(function() {
  var _listenerStore = {};
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

  // P1b S3 incr-c：fetch 返回 Response 对象（spec-compliance：ok/status/text()/json()）。
  // body 经 `__zw_fetch` 抓取；`__zwResolveCallback` 触发 pending resolver，包装为 Response。
  // body 以 `__zw_fetch_error` 开头 → ok:false（错误标记约定，见 register_fetch_callback）。
  function _makeResponse(body) {
    var ok = typeof body === 'string' && body.indexOf('__zw_fetch_error') !== 0;
    return {
      ok: ok,
      status: ok ? 200 : 0,
      statusText: ok ? 'OK' : 'Error',
      text: function() { return Promise.resolve(ok ? body : ''); },
      json: function() { return Promise.resolve(JSON.parse(ok ? body : 'null')); }
    };
  }

  // P1b S3 incr-a：`fetch(url)` 经 `__zw_fetch(id, url)` 回调异步抓取 + Promise（incr-c
  // 起 resolve Response 对象）。`__zw_fetch` 未注册（engine/renderer/reftest 路径无
  // browser fetch handler）时 resolve ok:false Response（stub，避免悬挂，零回归）。
  if (!globalThis.fetch) {
    globalThis.fetch = function(url) {
      if (typeof __zw_fetch !== 'function') {
        return Promise.resolve(_makeResponse('__zw_fetch_error:no-handler'));
      }
      return new Promise(function(resolve) {
        globalThis.__zw_fetch_counter = (globalThis.__zw_fetch_counter | 0) + 1;
        var id = '__zwfid:' + globalThis.__zw_fetch_counter;
        globalThis.__zw_pending[id] = function(body) { resolve(_makeResponse(body)); };
        try {
          __zw_fetch(id, url);
        } catch (_e) {
          resolve(_makeResponse('__zw_fetch_error:throw'));
        }
      });
    };
  }

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
  // ③ rootMargin 暂按 0 处理（defer 像素/% 展开）；④ root 为元素时取其 selector rect。
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
  globalThis.IntersectionObserver = function(callback, options) {
    this._callback = callback;
    var opts = options || {};
    this._thresholds = _io_normThresholds(opts.threshold);
    // root：null（默认 viewport）或元素（取其 __zwSelector 的 rect）。
    this._rootSel = (opts.root && opts.root.__zwSelector) ? opts.root.__zwSelector : null;
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
  // ② contentRect 取 gBCR rect（≈border-box，真实浏览器报 content-box，padding/border 扣除为 follow-up）；
  // ③ borderBoxSize/contentBoxSize 近似为单元素数组（inlineSize=width、blockSize=height）。
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
          entries.push({
            target: t.proxy,
            contentRect: _io_domRect(r.x, r.y, r.w, r.h),
            // borderBoxSize / contentBoxSize：单元素数组，inlineSize=width、blockSize=height（近似）。
            borderBoxSize: [{ inlineSize: r.w, blockSize: r.h }],
            contentBoxSize: [{ inlineSize: r.w, blockSize: r.h }],
            devicePixelContentBoxSize: [{ inlineSize: r.w, blockSize: r.h }],
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
  globalThis.__zw_reset_form_state = function() { _inputValues = {}; _classCache = {}; _customValidity = {}; _indeterminate = {}; _textSelection = {}; _outputDefault = {}; _outputValue = {}; };

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

  // crypto——Web Crypto 起步：randomUUID（UUID v4）+ getRandomValues（TypedArray 填充）。高频
  //（id 生成 / analytics / 随机字节）。**已知限制**：Math.random-based（**非 CSPRNG**），对 id 生成 /
  // 非安全随机字节主流用途足够；安全敏感场景（token / 密钥）需 host OS-random（如 getrandom crate）
  // 接入（follow-up，届时 randomUUID + getRandomValues 一并升级 CSPRNG）。
  globalThis.crypto = globalThis.crypto || {
    randomUUID: function () {
      var h = '0123456789abcdef';
      var s = '';
      for (var i = 0; i < 36; i++) {
        if (i === 8 || i === 13 || i === 18 || i === 23) s += '-';
        else if (i === 14) s += '4';
        else if (i === 19) s += h[(Math.random() * 4) | 0 | 8]; // y ∈ 8,9,a,b
        else s += h[(Math.random() * 16) | 0];
      }
      return s;
    },
    // getRandomValues(typedArray)：spec 限定 TypedArray（Int8..Uint32 / BigInt64/BigUint64），≤65536
    // 字节。填**底层字节 buffer**（Uint8Array 视图）→ 任意 typed 视图得随机值（含多字节 / 共享 buffer
    // 偏移）。Math.random 字节级（非 CSPRNG）。
    getRandomValues: function (arr) {
      if (typeof ArrayBuffer === 'undefined' || !ArrayBuffer.isView(arr)) {
        throw new TypeError('getRandomValues: argument must be a TypedArray');
      }
      if (arr.byteLength > 65536)
        throw new DOMException("The ArrayBufferView byte length (" + arr.byteLength + ") exceeds 65536.", 'QuotaExceededError');
      var view = new Uint8Array(arr.buffer, arr.byteOffset, arr.byteLength);
      for (var i = 0; i < view.length; i++) view[i] = (Math.random() * 256) | 0;
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
  // crypto.subtle.digest（R2793）——SHA-1/256/384/512 哈希（SRI / JWT / 内容哈希高频）。委托 host
  // `__zw_crypto_subtle_digest(algo, bytesCsv)`（RustCrypto sha1/sha2）；返 Promise<ArrayBuffer>（Uint8Array）。
  // algo 取串或 {name}；unsupported algo / host 未注册 → reject NotSupportedError。**scope 仅 digest**
  //（HMAC/sign/verify/encrypt/importKey/deriveBits defer——WebCrypto 大表面，digest 为最高频子集）。
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
          reject(new DOMException("Unsupported hash algorithm: '" + a + "'", 'NotSupportedError'));
          return;
        }
        var parts = out.split(',');
        var arr = new Uint8Array(parts.length);
        for (var i = 0; i < parts.length; i++) arr[i] = +parts[i];
        resolve(arr);
      });
    }
  };

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
    // clipboard（R2817）——异步剪贴板 API（复制按钮 ubiquitous）。headless 无真剪贴板 → resolving
    // Promise stubs（readText→''，writeText/read/write→undefined），让 modern 脚本 feature-detect 后路径执行不抛。
    clipboard: {
      readText: function() { return Promise.resolve(''); },
      writeText: function(_text) { return Promise.resolve(undefined); },
      read: function() { return Promise.resolve([]); },
      write: function(_data) { return Promise.resolve(undefined); },
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
    self.status = 0;
    self.statusText = '';
    self.responseText = '';
    self.response = '';
    self.onreadystatechange = null;
    self.onload = null;
    self.onerror = null;
    self.open = function(_method, _url) { self.readyState = 1; };
    self.send = function(_body) {
      self.readyState = 4;
      self.status = 404;
      self.statusText = 'Not Found';
      if (typeof self.onreadystatechange === 'function') self.onreadystatechange();
      if (typeof self.onload === 'function') self.onload();
    };
    self.abort = function() {};
    self.setRequestHeader = function() {};
    self.getResponseHeader = function() { return null; };
    self.getAllResponseHeaders = function() { return ''; };
  };

  function _ieEventType(type) {
    var s = String(type);
    return s.indexOf('on') === 0 ? s.slice(2) : s;
  }

  function _attachEventForKey(key, type, fn) {
    var t = _ieEventType(type);
    if (!_listenerStore[key]) _listenerStore[key] = {};
    if (!_listenerStore[key][t]) _listenerStore[key][t] = [];
    _listenerStore[key][t].push({ fn: fn, capture: false });
  }

  function _detachEventForKey(key, type, fn) {
    var t = _ieEventType(type);
    if (!_listenerStore[key] || !_listenerStore[key][t]) return;
    _listenerStore[key][t] = _listenerStore[key][t].filter(function(l) { return l.fn !== fn; });
  }

  // addEventListener 第三参 `opts` 的 capture 提取：支持 legacy 布尔形式（`addEventListener(t, fn, true)`
  // = capture）与对象形式（`{ capture: true }`）。旧实现仅认对象形式，布尔 true 被忽略 → capture listener
  // 注册不上（capture 阶段 R2693 因此对布尔形式失效）。removeEventListener 第三参同语义（useCapture
  // 须匹配才移除，spec）。
  function _optCapture(opts) {
    return !!(opts === true || (opts && opts.capture));
  }

  // addEventListener `opts.once` 提取（仅对象形式 `{ once: true }`；布尔形式无 once 语义）。
  function _optOnce(opts) {
    return !!(opts && opts.once);
  }

  function _globalAddEventListener(type, fn, opts) {
    var key = _elKey('html', null);
    var t = String(type);
    if (!_listenerStore[key]) _listenerStore[key] = {};
    if (!_listenerStore[key][t]) _listenerStore[key][t] = [];
    _listenerStore[key][t].push({ fn: fn, capture: _optCapture(opts), once: _optOnce(opts) });
  }

  // removeEventListener：spec 要求 useCapture（第三参）须与注册时匹配才移除——故
  // `addEventListener(t, fn, true)` 的 capture 注册仅 `removeEventListener(t, fn, true)` 能移除，
  // `removeEventListener(t, fn)`（capture=false）不动它。旧实现仅按 fn 过滤，误删 capture 注册。
  function _globalRemoveEventListener(type, fn, opts) {
    var key = _elKey('html', null);
    var t = String(type);
    if (!_listenerStore[key] || !_listenerStore[key][t]) return;
    var cap = _optCapture(opts);
    _listenerStore[key][t] = _listenerStore[key][t].filter(function(l) {
      return !(l.fn === fn && l.capture === cap);
    });
  }

  globalThis.Node = function Node() {};
  globalThis.Element = function Element() {};
  globalThis.HTMLElement = function HTMLElement() {};
  globalThis.Node.prototype = {};
  globalThis.Element.prototype = Object.create(globalThis.Node.prototype);
  globalThis.HTMLElement.prototype = Object.create(globalThis.Element.prototype);
  // Node.DOCUMENT_POSITION_* 静态常量（compareDocumentPosition bitmask，R2815）——库常读 Node.DOCUMENT_POSITION_FOLLOWING 等。
  globalThis.Node.DOCUMENT_POSITION_DISCONNECTED = 1;
  globalThis.Node.DOCUMENT_POSITION_PRECEDING = 2;
  globalThis.Node.DOCUMENT_POSITION_FOLLOWING = 4;
  globalThis.Node.DOCUMENT_POSITION_CONTAINS = 8;
  globalThis.Node.DOCUMENT_POSITION_CONTAINED_BY = 16;
  globalThis.Node.DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC = 32;
  globalThis.Element.prototype.addEventListener = function(type, fn, opts) {
    _globalAddEventListener(type, fn, opts);
  };
  globalThis.Element.prototype.removeEventListener = function(type, fn, opts) {
    _globalRemoveEventListener(type, fn, opts);
  };

  // customElements（CustomElementRegistry，R2813）——web components 生态门控（lit / stencil / fast 及所有
  // custom-element 库 feature-detect `window.customElements` + define/whenDefined）。**scoped registry slice**：
  // define/get/getName/whenDefined（同步 bookkeeping + whenDefined Promise）+ upgrade stub。**诚实 defer**：
  // element 实例化 upgrade（element 创建路径 `__zw_create_element` 返 generic Proxy，非 ctor 实例）+
  // connectedCallback/disconnectedCallback/attributeChangedCallback（需 mutation 观察）——深项，记后续 slice。
  // 本 slice 提供 feature-detection + 注册 + 查询 + whenDefined await（库 bootstrap 高频），不谎称 upgrade 生效。
  var _ce_registry = {};       // name → { ctor, options }
  var _ce_byCtor = new Map();  // ctor → name（getName 反查）
  var _ce_pending = {};        // name → [resolve]（whenDefined 挂起，define 时触发）
  var _CE_RESERVED = {
    'annotation-xml': 1, 'color-profile': 1, 'font-face': 1, 'font-face-src': 1,
    'font-face-uri': 1, 'font-face-format': 1, 'font-face-name': 1, 'missing-glyph': 1,
  };
  // 有效 custom element 名：首字符小写 ASCII 字母 + 含连字符 + 仅小写字母/数字/./-（spec PotentialCustomElementName
  // 简化，不含 uppercase / PASCII）。reserved 名拒。
  function _ce_validName(name) {
    if (typeof name !== 'string') return false;
    return /^[a-z][a-z0-9.-]*-[a-z0-9.-]*$/.test(name) && !_CE_RESERVED[name];
  }
  globalThis.customElements = globalThis.customElements || {
    define: function (name, ctor, options) {
      if (!_ce_validName(name)) {
        throw new Error("Failed to execute 'define' on 'CustomElementRegistry': \"" + name + "\" is not a valid custom element name");
      }
      if (typeof ctor !== 'function') {
        throw new TypeError("Failed to execute 'define' on 'CustomElementRegistry': parameter 2 is not a constructor");
      }
      if (_ce_registry[name]) {
        throw new Error("Failed to execute 'define' on 'CustomElementRegistry': the name \"" + name + "\" has already been used with this registry");
      }
      if (_ce_byCtor.has(ctor)) {
        throw new Error("Failed to execute 'define' on 'CustomElementRegistry': this constructor has already been used with this registry");
      }
      _ce_registry[name] = { ctor: ctor, options: options || {} };
      _ce_byCtor.set(ctor, name);
      var waiters = _ce_pending[name];
      if (waiters) {
        delete _ce_pending[name];
        for (var i = 0; i < waiters.length; i++) { try { waiters[i](ctor); } catch (_e) {} }
      }
    },
    get: function (name) {
      var entry = _ce_registry[name];
      return entry ? entry.ctor : undefined;
    },
    getName: function (ctor) {
      return _ce_byCtor.get(ctor) || null;
    },
    // whenDefined(name)：valid name → Promise<ctor>（已定义立即 resolve，否则挂起至 define 触发）；
    // invalid name → Promise reject（spec 一致，不同步抛）。Promise resolve 异步（microtask）。
    whenDefined: function (name) {
      if (!_ce_validName(name)) {
        return Promise.reject(new Error("Failed to execute 'whenDefined' on 'CustomElementRegistry': \"" + name + "\" is not a valid custom element name"));
      }
      var entry = _ce_registry[name];
      if (entry) return Promise.resolve(entry.ctor);
      return new Promise(function (resolve) {
        (_ce_pending[name] = _ce_pending[name] || []).push(resolve);
      });
    },
    // upgrade(root)：升级 root 子树 custom elements。**defer**（element 创建未接 ctor——proxy 非 ctor 实例，
    // upgrade 深项后续 slice）。spec 返 undefined，本 stub no-op 不抛（避免中断脚本）。
    upgrade: function (_root) {},
  };

  function _elKey(sel, handle) {
    return handle ? ('@' + handle) : sel;
  }

  // Constraint Validation ValidityState（R2825）。customError 由 setCustomValidity 跟踪（非空消息→invalid）；
  // 原生约束（valueMissing/typeMismatch/patternMismatch/tooLong/tooShort/rangeUnderflow/rangeOverflow/
  // stepMismatch/badInput）headless 不强制，恒 false（permissive valid——表单校验库 checkValidity 走 valid 路径）。
  function _validityState(key) {
    var hasCustom = _customValidity[key] != null && _customValidity[key] !== '';
    return {
      valueMissing: false, typeMismatch: false, patternMismatch: false,
      tooLong: false, tooShort: false, rangeUnderflow: false, rangeOverflow: false,
      stepMismatch: false, badInput: false, customError: hasCustom,
      valid: !hasCustom,
    };
  }

  // Web Animations Animation 对象（el.animate permissive stub，R2827）。headless 无真时间轴 / 关键帧应用
  // → 动画「瞬间完成」：创建即 playState='running'，execute 末 _defer microtask 后 playState='finished' +
  // finished Promise resolve + onfinish 触发（除非 cancel）。让 modern 动画库（Framer Motion / GSAP / Lottie）
  // feature-detect `el.animate` + 链式（await anim.finished / anim.onfinish）通过——动画不真播放，但回调链走通。
  function _makeAnimation(options) {
    var anim = {
      playState: 'running',
      currentTime: 0,
      startTime: 0,
      playbackRate: 1,
      duration: 0,
      id: '',
      onfinish: null,
      oncancel: null,
      onremove: null,
      _cancelled: false,
      play: function () { anim.playState = 'running'; },
      pause: function () { anim.playState = 'paused'; },
      cancel: function () { anim._cancelled = true; anim.playState = 'idle'; },
      finish: function () { anim.playState = 'finished'; },
      reverse: function () { anim.playbackRate = -anim.playbackRate; return anim; },
      updatePlaybackRate: function (rate) { anim.playbackRate = rate; },
      commitStyles: function () {},
      persist: function () {},
      addEventListener: function () {},
      removeEventListener: function () {},
      dispatchEvent: function () { return true; },
    };
    // options：number=duration(ms) / object={duration,id,...}。提取 duration（finish 后 currentTime 用）+ id。
    if (options != null) {
      if (typeof options === 'number') anim.duration = options;
      else {
        if (typeof options.duration === 'number') anim.duration = options.duration;
        if (options.id != null) anim.id = String(options.id);
      }
    }
    var resolveFinish;
    anim._finishedP = new Promise(function (r) { resolveFinish = r; });
    Object.defineProperty(anim, 'finished', { get: function () { return anim._finishedP; } });
    // headless 瞬间完成（无真时间轴）—— microtask 后 finished + onfinish（cancel 则 idle 不完成）。
    _defer(function () {
      if (!anim._cancelled) {
        anim.playState = 'finished';
        anim.currentTime = anim.duration;
        resolveFinish(anim);
        if (typeof anim.onfinish === 'function') {
          try { anim.onfinish({ type: 'finish', target: anim, currentTime: anim.currentTime }); } catch (_e) {}
        }
      }
    });
    return anim;
  }

  // 读元素当前 class（缓存优先，lazy-init 自 snapshot）。className get 与 classList 共用，
  // 使同脚本内连续 class 操作看到累积状态而非各自读 stale snapshot。
  function _readClass(key, sel, handle) {
    if (_classCache[key] != null) return _classCache[key];
    var v = (handle ? __zw_get_attr_handle(handle, 'class') : __zw_get_attr(sel, 'class')) || '';
    _classCache[key] = v;
    return v;
  }

  function _classListProxy(sel, handle) {
    var key = _elKey(sel, handle);
    var write = function(v) {
      _classCache[key] = v;
      if (handle) __zw_set_attr_handle(handle, 'class', v);
      else __zw_set_attr(sel, 'class', v);
      _mo_notify(sel, handle, { type: 'attributes', attributeName: 'class' });
    };
    return {
      add: function(c) {
        var parts = _readClass(key, sel, handle).split(/\s+/).filter(Boolean);
        if (parts.indexOf(c) < 0) parts.push(c);
        write(parts.join(' '));
      },
      remove: function(c) {
        var parts = _readClass(key, sel, handle)
          .split(/\s+/)
          .filter(Boolean)
          .filter(function(x) { return x !== c; });
        write(parts.join(' '));
      },
      toggle: function(c) {
        var parts = _readClass(key, sel, handle).split(/\s+/).filter(Boolean);
        var i = parts.indexOf(c);
        var on;
        if (i >= 0) {
          parts.splice(i, 1);
          on = false;
        } else {
          parts.push(c);
          on = true;
        }
        write(parts.join(' '));
        return on;
      },
      contains: function(c) {
        return _readClass(key, sel, handle).split(/\s+/).indexOf(c) >= 0;
      }
    };
  }

  // 派发某元素 key 上的事件 listener。`phase`：`'all'`（target 阶段，capture+非 capture，默认）、
  // `'capture'`（仅 capture listener，捕获期祖先用）、`'bubble'`（仅非 capture，冒泡期祖先用）。
  // `thisObj`：handler 内 `this` 与 `event.currentTarget`（默认 event.target）。`stopImmediatePropagation`
  // 中断当前节点内后续 listener。`once` listener（`{once:true}` 注册）派发后自动移除——用快照迭代，
  // 派发完一次性从原 list 滤除已触发的 once 条目（不扰动迭代；reentrancy 下按对象引用滤除安全）。
  function _dispatchToListeners(key, event, phase, thisObj) {
    var listeners = _listenerStore[key];
    if (!listeners || !listeners[event.type]) return !event._defaultPrevented;
    var list = listeners[event.type];
    var ctx = thisObj || event.target;
    event.currentTarget = ctx;
    var snap = list.slice();
    var firedOnce = null;
    var fire = function(entry) {
      entry.fn.call(ctx, event);
      if (entry.once) {
        if (!firedOnce) firedOnce = [];
        firedOnce.push(entry);
      }
    };
    if (phase !== 'bubble') {
      for (var i = 0; i < snap.length; i++) {
        if (snap[i].capture) {
          fire(snap[i]);
          if (event._immediateStopped) break;
        }
      }
    }
    if (phase !== 'capture' && !event._immediateStopped) {
      for (var j = 0; j < snap.length; j++) {
        if (!snap[j].capture) {
          fire(snap[j]);
          if (event._immediateStopped) break;
        }
      }
    }
    if (firedOnce) {
      listeners[event.type] = list.filter(function(e) { return firedOnce.indexOf(e) < 0; });
    }
    return !event._defaultPrevented;
  }

  // 按规范三阶段派发事件：①capture（root→target 的祖先，capture-only）②target（capture+非 capture，
  // AT_TARGET）③bubble（target→root 的祖先，非 capture，仅 event.bubbles）。事件委托基础：祖先 listener
  // 现在经捕获/冒泡两期触发（R2692 仅冒泡、R2693 补捕获）。`event.currentTarget` 随阶段更新。
  // 仅 sel-based target 且 `__zw_parent` 注册时走 capture/bubble（polyfill/handle-only detached 无父链 →
  // 仅 target，保旧行为）。kill-switch：`globalThis.__zw_no_capture` 关捕获期、`__zw_no_bubble` 关冒泡期。
  function _dispatchWithBubble(targetKey, targetSel, targetHandle, event) {
    var target = _makeProxy(targetSel, targetHandle);
    event.target = target;

    // 祖先链 target→root（[直接父, ..., html]）；无 __zw_parent / handle-only → 空 → 仅 target 派发。
    var chain = [];
    if (targetSel && typeof __zw_parent === 'function') {
      var cur = targetSel;
      while (true) {
        var p;
        try { p = __zw_parent(cur); } catch (_e) { p = ''; }
        if (!p) break;
        chain.push(p);
        cur = p;
      }
    }
    var propagate = chain.length > 0;

    // ① capture 阶段：root→target 方向（chain 反序），祖先派发 capture-only。
    if (propagate && !globalThis.__zw_no_capture) {
      for (var i = chain.length - 1; i >= 0; i--) {
        var capAnc = _wrapSelector(chain[i]);
        _dispatchToListeners(_elKey(chain[i], null), event, 'capture', capAnc);
        if (event._propagationStopped) return !event._defaultPrevented;
      }
    }

    // ② target 阶段：capture + 非 capture（AT_TARGET，保旧行为）。
    event.currentTarget = target;
    _dispatchToListeners(targetKey, event, 'all', target);
    if (event._propagationStopped) return !event._defaultPrevented;

    // ③ bubble 阶段：target→root 方向（chain 正序），祖先派发非 capture（仅 event.bubbles）。
    if (propagate && event.bubbles && !globalThis.__zw_no_bubble) {
      for (var k = 0; k < chain.length; k++) {
        var bAnc = _wrapSelector(chain[k]);
        _dispatchToListeners(_elKey(chain[k], null), event, 'bubble', bAnc);
        if (event._propagationStopped) break;
      }
    }
    return !event._defaultPrevented;
  }

  function _makeEvent(type, options) {
    options = options || {};
    var ev = {
      type: type,
      bubbles: !!options.bubbles,
      cancelable: !!options.cancelable,
      composed: false, // spec Event.composed 初值 false
      eventPhase: 0, // spec NONE=0
      isTrusted: false, // spec（合成事件恒 false）
      target: null,
      currentTarget: null,
      timeStamp: typeof __zw_performance_now === 'function'
        ? Number(__zw_performance_now())
        : (typeof Date.now === 'function' ? Date.now() : 0),
      detail: options.detail, // CustomEvent 用；Event 读得 undefined（spec 一致）
      defaultPrevented: false, // 公开镜像（dispatch 读 _defaultPrevented，勿删私字段）
      _defaultPrevented: false,
      _propagationStopped: false,
      _immediateStopped: false,
      preventDefault: function() { if (this.cancelable) { this.defaultPrevented = true; this._defaultPrevented = true; } },
      stopPropagation: function() { this._propagationStopped = true; },
      stopImmediatePropagation: function() {
        this._immediateStopped = true;
        this._propagationStopped = true;
      }
    };
    return ev;
  }

  function _tagFromSel(sel) {
    if (!sel) return 'DIV';
    if (sel.charAt(0) === '#') return 'DIV';
    if (sel.indexOf('.') >= 0) {
      var dot = sel.indexOf('.');
      var tag = sel.slice(0, dot);
      return tag ? tag.toUpperCase() : 'DIV';
    }
    return String(sel).toUpperCase();
  }

  // 真实 tag 名（修正 `_tagFromSel` 对 id-only 选择器恒猜 'DIV' 的正确性 bug——
  // `document.getElementById('foo').tagName` 对 `<span id=foo>` 错返 'DIV'）。优先 host 回调：
  // sel-based 元素经 `__zw_get_tag(sel)`（query_tag_from_html，真实 tag），handle-only（detached
  // createElement）经 `__zw_get_tag_handle(handle)`（CreateElement 记录的 tag）。host 未注册
  // （polyfill/WebView 路径）或未命中 → fallback `_tagFromSel`（启发式，保旧行为）。
  // tag 取小写 local_name，tagName/nodeName 在 HTML 命名空间须大写 → 统一 toUpperCase。
  function _realTag(sel, handle) {
    if (sel && typeof __zw_get_tag === 'function') {
      try { var t = __zw_get_tag(sel); if (t) return t.toUpperCase(); } catch (_e) {}
    }
    if (handle && typeof __zw_get_tag_handle === 'function') {
      try { var ht = __zw_get_tag_handle(handle); if (ht) return ht.toUpperCase(); } catch (_e) {}
    }
    return _tagFromSel(sel);
  }

  // P1a select：经 host `__zw_get_tag` 判元素是否为某 tag（selector-identity 元素）。
  // `_tagFromSel` 是启发式（id-only 选择器猜 DIV），不足以判 SELECT；host 查询准确。
  function _isTag(sel, tagUpper) {
    if (!sel || typeof __zw_get_tag !== 'function') return false;
    try { return __zw_get_tag(sel).toUpperCase() === tagUpper; } catch (_e) { return false; }
  }

  // text control 选区（R2844）：判元素是否为支持选区的 text control——TEXTAREA，或 INPUT 的 type 属于
  // {text, search, url, tel, password, 空}（Chromium 150 oracle：这些 type selectionStart/End 返数值；
  // number/email/date/range/color/checkbox 等 → null，非 text control）。无 type 属性 / 无效 type 归 text。
  var _TEXT_SEL_TYPES = { '': 1, text: 1, search: 1, url: 1, tel: 1, password: 1 };
  function _isTextControl(sel, handle) {
    var tag = _realTag(sel, handle);
    if (tag === 'TEXTAREA') return true;
    if (tag !== 'INPUT') return false;
    var ty;
    try { ty = handle ? __zw_get_attr_handle(handle, 'type') : __zw_get_attr(sel, 'type'); } catch (_e) { ty = ''; }
    return Object.prototype.hasOwnProperty.call(_TEXT_SEL_TYPES, (ty || '').toLowerCase());
  }
  // text control 当前 value 串（mirror value getter 的 lazy-init 逻辑，仅读不改缓存——选区 clamp 须 length）。
  function _controlValue(sel, handle, key) {
    if (_inputValues[key] != null) return String(_inputValues[key]);
    var v = '';
    if (!handle && sel && _isTag(sel, 'TEXTAREA')) {
      try { v = __zw_get_text(sel) || ''; } catch (_e) {}
    } else {
      try {
        var va = handle ? __zw_get_attr_handle(handle, 'value') : (sel ? __zw_get_attr(sel, 'value') : null);
        if (va != null) v = va;
      } catch (_e) {}
    }
    return String(v);
  }
  // 选区偏移 clamp：把任意输入归一为 [0, len] 内整数（Chromium 对超界/负值/非数 clamp 到边界，非抛）。
  function _clampSelOffset(v, len) {
    var n = (typeof v === 'number') ? Math.floor(v) : parseInt(v, 10);
    if (isNaN(n)) n = 0;
    if (n < 0) n = 0;
    if (n > len) n = len;
    return n;
  }
  // 取/建元素选区对象（getter 用默认 {0,0,'forward'}，不污染 map；setter/method 先 ensure 再 mutate）。
  function _selObj(key) {
    if (!_textSelection[key]) _textSelection[key] = { start: 0, end: 0, direction: 'forward' };
    return _textSelection[key];
  }

  // `el.parentNode` / `parentElement`：经 host `__zw_parent(sel)` 返真实元素父选择器
  //（修正旧 stub 对嵌套元素恒返 body 的 bug）。handle-only（detached）或无回调 → fallback stub
  //（detached 元素无真实 parent；html/body/head 用文档结构近似）。
  function _parentNodeFor(sel, handle) {
    if (sel && typeof __zw_parent === 'function') {
      try {
        var p = __zw_parent(sel);
        if (p) return _wrapSelector(p);
        return null; // html 根 / 未命中 → 无元素父
      } catch (_e) { return null; }
    }
    // fallback（无 host 回调路径，如 polyfill）：文档结构近似。
    if (sel === 'html') return null;
    if (sel === 'body' || sel === 'head') return _wrapSelector('html');
    return _wrapSelector('body');
  }

  // 祖先链（self → root，含两端，sel 数组）——经 `__zw_parent` 上行。guard 防环。供 getRootNode /
  // compareDocumentPosition（R2815）。sel 缺失（handle-only detached）→ 空数组。
  function _ancestorChain(sel) {
    var chain = [];
    if (!sel || typeof __zw_parent !== 'function') return chain;
    var cur = sel, guard = 0;
    while (cur && guard < 4096) {
      chain.push(cur);
      var p = '';
      try { p = __zw_parent(cur) || ''; } catch (_e) { p = ''; }
      if (!p || p === cur) break;
      cur = p;
      guard++;
    }
    return chain;
  }

  // 最小 detached Document（供 document.implementation.createDocument/createHTMLDocument，R2815）。
  // **已知限制**：hollow——无 detached tree proxy infra，querySelector 返 null（jQuery/DOMPurify 真 detached
  // 解析需后续 detached-tree slice）。满足 feature-detection + 基本 node 工厂。
  function _makeDetachedDocument(title) {
    return {
      nodeType: 9,
      nodeName: '#document',
      documentElement: { nodeType: 1, tagName: 'HTML', nodeName: 'HTML', childNodes: [] },
      head: { nodeType: 1, tagName: 'HEAD', nodeName: 'HEAD', childNodes: [] },
      body: { nodeType: 1, tagName: 'BODY', nodeName: 'BODY', childNodes: [] },
      title: title != null ? String(title) : '',
      querySelector: function() { return null; },
      querySelectorAll: function() { return []; },
      getElementById: function() { return null; },
      createElement: function(t) { var n = String(t).toUpperCase(); return { nodeType: 1, tagName: n, nodeName: n }; },
      createTextNode: function(t) { return { nodeType: 3, nodeName: '#text', nodeValue: String(t) }; },
    };
  }

  // 节点结构签名（供 isEqualNode，R2819）：type 前缀 + 序列化（元素→outerHTML 含 tag/属性/子树；
  // text→nodeValue；comment→nodeValue）。两节点签名相等即结构相等。**已知限制**：属性序敏感
  //（spec isEqualNode 属性序无关——outerHTML 按序序列化，故属性序不同会判不等；实际库属性序一致，足够）。
  function _nodeSig(sel, handle) {
    if (handle && _commentHandles[handle]) {
      var cv = (typeof __zw_get_text_handle === 'function') ? (__zw_get_text_handle(handle) || '') : '';
      return '8:' + cv;
    }
    if (handle && _textHandles[handle]) {
      var tv = (typeof __zw_get_text_handle === 'function') ? (__zw_get_text_handle(handle) || '') : '';
      return '3:' + tv;
    }
    if (sel && typeof __zw_get_outer_html === 'function') {
      try { return '1:' + __zw_get_outer_html(sel); } catch (_e) {}
    }
    if (handle && typeof __zw_get_inner_html_handle === 'function') {
      try { return '1:' + (__zw_get_inner_html_handle(handle) || ''); } catch (_e) {}
    }
    return '?';
  }

  // `sel` 支持单选择器串或多选择器数组——多 tag 集合（links=a[href]+area[href]、embeds/plugins=
  // embed+object）须逐选择器查询后 concat（querySelectorAll 顶层不支持逗号选择器列表，仅 :is()/:where()
  // 内部支持）。disjoint tag 故无需去重。R2833：扩展自单串以修正 links（旧返全部 `<a>`，spec 仅 a[href]）。
  // `has` trap（R2833）使 `Array.prototype.map/forEach/filter.call(coll, fn)` 等数组方法工作——它们先经
  // HasProperty 判定索引存在性，无 has trap 时 {length:0} target 对数值索引恒判 absent 致迭代得空。
  function _liveQueryCollection(sel) {
    var sels = Array.isArray(sel) ? sel : [sel];
    function snapshot() {
      var list = [];
      for (var i = 0; i < sels.length; i++) {
        var found = globalThis.document.querySelectorAll(sels[i]);
        for (var j = 0; j < found.length; j++) list.push(found[j]);
      }
      return list;
    }
    return new Proxy({ length: 0 }, {
      get: function(_t, prop) {
        var list = snapshot();
        if (prop === 'length') return list.length;
        if (prop === 'item') return function(i) { return list[i] || null; };
        var idx = parseInt(prop, 10);
        if (!isNaN(idx) && String(idx) === String(prop)) return list[idx];
        return list[prop];
      },
      has: function(_t, prop) {
        var idx = parseInt(prop, 10);
        if (!isNaN(idx) && String(idx) === String(prop)) {
          var list = snapshot();
          return idx >= 0 && idx < list.length;
        }
        return false;
      }
    });
  }

  // P1a select：`select.options` 集合（live）——`length`/索引访问/`item(i)` + `selectedIndex`/`value`
  // （与 select 同）。每次访问经 `querySelectorAll(sel + ' option')`（live，反映 dom_html）。
  // 各 option 经 R2664 唯一选择器，`.value`/`.selected` 读对。
  function _selectOptions(sel) {
    return new Proxy({}, {
      get: function(_t, prop) {
        var list = globalThis.document.querySelectorAll(sel + ' option');
        if (prop === 'length') return list.length;
        if (prop === 'item') return function(i) { return list[i] || null; };
        if (prop === 'selectedIndex') {
          try { return parseInt(__zw_select_index(sel), 10); } catch (_e) { return -1; }
        }
        if (prop === 'value') {
          try { return __zw_select_value(sel); } catch (_e) { return ''; }
        }
        var idx = parseInt(prop, 10);
        if (!isNaN(idx) && String(idx) === String(prop)) return list[idx];
        return undefined;
      }
    });
  }

  // P1a select：`select.selectedOptions`——选中 option 数组（各 `.selected`=true）。
  function _selectSelectedOptions(sel) {
    var list = globalThis.document.querySelectorAll(sel + ' option');
    var out = [];
    for (var i = 0; i < list.length; i++) {
      if (list[i].selected) out.push(list[i]);
    }
    return out;
  }

  // `el.style` CSSStyleDeclaration 代理：per-property get/set（`style.color`/`style.color='red'`）
  // + 方法（`setProperty`/`getPropertyValue`/`removeProperty`）+ `cssText` 整体读写 + `item`/`length`
  // 枚举。旧实现仅 per-property get/set，缺方法（调用即 TypeError）与 cssText（get 返 ''、set 误当
  // 属性名）。底层走 `__zw_set_style`/`__zw_get_attr('style')`；removeProperty 经 `__zw_remove_style`
  // 真移除声明（SetStyle 空值仍 push，不移除）；cssText set 经 `__zw_set_attr` 整体替换。
  // `el.attributes`（NamedNodeMap，只读快照）：length / item(i) / getNamedItem(name) / 数值索引 /
  // Symbol.iterator，每项 Attr-like {name,value,localName,...}。经 `__zw_attr_names`+`__zw_get_attr`。
  // handle-only（无 attr_names 变体）→ 空集；setNamedItem/removeNamedItem 只读 no-op（deferred 模式下
  // 改属性走 setAttribute/removeAttribute，NamedNodeMap 为只读快照视图）。
  function _attributesProxy(sel, handle) {
    var readNames = function() {
      if (!sel || typeof __zw_attr_names !== 'function') return [];
      try {
        var n = __zw_attr_names(sel);
        return n ? n.split('|').filter(Boolean) : [];
      } catch (_e) { return []; }
    };
    var attrObj = function(name) {
      var v = handle ? __zw_get_attr_handle(handle, name) : __zw_get_attr(sel, name);
      return {
        name: name,
        value: v || '',
        namespaceURI: null,
        prefix: null,
        localName: name,
        specified: true,
        ownerElement: _makeProxy(sel, handle)
      };
    };
    return new Proxy({}, {
      get: function(_t, p) {
        if (p === 'length') return readNames().length;
        if (p === 'item') {
          return function(i) {
            var names = readNames();
            var idx = i | 0;
            return idx >= 0 && idx < names.length ? attrObj(names[idx]) : null;
          };
        }
        if (p === 'getNamedItem') {
          return function(name) {
            var names = readNames();
            var n = String(name);
            return names.indexOf(n) >= 0 ? attrObj(n) : null;
          };
        }
        if (p === 'setNamedItem' || p === 'removeNamedItem') {
          return function() { return null; }; // 只读快照
        }
        if (p === Symbol.iterator) {
          return function() {
            var list = readNames().map(attrObj);
            var k = 0;
            return {
              next: function() {
                return k < list.length ? { value: list[k++], done: false } : { value: undefined, done: true };
              }
            };
          };
        }
        var names = readNames();
        var idx = parseInt(p, 10);
        if (!isNaN(idx) && String(idx) === String(p) && idx >= 0 && idx < names.length) {
          return attrObj(names[idx]);
        }
        return undefined;
      },
      has: function(_t, p) {
        // Array.prototype.map/forEach 经 `k in O`（HasProperty）判定——须对有效数值索引返 true，
        // 否则索引被当 hole 跳过（map 出空槽）。匹配 real NamedNodeMap 的 array-like 语义。
        if (p === 'length') return true;
        var names = readNames();
        var idx = parseInt(p, 10);
        return !isNaN(idx) && String(idx) === String(p) && idx >= 0 && idx < names.length;
      }
    });
  }

  // style 属性名归一：JS per-property 访问用 camelCase（`el.style.fontSize`），CSS 须 kebab-case
  //（`font-size`）；camelCase 直存 style 属性会被 CSS parser 忽略 → 渲染静默失效。归一 camelCase→
  // kebab（复用 `_camelToKebab`，对已 kebab 幂等）；`cssFloat`→`float`（JS 保留字特例）；`--custom`
  // 自定义属性大小写敏感，原样不转。
  function _stylePropName(name) {
    var s = String(name).trim();
    if (s === 'cssFloat') return 'float';
    if (s.charAt(0) === '-' && s.charAt(1) === '-') return s;
    return _camelToKebab(s);
  }

  function _styleProxy(sel, handle) {
    var readRaw = function() {
      return (handle ? __zw_get_attr_handle(handle, 'style') : __zw_get_attr(sel, 'style')) || '';
    };
    var readProp = function(name) {
      var raw = readRaw();
      if (!raw) return '';
      var want = _stylePropName(name).toLowerCase();
      var parts = raw.split(';');
      for (var i = 0; i < parts.length; i++) {
        var kv = parts[i].split(':');
        if (kv[0] && kv[0].trim().toLowerCase() === want) return (kv[1] || '').trim();
      }
      return '';
    };
    var setProp = function(name, value) {
      var prop = _stylePropName(name);
      if (handle) __zw_set_style_handle(handle, prop, String(value));
      else __zw_set_style(sel, prop, String(value));
      _mo_notify(sel, handle, { type: 'attributes', attributeName: 'style' });
    };
    var propNames = function() {
      var raw = readRaw();
      return raw
        .split(';')
        .map(function(s) { return s.split(':')[0].trim(); })
        .filter(Boolean);
    };
    return new Proxy({}, {
      get: function(_t, p) {
        var ps = String(p);
        if (ps === 'cssText') return readRaw();
        if (ps === 'length') return propNames().length;
        if (ps === 'getPropertyValue') return function(name) { return readProp(name); };
        if (ps === 'getPropertyPriority') return function() { return ''; }; // !priority 未跟踪
        if (ps === 'setProperty') return function(name, value) { setProp(name, value); return undefined; };
        if (ps === 'removeProperty') {
          return function(name) {
            var prev = readProp(name);
            var prop = _stylePropName(name);
            if (handle && typeof __zw_remove_style_handle === 'function') {
              __zw_remove_style_handle(handle, prop);
            } else if (!handle && typeof __zw_remove_style === 'function') {
              __zw_remove_style(sel, prop);
            }
            _mo_notify(sel, handle, { type: 'attributes', attributeName: 'style' });
            return prev;
          };
        }
        if (ps === 'item') return function(i) { return propNames()[i | 0] || ''; };
        return readProp(ps);
      },
      set: function(_t, p, v) {
        var ps = String(p);
        if (ps === 'cssText') {
          // 整体替换 style 属性（解析由 host/style-system 在 render 时处理）。
          if (handle) __zw_set_attr_handle(handle, 'style', String(v));
          else __zw_set_attr(sel, 'style', String(v));
          _mo_notify(sel, handle, { type: 'attributes', attributeName: 'style' });
          return true;
        }
        setProp(ps, v);
        return true;
      }
    });
  }



  function _makeProxy(sel, handle) {
    var key = _elKey(sel, handle);
    if (_proxyCache[key]) return _proxyCache[key];
    var proxy = new Proxy({}, {
      get: function(_t, prop) {
        if (prop === '__zwHandle') return handle;
        if (prop === '__zwSelector') return sel;
        if (prop === 'value') {
          // P1a select：<select>.value = 选中 option 的 value（HTML spec 语义，非 value 属性）。
          // selected 会随 host 设值变化，故不缓存（每次查 host 反映最新 dom_html）。
          if (!handle && sel && typeof __zw_select_value === 'function' && _isTag(sel, 'SELECT')) {
            try { return __zw_select_value(sel); } catch (_e) { return ''; }
          }
          // HTMLOutputElement.value（R2846）：spec 独立于 textContent——<output> 按 children 渲染非 value，
          // 设 .value 不触碰 DOM text。dirty（_outputValue 存在）→ 当前值；否则 → defaultValue（lazy textContent）。
          if (_realTag(sel, handle) === 'OUTPUT') {
            if (_outputValue[key] != null) return _outputValue[key];
            if (_outputDefault[key] == null) {
              _outputDefault[key] = handle ? (__zw_get_text_handle(handle) || '') : (__zw_get_text(sel) || '');
            }
            return _outputDefault[key];
          }
          // P1a form input：value get——per-element 缓存，lazy-init。
          // textarea 的 value 是其**文本内容**（非 value 属性，HTML spec）；input 是 value 属性。
          if (_inputValues[key] == null) {
            if (!handle && sel && _isTag(sel, 'TEXTAREA')) {
              _inputValues[key] = __zw_get_text(sel) || '';
            } else {
              var va = handle ? __zw_get_attr_handle(handle, 'value') : __zw_get_attr(sel, 'value');
              _inputValues[key] = (va == null) ? '' : va;
            }
          }
          return _inputValues[key];
        }
        // `input.valueAsNumber`（HTMLInputElement，R2836）——number/range 输入值↔数值转换（计算器/数量输入/
        // 校验库读 NaN 判非法）。type=number/range：parseFloat(value)（空/无效→NaN，parseFloat 对 '12px'
        // 等宽容近似 number 解析）；其他 type→NaN（date/month/week/time/datetime-local defer）。仅 INPUT。
        if (prop === 'valueAsNumber' && _realTag(sel, handle) === 'INPUT') {
          try {
            var vasT = (handle ? __zw_get_attr_handle(handle, 'type') : __zw_get_attr(sel, 'type')) || '';
            if (vasT.toLowerCase() !== 'number' && vasT.toLowerCase() !== 'range') return NaN;
            var vasV = _inputValues[key];
            if (vasV == null) vasV = (handle ? __zw_get_attr_handle(handle, 'value') : __zw_get_attr(sel, 'value')) || '';
            if (vasV === '') return NaN;
            var vasN = parseFloat(vasV);
            return isNaN(vasN) ? NaN : vasN;
          } catch (_e) { return NaN; }
        }
        // text-control 选区 getter（R2844）：selectionStart / selectionEnd / selectionDirection。
        // 仅 text control（_isTextControl gate）。默认 {0, 0, 'forward'}（Chromium 150 oracle 锚定）。
        // 文本编辑器 / 自动选择 / Range 算法读选区状态高频。非 text control 落 undefined（Chrome 返 null，
        // `!= null` 判定两者皆过——documented 微差）。getter 不污染 _textSelection（纯读）。
        if ((prop === 'selectionStart' || prop === 'selectionEnd' || prop === 'selectionDirection') &&
            _isTextControl(sel, handle)) {
          var gs = _textSelection[key] || { start: 0, end: 0, direction: 'forward' };
          if (prop === 'selectionStart') return gs.start;
          if (prop === 'selectionEnd') return gs.end;
          return gs.direction;
        }
        // `el.setSelectionRange(start, end, direction?)`（HTMLInputElement.textarea，R2844）——设选区。
        // Chromium 150 oracle 锚定：start/end clamp [0, len]；end<start → start 折叠到 end（setSR(4,2)→{2,2}）；
        // direction 缺省 'forward'，否则取给定值（'backward'/'none'，其他归 'forward'）。仅 text control。
        if (prop === 'setSelectionRange' && _isTextControl(sel, handle)) {
          return function(s, e, dir) {
            var len = _controlValue(sel, handle, key).length;
            var ne = _clampSelOffset(e, len);
            var ns = _clampSelOffset(s, len);
            if (ne < ns) ns = ne;
            var d = (dir === 'backward' || dir === 'none') ? dir : 'forward';
            var so = _selObj(key);
            so.start = ns; so.end = ne; so.direction = d;
            return undefined;
          };
        }
        // `input.files`（HTMLInputElement，R2830）——FileList（上传表单读 length/迭代）。headless
        // 无真文件 → 共享空 FileList（length 0）；仅 INPUT（_isTag gate），非 input → undefined。
        if (prop === 'files' && _isTag(sel, 'INPUT')) {
          return _emptyFileList;
        }
        // `input.indeterminate`（HTMLInputElement，R2831）——JS-only IDL 布尔（非 reflected attr），
        // per-element `_indeterminate` map（默认 false）。checkbox「全选」tri-state UI 高频。仅 INPUT。
        if (prop === 'indeterminate' && _isTag(sel, 'INPUT')) {
          return _indeterminate[key] === true;
        }
        if (prop === 'checked' || prop === 'hidden' || prop === 'disabled') {
          // boolean reflected property（checked/hidden/disabled）——属性存在性（经 host `__zw_has_attr`）。
          if (typeof __zw_has_attr === 'function') {
            try { return __zw_has_attr(sel, String(prop)) === '1'; } catch (_e) {}
          }
          return false;
        }
        if (prop === 'selectedIndex') {
          // P1a select：选中 option 的索引（host `__zw_select_index`）。非 select → -1。
          if (!handle && sel && typeof __zw_select_index === 'function' && _isTag(sel, 'SELECT')) {
            try { return parseInt(__zw_select_index(sel), 10); } catch (_e) {}
          }
          return -1;
        }
        if (prop === 'selected') {
          // P1a select option：selected 属性存在性（boolean）。sel-based 经 host `__zw_has_attr`；
          // handle-based（`new Option()` 创建）经 `__zw_has_attr_handle`（句柄不在快照）。
          if (handle && typeof __zw_has_attr_handle === 'function') {
            try { return __zw_has_attr_handle(handle, 'selected') === '1'; } catch (_e) {}
          }
          if (!handle && sel && typeof __zw_has_attr === 'function') {
            try { return __zw_has_attr(sel, 'selected') === '1'; } catch (_e) {}
          }
          return false;
        }
        if (prop === 'options' && !handle && sel && _isTag(sel, 'SELECT')) {
          // P1a select：`select.options` live 集合（length/索引/item + selectedIndex/value）。
          return _selectOptions(sel);
        }
        if (prop === 'selectedOptions' && !handle && sel && _isTag(sel, 'SELECT')) {
          // P1a select：`select.selectedOptions` 选中 option 数组。
          return _selectSelectedOptions(sel);
        }
        // `select.add(element, before?)`（HTMLOptionsCollection，R2832）——追加 option（或插 before 前）。
        // 仅 SELECT（_realTag gate）；与 `new Option()` 配对做动态下拉填充。appendChild / insertBefore 复用。
        if (prop === 'add' && _realTag(sel, handle) === 'SELECT') {
          return function (element, before) {
            if (!element || !element.__zwHandle) return undefined;
            if (before == null) {
              if (handle) __zw_append_child_handle(handle, element.__zwHandle);
              else __zw_append_child(sel, element.__zwHandle);
            } else if (before.__zwSelector) {
              if (handle) __zw_insert_before_handle(handle, element.__zwHandle, before.__zwSelector);
              else __zw_insert_before(sel, element.__zwHandle, before.__zwSelector);
            }
            return undefined;
          };
        }
        // HTMLMediaElement 方法（play/pause/load/canPlayType，R2835）——仅 AUDIO/VIDEO（_realTag gate，
        // 支持 sel + handle 两种身份——new Audio 创建的 handle-based 亦可调）。headless 无音视频设备：
        // play 返 resolved Promise（spec：HTMLMediaElement.play() 返 Promise），pause/load no-op，
        // canPlayType 返 ''（保守「不可播放」）。使 `new Audio(url).play().then(...)` 不抛（媒体 UI 主模式）。
        if (prop === 'play' && (_realTag(sel, handle) === 'AUDIO' || _realTag(sel, handle) === 'VIDEO')) {
          return function () { return Promise.resolve(undefined); };
        }
        if (prop === 'pause' && (_realTag(sel, handle) === 'AUDIO' || _realTag(sel, handle) === 'VIDEO')) {
          return function () {};
        }
        if (prop === 'load' && (_realTag(sel, handle) === 'AUDIO' || _realTag(sel, handle) === 'VIDEO')) {
          return function () {};
        }
        if (prop === 'canPlayType' && (_realTag(sel, handle) === 'AUDIO' || _realTag(sel, handle) === 'VIDEO')) {
          return function () { return ''; };
        }
        // HTMLAnchorElement/HTMLAreaElement URL 分解 IDL 属性（href/pathname/search/hash/host/hostname/port/
        // protocol/origin/username/password，R2838）——经 `__zw_parse_url`（R2778 url crate）解析 href 属性
        // （base = 页面 location.href）取组件。`a.href` getter 返**绝对** URL（区别 getAttribute('href') 返
        // 原始串——jQuery .prop('href') vs .attr('href')）；其余组件返解析值；无 href / 未注册回调 / 解析失败
        // → 空值（href getter 回落原始串）。SPA 路由（读 a.pathname/a.search）/链接分析/analytics 高频。
        // **已知限制**：仅 getter；组件 setter（a.pathname='/x'）经 set-trap catch-all 误设 spurious 属性
        // （罕见，defer——a.href setter 经 catch-all 正确设 href 属性）。
        if ((_realTag(sel, handle) === 'A' || _realTag(sel, handle) === 'AREA') &&
            (prop === 'href' || prop === 'pathname' || prop === 'search' || prop === 'hash' ||
             prop === 'host' || prop === 'hostname' || prop === 'port' || prop === 'protocol' ||
             prop === 'origin' || prop === 'username' || prop === 'password')) {
          var aRaw = handle ? __zw_get_attr_handle(handle, 'href') : __zw_get_attr(sel, 'href');
          if (!aRaw) return '';
          if (typeof __zw_parse_url !== 'function') return prop === 'href' ? aRaw : '';
          try {
            var aBase = globalThis.location ? globalThis.location.href : '';
            var aJson = __zw_parse_url(aRaw, aBase);
            if (!aJson) return prop === 'href' ? aRaw : '';
            var aVal = JSON.parse(aJson)[prop];
            return aVal == null ? '' : aVal;
          } catch (_e) { return prop === 'href' ? aRaw : ''; }
        }
        // HTMLFormElement 反射 IDL 属性（action/method/enctype/target，R2839）——form 序列化 / AJAX 提交库
        // （jQuery/Axios form 插件）读 form.action/form.method 构提交请求高频。反射同名内容属性；
        // **method/enctype 有 spec 默认值 + 小写归一**（method: get/post/dialog，无效或空→'get'；
        // enctype: 三值，无效或空→'application/x-www-form-urlencoded'）。action/target 为纯串反射（无→''）。
        // setter 经 set-trap catch-al（setAttribute）近似工作（method/enctype 不小写归一，罕见 defer）。
        if (_realTag(sel, handle) === 'FORM' &&
            (prop === 'action' || prop === 'method' || prop === 'enctype' || prop === 'target')) {
          var fv = handle ? __zw_get_attr_handle(handle, prop) : __zw_get_attr(sel, prop);
          fv = fv || '';
          if (prop === 'method') {
            fv = fv.toLowerCase();
            if (fv !== 'get' && fv !== 'post' && fv !== 'dialog') fv = 'get';
          } else if (prop === 'enctype') {
            fv = fv.toLowerCase();
            if (fv !== 'application/x-www-form-urlencoded' && fv !== 'multipart/form-data' && fv !== 'text/plain') {
              fv = 'application/x-www-form-urlencoded';
            }
          }
          return fv;
        }
        // `label.htmlFor`（HTMLLabelElement，R2840）——反射 `for` 属性（label↔control 关联，表单库读）。
        if (prop === 'htmlFor' && _realTag(sel, handle) === 'LABEL') {
          return (handle ? __zw_get_attr_handle(handle, 'for') : __zw_get_attr(sel, 'for')) || '';
        }
        // `input.defaultValue`（HTMLInputElement，R2840）——反射**初始** `value` 属性（区别 `.value` 当前态；
        // form reset 逻辑 / 校验库读 defaultValue 判「值是否改过」）。
        if (prop === 'defaultValue' && _realTag(sel, handle) === 'INPUT') {
          return (handle ? __zw_get_attr_handle(handle, 'value') : __zw_get_attr(sel, 'value')) || '';
        }
        // `input.defaultChecked`（HTMLInputElement，R2840）——反射 `checked` 属性存在性（初始选中态，区别
        // `.checked` 当前态；复选框 reset 逻辑读）。sel 经 `__zw_has_attr`，handle 经 `__zw_has_attr_handle`。
        if (prop === 'defaultChecked' && _realTag(sel, handle) === 'INPUT') {
          if (handle && typeof __zw_has_attr_handle === 'function') {
            try { return __zw_has_attr_handle(handle, 'checked') === '1'; } catch (_e) {}
          }
          if (!handle && sel && typeof __zw_has_attr === 'function') {
            try { return __zw_has_attr(sel, 'checked') === '1'; } catch (_e) {}
          }
          return false;
        }
        // `.form`（form-associated 控件 INPUT/SELECT/TEXTAREA/BUTTON，R2841）——返所属 <form> 元素
        // （form owner）。form 校验 / 序列化库读 input.form 找 owner form 上下文高频。**spec 顺序**：
        // ① `form` 属性关联优先（`<input form="id">` → getElementById(id)，即使无 ancestor form）；
        // ② 否则最近 ancestor <form>（经 `_ancestorChain` 上行）。handle-only detached / 无 owner → null。
        if (prop === 'form') {
          var fcTag = _realTag(sel, handle);
          if (fcTag === 'INPUT' || fcTag === 'SELECT' || fcTag === 'TEXTAREA' || fcTag === 'BUTTON') {
            try {
              var formAttr = handle ? __zw_get_attr_handle(handle, 'form') : (sel ? __zw_get_attr(sel, 'form') : '');
              if (formAttr && globalThis.document && globalThis.document.getElementById) {
                var byId = globalThis.document.getElementById(formAttr);
                if (byId) return byId;
              }
              if (sel) {
                var fchain = _ancestorChain(sel);
                for (var fi = 1; fi < fchain.length; fi++) {
                  if ((__zw_get_tag(fchain[fi]) || '').toUpperCase() === 'FORM') return _wrapSelector(fchain[fi]);
                }
              }
            } catch (_e) {}
            return null;
          }
        }
        // `<tr>.rowIndex`（HTMLTableRowElement，R2842）——行在 table 中的位置（0-based，跨 thead/tbody/tfoot
        // 全部行，document order）；-1 若不在 table。data-table / 表格操作库读 rowIndex 定位行高频。
        // 经 _ancestorChain 找 owning TABLE + 元素作用域 querySelectorAll('tr')（R2673）+ proxy identity 计位。
        if (prop === 'rowIndex' && _realTag(sel, handle) === 'TR') {
          if (!sel) return -1;
          try {
            var riChain = _ancestorChain(sel);
            var riTable = null;
            for (var ri = 1; ri < riChain.length; ri++) {
              if ((__zw_get_tag(riChain[ri]) || '').toUpperCase() === 'TABLE') { riTable = riChain[ri]; break; }
            }
            if (!riTable) return -1;
            var riRows = _wrapSelector(riTable).querySelectorAll('tr');
            var riSelf = _wrapSelector(sel);
            for (var rk = 0; rk < riRows.length; rk++) if (riRows[rk] === riSelf) return rk;
            return -1;
          } catch (_e) { return -1; }
        }
        // `<td>`/`<th>`.cellIndex（HTMLTableCellElement，R2842）——单元格在行中的位置（0-based，td+th 混计
        // document order）；-1 若不在行。表格操作库读 cellIndex 定位列高频。经 :is(td, th) 单查询保序
        // （querySelectorAll 顶层不支持逗号列表，:is() 内部支持）。
        if (prop === 'cellIndex' && (_realTag(sel, handle) === 'TD' || _realTag(sel, handle) === 'TH')) {
          if (!sel) return -1;
          try {
            var ciChain = _ancestorChain(sel);
            var ciTr = null;
            for (var ci = 1; ci < ciChain.length; ci++) {
              if ((__zw_get_tag(ciChain[ci]) || '').toUpperCase() === 'TR') { ciTr = ciChain[ci]; break; }
            }
            if (!ciTr) return -1;
            var ciCells = _wrapSelector(ciTr).querySelectorAll(':is(td, th)');
            var ciSelf = _wrapSelector(sel);
            for (var ck = 0; ck < ciCells.length; ck++) if (ciCells[ck] === ciSelf) return ck;
            return -1;
          } catch (_e) { return -1; }
        }
        // `<tr>`.sectionRowIndex（HTMLTableRowElement，R2843）——行在其 section（thead/tbody/tfoot）内的位置
        //（0-based）；-1 若无 section（html5ever 为 table-直属 tr 插入隐式 tbody，故通常有 section）。
        // 同 rowIndex 模式：_ancestorChain 找最近 thead/tbody/tfoot → 元素作用域 querySelectorAll('tr') + identity。
        if (prop === 'sectionRowIndex' && _realTag(sel, handle) === 'TR') {
          if (!sel) return -1;
          try {
            var srChain = _ancestorChain(sel);
            var srSection = null;
            for (var si = 1; si < srChain.length; si++) {
              var stag = (__zw_get_tag(srChain[si]) || '').toUpperCase();
              if (stag === 'THEAD' || stag === 'TBODY' || stag === 'TFOOT') { srSection = srChain[si]; break; }
            }
            if (!srSection) return -1;
            var srRows = _wrapSelector(srSection).querySelectorAll('tr');
            var srSelf = _wrapSelector(sel);
            for (var ssk = 0; ssk < srRows.length; ssk++) if (srRows[ssk] === srSelf) return ssk;
            return -1;
          } catch (_e) { return -1; }
        }
        // `<option>`.index（HTMLOptionElement，R2849）——option 在其 select 中的位置（0-based，document order）；
        // 0 若不在 select（detached / handle-based，与 Chromium detached→0 一致）。form 库读 option.index 定位高频。
        // 同 R2842 rowIndex 模式：_ancestorChain 找 owning SELECT + 元素作用域 querySelectorAll('option') + identity。
        if (prop === 'index' && _realTag(sel, handle) === 'OPTION') {
          if (!sel) return 0;
          try {
            var oiChain = _ancestorChain(sel);
            var oiSelect = null;
            for (var oi = 1; oi < oiChain.length; oi++) {
              if ((__zw_get_tag(oiChain[oi]) || '').toUpperCase() === 'SELECT') { oiSelect = oiChain[oi]; break; }
            }
            if (!oiSelect) return 0;
            var oiOpts = _wrapSelector(oiSelect).querySelectorAll('option');
            var oiSelf = _wrapSelector(sel);
            for (var ok = 0; ok < oiOpts.length; ok++) if (oiOpts[ok] === oiSelf) return ok;
            return 0;
          } catch (_e) { return 0; }
        }
        // `<table>`.rows（HTMLTableElement，R2843）/ section.rows（HTMLTableSectionElement，R2845）——
        // table 内全部行（跨 thead/tbody/tfoot document order）/ section（thead/tbody/tfoot）作用域内行。
        // 元素作用域 querySelectorAll('tr') 返真数组（length/索引/迭代/Array 方法）。gate = TABLE 或
        // THEAD/TBODY/TFOOT（section-scoped）；textarea.rows 落 set-trap catch-al 反射不冲突（textarea 非 section）。
        if (prop === 'rows') {
          var rTag = _realTag(sel, handle);
          if (rTag === 'TABLE' || rTag === 'THEAD' || rTag === 'TBODY' || rTag === 'TFOOT') {
            if (!sel) return [];
            try { return _wrapSelector(sel).querySelectorAll('tr'); } catch (_e) { return []; }
          }
        }
        if (prop === 'tBodies' && _realTag(sel, handle) === 'TABLE') {
          if (!sel) return [];
          try { return _wrapSelector(sel).querySelectorAll('tbody'); } catch (_e) { return []; }
        }
        // `<table>`.caption / `<table>`.tHead / `<table>`.tFoot（HTMLTableElement，R2845）——table 的首个
        // caption / thead / tfoot 子元素（Chromium 150 oracle：querySelector 首匹配；无 → null）。表格分析 /
        // 序列化库读结构高频。仅 getter（setter 须 remove 既有 + insert 新建属 table 头部位置，复杂且罕见——
        // 落 catch-al 反射内容属性，documented 限制）。gate 仅 TABLE。
        if ((prop === 'caption' || prop === 'tHead' || prop === 'tFoot') && _realTag(sel, handle) === 'TABLE') {
          if (!sel) return null;
          var cTag = prop === 'tHead' ? 'thead' : (prop === 'tFoot' ? 'tfoot' : 'caption');
          try { return _wrapSelector(sel).querySelector(cTag); } catch (_e) { return null; }
        }
        // HTMLOptionElement 读属性（option.text/label/defaultSelected，R2832），仅 OPTION（_realTag gate，
        // 支持 sel + handle 两种身份——new Option 创建的 handle-based 亦可读）。
        if (prop === 'text' && _realTag(sel, handle) === 'OPTION') {
          // text = 显示文本（= textContent）。
          return handle ? __zw_get_text_handle(handle) : __zw_get_text(sel);
        }
        if (prop === 'label' && _realTag(sel, handle) === 'OPTION') {
          // label 属性；缺省回落 text。
          var lab = handle ? __zw_get_attr_handle(handle, 'label') : __zw_get_attr(sel, 'label');
          return lab || (handle ? __zw_get_text_handle(handle) : __zw_get_text(sel)) || '';
        }
        if (prop === 'defaultSelected' && _realTag(sel, handle) === 'OPTION') {
          // defaultSelected = 'selected' 属性存在性（boolean reflected）。sel-based 经 `__zw_has_attr`；
          // handle-based（`new Option()` 创建）经 `__zw_has_attr_handle`（句柄不在快照）。
          if (handle && typeof __zw_has_attr_handle === 'function') {
            try { return __zw_has_attr_handle(handle, 'selected') === '1'; } catch (_e) {}
          }
          if (!handle && sel && typeof __zw_has_attr === 'function') {
            try { return __zw_has_attr(sel, 'selected') === '1'; } catch (_e) {}
          }
          return false;
        }
        // HTMLOutputElement.defaultValue（R2846）：初始文本内容（lazy 捕获一次，跨 value 变更保持稳定——
        // Chromium 150 oracle：value=99 后 defaultValue 仍=初值）。output.value getter/setter 见上方 value 块 +
        // set-trap。表单计算器 `<output>` 显示结果高频。仅 OUTPUT；htmlFor 为 DOMSettableTokenList（复杂罕见，defer）。
        if (prop === 'defaultValue' && _realTag(sel, handle) === 'OUTPUT') {
          if (_outputDefault[key] == null) {
            _outputDefault[key] = handle ? (__zw_get_text_handle(handle) || '') : (__zw_get_text(sel) || '');
          }
          return _outputDefault[key];
        }
        if (prop === 'style') {
          return _styleProxy(sel, handle);
        }
        if (prop === 'classList') return _classListProxy(sel, handle);
        if (prop === 'className') {
          return _readClass(key, sel, handle);
        }
        if (prop === 'id') {
          return handle ? __zw_get_attr_handle(handle, 'id') : __zw_get_attr(sel, 'id');
        }
        // reflected 字符串属性（title/lang/dir）——get 反射同名 attribute（无 → ''）；同步 set→get 优先读
        // _reflectedAttrs 缓存（__zw_set_attr 异步入队，无缓存则 set 后 get 读 stale 快照）。
        if (prop === 'title' || prop === 'lang' || prop === 'dir') {
          var rc = _reflectedAttrs[key];
          if (rc && Object.prototype.hasOwnProperty.call(rc, prop)) return rc[prop];
          return (handle ? __zw_get_attr_handle(handle, prop) : __zw_get_attr(sel, prop)) || '';
        }
        // `el.tabIndex`——反射 tabindex 属性为数值；无属性 → -1（spec：非 tab 序元素默认 -1；
        // natively focusable 默认 0 简化为 -1，常见用法足）。同步 set→get 优先读缓存。
        if (prop === 'tabIndex') {
          var rtc = _reflectedAttrs[key];
          if (rtc && Object.prototype.hasOwnProperty.call(rtc, 'tabindex')) return rtc['tabindex'];
          var tiraw = handle ? __zw_get_attr_handle(handle, 'tabindex') : __zw_get_attr(sel, 'tabindex');
          var tin = parseInt(tiraw, 10);
          return isNaN(tin) ? -1 : tin;
        }
        // `el.contentEditable`——反射 contenteditable 属性（无 → 'inherit'，spec）；同步 set→get 优先读缓存。
        if (prop === 'contentEditable') {
          var cec = _reflectedAttrs[key];
          if (cec && Object.prototype.hasOwnProperty.call(cec, 'contenteditable')) return cec['contenteditable'];
          return (handle ? __zw_get_attr_handle(handle, 'contenteditable') : __zw_get_attr(sel, 'contenteditable')) || 'inherit';
        }
        // `el.isContentEditable`——计算 bool（contentEditable === 'true'）。**简化**：不沿祖先链解析
        // 'inherit'（spec：inherit 时看最近可编辑祖先）——本沙箱无渲染期可编辑态，元素自身 'true' 即 true。
        if (prop === 'isContentEditable') {
          var ced = _reflectedAttrs[key];
          var cval = ced && Object.prototype.hasOwnProperty.call(ced, 'contenteditable')
            ? ced['contenteditable']
            : ((handle ? __zw_get_attr_handle(handle, 'contenteditable') : __zw_get_attr(sel, 'contenteditable')) || 'inherit');
          return cval === 'true';
        }
        // `el.accessKey`——反射 accesskey 属性（无 → ''）；同步 set→get 优先读缓存。
        if (prop === 'accessKey') {
          var akc = _reflectedAttrs[key];
          if (akc && Object.prototype.hasOwnProperty.call(akc, 'accesskey')) return akc['accesskey'];
          return (handle ? __zw_get_attr_handle(handle, 'accesskey') : __zw_get_attr(sel, 'accesskey')) || '';
        }
        // `el.role`——反射 role 属性（无 → ''）；同步 set→get 优先读缓存。
        if (prop === 'role') {
          var rlc = _reflectedAttrs[key];
          if (rlc && Object.prototype.hasOwnProperty.call(rlc, 'role')) return rlc['role'];
          return (handle ? __zw_get_attr_handle(handle, 'role') : __zw_get_attr(sel, 'role')) || '';
        }
        // `el.ariaXxx`——反射 aria-* 属性（ariaLabel↔aria-label, ariaLabelledBy↔aria-labelledby, ...）。
        // 经 `_ariaAttrName` 通用映射覆盖全部 aria IDL 属性；无 → ''。同步 set→get 优先读缓存。
        var _ariaName = _ariaAttrName(prop);
        if (_ariaName) {
          var arc = _reflectedAttrs[key];
          if (arc && Object.prototype.hasOwnProperty.call(arc, _ariaName)) return arc[_ariaName];
          return (handle ? __zw_get_attr_handle(handle, _ariaName) : __zw_get_attr(sel, _ariaName)) || '';
        }
        // reflected 布尔/枚举全局属性（R2848/R2850）：autofocus/draggable/spellcheck/translate（R2848）
        // + inert/autocomplete（R2850）——旧 fallthrough 返 undefined（spec 须布尔/串）。getter 优先读
        // _reflectedAttrs 缓存（setter 写解析值，同步 set→get 即时），无缓存则 host attr 解析。spec 默认：
        // autofocus/draggable/inert=false，spellcheck/translate=true，autocomplete="on"（missing-default）。
        // autofocus/inert 为 boolean attr（presence 判定，has_attr）；autocomplete 为 enumerated 串反射。
        if (prop === 'autofocus' || prop === 'draggable' || prop === 'spellcheck' || prop === 'translate' || prop === 'inert' || prop === 'autocomplete') {
          var rfc = _reflectedAttrs[key];
          if (rfc && Object.prototype.hasOwnProperty.call(rfc, prop)) return rfc[prop];
          if (prop === 'autofocus' || prop === 'inert') {
            // boolean attr：presence（has_attr）→ true；缺省 → false。
            if (handle) {
              try { return __zw_has_attr_handle(handle, prop) === '1'; } catch (_e) { return false; }
            }
            return typeof __zw_has_attr === 'function' && __zw_has_attr(sel, prop) === '1';
          }
          if (prop === 'autocomplete') {
            // enumerated 串反射：attr 值（缺省 → "on"，spec missing-default）。__zw_get_attr 缺省返 "" 故 "" 亦判缺省。
            var acRaw = handle ? __zw_get_attr_handle(handle, 'autocomplete') : __zw_get_attr(sel, 'autocomplete');
            return (acRaw == null || acRaw === '') ? 'on' : String(acRaw);
          }
          var rfRaw = handle ? __zw_get_attr_handle(handle, prop) : __zw_get_attr(sel, prop);
          rfRaw = (rfRaw == null) ? '' : String(rfRaw).toLowerCase();
          if (prop === 'draggable') return rfRaw === 'true';   // "true"→true，余（"false"/""/缺省 auto）→false（简化）
          if (prop === 'spellcheck') return rfRaw !== 'false'; // "false"→false，余（含缺省）→true（spec 默认 true）
          return rfRaw !== 'no';                               // translate："no"→false，余→true（默认 true）
        }
        // reflected unsigned-long 维度属性（R2851）：IMG/IFRAME `.width`/`.height`（反射 width/height 内容属性
        // 为非负整数，缺省/不可解析 → 0；spec「reflect unsigned long」算法）+ IMG `.naturalWidth`/`.naturalHeight`
        // （固有像素尺寸，headless 无真图加载 → 恒 0，spec unloaded→0 一致）。响应式/布局 JS 读 img.width 高频。
        // CANVAS（缺省 300/150 且 setter 改 bitmap，特殊）/ VIDEO/EMBED defer。
        if (prop === 'width' || prop === 'height' || prop === 'naturalWidth' || prop === 'naturalHeight') {
          var rgTag = _realTag(sel, handle);
          if (rgTag === 'IMG' && (prop === 'naturalWidth' || prop === 'naturalHeight')) {
            return 0;  // headless 无真图加载（onload 不触发）→ 固有尺寸 0（spec unloaded→0）。
          }
          if ((rgTag === 'IMG' || rgTag === 'IFRAME') && (prop === 'width' || prop === 'height')) {
            // sync set→get 优先读缓存（setter 写数值）；无缓存则解析 width/height 内容属性（缺省/非负整数失败 → 0）。
            var drc = _reflectedAttrs[key];
            if (drc && Object.prototype.hasOwnProperty.call(drc, prop)) return drc[prop];
            var dRaw = handle ? __zw_get_attr_handle(handle, prop) : __zw_get_attr(sel, prop);
            var dN = parseInt(dRaw, 10);
            return (isNaN(dN) || dN < 0) ? 0 : dN;
          }
        }
        // `el.dataset`——`data-*` 属性的 camelCase 键对象（get/set/has/delete/枚举）。
        // dataset.fooBar ↔ data-foo-bar 属性。handle 脱离 DOM 元素枚举受限（无 attr-names-handle）。
        if (prop === 'dataset') {
          return _datasetProxy(sel, handle);
        }
        if (prop === 'textContent') {
          return handle ? __zw_get_text_handle(handle) : __zw_get_text(sel);
        }
        if (prop === 'innerHTML') {
          return handle ? __zw_get_inner_html_handle(handle) : __zw_get_inner_html(sel);
        }
        // `element.outerHTML`（getter）：含自身 tag/属性 + 子树序列化。仅 sel-based（已挂载）
        // 元素经 host `__zw_get_outer_html` 真实序列化；handle-only（detached）无 tag host 查询，
        // best-effort 返 innerHTML（无 wrapper，罕见读取场景）。
        if (prop === 'outerHTML') {
          if (sel && typeof __zw_get_outer_html === 'function') {
            try { return __zw_get_outer_html(sel); } catch (_e) { return ''; }
          }
          return handle && typeof __zw_get_inner_html_handle === 'function'
            ? (__zw_get_inner_html_handle(handle) || '')
            : '';
        }
        if (prop === 'parentNode' || prop === 'parentElement') {
          return _parentNodeFor(sel, handle);
        }
        // 元素遍历/导航 API（仅元素子/兄弟，跳过文本/注释）。handle（脱离 DOM，无 sel）→ null/[]。
        if (prop === 'children') {
          return sel && typeof __zw_element_children === 'function'
            ? _splitSelectors(__zw_element_children(sel)) : [];
        }
        if (prop === 'firstElementChild' || prop === 'lastElementChild' || prop === 'childElementCount') {
          if (!sel || typeof __zw_element_children !== 'function') {
            return prop === 'childElementCount' ? 0 : null;
          }
          var kids = _splitSelectors(__zw_element_children(sel));
          if (prop === 'childElementCount') return kids.length;
          if (!kids.length) return null;
          return prop === 'firstElementChild' ? kids[0] : kids[kids.length - 1];
        }
        if (prop === 'previousElementSibling' || prop === 'nextElementSibling') {
          if (!sel || typeof __zw_element_siblings !== 'function') return null;
          try {
            var parts = __zw_element_siblings(sel).split('|');
            var hit = prop === 'previousElementSibling' ? parts[0] : parts[1];
            return hit ? _wrapSelector(hit) : null;
          } catch (_e) { return null; }
        }
        // 节点级遍历（含文本/注释，区别于上面的 element-only 版）：childNodes / firstChild /
        // lastChild（子列表，经 __zw_child_nodes JSON）/ previousSibling / nextSibling（兄弟，经
        // __zw_sibling_nodes JSON）。文本/注释节点返静态对象（_wrapNodeEntry）。仅 sel-based 目标。
        if (prop === 'childNodes') {
          return _childNodeList(sel, handle);
        }
        if (prop === 'firstChild' || prop === 'lastChild') {
          var cn = _childNodeList(sel, handle);
          if (!cn.length) return null;
          return prop === 'firstChild' ? cn[0] : cn[cn.length - 1];
        }
        if (prop === 'previousSibling' || prop === 'nextSibling') {
          if (!sel || typeof __zw_sibling_nodes !== 'function') return null;
          try {
            var pair = JSON.parse(__zw_sibling_nodes(sel) || '{"p":null,"n":null}');
            var en = prop === 'previousSibling' ? pair.p : pair.n;
            return _wrapNodeEntry(en, _parentNodeFor(sel, handle));
          } catch (_e) { return null; }
        }
        // `el.contains(other)`——other 是否为 el 的后代或 el 自身（沿 parent 链）。
        if (prop === 'contains') {
          return function(other) {
            if (!sel || typeof __zw_contains !== 'function') return false;
            var otherSel = other && other.__zwSelector;
            if (!otherSel) return false;
            try { return __zw_contains(sel, otherSel) === '1'; } catch (_e) { return false; }
          };
        }
        // `el.getRootNode()`——沿 parent 链到根（通常 html），返根 proxy。sel 缺失 → 返自身。
        if (prop === 'getRootNode') {
          return function() {
            if (!sel) return _makeProxy(sel, handle);
            var chain = _ancestorChain(sel);
            return _wrapSelector(chain.length ? chain[chain.length - 1] : sel);
          };
        }
        // `el.isConnected`（只读 boolean，spec Node.isConnected：节点是否连入 document）——框架 / 库
        // 高频判活（jQuery cleanData、React commit-phase、mutation handler `if (!node.isConnected) return`；
        // 缺失则恒 undefined（falsy）→ 在档元素被误判 detached，脚本取错分支）。
        // ① sel-based（parsed 元素 / querySelector·getElementById 结果 / html·body·head）→ 经 `__zw_contains
        //   ('html', sel)`（element_contains 自含，html 自身亦命中）判定是否在 documentElement 子树内——亦
        //   正确反映 `el.remove()` / `removeChild` 后的 detach（selector 不再在档 → 返 '0'）；无 `__zw_contains`
        //   回调路径 → fallback true（sel-based parsed 元素构造即在树内）。
        // ② handle-only（createElement / createTextNode / createComment / DocumentFragment，未挂载）→ false；
        // ③ 已 appendChild 的 handle 元素 best-effort：`__zw_getBoundingClientRect(handle)` 非空 ⇒ 已在布局
        //   树（= 已连入文档）→ true（复用 R2661 handle→NodeId 解析）。**已知限制**：append 后未跑 layout
        //   的同一 execute 内可能暂报 false（layout-dependent probe）；text/comment 节点无布局 rect → append
        //   后仍报 false（少见 `textNode.isConnected` 检查，documented）。Document 节点恒 connected（见 literal）。
        if (prop === 'isConnected') {
          if (sel) {
            if (typeof __zw_contains === 'function') {
              try { return __zw_contains('html', sel) === '1'; } catch (_e) { return true; }
            }
            return true;
          }
          if (handle && typeof __zw_getBoundingClientRect === 'function') {
            try { return __zw_getBoundingClientRect(handle) !== ''; } catch (_e) { return false; }
          }
          return false;
        }
        // `el.hasChildNodes()`（spec Node.hasChildNodes：是否有任意子节点含文本/注释）——树遍历 / diff /
        // 子节点存在性检查高频。经既有 `_childNodeList`（元素查 `__zw_child_nodes`；handle-only 返 []）取
        // length>0。text/comment 节点本身无子（spec）；DocumentFragment 子节点经 host flatten 跟踪，
        // handle-only _childNodeList 暂返 [] → 报 false（detached fragment 检查少见，documented）。
        if (prop === 'hasChildNodes') {
          return function() { return _childNodeList(sel, handle).length > 0; };
        }
        // `el.isSameNode(other)`——节点身份相等（deprecated，等价 ===；proxy 缓存使同节点同 proxy，
        // 但经 _elKey 比较更鲁棒：sel/handle 一致即同节点）。
        if (prop === 'isSameNode') {
          return function(other) {
            if (!other) return false;
            var otherSel = other.__zwSelector || '';
            var otherHandle = other.__zwHandle || null;
            return _elKey(sel, handle) === _elKey(otherSel, otherHandle);
          };
        }
        // `el.isEqualNode(other)`——节点结构相等（node-equality 三件套：isSameNode 身份 / compareDocumentPosition
        // 位置 / isEqualNode 结构）。testing/diff 库高频。经 `_nodeSig` 序列化签名比对（元素 outerHTML / text·comment
        // nodeValue）。**已知限制**：属性序敏感（spec 序无关）；handle/detached 元素 outerHTML 仅 innerHTML 回落。
        if (prop === 'isEqualNode') {
          return function(other) {
            if (!other || typeof other !== 'object') return false;
            var oSel = other.__zwSelector || '';
            var oHandle = other.__zwHandle || null;
            return _nodeSig(sel, handle) === _nodeSig(oSel, oHandle);
          };
        }
        // `el.compareDocumentPosition(other)`——bitmask 描述 other 相对 el 的文档位置（树算法 / 库排序高频）。
        // 经 `_ancestorChain`（self/other→root）+ LCA + `__zw_element_children` 子序比较。已知限制：仅 sel-based
        // element（text/comment 节点无 sel → DISCONNECTED 兜底）；不同树 → DISCONNECTED|IMPL。
        if (prop === 'compareDocumentPosition') {
          return function(other) {
            var FOLLOWING = 4, PRECEDING = 2, CONTAINS = 8, CONTAINED_BY = 16, DISCONNECTED = 1, IMPL = 32;
            var otherSel = other && other.__zwSelector;
            if (!sel || !otherSel) return DISCONNECTED | IMPL;
            if (sel === otherSel) return 0;
            var aChain = _ancestorChain(sel);
            var bChain = _ancestorChain(otherSel);
            if (!aChain.length || !bChain.length) return DISCONNECTED | IMPL;
            if (aChain[aChain.length - 1] !== bChain[bChain.length - 1]) return DISCONNECTED | IMPL;
            // other 是 this 的祖先 → other contains this + other precedes this（doc 序）。
            if (aChain.indexOf(otherSel) >= 0) return CONTAINS | PRECEDING;
            // this 是 other 的祖先 → this contains other + other follows this。
            if (bChain.indexOf(sel) >= 0) return CONTAINED_BY | FOLLOWING;
            // 共同祖先非直系：root→node 反转链找 LCA；扫描 LCA element children 的**原始 selector 串**
            //（_splitSelectors 会包成 proxy，故直接 split '|'），经 `__zw_contains`（节点包含，selector-format
            // 无关）定位含 this / other 的子，序比较。
            var ra = aChain.slice().reverse(), rb = bChain.slice().reverse();
            var i = 0;
            while (i < ra.length && i < rb.length && ra[i] === rb[i]) i++;
            var lca = ra[i - 1];
            if (lca && typeof __zw_element_children === 'function' && typeof __zw_contains === 'function') {
              try {
                var kids = String(__zw_element_children(lca) || '').split('|').filter(Boolean);
                var ti = -1, oi = -1;
                for (var k = 0; k < kids.length && (ti < 0 || oi < 0); k++) {
                  if (ti < 0 && __zw_contains(kids[k], sel) === '1') ti = k;
                  if (oi < 0 && __zw_contains(kids[k], otherSel) === '1') oi = k;
                }
                if (ti >= 0 && oi >= 0) return ti < oi ? FOLLOWING : PRECEDING;
              } catch (_e) {}
            }
            return FOLLOWING; // 兜底
          };
        }
        // DocumentFragment handle（nodeType 11 / '#document-fragment'）/ Comment（nodeType 8 / '#comment'）/
        // Text（nodeType 3 / '#text'）——均为 create 句柄无 selector，经 handle set 区别于普通元素句柄。
        var isFrag = handle && _fragmentHandles[handle];
        var isComment = handle && _commentHandles[handle];
        var isText = handle && _textHandles[handle];
        if (prop === 'tagName') {
          return (isFrag || isComment || isText) ? undefined : _realTag(sel, handle);
        }
        if (prop === 'nodeName') {
          return isFrag ? '#document-fragment'
            : isComment ? '#comment'
            : isText ? '#text'
            : _realTag(sel, handle);
        }
        if (prop === 'nodeType') {
          return isFrag ? 11 : (isComment ? 8 : (isText ? 3 : 1));
        }
        // Text/Comment 节点的 nodeValue/data = 文本（经 __zw_get_text_handle 读回，element 的 nodeValue 为 null）。
        if ((isText || isComment) && (prop === 'nodeValue' || prop === 'data')) {
          return handle ? __zw_get_text_handle(handle) : '';
        }
        // CharacterData 数据编辑方法（R2823，text/comment 节点）+ Text.splitText。仅 handle-based
        // 文本/注释节点（createTextNode/createComment 所建——parsed DOM 文本节点为 _wrapNodeEntry 静态
        // 快照无 handle）。读经 __zw_get_text_handle（query_text_from_mutations 反向 replay 取最新值，
        // 故多次编辑 compose 正确），写经 __zw_set_text_handle（追加 SetTextOnHandle mutation）。offset
        // 越界 clamp（spec 抛 IndexSizeError，此处 permissive 不抛）。contentEditable 编辑库（ProseMirror
        // / Slate / Quill）+ Range/Selection 高频。
        if ((isText || isComment) && prop === 'length') {
          return handle ? __zw_get_text_handle(handle).length : 0;
        }
        if ((isText || isComment) && prop === 'appendData') {
          return function (s) {
            if (handle) __zw_set_text_handle(handle, __zw_get_text_handle(handle) + String(s == null ? '' : s));
            return undefined;
          };
        }
        if ((isText || isComment) && prop === 'deleteData') {
          return function (offset, count) {
            if (!handle) return undefined;
            var cur = __zw_get_text_handle(handle);
            var o = offset | 0, c = count | 0;
            if (o < 0) o = 0;
            if (c < 0) c = 0;
            __zw_set_text_handle(handle, cur.slice(0, o) + cur.slice(o + c));
            return undefined;
          };
        }
        if ((isText || isComment) && prop === 'insertData') {
          return function (offset, s) {
            if (!handle) return undefined;
            var cur = __zw_get_text_handle(handle);
            var o = offset | 0;
            if (o < 0) o = 0;
            __zw_set_text_handle(handle, cur.slice(0, o) + String(s == null ? '' : s) + cur.slice(o));
            return undefined;
          };
        }
        if ((isText || isComment) && prop === 'replaceData') {
          return function (offset, count, s) {
            if (!handle) return undefined;
            var cur = __zw_get_text_handle(handle);
            var o = offset | 0, c = count | 0;
            if (o < 0) o = 0;
            if (c < 0) c = 0;
            __zw_set_text_handle(handle, cur.slice(0, o) + String(s == null ? '' : s) + cur.slice(o + c));
            return undefined;
          };
        }
        if ((isText || isComment) && prop === 'substringData') {
          return function (offset, count) {
            if (!handle) return '';
            var cur = __zw_get_text_handle(handle);
            var o = offset | 0, c = count | 0;
            if (o < 0) o = 0;
            if (c < 0) c = 0;
            return cur.slice(o, o + c);
          };
        }
        // Text.splitText(offset)——在 offset 拆分：原节点保 [0,offset)，返新 text 节点含 [offset,)。
        // 仅 text（comment 无 splitText）。offset clamp 到 [0,length]；新节点经 createTextNode 建（handle-based，可后续编辑）。
        if (isText && prop === 'splitText') {
          return function (offset) {
            var cur = handle ? __zw_get_text_handle(handle) : '';
            var o = offset | 0;
            if (o < 0) o = 0;
            if (o > cur.length) o = cur.length;
            var tail = cur.slice(o);
            if (handle) __zw_set_text_handle(handle, cur.slice(0, o));
            return globalThis.document.createTextNode(tail);
          };
        }
        if (prop === 'ownerDocument') {
          return globalThis.document;
        }
        if (prop === 'getAttribute') {
          return function(name) {
            return handle ? __zw_get_attr_handle(handle, name) : __zw_get_attr(sel, name);
          };
        }
        if (prop === 'setAttribute') {
          return function(name, value) {
            var n = String(name);
            var v = String(value);
            // 同步客户端缓存：class→_classCache、value→_inputValues，使 setAttribute 与
            // classList/className、.value getter 协作一致（否则后续 classList.add 读 stale 缓存丢值）。
            if (n === 'class') _classCache[key] = v;
            else if (n === 'value') _inputValues[key] = v;
            if (handle) __zw_set_attr_handle(handle, n, v);
            else __zw_set_attr(sel, n, v);
            _mo_notify(sel, handle, { type: 'attributes', attributeName: n });
          };
        }
        if (prop === 'removeAttribute') {
          return function(name) {
            var n = String(name);
            // sel-based：真移除（__zw_remove_attr / RemoveAttr，R2657）——区别于 set-empty 残留
            // `attr=""`（boolean 属性 checked/disabled 设空值仍 present → hasAttribute 误 true）。
            // handle-only（无 remove-handle 变体）/ 无回调 → fallback set-empty。
            // 同步客户端缓存（class/value），使后续 classList/.value 反映移除。
            if (n === 'class') _classCache[key] = '';
            else if (n === 'value') _inputValues[key] = '';
            if (handle) __zw_set_attr_handle(handle, n, '');
            else if (typeof __zw_remove_attr === 'function') __zw_remove_attr(sel, n);
            else __zw_set_attr(sel, n, '');
            _mo_notify(sel, handle, { type: 'attributes', attributeName: n });
          };
        }
        // `el.hasAttribute(name)`——属性存在性（boolean 属性 checked/disabled/hidden、data-* 检查常用）。
        // sel-based 经 host `__zw_has_attr`（"1"/"0"）；handle-only（无 has-attr-handle 变体）→ false。
        if (prop === 'hasAttribute') {
          return function(name) {
            if (!sel || typeof __zw_has_attr !== 'function') return false;
            try { return __zw_has_attr(sel, String(name)) === '1'; } catch (_e) { return false; }
          };
        }
        // `el.focus()` / `el.blur()`——焦点状态追踪（document.activeElement 对）。纯 in-JS 状态：
        // focus 记当前 key，blur 清当前 key。**已知限制**：① 无真键盘焦点（纯状态，无输入焦点点亮）；
        // ② 不派发 focus/blur 事件；③ 不校验可聚焦性（非聚焦元素仍记焦点）；④ 无 tabindex 焦点序。
        if (prop === 'focus') {
          return function() { _activeElKey = key; };
        }
        if (prop === 'blur') {
          return function() { if (_activeElKey === key) _activeElKey = null; };
        }
        // 全屏 / 指针锁 / 滚动（R2817）——headless 无真全屏/指针锁/滚动 → Promise resolve（fullscreen）
        // 或 no-op（pointerLock/scroll*）。feature-detect + modern 交互脚本不抛。
        if (prop === 'requestFullscreen') {
          return function() { return Promise.resolve(undefined); };
        }
        if (prop === 'requestPointerLock') {
          return function() {};
        }
        if (prop === 'scrollIntoView' || prop === 'scrollTo' || prop === 'scrollBy') {
          return function() {};
        }
        // `el.hasAttributes()`——是否有任意属性（经 `__zw_attr_names` 非空判定）。
        if (prop === 'hasAttributes') {
          return function() {
            if (!sel || typeof __zw_attr_names !== 'function') return false;
            try { return __zw_attr_names(sel).length > 0; } catch (_e) { return false; }
          };
        }
        // `el.getAttributeNames()`——属性名数组（经 `__zw_attr_names` "|"-split）。
        if (prop === 'getAttributeNames') {
          return function() {
            if (!sel || typeof __zw_attr_names !== 'function') return [];
            try {
              var n = __zw_attr_names(sel);
              return n ? n.split('|').filter(Boolean) : [];
            } catch (_e) { return []; }
          };
        }
        // `el.toggleAttribute(name, force?)`——切换属性存在性，返切换后是否存在。决策经 host
        // `__zw_toggle_attribute`（DomMutation::ToggleAttribute，apply 时读当前存在性决定），故连续
        // toggle 正确复合（朴素 shim 读 stale snapshot 决定会都 add）。返值用 snapshot presence 近似
        //（单次正确；连续下 mutation 正确、返值 stale，可接受）。
        if (prop === 'toggleAttribute') {
          return function(name, force) {
            var n = String(name);
            var hasForce = force !== undefined;
            var snapHas = (sel && typeof __zw_has_attr === 'function')
              ? (__zw_has_attr(sel, n) === '1')
              : false;
            if (sel && typeof __zw_toggle_attribute === 'function') {
              var fArg = hasForce ? (force ? '1' : '0') : '';
              __zw_toggle_attribute(sel, n, fArg);
              _mo_notify(sel, handle, { type: 'attributes', attributeName: n });
            } else if (handle) {
              // handle-only（无 toggle/has-attr handle 变体）：best-effort client-side。
              var want = hasForce ? !!force : !snapHas;
              if (want) __zw_set_attr_handle(handle, n, '');
            }
            return hasForce ? !!force : !snapHas;
          };
        }
        // `el.attributes`（NamedNodeMap 只读快照）——属性枚举（序列化/属性拷贝常用）。
        if (prop === 'attributes') {
          return _attributesProxy(sel, handle);
        }
        // `el.matches(selector)` / `el.matchesSelector`——元素是否匹配选择器（含组合器，经 host
        // `__zw_matches` 全匹配集判定）。handle（未挂载 DOM 的 createElement）无 sel → false。
        if (prop === 'matches' || prop === 'matchesSelector' || prop === 'webkitMatchesSelector') {
          return function(selector) {
            if (!sel || typeof __zw_matches !== 'function') return false;
            try { return __zw_matches(sel, String(selector)) === '1'; } catch (_e) { return false; }
          };
        }
        // `el.closest(selector)`——自身或最近祖先首个匹配元素（proxy），无匹配 null。经 host
        // `__zw_closest`（parent_node 链 + 全匹配集），返唯一选择器后包 proxy。
        if (prop === 'closest') {
          return function(selector) {
            if (!sel || typeof __zw_closest !== 'function') return null;
            try {
              var hit = __zw_closest(sel, String(selector));
              return hit ? _wrapSelector(hit) : null;
            } catch (_e) { return null; }
          };
        }
        if (prop === 'addEventListener') {
          return function(type, fn, opts) {
            if (!_listenerStore[key]) _listenerStore[key] = {};
            if (!_listenerStore[key][type]) _listenerStore[key][type] = [];
            _listenerStore[key][type].push({ fn: fn, capture: _optCapture(opts), once: _optOnce(opts) });
          };
        }
        if (prop === 'removeEventListener') {
          return function(type, fn, opts) {
            if (!_listenerStore[key] || !_listenerStore[key][type]) return;
            var cap = _optCapture(opts);
            _listenerStore[key][type] = _listenerStore[key][type].filter(function(l) {
              return !(l.fn === fn && l.capture === cap);
            });
          };
        }
        if (prop === 'attachEvent') {
          return function(type, fn) {
            _attachEventForKey(key, type, fn);
          };
        }
        if (prop === 'detachEvent') {
          return function(type, fn) {
            _detachEventForKey(key, type, fn);
          };
        }
        if (prop === 'dispatchEvent') {
          return function(event) {
            return _dispatchWithBubble(key, sel, handle, event);
          };
        }
        if (prop === 'click') {
          return function() {
            var ev = _makeEvent('click', { bubbles: true, cancelable: true });
            return _dispatchWithBubble(key, sel, handle, ev);
          };
        }
        // Constraint Validation API（R2825）——表单校验库（checkValidity gate submit / setCustomValidity
        // 自定义错误 / validity.valid 读 / validationMessage 显示）高频。customError 由 _customValidity 跟踪；
        // 原生约束 headless 不强制（permissive valid）。checkValidity/reportValidity invalid 时派发 'invalid'
        // 事件（cancelable，非 bubble，经 _dispatchWithBubble）。
        if (prop === 'checkValidity' || prop === 'reportValidity') {
          return function() {
            var v = _validityState(key);
            if (!v.valid) {
              _dispatchWithBubble(key, sel, handle, _makeEvent('invalid', { cancelable: true, bubbles: false }));
            }
            return v.valid;
          };
        }
        if (prop === 'setCustomValidity') {
          return function(msg) {
            _customValidity[key] = (msg == null) ? '' : String(msg);
            return undefined;
          };
        }
        if (prop === 'validity') return _validityState(key);
        if (prop === 'validationMessage') return _customValidity[key] != null ? _customValidity[key] : '';
        if (prop === 'willValidate') return true;
        // `el.select()`（HTMLInputElement/TextArea，R2826/R2844）——选中文本（legacy copy 模式
        // `el.select(); document.execCommand('copy')` 配对，及自动全选场景）。headless 无真文本选择渲染，
        // 但 text control（R2844）须更新 _textSelection 使后续 selectionStart/End 反映全选（Chromium 150
        // oracle：select()→{0, value.length, 'forward'}）；非 text control 仍 no-op（无选区概念）。
        if (prop === 'select') {
          return function() {
            if (_isTextControl(sel, handle)) {
              var so = _selObj(key);
              so.start = 0;
              so.end = _controlValue(sel, handle, key).length;
              so.direction = 'forward';
            }
            return undefined;
          };
        }
        // `el.animate(keyframes, options)`（Web Animations API，R2827）——modern 动画库（Framer Motion /
        // GSAP / Lottie）feature-detect + 链式。headless 无真时间轴 → `_makeAnimation` permissive stub
        //（瞬间完成：playState 'running'→'finished' + finished Promise + onfinish）。关键帧不真应用（documented）。
        if (prop === 'animate') {
          return function (_keyframes, options) { return _makeAnimation(options); };
        }
        // `el.cloneNode(deep)`——克隆元素（返新 handle proxy，detached）。复用既有回调组合：
        // create(tag) + 逐属性 set_attr_handle + (deep) set_inner_html_handle。sel-based 源完整；
        // handle 源 tag/attrs 受限（无 get_tag/attr_names handle 变体，best-effort）。
        // `Node.normalize()`（R2853）——合并相邻 Text 子节点 + 移除空 Text。snapshot 模型下元素文本为
        // 单一串（无独立 Text 子节点暴露），故 normalize 为语义正确的 no-op（DOM 态已「normalized」）。
        // 提供 no-op 防 `el.normalize()` 防御性调用（rich-text 编辑器 / innerHTML 后清理）抛 TypeError。
        if (prop === 'normalize') {
          return function() {};
        }
        if (prop === 'cloneNode') {
          return function(deep) {
            var srcTag = 'div';
            if (sel && typeof __zw_get_tag === 'function') {
              try { var t = __zw_get_tag(sel); if (t) srcTag = t; } catch (_e) {}
            }
            var nh = __zw_create_element(srcTag);
            // 复制属性（仅 sel-based 有 attr_names 枚举）。
            if (sel && typeof __zw_attr_names === 'function') {
              try {
                var names = __zw_attr_names(sel);
                if (names) {
                  names.split('|').filter(Boolean).forEach(function(n) {
                    __zw_set_attr_handle(nh, n, __zw_get_attr(sel, n) || '');
                  });
                }
              } catch (_e) {}
            }
            // deep：复制后代（innerHTML）。
            if (deep) {
              try {
                var ih = handle
                  ? __zw_get_inner_html_handle(handle)
                  : (sel ? __zw_get_inner_html(sel) : null);
                if (ih) __zw_set_inner_html_handle(nh, ih);
              } catch (_e) {}
            }
            return _wrapHandle(nh);
          };
        }
        if (prop === 'appendChild') {
          return function(child) {
            if (child && child.__zwHandle) {
              // DocumentFragment：flatten 子节点到 this（fragment 自身不入树），区别于 append 节点自身。
              if (_fragmentHandles[child.__zwHandle] && typeof __zw_append_fragment_children === 'function') {
                if (handle) __zw_append_fragment_children_handle(handle, child.__zwHandle);
                else __zw_append_fragment_children(sel, child.__zwHandle);
              } else if (handle) {
                __zw_append_child_handle(handle, child.__zwHandle);
              } else {
                __zw_append_child(sel, child.__zwHandle);
              }
              _mo_notify(sel, handle, { type: 'childList', addedNodes: [child], removedNodes: [] });
            }
            return child;
          };
        }
        if (prop === 'removeChild') {
          return function(child) {
            if (child && child.__zwHandle) {
              __zw_remove_handle(child.__zwHandle);
              _mo_notify(sel, handle, { type: 'childList', addedNodes: [], removedNodes: [child] });
            }
            return child;
          };
        }
        if (prop === 'insertBefore') {
          return function(newNode, refNode) {
            if (newNode && newNode.__zwHandle) {
              // DocumentFragment：flatten 子节点（refNode 非 null 时插到 ref 前，null 时 append）。
              if (_fragmentHandles[newNode.__zwHandle]) {
                if (refNode == null) {
                  if (handle && typeof __zw_append_fragment_children_handle === 'function')
                    __zw_append_fragment_children_handle(handle, newNode.__zwHandle);
                  else if (typeof __zw_append_fragment_children === 'function')
                    __zw_append_fragment_children(sel, newNode.__zwHandle);
                } else if (refNode.__zwSelector) {
                  if (handle && typeof __zw_insert_fragment_before_handle === 'function')
                    __zw_insert_fragment_before_handle(handle, newNode.__zwHandle, refNode.__zwSelector);
                  else if (typeof __zw_insert_fragment_before === 'function')
                    __zw_insert_fragment_before(sel, newNode.__zwHandle, refNode.__zwSelector);
                }
              } else if (refNode == null) {
                // `insertBefore(node, null)` 等价于 appendChild。
                if (handle) __zw_append_child_handle(handle, newNode.__zwHandle);
                else __zw_append_child(sel, newNode.__zwHandle);
              } else if (refNode.__zwSelector) {
                if (handle) __zw_insert_before_handle(handle, newNode.__zwHandle, refNode.__zwSelector);
                else __zw_insert_before(sel, newNode.__zwHandle, refNode.__zwSelector);
              }
              // refNode 为 create 句柄（无 selector）时不支持（罕见）。
              _mo_notify(sel, handle, { type: 'childList', addedNodes: [newNode], removedNodes: [] });
            }
            return newNode;
          };
        }
        // `parent.replaceChild(newChild, oldChild)`：在 oldChild 位置前插入 newChild，再移除
        // oldChild（spec replace 语义）。newChild 须为 create 句柄节点；oldChild 须有 selector
        //（selector-identity 子节点，作 insert ref）。返回 oldChild（spec）。
        if (prop === 'replaceChild') {
          return function(newChild, oldChild) {
            if (newChild && newChild.__zwHandle && oldChild && oldChild.__zwSelector) {
              // DocumentFragment：flatten 子到 old 前（非插 fragment 节点本身），再移除 old。
              if (_fragmentHandles[newChild.__zwHandle]) {
                if (handle && typeof __zw_insert_fragment_before_handle === 'function')
                  __zw_insert_fragment_before_handle(handle, newChild.__zwHandle, oldChild.__zwSelector);
                else if (typeof __zw_insert_fragment_before === 'function')
                  __zw_insert_fragment_before(sel, newChild.__zwHandle, oldChild.__zwSelector);
              } else if (handle) {
                __zw_insert_before_handle(handle, newChild.__zwHandle, oldChild.__zwSelector);
              } else {
                __zw_insert_before(sel, newChild.__zwHandle, oldChild.__zwSelector);
              }
              __zw_remove(oldChild.__zwSelector);
              _mo_notify(sel, handle, {
                type: 'childList',
                addedNodes: [newChild],
                removedNodes: [oldChild],
              });
            }
            return oldChild;
          };
        }
        if (prop === 'remove') {
          return function() {
            if (handle) __zw_remove_handle(handle);
            else __zw_remove(sel);
          };
        }
        // `element.replaceWith(...nodesOrStrings)`：用新节点序列替换自身（self 级，区别于
        // replaceChild 的 parent 级）。= 先 before(...args) 作前兄弟插入（正序保参数序），再 remove 自身。
        // 复用 _insertAdjacentVariadic（beforebegin 正序）+ remove。仅 sel-based 目标（需 parent）。
        if (prop === 'replaceWith') {
          return function() {
            if (sel) {
              _insertAdjacentVariadic(sel, 'beforebegin', arguments, false);
              if (handle) __zw_remove_handle(handle);
              else __zw_remove(sel);
            }
            return undefined;
          };
        }
        // `Element.append(...nodesOrStrings)`（现代 API，区别于 appendChild）：
        // 追加多个节点/字符串，字符串自动包成 Text 节点。复用既有 appendChild +
        // createTextNode 回调，无需新增 Rust 端 callback。
        if (prop === 'append') {
          return function() {
            var added = _appendVariadic(sel, handle, arguments);
            if (added.length > 0) {
              _mo_notify(sel, handle, { type: 'childList', addedNodes: added, removedNodes: [] });
            }
            return undefined;
          };
        }
        // `element.replaceChildren(...nodesOrStrings)`（现代 API，R2822）：移除全部现有子 + 追加新子
        // （clear-and-populate 原子语义，Vue3/lit/Svelte/手写代码高频）。清空经 set_inner_html('')，
        // 追加复用 _appendVariadic；MO childList 同时上报 removedNodes（旧子快照）+ addedNodes（新子）。
        if (prop === 'replaceChildren') {
          return function() {
            var removed = _childNodeList(sel, handle);
            if (handle && typeof __zw_set_inner_html_handle === 'function') __zw_set_inner_html_handle(handle, '');
            else if (typeof __zw_set_inner_html === 'function') __zw_set_inner_html(sel, '');
            var added = _appendVariadic(sel, handle, arguments);
            if (removed.length > 0 || added.length > 0) {
              _mo_notify(sel, handle, { type: 'childList', addedNodes: added, removedNodes: removed });
            }
            return undefined;
          };
        }
        // `element.prepend(...nodesOrStrings)`（现代 API，区别于 appendChild/append）：插为元素
        // **首子**（保持参数序）。经 insertAdjacent afterbegin + 反序（见 _insertAdjacentVariadic）。
        // 仅 sel-based 目标；handle-only detached 无操作。
        if (prop === 'prepend') {
          return function() {
            _insertAdjacentVariadic(sel, 'afterbegin', arguments, true);
            return undefined;
          };
        }
        // `element.before(...nodesOrStrings)`：插为元素**前兄弟**（保持参数序）。beforebegin 正序。
        if (prop === 'before') {
          return function() {
            _insertAdjacentVariadic(sel, 'beforebegin', arguments, false);
            return undefined;
          };
        }
        // `element.after(...nodesOrStrings)`：插为元素**后兄弟**（保持参数序）。afterend 反序。
        if (prop === 'after') {
          return function() {
            _insertAdjacentVariadic(sel, 'afterend', arguments, true);
            return undefined;
          };
        }
        // `element.insertAdjacentHTML(position, text)`（P1a）：解析 HTML 片段并按 position 插入——
        // `beforeend`（末子）/`afterbegin`（首子）/`beforebegin`（前兄弟）/`afterend`（后兄弟）。
        // 服务端原子完成（fragment parse + copy + parent 遍历，见 DomMutation::InsertAdjacentHtml）。
        // 仅 sel-based（已挂载）元素经 host `__zw_insert_adjacent_html`；handle-only（createElement
        // detached）无 sel → 无操作（beforeend/afterbegin 因脱离文档树无意义，beforebegin/afterend 需
        // parent——spec 对 detached 元素本就抛错，此处静默无操作更安全）。
        if (prop === 'insertAdjacentHTML') {
          return function(position, text) {
            if (sel && typeof __zw_insert_adjacent_html === 'function') {
              try {
                __zw_insert_adjacent_html(sel, String(position), String(text));
                _mo_notify(sel, handle, { type: 'childList', addedNodes: [], removedNodes: [] });
              } catch (_e) {}
            }
            return undefined;
          };
        }
        // `element.insertAdjacentText(position, text)`（P1a）：文本作**字面 Text 节点**（不解析
        // HTML）按 position 插入——区别于 insertAdjacentHTML（解析片段）。仅 sel-based（已挂载）
        // 元素经 host `__zw_insert_adjacent_text`；handle-only detached 无操作（同 insertAdjacentHTML）。
        if (prop === 'insertAdjacentText') {
          return function(position, text) {
            if (sel && typeof __zw_insert_adjacent_text === 'function') {
              try {
                __zw_insert_adjacent_text(sel, String(position), String(text));
                _mo_notify(sel, handle, { type: 'childList', addedNodes: [], removedNodes: [] });
              } catch (_e) {}
            }
            return undefined;
          };
        }
        // `element.insertAdjacentElement(position, element)`（P1a）：既有节点按 position 移动插入。
        // 仅接受 create 句柄节点（element.__zwHandle）；sel-based 参考元素经 host
        // `__zw_insert_adjacent_element`，复用 append_child 自动 reparent 移动语义。
        // 返插入的元素（spec）；handle-only 目标或非节点参数 → null（spec 非法 element 抛 TypeError，
        // 此处宽容返 null 避免中断脚本）。
        if (prop === 'insertAdjacentElement') {
          return function(position, element) {
            if (
              sel &&
              element &&
              typeof __zw_insert_adjacent_element === 'function' &&
              element.__zwHandle
            ) {
              try {
                __zw_insert_adjacent_element(sel, String(position), element.__zwHandle);
                _mo_notify(sel, handle, { type: 'childList', addedNodes: [element], removedNodes: [] });
                return element;
              } catch (_e) {}
            }
            return null;
          };
        }
        if (prop === 'querySelector') {
          // 元素**子树**作用域（spec：仅后代，不含元素自身）。经 host `__zw_query_match_sub(sel, q)`
          // 在 elem 子树内查首个匹配。handle（未挂载 DOM，无 sel）→ null（脱离文档树无后代）。
          return function(q) {
            if (!sel || typeof __zw_query_match_sub !== 'function') return null;
            try {
              var hit = __zw_query_match_sub(sel, String(q));
              return hit ? _wrapSelector(hit) : null;
            } catch (_e) { return null; }
          };
        }
        if (prop === 'querySelectorAll') {
          // 元素**子树**作用域（spec：仅后代）。经 host `__zw_query_all_sub(sel, q)`。
          return function(q) {
            if (!sel || typeof __zw_query_all_sub !== 'function') return [];
            try {
              var all = __zw_query_all_sub(sel, String(q));
              if (!all) return [];
              return all.split('|').filter(Boolean).map(_wrapSelector);
            } catch (_e) { return []; }
          };
        }
        // `form.elements`（HTMLFormControlsCollection，R2829）——表单控件集合（jQuery serialize /
        // FormData / 校验库迭代高频）。仅 HTMLFormElement（_realTag==='FORM' gate）；非 form → undefined。
        // `_formControls(sel)` 查 '*' 全后代客户端按 tag 过滤（tree order）+ namedItem。
        if (prop === 'elements' && _realTag(sel, handle) === 'FORM') {
          var controls = _formControls(sel);
          // array-like collection + namedItem（id 或 name 首匹配）。
          controls.namedItem = function (name) {
            var n = String(name);
            for (var i = 0; i < controls.length; i++) {
              var c = controls[i];
              if (c && c.id === n) return c;
              try { if (c && c.getAttribute && c.getAttribute('name') === n) return c; } catch (_e2) {}
            }
            return null;
          };
          return controls;
        }
        // `form.length`（HTMLFormElement）= 控件数；非 form 透传（不拦截）。
        if (prop === 'length' && _realTag(sel, handle) === 'FORM') {
          return _formControls(sel).length;
        }
        // 布局测量 API：`el.getBoundingClientRect()` 返真实 DOMRect（P1a gBCR path C）。
        // selector-identity 元素（querySelector/getElementById，sel=stable_selector）→ host
        // `__zw_getBoundingClientRect(sel)` 解析 dom_html→NodeId→layout-rect snapshot 返 "x,y,w,h"。
        // host 未注册 / 未命中 / handle-identity（createElement，sel 为空）→ 零 rect（= 旧行为，零回归；
        // 作 reflow 触发器语义仍正确——返回值多被丢弃）。注：rect 反映「上次 render」（stale-but-non-zero），
        // 改样式后同脚本内即读见 pre-change rect（force-reflow-on-demand 为 follow-up）。
        // offsetWidth/offsetHeight/clientWidth/Top/Left 等布局几何属性从同一 rect 派生（见 get trap 末段）。
        if (prop === 'getBoundingClientRect') {
          return function() {
            // identity = selector（querySelector/getElementById 元素）或 handle（createElement
            // 元素，path A）。sel 空时用 handle，host RectBridge handler 查持久 handle→selector map
            // 解析；map 未命中/未注册 → 空串 → 零 rect（= 旧行为，零回归）。
            return _domRectFromId(sel || handle) || { x: 0, y: 0, top: 0, left: 0, right: 0, bottom: 0, width: 0, height: 0, toJSON: function() { return this; } };
          };
        }
        // `el.getClientRects()`（R2828）——DOMRectList（浮层定位库 popper.js/tether 取 [0] 测量）。
        // headless 无逐 line-box 布局 → 返**单元素 bounding rect** 数组（与 getBoundingClientRect 同源 _domRectFromId）；
        // inline 多行收缩为单 rect（无 per-line-box，documented）；handle-only detached 无 layout → []。
        if (prop === 'getClientRects') {
          return function() {
            var r = _domRectFromId(sel || handle);
            return r ? [r] : [];
          };
        }
        // 布局几何属性：offsetWidth/offsetHeight/clientWidth/clientHeight/offsetTop/offsetLeft。
        // 旧返 undefined → `el.offsetWidth > 0` visibility 检查误判 false（元素被当隐藏）。
        // 现从既有 __zw_getBoundingClientRect rect 派生（rect 反映上次 render，stale-but-non-zero
        // 同 gBCR）。无 rect（未渲染/handle 未映射）→ 0（detached 元素 offsetWidth=0 语义）。
        // 注：offsetWidth/Height 为 border-box（rect 即 border-box，精确）；clientWidth/Height 应为
        // content-box（缺 border 数据，此处≈offset，近似）；offsetTop/Left 应相对 offsetParent（此处
        // 相对 viewport，顶层元素精确、嵌套近似）——近似对 visibility/sizing 检查足够。
        if (prop === 'offsetWidth' || prop === 'clientWidth') {
          var r = _layoutRect(sel, handle);
          return r ? r.w : 0;
        }
        if (prop === 'offsetHeight' || prop === 'clientHeight') {
          var r = _layoutRect(sel, handle);
          return r ? r.h : 0;
        }
        if (prop === 'offsetTop') {
          var r = _layoutRect(sel, handle);
          return r ? r.y : 0;
        }
        if (prop === 'offsetLeft') {
          var r = _layoutRect(sel, handle);
          return r ? r.x : 0;
        }
        // scrollWidth/scrollHeight：滚动内容尺寸。布局 rect 无 overflow 数据（不含滚动展开量），
        // 近似为 client 尺寸（同 offsetWidth/Height 的 border-box 近似）。对「content 是否溢出」
        // 精确判定不足，但对 `el.scrollHeight > 0` 等 sizing 检查足够（消除旧 undefined 返回）。
        if (prop === 'scrollWidth') {
          var r = _layoutRect(sel, handle);
          return r ? r.w : 0;
        }
        if (prop === 'scrollHeight') {
          var r = _layoutRect(sel, handle);
          return r ? r.h : 0;
        }
        // scrollTop/scrollLeft：滚动偏移。当前无滚动状态跟踪 → 恒 0（无滚动行为，符合默认未滚动语义）。
        if (prop === 'scrollTop' || prop === 'scrollLeft') {
          return 0;
        }
        // offsetParent：最近 positioned 祖先（position != static）或 body，detached/hidden → null。
        // 布局 rect 无 style 信息，无法精确算 positioned 祖先；近似：有 rect（已渲染）→ body proxy，
        // 无 rect（detached/display:none）→ null。dominant 用法 `el.offsetParent === null` 可见性判定
        // 正确（visible→非 null body / hidden→null）；`offsetTop - offsetParent.offsetTop` 嵌套坐标
        // 为近似（offsetTop 本就 viewport-relative，见上注）。
        if (prop === 'offsetParent') {
          var rp = _layoutRect(sel, handle);
          return rp ? _wrapSelector('body') : null;
        }
        return undefined;
      },
      set: function(_t, prop, value) {
        var p = String(prop);
        var moAttr = null;
        if (p === 'textContent' || p === 'innerHTML') {
          if (p === 'innerHTML') {
            if (handle) __zw_set_inner_html_handle(handle, String(value));
            else __zw_set_inner_html(sel, String(value));
          } else if (handle) {
            __zw_set_text_handle(handle, String(value));
          } else {
            __zw_set_text(sel, String(value));
          }
          // textContent/innerHTML = characterData 类，incr 仅支持 attributes + childList，不 notify。
        } else if (p === 'outerHTML') {
          // outerHTML setter：整体替换元素为解析后的片段。仅 sel-based（需父节点）；
          // handle-only（detached）无父 → 无操作（spec 对无父元素赋 outerHTML 抛错，静默更安全）。
          if (sel && typeof __zw_set_outer_html === 'function') {
            try {
              __zw_set_outer_html(sel, String(value));
              _mo_notify(sel, handle, { type: 'childList', addedNodes: [], removedNodes: [] });
            } catch (_e) {}
          }
          return true;
        } else if (p === 'className') {
          _classCache[key] = String(value);
          if (handle) __zw_set_attr_handle(handle, 'class', String(value));
          else __zw_set_attr(sel, 'class', String(value));
          moAttr = 'class';
        } else if (p === 'id') {
          if (handle) __zw_set_attr_handle(handle, 'id', String(value));
          else __zw_set_attr(sel, 'id', String(value));
          moAttr = 'id';
        } else if (p === 'title' || p === 'lang' || p === 'dir') {
          // reflected 字符串属性 set——写同名 attribute + 同步客户端缓存（set 后 get 读缓存）。
          var rcb = _reflectedAttrs[key] || (_reflectedAttrs[key] = {});
          rcb[p] = String(value);
          if (handle) __zw_set_attr_handle(handle, p, String(value));
          else __zw_set_attr(sel, p, String(value));
          moAttr = p;
        } else if (p === 'tabIndex') {
          // tabIndex set——反射为 tabindex 属性（数值）；NaN 忽略（spec 抛，lenient 不抛）。同步缓存。
          var tisv = parseInt(value, 10);
          if (!isNaN(tisv)) {
            var rtc2 = _reflectedAttrs[key] || (_reflectedAttrs[key] = {});
            rtc2['tabindex'] = tisv;
            if (handle) __zw_set_attr_handle(handle, 'tabindex', String(tisv));
            else __zw_set_attr(sel, 'tabindex', String(tisv));
            moAttr = 'tabindex';
          }
        } else if (p === 'contentEditable') {
          // contentEditable set——反射 contenteditable 属性（lenient：spec 仅接受 true/false/plaintext-only
          // 否则抛 SyntaxError，本沙箱不抛直接设串避免中断脚本）。同步缓存。
          var cec2 = _reflectedAttrs[key] || (_reflectedAttrs[key] = {});
          cec2['contenteditable'] = String(value);
          if (handle) __zw_set_attr_handle(handle, 'contenteditable', String(value));
          else __zw_set_attr(sel, 'contenteditable', String(value));
          moAttr = 'contenteditable';
        } else if (p === 'accessKey') {
          // accessKey set——反射 accesskey 属性（串）。同步缓存。
          var akc2 = _reflectedAttrs[key] || (_reflectedAttrs[key] = {});
          akc2['accesskey'] = String(value);
          if (handle) __zw_set_attr_handle(handle, 'accesskey', String(value));
          else __zw_set_attr(sel, 'accesskey', String(value));
          moAttr = 'accesskey';
        } else if (p === 'role') {
          // role set——反射 role 属性（串）。同步缓存。
          var rlc2 = _reflectedAttrs[key] || (_reflectedAttrs[key] = {});
          rlc2['role'] = String(value);
          if (handle) __zw_set_attr_handle(handle, 'role', String(value));
          else __zw_set_attr(sel, 'role', String(value));
          moAttr = 'role';
        } else if (_ariaAttrName(p)) {
          // ariaXxx set——反射 aria-* 属性（ariaLabel→aria-label, ariaLabelledBy→aria-labelledby...）。
          // 通用映射覆盖全部 aria IDL 属性。同步缓存。
          var ariaAttr = _ariaAttrName(p);
          var arc2 = _reflectedAttrs[key] || (_reflectedAttrs[key] = {});
          arc2[ariaAttr] = String(value);
          if (handle) __zw_set_attr_handle(handle, ariaAttr, String(value));
          else __zw_set_attr(sel, ariaAttr, String(value));
          moAttr = ariaAttr;
        } else if (p === 'value') {
          // P1a select：编程设 `<select>.value = value` → 记 SelectOption mutation（apply 时
          // mark 匹配 option selected + deselect 兄弟）。匹配浏览器：编程设值不自动派 change。
          if (!handle && sel && typeof __zw_select_option === 'function' && _isTag(sel, 'SELECT')) {
            __zw_select_option(sel, String(value));
            // SelectOption 改的是子 option 的 selected 属性，非 select 元素自身的属性 mutation；
            // 不发 select 的 attributes MO 通知（语义正确）。
          } else if (_realTag(sel, handle) === 'OUTPUT') {
            // HTMLOutputElement.value setter（R2846）：dirty + 存当前值。spec：value 独立于 textContent——
            // <output> 按 children 渲染非 value，故设 .value 不写 DOM text（与 textarea 区分）。
            _outputValue[key] = String(value);
          } else {
            _inputValues[key] = String(value);
            // textarea 的 value ↔ **文本内容**（非 value 属性，HTML spec）——写 content 而非属性。
            // input 走 value 属性 mutation。
            if (!handle && sel && _isTag(sel, 'TEXTAREA')) {
              __zw_set_text(sel, String(value));
            } else if (handle) {
              __zw_set_attr_handle(handle, 'value', String(value));
            } else {
              __zw_set_attr(sel, 'value', String(value));
              moAttr = 'value';
            }
          }
        } else if (p === 'valueAsNumber') {
          // `input.valueAsNumber = n`（HTMLInputElement，R2836）——number/range：NaN→''，否则 String(n)→设
          // value 属性 + 缓存（复用 value 同步路径）。其他 type / 非 INPUT：no-op（date/time defer；分支
          // 终止不 fallthrough 致误设 'valueAsNumber' 内容属性）。仅 INPUT（_realTag gate）。
          if (_realTag(sel, handle) === 'INPUT') {
            var vsT = (handle ? __zw_get_attr_handle(handle, 'type') : __zw_get_attr(sel, 'type')) || '';
            if (vsT.toLowerCase() === 'number' || vsT.toLowerCase() === 'range') {
              var vsS = (typeof value === 'number' && isNaN(value)) ? '' : String(value);
              _inputValues[key] = vsS;
              if (handle) __zw_set_attr_handle(handle, 'value', vsS);
              else { __zw_set_attr(sel, 'value', vsS); moAttr = 'value'; }
            }
          }
        } else if (p === 'indeterminate') {
          // JS-only IDL 布尔（非 reflected attr）—— per-element state map（默认 false）。无属性 mutation。
          _indeterminate[key] = !!value;
        } else if (p === 'selectionStart' || p === 'selectionEnd' || p === 'selectionDirection') {
          // text-control 选区 setter（R2844）。Chromium 150 oracle 锚定：保持 0≤start≤end≤len 不变式——
          // 设 start 超 end → end 跟到 start（{start:99}→ end 升到 start）；设 end 低于 start → end 升回 start
          //（{end:-5}→ end 升到 start，不降）；start/end 均 clamp [0, len]；direction 仅接受 forward/backward/none。
          if (_isTextControl(sel, handle)) {
            var so = _selObj(key);
            if (p === 'selectionStart') {
              var nsLen = _controlValue(sel, handle, key).length;
              var ns2 = _clampSelOffset(value, nsLen);
              if (ns2 > so.end) so.end = ns2;
              so.start = ns2;
            } else if (p === 'selectionEnd') {
              var neLen = _controlValue(sel, handle, key).length;
              var ne2 = _clampSelOffset(value, neLen);
              if (ne2 < so.start) ne2 = so.start;
              so.end = ne2;
            } else {
              so.direction = (value === 'backward' || value === 'none') ? value : 'forward';
            }
          }
        } else if (p === 'htmlFor') {
          // `label.htmlFor = x`（R2840）——反射 `for` 属性（attr 名映射 htmlFor→for）。仅 LABEL。
          if (_realTag(sel, handle) === 'LABEL') {
            if (handle) __zw_set_attr_handle(handle, 'for', String(value));
            else { __zw_set_attr(sel, 'for', String(value)); moAttr = 'for'; }
          }
        } else if (p === 'defaultValue') {
          // `input.defaultValue = x`（R2840）——反射 `value` 属性（初始值；attr 名映射 defaultValue→value）。
          // 仅设 value 属性，不联动 .value 当前态（spec 仅当当前值等于旧 defaultValue 时联动——罕见 defer）。
          if (_realTag(sel, handle) === 'INPUT') {
            if (handle) __zw_set_attr_handle(handle, 'value', String(value));
            else { __zw_set_attr(sel, 'value', String(value)); moAttr = 'value'; }
          } else if (_realTag(sel, handle) === 'OUTPUT') {
            // `output.defaultValue = x`（R2846）——更新捕获的初值缓存（不联动 textContent/.value 当前态——
            // spec 仅当未 dirty 时联动，罕见 defer；Chromium 150 oracle：dirty 时设 defaultValue 不改 value）。
            _outputDefault[key] = String(value);
          }
        } else if (p === 'defaultChecked') {
          // `input.defaultChecked = x`（R2840）——boolean 反射 `checked` 属性（truthy→设存在，falsy→移除）。
          if (_realTag(sel, handle) === 'INPUT') {
            if (value) {
              if (handle) __zw_set_attr_handle(handle, 'checked', '');
              else { __zw_set_attr(sel, 'checked', ''); moAttr = 'checked'; }
            } else if (!handle && typeof __zw_remove_attr === 'function') {
              __zw_remove_attr(sel, 'checked'); moAttr = 'checked';
            }
          }
        } else if (p === 'hidden' || p === 'checked' || p === 'disabled' || p === 'selected') {
          // boolean reflected property：truthy → 设存在（空值，has_attr=true）；falsy → 真移除
          // （has_attr=false）。修正旧 fallthrough 写空串致 falsy 仍 present 的 bug。
          if (value) {
            if (handle) __zw_set_attr_handle(handle, p, '');
            else __zw_set_attr(sel, p, '');
            moAttr = p;
          } else if (!handle && typeof __zw_remove_attr === 'function') {
            __zw_remove_attr(sel, p);
            moAttr = p;
          }
          // handle falsy：无 remove-handle 变体 → 不设（detach 元素 append 时默认无该布尔属性）。
        } else if (p === 'autofocus' || p === 'draggable' || p === 'spellcheck' || p === 'translate' || p === 'inert' || p === 'autocomplete') {
          // reflected 布尔/枚举全局属性（R2848/R2850）：autofocus/draggable/spellcheck/translate（R2848）
          // + inert/autocomplete（R2850）。autofocus/inert=boolean presence（truthy 设空值 / falsy 真移除）；
          // autocomplete=enumerated 串（任意值写 attr）；draggable/spellcheck="true"/"false"；translate="yes"/"no"。
          var rc4 = _reflectedAttrs[key] || (_reflectedAttrs[key] = {});
          if (p === 'autofocus' || p === 'inert') {
            var bsv = !!value;
            rc4[p] = bsv;
            if (bsv) {
              if (handle) __zw_set_attr_handle(handle, p, '');
              else { __zw_set_attr(sel, p, ''); moAttr = p; }
            } else if (!handle && typeof __zw_remove_attr === 'function') {
              __zw_remove_attr(sel, p); moAttr = p;
            }
          } else if (p === 'autocomplete') {
            rc4[p] = String(value);
            if (handle) __zw_set_attr_handle(handle, 'autocomplete', String(value));
            else { __zw_set_attr(sel, 'autocomplete', String(value)); moAttr = 'autocomplete'; }
          } else {
            var sv = !!value;
            rc4[p] = sv;
            var attrV = (p === 'translate') ? (sv ? 'yes' : 'no') : (sv ? 'true' : 'false');
            if (handle) __zw_set_attr_handle(handle, p, attrV);
            else { __zw_set_attr(sel, p, attrV); moAttr = p; }
          }
        } else if ((p === 'width' || p === 'height') && (_realTag(sel, handle) === 'IMG' || _realTag(sel, handle) === 'IFRAME')) {
          // reflected unsigned-long 维度 setter（R2851）：parseInt 归一（NaN/负 → 0）→ 缓存数值 + 写 width/height
          // 内容属性（getter 优先读缓存保 sync set→get）。
          var wv = parseInt(value, 10);
          if (isNaN(wv) || wv < 0) wv = 0;
          var wrc = _reflectedAttrs[key] || (_reflectedAttrs[key] = {});
          wrc[p] = wv;
          if (handle) __zw_set_attr_handle(handle, p, String(wv));
          else { __zw_set_attr(sel, p, String(wv)); moAttr = p; }
        } else {
          if (handle) __zw_set_attr_handle(handle, p, String(value));
          else __zw_set_attr(sel, p, String(value));
          moAttr = p;
        }
        if (moAttr) _mo_notify(sel, handle, { type: 'attributes', attributeName: moAttr });
        return true;
      }
    });
    _proxyCache[key] = proxy;
    return proxy;
  }

  function _wrapSelector(sel) {
    return _makeProxy(sel, null);
  }

  function _wrapHandle(handle) {
    return _makeProxy(null, handle);
  }

  // canvas 元素 + 2d 上下文 proxy（R2795，canvas slice 1）。host 持 CanvasContext 注册表，JS 经
  // `__zw_canvas_op(handle, op, ...args)` 串参派发。`getContext('2d')` 首次调时创建 host 上下文（返 id），
  // 后续返回同一 proxy。host 未注册 → getContext 返 null（no-throw 回落）。width/height 默认 300×150（spec）。
  // **fillRect 经 path 实现**（host fill_rect 便捷法不写 pixel_buffer，path-based fill 经 blit 写）。
  // **canvas 为 standalone 对象**（非 host-backed 元素 proxy——canvas 主要经 context 离屏绘制，不需 DOM
  // 树挂载；DOM 集成/appendChild 为 follow-up）。
  function _zwMakeCanvas() {
    var el = {
      nodeType: 1,
      tagName: 'CANVAS',
      nodeName: 'CANVAS',
      localName: 'canvas',
      width: 300,
      height: 150,
      style: {},
      _ctx: null
    };
    el.getContext = function (type) {
      if (String(type) !== '2d') return null; // 仅 2d；webgl/webgl2 defer
      if (el._ctx) return el._ctx;
      if (typeof __zw_canvas_op !== 'function') return null;
      var id = __zw_canvas_op('0', 'getContext2d', String(el.width), String(el.height));
      if (!id || String(id).charAt(0) === '!') return null;
      el._ctx = _zwMakeCtx2d(String(id));
      el._ctx.canvas = el;
      return el._ctx;
    };
    // toDataURL（R2797，canvas slice 3）：PNG 导出。host 编码 ctx.pixel_buffer → PNG（csv 字节）→
    // shim 转 Latin-1 → btoa → `data:image/png;base64,...`（复用 btoa，无 base64 dep）。仅 'image/png'
    //（type 参数忽略，jpeg/webp defer）；host 未注册 / 编码失败 → `data:,` 回落。无 ctx 时惰性创建。
    el.toDataURL = function (_type) {
      if (typeof __zw_canvas_op !== 'function') return 'data:,';
      if (!el._ctx) el.getContext('2d');
      if (!el._ctx) return 'data:,';
      var csv = String(__zw_canvas_op(el._ctx._handle, 'toDataURL'));
      if (!csv) return 'data:,';
      var nums = csv.split(',');
      var s = '';
      for (var i = 0; i < nums.length; i++) s += String.fromCharCode(+nums[i]);
      return 'data:image/png;base64,' + btoa(s);
    };
    return el;
  }
  function _zwMakeCtx2d(h) {
    var ctx = { _handle: h, canvas: null, _fs: '#000000', _ss: '#000000', _lw: 1.0 };
    Object.defineProperty(ctx, 'fillStyle', {
      set: function (v) { this._fs = String(v); __zw_canvas_op(h, 'setFillStyle', String(v)); },
      get: function () { return this._fs; }
    });
    Object.defineProperty(ctx, 'strokeStyle', {
      set: function (v) { this._ss = String(v); __zw_canvas_op(h, 'setStrokeStyle', String(v)); },
      get: function () { return this._ss; }
    });
    Object.defineProperty(ctx, 'lineWidth', {
      set: function (v) { this._lw = +v; __zw_canvas_op(h, 'setLineWidth', String(v)); },
      get: function () { return this._lw; }
    });
    ctx.beginPath = function () { __zw_canvas_op(h, 'beginPath'); };
    ctx.closePath = function () { __zw_canvas_op(h, 'closePath'); };
    ctx.moveTo = function (x, y) { __zw_canvas_op(h, 'moveTo', String(x), String(y)); };
    ctx.lineTo = function (x, y) { __zw_canvas_op(h, 'lineTo', String(x), String(y)); };
    ctx.arc = function (x, y, r, s, e) {
      __zw_canvas_op(h, 'arc', String(x), String(y), String(r), String(s), String(e));
    };
    ctx.fill = function () { __zw_canvas_op(h, 'fill'); };
    ctx.stroke = function () { __zw_canvas_op(h, 'stroke'); };
    ctx.fillRect = function (x, y, w, hh) {
      __zw_canvas_op(h, 'fillRect', String(x), String(y), String(w), String(hh));
    };
    ctx.strokeRect = function (x, y, w, hh) {
      __zw_canvas_op(h, 'strokeRect', String(x), String(y), String(w), String(hh));
    };
    ctx.clearRect = function (x, y, w, hh) {
      __zw_canvas_op(h, 'clearRect', String(x), String(y), String(w), String(hh));
    };
    // ── slice 2：path 曲线 / 状态栈 / transforms / line 样式 / globalAlpha（R2796）──
    ctx.quadraticCurveTo = function (cpx, cpy, x, y) {
      __zw_canvas_op(h, 'quadraticCurveTo', String(cpx), String(cpy), String(x), String(y));
    };
    ctx.bezierCurveTo = function (cp1x, cp1y, cp2x, cp2y, x, y) {
      __zw_canvas_op(h, 'bezierCurveTo', String(cp1x), String(cp1y), String(cp2x), String(cp2y), String(x), String(y));
    };
    ctx.ellipse = function (x, y, rx, ry, rotation, start, end /*, ccw */) {
      __zw_canvas_op(h, 'ellipse', String(x), String(y), String(rx), String(ry), String(rotation), String(start), String(end));
    };
    ctx.arcTo = function (x1, y1, x2, y2, r) {
      __zw_canvas_op(h, 'arcTo', String(x1), String(y1), String(x2), String(y2), String(r));
    };
    ctx.rect = function (x, y, w, hh) {
      __zw_canvas_op(h, 'rect', String(x), String(y), String(w), String(hh));
    };
    ctx.clip = function () { __zw_canvas_op(h, 'clip'); };
    ctx.save = function () { __zw_canvas_op(h, 'save'); };
    ctx.restore = function () { __zw_canvas_op(h, 'restore'); };
    ctx.translate = function (tx, ty) { __zw_canvas_op(h, 'translate', String(tx), String(ty)); };
    ctx.rotate = function (angle) { __zw_canvas_op(h, 'rotate', String(angle)); };
    ctx.scale = function (sx, sy) { __zw_canvas_op(h, 'scale', String(sx), String(sy)); };
    ctx.setTransform = function (a, b, c, d, e, ff) {
      __zw_canvas_op(h, 'setTransform', String(a), String(b), String(c), String(d), String(e), String(ff));
    };
    ctx.transform = function (a, b, c, d, e, ff) {
      __zw_canvas_op(h, 'transform', String(a), String(b), String(c), String(d), String(e), String(ff));
    };
    // globalAlpha / lineDash / lineJoin / lineCap：getter+setter（client-side 存值 + push host）。
    ctx._ga = 1.0;
    Object.defineProperty(ctx, 'globalAlpha', {
      set: function (v) { this._ga = +v; __zw_canvas_op(h, 'setGlobalAlpha', String(v)); },
      get: function () { return this._ga; }
    });
    ctx.setLineDash = function (segs) {
      var s = (segs && segs.length != null) ? Array.prototype.join.call(segs, ',') : String(segs);
      __zw_canvas_op(h, 'setLineDash', s);
    };
    ctx._lj = 'miter';
    Object.defineProperty(ctx, 'lineJoin', {
      set: function (v) { this._lj = String(v); __zw_canvas_op(h, 'setLineJoin', String(v)); },
      get: function () { return this._lj; }
    });
    ctx._lc = 'butt';
    Object.defineProperty(ctx, 'lineCap', {
      set: function (v) { this._lc = String(v); __zw_canvas_op(h, 'setLineCap', String(v)); },
      get: function () { return this._lc; }
    });
    // ── slice 4：globalCompositeOperation / shadow / putImageData（R2798）──
    // 客户端镜像串 + push host（同 lineJoin/lineCap 模式）。getter 取客户端镜像，免 host 往返。
    // **已知限制**：composite 仅对 stroke/rect-blit 生效（host composite_pixel），path-based fillRect 不消费。
    ctx._gco = 'source-over';
    Object.defineProperty(ctx, 'globalCompositeOperation', {
      set: function (v) { this._gco = String(v); __zw_canvas_op(h, 'setCompositeOperation', String(v)); },
      get: function () { return this._gco; }
    });
    ctx._sc = 'rgba(0, 0, 0, 0)';
    Object.defineProperty(ctx, 'shadowColor', {
      set: function (v) { this._sc = String(v); __zw_canvas_op(h, 'setShadowColor', String(v)); },
      get: function () { return this._sc; }
    });
    ctx._sb = 0;
    Object.defineProperty(ctx, 'shadowBlur', {
      set: function (v) { this._sb = +v; __zw_canvas_op(h, 'setShadowBlur', String(v)); },
      get: function () { return this._sb; }
    });
    ctx._sox = 0;
    Object.defineProperty(ctx, 'shadowOffsetX', {
      set: function (v) { this._sox = +v; __zw_canvas_op(h, 'setShadowOffsetX', String(v)); },
      get: function () { return this._sox; }
    });
    ctx._soy = 0;
    Object.defineProperty(ctx, 'shadowOffsetY', {
      set: function (v) { this._soy = +v; __zw_canvas_op(h, 'setShadowOffsetY', String(v)); },
      get: function () { return this._soy; }
    });
    // putImageData(imagedata, dx, dy)：序列化 data → csv，dx/dy/w/h 串参派发。host 1:1 写 pixel_buffer。
    ctx.putImageData = function (img, dx, dy) {
      if (!img || !img.data) return;
      var d = img.data;
      var n = d.length;
      // 分片拼接（避免超大数据单次 += 触发大字符串重分配；测试用小图，正常路径即可）。
      var chunks = [];
      for (var i = 0; i < n; i++) {
        chunks.push((i ? ',' : '') + d[i]);
      }
      __zw_canvas_op(h, 'putImageData', String(dx | 0), String(dy | 0),
        String(img.width | 0), String(img.height | 0), chunks.join(''));
    };
    // drawImage（R2799，canvas slice 5）：源 canvas → 本 ctx。3 spec 重载（arg 数 3/5/9）：
    //   drawImage(image, dx, dy) / drawImage(image, dx, dy, dw, dh) /
    //   drawImage(image, sx, sy, sw, sh, dx, dy, dw, dh)。
    // **源限 canvas 元素**（canvas-to-canvas）：经源 canvas 既有 getImageData 取全 RGBA wire 串作源传 host；
    // HTMLImageElement/`<img>` decode defer。host draw_image* 真栅格（source-over alpha 混合）。
    ctx.drawImage = function (image) {
      if (typeof __zw_canvas_op !== 'function') return;
      // 源须为 canvas 元素（有 _ctx._handle + width/height）。未 getContext 则惰性建。
      if (!image || typeof image.getContext !== 'function') return;
      if (!image._ctx) image.getContext('2d');
      if (!image._ctx) return;
      var srcHandle = image._ctx._handle;
      var sw = image.width | 0;
      var sh = image.height | 0;
      if (sw <= 0 || sh <= 0) return;
      var wire = String(__zw_canvas_op(srcHandle, 'getImageData', '0', '0', String(sw), String(sh)));
      var a = arguments;
      if (a.length === 3) {
        __zw_canvas_op(h, 'drawImage', wire, String(a[1]), String(a[2]));
      } else if (a.length === 5) {
        __zw_canvas_op(h, 'drawImageScaled', wire,
          String(a[1]), String(a[2]), String(a[3]), String(a[4]));
      } else if (a.length === 9) {
        __zw_canvas_op(h, 'drawImageSliced', wire,
          String(a[1]), String(a[2]), String(a[3]), String(a[4]),
          String(a[5]), String(a[6]), String(a[7]), String(a[8]));
      }
    };
    ctx.getImageData = function (x, y, w, hh) {
      if (typeof __zw_canvas_op !== 'function') return null;
      var r = String(__zw_canvas_op(h, 'getImageData', String(x), String(y), String(w), String(hh)));
      if (!r) return null;
      var parts = r.split(';');
      var dims = parts[0].split(':');
      var nums = parts[1] ? parts[1].split(',') : [];
      var arr = new Uint8ClampedArray(nums.length);
      for (var i = 0; i < nums.length; i++) arr[i] = +nums[i];
      return { width: +dims[0], height: +dims[1], data: arr };
    };
    return ctx;
  }

  // `|` 分隔的选择器串 → 元素 proxy 数组（空串/无回调 → []）。供 children 等导航 API。
  function _splitSelectors(joined) {
    if (!joined) return [];
    return joined.split('|').filter(Boolean).map(_wrapSelector);
  }

  // 节点级遍历：把 __zw_child_nodes/__zw_sibling_nodes 返的 JSON 条目（{k:'E'|'T'|'C',...}）
  // 转 proxy/对象。元素 → _wrapSelector；文本/注释 → 纯对象（nodeType 3/8，纯读快照非 live，
  // parentNode=parentProxy）。文本节点无 selector，故用静态对象（nodeValue/textContent/data 只读）。
  function _wrapNodeEntry(entry, parentProxy) {
    if (!entry) return null;
    if (entry.k === 'E') return _wrapSelector(entry.s);
    var isComment = entry.k === 'C';
    var text = entry.v != null ? entry.v : '';
    return {
      nodeType: isComment ? 8 : 3,
      nodeName: isComment ? '#comment' : '#text',
      nodeValue: text,
      textContent: text,
      data: text,
      length: text.length,
      parentNode: parentProxy,
      parentElement: parentProxy,
      previousSibling: null,
      nextSibling: null,
      __zwIsText: true,
    };
  }

  // `el.childNodes`（含文本/注释）：解析 __zw_child_nodes JSON 数组 → 节点数组（快照，非 live）。
  function _childNodeList(sel, handle) {
    if (!sel || typeof __zw_child_nodes !== 'function') return [];
    try {
      var arr = JSON.parse(__zw_child_nodes(sel) || '[]');
      var parent = handle ? _wrapHandle(handle) : _wrapSelector(sel);
      return arr.map(function(e) { return _wrapNodeEntry(e, parent); });
    } catch (_e) { return []; }
  }

  // `prepend`/`before`/`after` 共用：variadic 节点/字符串按 position 经 insertAdjacent*
  // 回调插入。仅 sel-based（已挂载）目标；handle-only（detached）无操作（同 insertAdjacent 家族）。
  // `reverseOrder`：afterbegin（prepend）/afterend（after）需反序插入以保持「参数序 == DOM 序」
  //（每插一项后参考子/兄弟前移）；beforebegin（before）正序即可（参考 = target 固定）。
  function _insertAdjacentVariadic(sel, position, args, reverseOrder) {
    if (!sel || typeof __zw_insert_adjacent_element !== 'function') return;
    var items = [];
    for (var i = 0; i < args.length; i++) {
      var a = args[i];
      if (a == null) continue;
      items.push(a);
    }
    if (reverseOrder) items.reverse();
    for (var k = 0; k < items.length; k++) {
      var item = items[k];
      try {
        if (typeof item === 'object' && item.__zwHandle) {
          __zw_insert_adjacent_element(sel, position, item.__zwHandle);
        } else {
          __zw_insert_adjacent_text(sel, position, String(item));
        }
      } catch (_e) {}
    }
  }

  // append/replaceChildren 共用：variadic 节点/字符串追加到 this 末尾（DocumentFragment flatten）。
  // 返 added 列表（供 MO childList notify）。节点经 handle/selector append_child；字符串建 text 节点 append。
  function _appendVariadic(sel, handle, args) {
    var added = [];
    for (var i = 0; i < args.length; i++) {
      var item = args[i];
      if (item == null) continue;
      if (typeof item === 'object' && item.__zwHandle) {
        // DocumentFragment：flatten 子节点到 this。
        if (_fragmentHandles[item.__zwHandle] && typeof __zw_append_fragment_children === 'function') {
          if (handle) __zw_append_fragment_children_handle(handle, item.__zwHandle);
          else __zw_append_fragment_children(sel, item.__zwHandle);
        } else if (handle) {
          __zw_append_child_handle(handle, item.__zwHandle);
        } else {
          __zw_append_child(sel, item.__zwHandle);
        }
        added.push(item);
      } else {
        var tn = __zw_create_text(String(item));
        if (handle) __zw_append_child_handle(handle, tn);
        else __zw_append_child(sel, tn);
        added.push({ __zwHandle: tn, __zwSelector: '' });
      }
    }
    return added;
  }

  // 元素的布局 rect（{x,y,w,h}），经 `__zw_getBoundingClientRect`（与 getBoundingClientRect 同源）。
  // 无回调/未命中/handle 未映射 → null（调用方返 0）。rect 反映上次 render（stale-but-non-zero）。
  function _layoutRect(sel, handle) {
    var id = sel || handle;
    if (id && typeof __zw_getBoundingClientRect === 'function') {
      try {
        var s = __zw_getBoundingClientRect(id);
        if (s && s.indexOf(',') >= 0) {
          var p = s.split(',');
          return { x: +p[0], y: +p[1], w: +p[2], h: +p[3] };
        }
      } catch (_e) {}
    }
    return null;
  }

  // getBoundingClientRect/getClientRects 共用（R2828）：从 `__zw_getBoundingClientRect(id)` 解析
  // "x,y,w,h" → 完整 DOMRect（x/y/top/left/right/bottom/width/height + toJSON）。id = selector 或 handle。
  // 未注册 / 未命中 / 无 layout（handle-only detached）→ null（getBoundingClientRect 落零 rect，getClientRects 落 []）。
  function _domRectFromId(id) {
    if (id && typeof __zw_getBoundingClientRect === 'function') {
      try {
        var s = __zw_getBoundingClientRect(id);
        if (s && s.indexOf(',') >= 0) {
          var p = s.split(',');
          var x = +p[0], y = +p[1], w = +p[2], h = +p[3];
          return {
            x: x, y: y, top: y, left: x, right: x + w, bottom: y + h,
            width: w, height: h, toJSON: function () { return this; },
          };
        }
      } catch (_e) {}
    }
    return null;
  }

  // form.elements 表单控件集合（R2829）：form 后代中 input/button/select/textarea，**tree order**。
  // host `__zw_query_all_sub` 不支持逗号列表 / '*' 通用选择器 → 经 `childNodes` 递归下降遍历子树
  //（tree order 天然）客户端按 tag 过滤。供 form.elements（+ namedItem）+ form.length 共用。
  var _formControlTags = { INPUT: 1, BUTTON: 1, SELECT: 1, TEXTAREA: 1 };
  function _formControls(sel) {
    var controls = [];
    if (!sel) return controls;
    // 递归下降：childNodes 遍历子树（element 子递归，text/comment 跳过），tag 命中收集。
    function walk(parentProxy) {
      var kids = (parentProxy && parentProxy.childNodes) || [];
      for (var i = 0; i < kids.length; i++) {
        var k = kids[i];
        if (k && k.nodeType === 1) {
          if (_formControlTags[k.tagName]) controls.push(k);
          walk(k);
        }
      }
    }
    try { walk(_wrapSelector(sel)); } catch (_e) {}
    return controls;
  }

  // dataset 键转换：camelCase ↔ data-kebab-case（fooBar ↔ data-foo-bar）。
  function _camelToKebab(s) {
    return s.replace(/[A-Z]/g, function(m) { return '-' + m.toLowerCase(); });
  }

  // ARIA IDL 属性名 → content 属性名（element.ariaXxx ↔ aria-xxx）。
  // **不同于 _camelToKebab**：ariaLabelledBy → aria-labelledby（单 hyphen，非 aria-labelled-by）。
  // 规则：aria + 大写首字母 + 余 → aria- + 全小写(余)。非 aria 前缀 / 首字母非大写 → null。
  function _ariaAttrName(prop) {
    if (typeof prop !== 'string' || prop.length < 5 || prop.slice(0, 4) !== 'aria') return null;
    var rest = prop.slice(4);
    var head = rest.charAt(0);
    if (head < 'A' || head > 'Z') return null;
    return 'aria-' + rest.toLowerCase();
  }
  function _kebabToCamel(s) {
    return s.replace(/-([a-z])/g, function(_, c) { return c.toUpperCase(); });
  }

  // `el.dataset`——data-* 属性的 camelCase 键对象。get/set/has/delete/枚举（ownKeys）。
  // 注：mutate（set/delete）记 mutation，apply 在脚本末尾——同脚本内即读见旧值（stale，
  // 同 setAttribute 既有模式）；枚举读 dom_html 当前属性名。
  function _datasetProxy(sel, handle) {
    var attrOf = function(key) { return 'data-' + _camelToKebab(String(key)); };
    var readAttr = function(name) {
      return handle ? __zw_get_attr_handle(handle, name) : __zw_get_attr(sel, name);
    };
    var hasAttrFn = function(name) {
      try { return (handle ? false : __zw_has_attr(sel, name)) === '1'; } catch (_e) { return false; }
    };
    var dataKeys = function() {
      // 仅 sel-based 支持枚举（无 attr-names-handle）；data-* → camelCase 键。
      if (handle || typeof __zw_attr_names !== 'function') return [];
      try {
        var names = __zw_attr_names(sel);
        if (!names) return [];
        return names.split('|').filter(function(n) { return n.indexOf('data-') === 0; })
                     .map(function(n) { return _kebabToCamel(n.slice(5)); });
      } catch (_e) { return []; }
    };
    return new Proxy({}, {
      get: function(_t, key) {
        if (typeof key !== 'string') return undefined;
        if (key === 'then') return undefined; // 防 Promise 化误判
        var name = attrOf(key);
        // 缺失属性 → undefined（__zw_get_attr 对缺失返空串，须用 has_attr 区分）。
        if (!hasAttrFn(name)) return undefined;
        var v = readAttr(name);
        return v == null ? '' : v;
      },
      set: function(_t, key, value) {
        if (typeof key !== 'string') return false;
        var name = attrOf(key);
        if (handle) __zw_set_attr_handle(handle, name, String(value));
        else __zw_set_attr(sel, name, String(value));
        _mo_notify(sel, handle, { type: 'attributes', attributeName: name });
        return true;
      },
      has: function(_t, key) {
        return typeof key === 'string' && hasAttrFn(attrOf(key));
      },
      deleteProperty: function(_t, key) {
        if (typeof key !== 'string') return false;
        var name = attrOf(key);
        if (handle) __zw_set_attr_handle(handle, name, '');
        else if (typeof __zw_remove_attr === 'function') __zw_remove_attr(sel, name);
        else __zw_set_attr(sel, name, '');
        _mo_notify(sel, handle, { type: 'attributes', attributeName: name });
        return true;
      },
      ownKeys: function() { return dataKeys(); },
      getOwnPropertyDescriptor: function(_t, key) {
        if (typeof key !== 'string' || !hasAttrFn(attrOf(key))) return undefined;
        return { configurable: true, enumerable: true, value: readAttr(attrOf(key)), writable: true };
      }
    });
  }

  // Event/CustomEvent/KeyboardEvent——DOM 事件构造器（R2779 spec-completeness）。_makeEvent 造数据
  // 对象（含 spec 字段 composed/eventPhase/isTrusted/timeStamp/defaultPrevented），构造器置 [[Prototype]]
  // 使 instanceof 成立（chromium 一致：new Event() instanceof Event、new CustomEvent() instanceof Event）。
  // dispatch 读 _-prefixed 私字段（_defaultPrevented 等，勿改名）；公开 defaultPrevented 经 preventDefault
  // 镜像同步。initEvent legacy API 在 Event.prototype。
  globalThis.Event = function Event(type, options) {
    var ev = _makeEvent(type, options);
    Object.setPrototypeOf(ev, globalThis.Event.prototype);
    return ev;
  };
  if (typeof globalThis.Event.prototype.initEvent !== 'function') {
    globalThis.Event.prototype.initEvent = function (type, bubbles, cancelable) {
      this.type = type;
      this.bubbles = !!bubbles;
      this.cancelable = !!cancelable;
      this.defaultPrevented = false;
      this._defaultPrevented = false;
    };
  }

  globalThis.CustomEvent = function CustomEvent(type, options) {
    var ev = _makeEvent(type, options);
    Object.setPrototypeOf(ev, globalThis.CustomEvent.prototype);
    return ev;
  };
  globalThis.CustomEvent.prototype = Object.create(globalThis.Event.prototype);
  globalThis.CustomEvent.prototype.constructor = globalThis.CustomEvent;
  // initCustomEvent——legacy 合成事件初始化（与 createEvent('CustomEvent') + initEvent 配对，spec）。
  // 镜像 initEvent 设 type/bubbles/cancelable + 设 detail。guard 幂等（不覆盖既有定义）。
  if (typeof globalThis.CustomEvent.prototype.initCustomEvent !== 'function') {
    globalThis.CustomEvent.prototype.initCustomEvent = function (type, bubbles, cancelable, detail) {
      this.type = type;
      this.bubbles = !!bubbles;
      this.cancelable = !!cancelable;
      this.detail = detail;
      this.defaultPrevented = false;
      this._defaultPrevented = false;
    };
  }

  globalThis.KeyboardEvent = function KeyboardEvent(type, options) {
    var ev = _makeEvent(type, options);
    Object.setPrototypeOf(ev, globalThis.KeyboardEvent.prototype);
    ev.key = (options && options.key) || '';
    ev.code = (options && (options.code || options.key)) || '';
    return ev;
  };
  globalThis.KeyboardEvent.prototype = Object.create(globalThis.Event.prototype);
  globalThis.KeyboardEvent.prototype.constructor = globalThis.KeyboardEvent;

  // Event 子类簇（R2811）——UIEvent / MouseEvent / FocusEvent / WheelEvent / PointerEvent / InputEvent。
  // 现代输入事件表面：feature-detection（`'PointerEvent' in window`）+ `new MouseEvent('click',{clientX,...})`
  // 合成派发（测试 / 库 / 事件总线高频）。统一经 [`_defineEventSubclass`] 工厂建（复用 `_makeEvent` + 原型链
  // extends parent）。**已知限制**：① 仅构造期填字段（无真事件循环派发——同 Event/KeyboardEvent 既有简化）；
  // ② getModifierState 仅跟踪 Alt/Control/Meta/Shift（CapsLock/NumLock 等未跟踪→false）；③ pageX/pageY
  // 存值非计算（spec 计算自 clientX+scroll，本沙箱无滚动→取存值或 0）。
  function _defineEventSubclass(name, parentName, props) {
    if (globalThis[name]) return globalThis[name];
    var Parent = globalThis[parentName] || globalThis.Event;
    var Ctor = function (type, options) {
      var ev = _makeEvent(type, options);
      Object.setPrototypeOf(ev, Ctor.prototype);
      var o = options || {};
      for (var i = 0; i < props.length; i++) {
        var p = props[i];
        ev[p[0]] = o[p[1]] != null ? o[p[1]] : p[2];
      }
      return ev;
    };
    Ctor.prototype = Object.create(Parent.prototype);
    Ctor.prototype.constructor = Ctor;
    globalThis[name] = Ctor;
    return Ctor;
  }
  // UIEvent（Event 子类）：view（默认 null）/ detail（默认 0）。
  _defineEventSubclass('UIEvent', 'Event', [
    ['view', 'view', null],
    ['detail', 'detail', 0],
  ]);
  // MouseEvent（UIEvent 子类）：坐标 / 修饰键 / button / buttons / relatedTarget。
  var MouseEventCtor = _defineEventSubclass('MouseEvent', 'UIEvent', [
    ['screenX', 'screenX', 0], ['screenY', 'screenY', 0],
    ['clientX', 'clientX', 0], ['clientY', 'clientY', 0],
    ['pageX', 'pageX', 0], ['pageY', 'pageY', 0],
    ['offsetX', 'offsetX', 0], ['offsetY', 'offsetY', 0],
    ['ctrlKey', 'ctrlKey', false], ['shiftKey', 'shiftKey', false],
    ['altKey', 'altKey', false], ['metaKey', 'metaKey', false],
    ['button', 'button', 0], ['buttons', 'buttons', 0],
    ['relatedTarget', 'relatedTarget', null], ['region', 'region', null],
  ]);
  // getModifierState——修饰键状态查询（PointerEvent/WheelEvent 经原型链继承）。仅 4 个 tracked 修饰键。
  MouseEventCtor.prototype.getModifierState = function (key) {
    var k = String(key);
    if (k === 'Alt') return !!this.altKey;
    if (k === 'Control') return !!this.ctrlKey;
    if (k === 'Meta') return !!this.metaKey;
    if (k === 'Shift') return !!this.shiftKey;
    return false;
  };
  // FocusEvent（UIEvent 子类）：relatedTarget。
  _defineEventSubclass('FocusEvent', 'UIEvent', [
    ['relatedTarget', 'relatedTarget', null],
  ]);
  // WheelEvent（MouseEvent 子类）：delta + deltaMode + DOM_DELTA_* 静态常量。
  var WheelEventCtor = _defineEventSubclass('WheelEvent', 'MouseEvent', [
    ['deltaX', 'deltaX', 0], ['deltaY', 'deltaY', 0], ['deltaZ', 'deltaZ', 0],
    ['deltaMode', 'deltaMode', 0],
  ]);
  WheelEventCtor.DOM_DELTA_PIXEL = 0;
  WheelEventCtor.DOM_DELTA_LINE = 1;
  WheelEventCtor.DOM_DELTA_PAGE = 2;
  // PointerEvent（MouseEvent 子类）：pointer 字段。
  _defineEventSubclass('PointerEvent', 'MouseEvent', [
    ['pointerId', 'pointerId', 0], ['width', 'width', 1], ['height', 'height', 1],
    ['pressure', 'pressure', 0], ['tiltX', 'tiltX', 0], ['tiltY', 'tiltY', 0],
    ['pointerType', 'pointerType', ''], ['isPrimary', 'isPrimary', false],
    ['twist', 'twist', 0], ['tangentialPressure', 'tangentialPressure', 0],
  ]);
  // InputEvent（UIEvent 子类）：data / inputType / isComposing / dataTransfer。
  _defineEventSubclass('InputEvent', 'UIEvent', [
    ['data', 'data', null], ['isComposing', 'isComposing', false],
    ['inputType', 'inputType', ''], ['dataTransfer', 'dataTransfer', null],
  ]);
  // Event 子类簇 #2（R2812）——均 extends Event：HashChangeEvent（SPA hash 路由）/ PopStateEvent（history
  // 路由）/ StorageEvent（跨标签页 storage 同步）/ ProgressEvent（XHR/资源加载进度）/ TransitionEvent·
  // AnimationEvent（CSS 过渡/动画回调）。feature-detection + `new X(type, init)` 合成派发高频。复用
  // [`_defineEventSubclass`]（R2811）。**已知限制**：仅构造期填字段（无真事件派发——同既有简化）。
  _defineEventSubclass('HashChangeEvent', 'Event', [
    ['oldURL', 'oldURL', ''], ['newURL', 'newURL', ''],
  ]);
  _defineEventSubclass('PopStateEvent', 'Event', [
    ['state', 'state', null],
  ]);
  _defineEventSubclass('StorageEvent', 'Event', [
    ['key', 'key', null], ['newValue', 'newValue', null], ['oldValue', 'oldValue', null],
    ['url', 'url', ''], ['storageArea', 'storageArea', null],
  ]);
  _defineEventSubclass('ProgressEvent', 'Event', [
    ['lengthComputable', 'lengthComputable', false], ['loaded', 'loaded', 0], ['total', 'total', 0],
  ]);
  _defineEventSubclass('TransitionEvent', 'Event', [
    ['propertyName', 'propertyName', ''], ['elapsedTime', 'elapsedTime', 0], ['pseudoElement', 'pseudoElement', ''],
  ]);
  _defineEventSubclass('AnimationEvent', 'Event', [
    ['animationName', 'animationName', ''], ['elapsedTime', 'elapsedTime', 0], ['pseudoElement', 'pseudoElement', ''],
  ]);

  // EventTarget——事件目标基类型（独立构造器，R2779）。库常用 `new EventTarget()` / `extends EventTarget`
  // 做事件发射器（pub-sub / 自定义事件总线）。元素 / document / window 经各自 addEventListener 路径；
  // 本构造器提供自包含 listener map（与 DOM 元素事件系统独立，派发事件不冒泡到 DOM，spec 一致）。
  // **已知限制**：仅 target 阶段（EventTarget 无 DOM 父链，无跨节点 capture/bubble；capture listener
  // 在 target 阶段同 fire）；dispatchEvent 返 `!defaultPrevented`（spec 一致）。
  function EventTarget() {
    this._et_listeners = {};
  }
  EventTarget.prototype.addEventListener = function (type, cb, opts) {
    if (typeof cb !== 'function' || typeof type !== 'string') return;
    var capture = opts === true || (opts && opts.capture) ? '|cap' : '';
    var key = type + capture;
    (this._et_listeners[key] = this._et_listeners[key] || []).push(cb);
  };
  EventTarget.prototype.removeEventListener = function (type, cb, opts) {
    if (typeof cb !== 'function' || typeof type !== 'string') return;
    var capture = opts === true || (opts && opts.capture) ? '|cap' : '';
    var arr = this._et_listeners[type + capture];
    if (arr) {
      var i = arr.indexOf(cb);
      if (i >= 0) arr.splice(i, 1);
    }
  };
  EventTarget.prototype.dispatchEvent = function (event) {
    if (event == null || typeof event.type !== 'string') {
      event = _makeEvent(event == null ? '' : String(event && event.type), {});
    }
    var target = this;
    event.target = target;
    event.currentTarget = target;
    var suffixes = ['', '|cap'];
    for (var s = 0; s < suffixes.length; s++) {
      var arr = target._et_listeners[event.type + suffixes[s]];
      if (!arr) continue;
      arr = arr.slice();
      for (var i = 0; i < arr.length; i++) {
        if (event._immediateStopped) break;
        try { arr[i].call(target, event); } catch (_) {}
      }
    }
    return !event._defaultPrevented;
  };
  globalThis.EventTarget = globalThis.EventTarget || EventTarget;

  // matchMedia——window.matchMedia(query) 响应式设计 / viewport 查询（modern 站点高频，shim 曾缺失）。
  // 委托 host `__zw_match_media(query, w, h)`（spec-correct via zero_css_parser::media_query）。返
  // MediaQueryList（extends EventTarget R2779）：media/matches + addEventListener('change') + legacy
  // addListener/removeListener。**已知限制**：change 事件需 host resize 跟踪派发（当前无，addListener
  // 注册有效但不触发；matches 为查询时刻快照，spec 一致）。
  function MediaQueryList(media, matches) {
    this._et_listeners = {}; // EventTarget 内部 listener map（EventTarget 构造器未自动调，手动初始化）
    this.media = media;
    this.matches = matches;
  }
  MediaQueryList.prototype = Object.create(EventTarget.prototype);
  MediaQueryList.prototype.constructor = MediaQueryList;
  // legacy 别名（旧 API：addListener/removeListener → change 事件）。
  MediaQueryList.prototype.addListener = function (cb) { this.addEventListener('change', cb); };
  MediaQueryList.prototype.removeListener = function (cb) { this.removeEventListener('change', cb); };
  globalThis.MediaQueryList = globalThis.MediaQueryList || MediaQueryList;
  function matchMedia(query) {
    var q = String(query);
    var matches = false;
    if (typeof __zw_match_media === 'function') {
      var raw = __zw_match_media(q, globalThis.innerWidth || 0, globalThis.innerHeight || 0);
      try { var p = JSON.parse(raw); matches = !!p.matches; } catch (_) {}
    }
    return new MediaQueryList(q, matches);
  }
  globalThis.matchMedia = globalThis.matchMedia || matchMedia;

  // MessageEvent——message 事件（Window.postMessage / MessagePort / BroadcastChannel 派发）。extends
  // Event（R2779），加 data/origin/source/ports。复用 _makeEvent 造数据对象 + 置 [[Prototype]]。
  function MessageEvent(type, options) {
    var ev = _makeEvent(type, options);
    Object.setPrototypeOf(ev, MessageEvent.prototype);
    ev.data = options && options.data !== undefined ? options.data : null;
    ev.origin = (options && options.origin) || '';
    ev.source = (options && options.source) || null;
    ev.ports = [];
    return ev;
  }
  MessageEvent.prototype = Object.create(Event.prototype);
  MessageEvent.prototype.constructor = MessageEvent;
  globalThis.MessageEvent = globalThis.MessageEvent || MessageEvent;

  // MessagePort——消息端口（MessageChannel 双端口之一，部分库经此做结构化通信）。extends EventTarget
  //（R2779）。postMessage 经 structuredClone（R2773）深拷贝消息 + queueMicrotask（R2774）**异步**派发
  // 'message' 事件到配对端口（spec 为 task；sandbox 经 execute 末 microtask checkpoint 派发，下 execute
  // 可读）。onmessage 属性 setter 内部走 addEventListener('message')。**已知限制**：无 transfer 列表
  //（Transferable 移植，罕见用法）；同执行上下文端口对（跨 worker/进程通信需 host 接线，defer）。
  function MessagePort() {
    this._et_listeners = {}; // EventTarget 内部 listener map（构造器未自动调，手动初始化）
    this._other = null; // 配对端口（MessageChannel 构造时互连）
    this._closed = false;
    this._onmessage = null;
  }
  MessagePort.prototype = Object.create(EventTarget.prototype);
  MessagePort.prototype.constructor = MessagePort;
  MessagePort.prototype.postMessage = function (message) {
    if (this._closed || !this._other) return;
    var data = typeof structuredClone === 'function' ? structuredClone(message) : message;
    var other = this._other;
    if (typeof queueMicrotask === 'function') {
      queueMicrotask(function () {
        if (other._closed) return;
        other.dispatchEvent(new MessageEvent('message', { data: data, origin: '' }));
      });
    }
  };
  MessagePort.prototype.start = function () {}; // 始终 active（polyfill 简化）
  MessagePort.prototype.close = function () {
    this._closed = true;
    if (this._other) this._other._other = null; // 断开配对
    this._other = null;
  };
  Object.defineProperty(MessagePort.prototype, 'onmessage', {
    configurable: true,
    enumerable: true,
    get: function () { return this._onmessage || null; },
    set: function (cb) {
      if (this._onmessage) this.removeEventListener('message', this._onmessage);
      if (typeof cb === 'function') {
        this._onmessage = cb;
        this.addEventListener('message', cb);
      } else {
        this._onmessage = null;
      }
    },
  });
  globalThis.MessagePort = globalThis.MessagePort || MessagePort;

  // MessageChannel——双端口通信通道（port1/port2 互连，postMessage 经异步 message 事件派发到对端）。
  function MessageChannel() {
    if (!(this instanceof MessageChannel)) return new MessageChannel();
    var p1 = new MessagePort();
    var p2 = new MessagePort();
    p1._other = p2;
    p2._other = p1;
    this.port1 = p1;
    this.port2 = p2;
  }
  globalThis.MessageChannel = globalThis.MessageChannel || MessageChannel;

  // BroadcastChannel——同源广播通信（所有同名 channel 实例互收消息，**发送者不收自己**）。extends
  // EventTarget（R2779）。postMessage 经 structuredClone（R2773）深拷贝 + queueMicrotask（R2782 同款异步
  // 派发）到所有同名其他实例。注册表 `_bc_registry`（name → channel 数组）同 JS 上下文共享。**已知限制**：
  // 仅同 JS 上下文广播（跨 worker/进程广播需 host 接线，defer）；sender 不收自己（spec 一致）。
  var _bc_registry = {};
  function BroadcastChannel(name) {
    if (!(this instanceof BroadcastChannel)) return new BroadcastChannel(name);
    this._et_listeners = {}; // EventTarget 内部 listener map（构造器未自动调，手动初始化）
    this._name = String(name);
    this._closed = false;
    this._onmessage = null;
    (_bc_registry[this._name] = _bc_registry[this._name] || []).push(this);
  }
  BroadcastChannel.prototype = Object.create(EventTarget.prototype);
  BroadcastChannel.prototype.constructor = BroadcastChannel;
  Object.defineProperty(BroadcastChannel.prototype, 'name', {
    configurable: true,
    enumerable: true,
    get: function () { return this._name; },
  });
  BroadcastChannel.prototype.postMessage = function (message) {
    if (this._closed) return;
    var data = typeof structuredClone === 'function' ? structuredClone(message) : message;
    var sender = this;
    var name = this._name;
    if (typeof queueMicrotask === 'function') {
      queueMicrotask(function () {
        var peers = _bc_registry[name];
        if (!peers) return;
        peers = peers.slice();
        for (var i = 0; i < peers.length; i++) {
          var p = peers[i];
          if (p === sender || p._closed) continue;
          p.dispatchEvent(new MessageEvent('message', { data: data, origin: '' }));
        }
      });
    }
  };
  BroadcastChannel.prototype.close = function () {
    if (this._closed) return;
    this._closed = true;
    var peers = _bc_registry[this._name];
    if (peers) {
      var i = peers.indexOf(this);
      if (i >= 0) peers.splice(i, 1);
      if (peers.length === 0) delete _bc_registry[this._name];
    }
  };
  Object.defineProperty(BroadcastChannel.prototype, 'onmessage', {
    configurable: true,
    enumerable: true,
    get: function () { return this._onmessage || null; },
    set: function (cb) {
      if (this._onmessage) this.removeEventListener('message', this._onmessage);
      if (typeof cb === 'function') {
        this._onmessage = cb;
        this.addEventListener('message', cb);
      } else {
        this._onmessage = null;
      }
    },
  });
  globalThis.BroadcastChannel = globalThis.BroadcastChannel || BroadcastChannel;

  // CSS——CSS 命名空间（escape 选择器转义 + supports 特性检测）。escape 纯 JS（CSSOM escape 算法，
  // 本地 Chromium 150 oracle 锚定）；supports 委托 host `__zw_css_supports`（known-property gate +
  // apply，两参声明 / 单参条件 not/括号/声明）。**已知限制**：supports 的 and/or 深嵌套未实现
  //（罕见，单声明/not/括号覆盖主流）；supports 语义近似「ZW 能 apply」（偏乐观）。
  globalThis.CSS = globalThis.CSS || {
    escape: function (str) {
      var s = String(str);
      var out = '';
      for (var i = 0; i < s.length; i++) {
        var c = s.charAt(i);
        var code = s.charCodeAt(i);
        var isIdent = (code >= 0x30 && code <= 0x39) // 0-9
          || (code >= 0x41 && code <= 0x5a) || (code >= 0x61 && code <= 0x7a) // A-Z a-z
          || c === '_' || c === '-' || code >= 0x80; // _ - 非 ASCII
        if (i === 0 && code >= 0x30 && code <= 0x39) {
          out += '\\' + code.toString(16) + ' '; // 首字符数字 → \hex + 空格（终止 hex 转义）
        } else if (i === 0 && c === '-' && (s.length === 1 || (s.charCodeAt(1) >= 0x30 && s.charCodeAt(1) <= 0x39))) {
          out += '\\-'; // 首字符 - 且后随数字（或仅 -）→ \-
        } else if (code < 0x20 || code === 0x7f) {
          out += '\\' + code.toString(16) + ' '; // 控制字符 → \hex + 空格
        } else if (isIdent) {
          out += c;
        } else {
          out += '\\' + c; // 特殊字符 → \char
        }
      }
      return out;
    },
    supports: function (prop, value) {
      if (typeof __zw_css_supports !== 'function') return false;
      if (arguments.length >= 2) return __zw_css_supports(String(prop), String(value)) === '1';
      return __zw_css_supports(String(prop)) === '1';
    },
  };

  // document.cookie 的 in-JS 存储（name → value）。document.cookie setter 写入，getter 序列化。
  // 不接真 cookie jar（host-layer defer）；per-上下文（无 origin 隔离）。
  var _doc_cookies = {};

  // document.title 缓存。null = 未初始化（惰性读 <title> 文本）；string = 显式 set 或已读。
  // getter 首访读 document.querySelector('title').textContent（空白折叠）；setter 仅更新缓存。
  var _doc_title = null;

  // document.activeElement 焦点追踪。null = 无焦点（activeElement 回落 body）；非空 = 焦点元素 key
  //（_elKey(sel,handle)）。focus()/blur() 经 Proxy get trap 操作。纯状态追踪，无真输入焦点/无事件派发。
  var _activeElKey = null;

  // NodeFilter 常量（spec）——createTreeWalker/createNodeIterator 的 whatToShow 掩码 + acceptNode 返回值。
  globalThis.NodeFilter = globalThis.NodeFilter || {
    SHOW_ALL: 0xFFFFFFFF,
    SHOW_ELEMENT: 0x1,
    SHOW_TEXT: 0x4,
    SHOW_CDATA_SECTION: 0x8,
    SHOW_PROCESSING_INSTRUCTION: 0x10,
    SHOW_COMMENT: 0x80,
    SHOW_DOCUMENT: 0x100,
    SHOW_DOCUMENT_TYPE: 0x200,
    SHOW_DOCUMENT_FRAGMENT: 0x400,
    FILTER_ACCEPT: 1,
    FILTER_REJECT: 2,
    FILTER_SKIP: 3,
    acceptNode: function () { return 1; }
  };

  // 内部：构造 TreeWalker/NodeIterator 共用的节点遍历器（R2803）。**eager pre-order** 经 `childNodes`
  // 递归收集子树（element 子为 selector-based proxy 可递归；文本/注释为静态叶节点），按 whatToShow 掩码 +
  // acceptNode 过滤。nextNode/previousNode 在过滤后序列上游走。TreeWalker 与 NodeIterator 共用（接口同）。
  // **已知限制**：① eager（非 lazy，spec TreeWalker 惰性——小树无碍，结果序一致）；② currentNode setter
  // 不重置游标（spec 应从 currentNode 续遍历）；③ 无 live/detach（NodeIterator 移除节点 detach defer）。
  function _makeNodeWalker(root, whatToShow, filter) {
    var wts = (whatToShow == null) ? 0xFFFFFFFF : (whatToShow | 0);
    var filterFn = null;
    if (typeof filter === 'function') filterFn = filter;
    else if (filter && typeof filter.acceptNode === 'function') filterFn = filter.acceptNode;
    function maskFor(node) {
      var nt = node && node.nodeType;
      // proxy 树仅含 element(1)/text(3)/comment(8)；其他 nodeType 不展示。
      return nt === 1 ? 0x1 : nt === 3 ? 0x4 : nt === 8 ? 0x80 : 0;
    }
    function check(node) {
      if ((wts & maskFor(node)) === 0) return 3; // 不在 whatToShow → SKIP（不入列，但仍遍历子树）
      if (!filterFn) return 1; // 无 filter → ACCEPT
      try { return filterFn(node) | 0; } catch (_e) { return 1; }
    }
    var accepted = [];
    // 深度优先 pre-order：ACCEPT/SKIP 入子树，REJECT 剪子树。
    function walk(node) {
      if (!node) return;
      var r = check(node);
      if (r === 1) accepted.push(node);
      if (r !== 2 && node.childNodes) {
        var kids = node.childNodes;
        for (var i = 0; i < kids.length; i++) walk(kids[i]);
      }
    }
    walk(root);
    var idx = -1;
    return {
      root: root,
      whatToShow: wts,
      filter: filter || null,
      currentNode: root,
      nextNode: function () {
        if (idx < accepted.length - 1) { idx++; this.currentNode = accepted[idx]; return accepted[idx]; }
        return null;
      },
      previousNode: function () {
        if (idx > 0) { idx--; this.currentNode = accepted[idx]; return accepted[idx]; }
        return null;
      }
    };
  }

  // CSSStyleSheet（R2808 读 / R2809 写 / R2810 per-rule style）——`<style>` 元素的样式表。cssRules 惰性经
  // host `__zw_style_rules`（解析 `<style>` 文本→StyleRule 序列化 \x1f/\x1e wire）→ CSSRule 数组（client cache）。
  // insertRule/deleteRule（R2809）：维护 client cache（同步读回真值）+ 从 cache 重建 `<style>` 文本经
  // `__zw_set_text` 写回（写源→下次 render 重解析 cascade；视觉生效异步，JS 契约同步）。
  // CSSRule.style（R2810）：per-rule CSSStyleDeclaration，backed by 规则声明块，mutation 同样 flush 写回。
  // **已知限制**：① 视觉生效于下次 render（写源 SetText 入队，cascade 异步）；② 仅 `<style>`（`<link>` defer 网络）；
  // ③ 每次访问 styleSheets 重新查询（live DOM，非缓存）；④ insertRule ruleText 仅按首 `{` 切分（best-effort）。
  // CSS Declaration 块文本（`prop: value; prop2: value2`）→ 有序 [{name, value}]。name 归一小写。
  // 供 [`_makeRuleStyle`] 解析 rule.cssText body 与 style.cssText 整体写。
  function _parseDeclarations(text) {
    var decls = [];
    var segs = String(text == null ? '' : text).split(';');
    for (var i = 0; i < segs.length; i++) {
      var seg = segs[i];
      var c = seg.indexOf(':');
      if (c < 0) continue;
      var name = seg.slice(0, c).trim();
      var val = seg.slice(c + 1).trim();
      if (name) decls.push({ name: name.toLowerCase(), value: val });
    }
    return decls;
  }

  // CSSRule.style per-rule CSSStyleDeclaration（R2810）——backed by 规则声明块（从 rule.cssText 的 `{ ... }`
  // body 解析为有序 declarations）。per-property get/set（camelCase↔kebab，复用 `_stylePropName`）+
  // getPropertyValue/setProperty/removeProperty + cssText 整体读写 + item/length 枚举。任一 mutation →
  // 重建 body → 更新 rule.cssText（selectorText 不变）→ 触发 parentSheet flushToOwner（复用 R2809 写回
  // `<style>` 源）。**已知限制**：① 视觉生效于下次 render（flush 写源→cascade 异步，同 R2809）；
  // ② `!important` 并入 value（getPropertyValue 含 '!important'、getPropertyPriority 返 ''，同 element.style
  // 既有简化）；③ 仅 type===1 StyleRule（@-rule 无 style）；④ set 空串 = remove（spec 一致，避免 emit `prop: `）。
  function _makeRuleStyle(rule, flushFn) {
    var bodyText = function () {
      var t = rule.cssText || '';
      var lo = t.indexOf('{');
      var hi = t.lastIndexOf('}');
      return lo >= 0 ? t.slice(lo + 1, hi >= 0 ? hi : t.length) : t;
    };
    var decls = _parseDeclarations(bodyText());
    function findIdx(name) {
      var want = String(name).toLowerCase();
      for (var i = 0; i < decls.length; i++) if (decls[i].name === want) return i;
      return -1;
    }
    function declsText() {
      return decls.map(function (d) { return d.name + ': ' + d.value; }).join('; ');
    }
    function rebuild() {
      var sel = rule.selectorText != null ? rule.selectorText : '';
      rule.cssText = sel + ' { ' + declsText() + ' }';
      if (typeof flushFn === 'function') { try { flushFn(); } catch (_e) {} }
    }
    function readProp(name) {
      var i = findIdx(_stylePropName(name));
      return i >= 0 ? decls[i].value : '';
    }
    function setProp(name, value) {
      var prop = _stylePropName(name).toLowerCase();
      var v = String(value == null ? '' : value).trim();
      var idx = findIdx(prop);
      if (v === '') { // 空串 = remove（spec 一致）
        if (idx >= 0) { decls.splice(idx, 1); rebuild(); }
        return;
      }
      if (idx >= 0) decls[idx].value = v;
      else decls.push({ name: prop, value: v });
      rebuild();
    }
    function removeProp(name) {
      var prop = _stylePropName(name).toLowerCase();
      var i = findIdx(prop);
      if (i < 0) return '';
      var prev = decls[i].value;
      decls.splice(i, 1);
      rebuild();
      return prev;
    }
    return new Proxy({}, {
      get: function (_t, p) {
        var ps = String(p);
        if (ps === 'cssText') return declsText();
        if (ps === 'length') return decls.length;
        if (ps === 'getPropertyValue') return function (name) { return readProp(name); };
        if (ps === 'getPropertyPriority') return function () { return ''; };
        if (ps === 'setProperty') return function (name, value) { setProp(name, value); return undefined; };
        if (ps === 'removeProperty') return function (name) { return removeProp(name); };
        if (ps === 'item') return function (i) { var d = decls[i | 0]; return d ? d.name : ''; };
        return readProp(ps);
      },
      set: function (_t, p, v) {
        var ps = String(p);
        if (ps === 'cssText') { decls = _parseDeclarations(String(v == null ? '' : v)); rebuild(); return true; }
        setProp(ps, v);
        return true;
      }
    });
  }

  function _ruleFromText(text, parentSheet, flushFn) {
    var t = String(text == null ? '' : text).trim();
    var brace = t.indexOf('{');
    var rule;
    if (brace >= 0) {
      var s = t.slice(0, brace).trim();
      var body = t.slice(brace + 1).replace(/}\s*$/, '').trim();
      rule = { type: 1, selectorText: s, cssText: s + ' { ' + body + ' }', style: null, parentStyleSheet: parentSheet };
    } else {
      rule = { type: 1, selectorText: t, cssText: t + ' { }', style: null, parentStyleSheet: parentSheet };
    }
    rule.style = _makeRuleStyle(rule, flushFn);
    return rule;
  }
  function _makeStyleSheet(owner) {
    var sel = owner && owner.__zwSelector;
    var rulesCache = null;
    function getRules() {
      if (rulesCache) return rulesCache;
      rulesCache = [];
      if (sel && typeof __zw_style_rules === 'function') {
        try {
          var wire = String(__zw_style_rules(sel));
          if (wire) {
            var entries = wire.split('\x1f');
            for (var i = 0; i < entries.length; i++) {
              var parts = entries[i].split('\x1e');
              if (parts.length >= 2) {
                var r = {
                  type: 1, // CSSRule.STYLE_RULE
                  selectorText: parts[0],
                  cssText: parts[1],
                  style: null, // 由 _makeRuleStyle 填（per-rule CSSStyleDeclaration，R2810）
                  parentStyleSheet: ss
                };
                r.style = _makeRuleStyle(r, flushToOwner);
                rulesCache.push(r);
              }
            }
          }
        } catch (_e) { rulesCache = []; }
      }
      return rulesCache;
    }
    // 从 cache 重建 `<style>` 文本（join cssText）+ 写回 owner 元素（下次 render 重解析 cascade）。
    function flushToOwner() {
      if (!sel || typeof __zw_set_text !== 'function') return;
      var text = getRules().map(function (r) { return r.cssText; }).join('\n');
      try { __zw_set_text(sel, text); } catch (_e) {}
    }
    var ss = {
      type: 'text/css',
      href: null,
      ownerNode: owner,
      owningElement: owner,
      disabled: false,
      title: '',
      parentStyleSheet: null,
      get cssRules() { return getRules(); },
      get rules() { return getRules(); },
      // insertRule(ruleText, index?)：splice 新规则入 cache + flush 重建 `<style>` 文本；返插入 index。
      insertRule: function (ruleText, index) {
        getRules(); // 确保从 host 填充 cache（若未读）
        var rule = _ruleFromText(ruleText, ss, flushToOwner);
        var idx = (index == null) ? rulesCache.length : (index | 0);
        if (idx < 0) idx = 0;
        if (idx > rulesCache.length) idx = rulesCache.length;
        rulesCache.splice(idx, 0, rule);
        flushToOwner();
        return idx;
      },
      // deleteRule(index)：移除 cache[index] + flush 重建。
      deleteRule: function (index) {
        getRules();
        var idx = (index | 0);
        if (idx >= 0 && idx < rulesCache.length) {
          rulesCache.splice(idx, 1);
          flushToOwner();
        }
      },
      // IE legacy 别名（addRule 返回 -1 = 失败 marker；CSS-in-JS 罕用，stub）。
      addRule: function () { return -1; },
      removeRule: function () {}
    };
    return ss;
  }

  globalThis.document = {
    querySelector: function(sel) {
      var hit = __zw_query_match(sel);
      return hit ? _wrapSelector(hit) : null;
    },
    getElementById: function(id) {
      return globalThis.document.querySelector('#' + id);
    },
    querySelectorAll: function(sel) {
      var all = __zw_query_all(sel);
      if (!all) return [];
      return all.split('|').filter(Boolean).map(_wrapSelector);
    },
    getElementsByClassName: function(cls) {
      return globalThis.document.querySelectorAll('.' + cls);
    },
    getElementsByTagName: function(tag) {
      return globalThis.document.querySelectorAll(tag);
    },
    createElement: function(tag) {
      tag = String(tag);
      if (tag.toLowerCase() === 'canvas') return _zwMakeCanvas();
      var handle = __zw_create_element(tag);
      return _wrapHandle(handle);
    },
    // `createElementNS(ns, tag)`：HTML 命名空间元素与 createElement 等价；
    // SVG 命名空间元素（filter/cursor 等）在本目标范围外，按通用元素创建（不渲染
    // 为 SVG 但避免 ReferenceError 中断脚本，crashtest 尤其依赖不抛）。
    createElementNS: function(_ns, tag) {
      var handle = __zw_create_element(String(tag));
      return _wrapHandle(handle);
    },
    createTextNode: function(text) {
      var handle = __zw_create_text(String(text));
      if (handle) _textHandles[handle] = true;
      return _wrapHandle(handle);
    },
    // `document.createComment(text)`——注释节点（nodeType 8，框架 placeholder/anchor 高频）。镜像 createTextNode，
    // 经 host `__zw_create_comment`（apply 时 doc.create_comment）。textContent/data/nodeValue 读回注释内容。
    createComment: function(text) {
      var handle = (typeof __zw_create_comment === 'function')
        ? __zw_create_comment(String(text)) : __zw_create_text(String(text));
      if (handle) _commentHandles[handle] = true;
      return _wrapHandle(handle);
    },
    // `document.createEvent(type)`——legacy 合成事件工厂（jQuery<3 / 旧库 / 分析脚本高频）。返空 type 事件，
    // 经 initEvent/initCustomEvent 填充后 dispatchEvent。type 大小写不敏感 + spec 别名（custom↔CustomEvent）；
    // 已知 Event 子类 type→对应构造器（R2779 / R2811 / R2812）；未知回落 Event（lenient，spec 抛
    // NotSupportedError——本沙箱不抛，避免中断脚本）。
    createEvent: function(type) {
      var t = String(type == null ? '' : type).toLowerCase();
      var map = {
        customevent: globalThis.CustomEvent, custom: globalThis.CustomEvent,
        keyboardevent: globalThis.KeyboardEvent,
        mouseevent: globalThis.MouseEvent,
        uievent: globalThis.UIEvent,
        focusevent: globalThis.FocusEvent,
        wheelevent: globalThis.WheelEvent,
        pointerevent: globalThis.PointerEvent,
        inputevent: globalThis.InputEvent,
        hashchangeevent: globalThis.HashChangeEvent,
        popstateevent: globalThis.PopStateEvent,
        storageevent: globalThis.StorageEvent,
        progressevent: globalThis.ProgressEvent,
        transitionevent: globalThis.TransitionEvent,
        animationevent: globalThis.AnimationEvent,
      };
      var Ctor = (map[t] && typeof map[t] === 'function') ? map[t] : globalThis.Event;
      // 构造器接收 (type, options)；createEvent 返**空 type** 事件（initEvent/initCustomEvent 设 type）。
      return new Ctor('');
    },
    // execCommand / queryCommand*（R2826）——legacy 编辑/剪贴板命令表面（旧 copy 按钮
    // `el.select(); document.execCommand('copy')` / clipboard.js feature-detect `queryCommandSupported('copy')`
    // / contentEditable 编辑器 format 命令）。headless 无真剪贴板/格式化 → permissive stub：
    // execCommand 返 true（copy/cut 不真写剪贴板——modern 路径走 navigator.clipboard；format 命令不真应用）；
    // queryCommandSupported/Enabled 返 true（feature-detect 通过）；queryCommandValue 返 ''。
    execCommand: function (_commandId, _showUI, _value) { return true; },
    queryCommandSupported: function (_commandId) { return true; },
    queryCommandEnabled: function (_commandId) { return true; },
    queryCommandValue: function (_commandId) { return ''; },
    // `document.createTreeWalker(root, whatToShow, filter)` / `createNodeIterator(...)`——DOM 子树遍历器
    //（库 / sanitizer / a11y tree walker 高频）。whatToShow 掩码 + acceptNode FILTER_ACCEPT/REJECT/SKIP。
    // 经 `_makeNodeWalker`（eager pre-order via childNodes 递归）。两者共用工厂（接口同：nextNode/previousNode）。
    createTreeWalker: function (root, whatToShow, filter) {
      return _makeNodeWalker(root, whatToShow, filter);
    },
    createNodeIterator: function (root, whatToShow, filter) {
      return _makeNodeWalker(root, whatToShow, filter);
    },
    // `document.createRange()`——新建 Range（R2804，Selection/Range）。详见 `_makeRange`。
    createRange: function () {
      return _makeRange();
    },
    // `document.createDocumentFragment()`：DocumentFragment（nodeType 11，轻量容器）。
    // 建 fragment（append 子节点经既有 append_child_handle）+ 标记 handle 到 _fragmentHandles
    //（供 nodeType=11 与 append 时 flatten 检测）。
    createDocumentFragment: function() {
      if (typeof __zw_create_document_fragment !== 'function') return _wrapHandle('');
      var handle = __zw_create_document_fragment();
      if (handle) _fragmentHandles[handle] = true;
      return _wrapHandle(handle);
    },
    // `document.adoptNode(node)`（R2818）——跨文档收养。单文档沙箱 → identity no-op（spec：同文档 adopt
    // 返节点自身）。返节点（不抛，feature-detection / 库跨文档逻辑兼容）。
    adoptNode: function(node) { return node; },
    // `document.importNode(node, deep?)`（R2818）——跨文档导入（克隆）。委托 `node.cloneNode(deep)`
    //（复用既有 clone 机制——建副本 + 复制属性 + deep 时复制子树）。无 cloneNode（非元素/detached）→ 返 node。
    importNode: function(node, deep) {
      return node && typeof node.cloneNode === 'function' ? node.cloneNode(!!deep) : node;
    },
    // `document.implementation`（DOMImplementation，R2815）——feature-detection（jQuery support 等查 hasFeature）
    // + createDocument/createHTMLDocument（返最小 hollow detached Document）。**已知限制**：detached tree 无
    // proxy infra，querySelector 返 null（jQuery/DOMPurify 真 detached 解析需后续 detached-tree slice）。
    implementation: {
      hasFeature: function() { return true; }, // spec：deprecated，恒返 true
      createDocument: function() { return _makeDetachedDocument(''); },
      createHTMLDocument: function(title) { return _makeDetachedDocument(title); },
      createDocumentType: function() { return null; },
    },
    documentElement: _wrapSelector('html'),
    body: _wrapSelector('body'),
    head: _wrapSelector('head'),
    // node-level 身份与连入态（Document 节点恒 connected + 恒有 documentElement 子）。`document.nodeType`
    // =9 / nodeName='#document'（Node 接口常查 `node.nodeType === 9` / `=== Node.DOCUMENT_NODE`）。
    nodeType: 9,
    nodeName: '#document',
    isConnected: true,
    hasChildNodes: function () { return true; },
    compatMode: 'CSS1Compat',
    characterSet: 'UTF-8',
    charset: 'UTF-8',
    contentType: 'text/html',
    readyState: 'complete',
    // fullscreen（R2817）——headless 无真全屏：fullscreenElement 恒 null，exitFullscreen 返 resolving Promise。
    fullscreenElement: null,
    fullscreenEnabled: true,
    exitFullscreen: function() { return Promise.resolve(undefined); },
    // document.title——getter 返首 <title> 文本（空白折叠，spec 一致）；首访惰性读 querySelector('title')
    // 并缓存；setter 更新缓存。**已知限制**：① setter 仅更新 in-JS 缓存，不写回 host DOM <title>（快照 proxy
    // 只读，无 head/title 建链）；② 不创建 <head><title>（spec 无 head 时应建——本沙箱无渲染 title 需求）。
    get title() {
      if (_doc_title !== null) return _doc_title;
      var t = null;
      try { t = globalThis.document.querySelector('title'); } catch (e) { t = null; }
      _doc_title = t && t.textContent ? String(t.textContent).replace(/\s+/g, ' ').trim() : '';
      return _doc_title;
    },
    set title(v) {
      _doc_title = v == null ? '' : String(v);
    },
    // document.URL / documentURI = 页面 URL（= location.href）；referrer = ''（无 referrer 追踪，
    // net-layer defer；standalone 渲染/reftest 无来源页，spec 空串可接受）。
    get URL() { return globalThis.location ? globalThis.location.href : ''; },
    get documentURI() { return globalThis.location ? globalThis.location.href : ''; },
    get referrer() { return ''; },
    // document.activeElement——当前焦点元素（focus()/blur() 操作 _activeElKey）；无焦点回落 body（spec）。
    get activeElement() {
      if (_activeElKey && _proxyCache[_activeElKey]) return _proxyCache[_activeElKey];
      return globalThis.document.body;
    },
    // Page Visibility + 焦点状态（R2824）——headless 页面恒「可见 + 已聚焦」。hidden=false /
    // visibilityState='visible' / hasFocus()=true（analytics/RUM 高频：GA 读 visibilityState/hidden，
    // hasFocus gate 操作；visibilitychange 事件 addEventListener 注册有效但永不触发——headless 无
    // 可见性变化源，documented）。webkit 前缀（legacy analytics / 旧 GA / jQuery 插件 feature-detect
    // `document.webkitHidden || document.hidden`）。
    get hidden() { return false; },
    get visibilityState() { return 'visible'; },
    webkitHidden: false,
    webkitVisibilityState: 'visible',
    hasFocus: function () { return true; },
    // document.cookie——get 返 "n=v; n=v" 串（仅 name=value，无属性）；set 解析 "n=v; Path=...; Max-Age=..."
    // 取首个 name=value 存/覆盖。**已知限制**：in-JS 存储（不接真 cookie jar / 不随 fetch 发送 / 无 origin
    // 隔离 / 无 expiry 淘汰——网络/origin 集成属 host-layer defer）；set-then-read 常见模式 tractable。
    get cookie() {
      var parts = [];
      for (var k in _doc_cookies) {
        if (Object.prototype.hasOwnProperty.call(_doc_cookies, k)) parts.push(k + '=' + _doc_cookies[k]);
      }
      return parts.join('; ');
    },
    set cookie(str) {
      var s = String(str == null ? '' : str);
      var first = s.split(';')[0];
      var eq = first.indexOf('=');
      if (eq < 0) return; // 无 name=value → 忽略
      var name = first.slice(0, eq).trim();
      var value = first.slice(eq + 1);
      if (!name) return;
      _doc_cookies[name] = value;
    },
    // `document.styleSheets`（R2808）——真 backing：`<style>` 元素 → CSSStyleSheet 数组（经
    // `__zw_query_all('style')` 查询 + `_makeStyleSheet`）。每次访问重新查询（live DOM）。
    get styleSheets() {
      var sels = (typeof __zw_query_all === 'function')
        ? String(__zw_query_all('style')).split('|').filter(Boolean) : [];
      var out = [];
      for (var i = 0; i < sels.length; i++) out.push(_makeStyleSheet(_wrapSelector(sels[i])));
      return out;
    },
    forms: _liveQueryCollection('form'),
    images: _liveQueryCollection('img'),
    scripts: _liveQueryCollection('script'),
    // links = a[href] + area[href]（R2833 修正：旧 `_liveQueryCollection('a')` 返全部 `<a>` 含 name-only
    // 锚，spec 仅带 href 的 a/area）。embeds/plugins = embed + object（同 spec）；anchors = a[name]（legacy 命名锚）。
    links: _liveQueryCollection(['a[href]', 'area[href]']),
    embeds: _liveQueryCollection(['embed', 'object']),
    plugins: _liveQueryCollection(['embed', 'object']),
    anchors: _liveQueryCollection('a[name]'),
    addEventListener: function(type, fn, opts) {
      _makeProxy('html', null).addEventListener(type, fn, opts);
    },
    removeEventListener: function(type, fn, opts) {
      _makeProxy('html', null).removeEventListener(type, fn, opts);
    },
    attachEvent: function(type, fn) {
      _attachEventForKey(_elKey('html', null), type, fn);
    },
    detachEvent: function(type, fn) {
      _detachEventForKey(_elKey('html', null), type, fn);
    }
  };
  globalThis.window = globalThis;
  globalThis.addEventListener = _globalAddEventListener;
  globalThis.removeEventListener = _globalRemoveEventListener;
  globalThis.window.attachEvent = function(type, fn) {
    _attachEventForKey(_elKey('html', null), type, fn);
  };
  globalThis.window.detachEvent = function(type, fn) {
    _detachEventForKey(_elKey('html', null), type, fn);
  };
  Object.defineProperty(globalThis.document, 'defaultView', {
    get: function() { return globalThis.window; }
  });

  // Selection / Range（R2804，缺失 Web API 续）。headless 无真用户选择——Selection 单例默认空
  //（rangeCount=0/isCollapsed=true/toString=''/anchorNode=null/focusNode=null/type='None'），selection-state-
  // checking 脚本（`if (getSelection().toString()) ...`）正确跳过选择分支。programmatic Range 经 setStart/
  // setEnd/selectNode* 设边界；toString 提取选区文本（精确覆盖 selectNode*/同文本节点 setStart·setEnd）。
  var _selection = null; // Selection 单例（惰性建，getSelection 返同一对象，spec 一致）

  // 递归收集 node 子树内文本节点 data（文档序，经 childNodes——element 子可递归，文本为静态叶）。
  function _descendantText(node, out) {
    if (!node) return;
    if (node.nodeType === 3 || node.__zwIsText) { out.push(node.nodeValue || ''); return; }
    var kids = node.childNodes;
    if (kids && kids.length) { for (var i = 0; i < kids.length; i++) _descendantText(kids[i], out); }
  }

  // 构造 Range（document.createRange / selectNode* 等用）。**已知限制**：① toString 精确覆盖 selectNode/
  // selectNodeContents（整节点子树文本）+ 同文本节点 setStart/setEnd（slice 偏移）；其余 setStart/setEnd
  // 组合 best-effort 取 commonAncestor 子树文本（跨节点偏移不精确截取）；② cloneContents best-effort 仅文本
  // 节点（DOM 克隆 defer）；③ deleteContents/extractContents/insertNode/surroundContents defer（DOM 变更复杂）；
  // ④ getBoundingClientRect/getClientRects 返空（无 layout 选择几何）；⑤ 无真 live（proxy 快照）。
  function _makeRange() {
    return {
      startContainer: null, startOffset: 0, endContainer: null, endOffset: 0,
      commonAncestorContainer: null, collapsed: true, _mode: null,
      setStart: function (node, off) { this.startContainer = node; this.startOffset = off | 0; this._recalc(); return this; },
      setEnd: function (node, off) { this.endContainer = node; this.endOffset = off | 0; this._recalc(); return this; },
      setStartBefore: function (node) { var p = node && node.parentNode; return p ? this.setStart(p, this._indexOf(p, node)) : this; },
      setStartAfter: function (node) { var p = node && node.parentNode; return p ? this.setStart(p, this._indexOf(p, node) + 1) : this; },
      setEndBefore: function (node) { var p = node && node.parentNode; return p ? this.setEnd(p, this._indexOf(p, node)) : this; },
      setEndAfter: function (node) { var p = node && node.parentNode; return p ? this.setEnd(p, this._indexOf(p, node) + 1) : this; },
      selectNode: function (node) {
        var p = (node && node.parentNode) || node;
        var i = this._indexOf(p, node);
        this.startContainer = p; this.startOffset = i;
        this.endContainer = p; this.endOffset = i + 1;
        this.commonAncestorContainer = p; this.collapsed = false; this._mode = { node: node, kind: 'node' };
        return this;
      },
      selectNodeContents: function (node) {
        var cnt = node && node.childNodes ? node.childNodes.length : 0;
        this.startContainer = node; this.startOffset = 0;
        this.endContainer = node; this.endOffset = cnt;
        this.commonAncestorContainer = node; this.collapsed = cnt === 0; this._mode = { node: node, kind: 'contents' };
        return this;
      },
      collapse: function (toStart) {
        if (toStart) { this.endContainer = this.startContainer; this.endOffset = this.startOffset; }
        else { this.startContainer = this.endContainer; this.startOffset = this.endOffset; }
        this.collapsed = true; this._mode = null; return this;
      },
      _indexOf: function (parent, node) {
        var kids = parent && parent.childNodes;
        if (!kids) return 0;
        for (var i = 0; i < kids.length; i++) if (kids[i] === node) return i;
        return 0;
      },
      _recalc: function () {
        this._mode = null;
        this.collapsed = (this.startContainer === this.endContainer && this.startOffset === this.endOffset);
        this.commonAncestorContainer = this.startContainer; // best-effort（spec 须最近共同祖先）
      },
      toString: function () {
        // 精确：selectNode/selectNodeContents → 整节点子树文本。
        if (this._mode) { var out = []; _descendantText(this._mode.node, out); return out.join(''); }
        // 精确：同文本节点 setStart/setEnd → slice 偏移。
        if (this.startContainer && this.startContainer === this.endContainer &&
            (this.startContainer.nodeType === 3 || this.startContainer.__zwIsText)) {
          var v = this.startContainer.nodeValue || '';
          var a = Math.min(this.startOffset, this.endOffset);
          var b = Math.max(this.startOffset, this.endOffset);
          return v.slice(a, b);
        }
        // best-effort：取 commonAncestor 子树文本（跨节点偏移不精确截取）。
        if (this.commonAncestorContainer) { var o2 = []; _descendantText(this.commonAncestorContainer, o2); return o2.join(''); }
        return '';
      },
      cloneContents: function () {
        // best-effort DocumentFragment：仅含选区文本（一个文本节点）。DOM 克隆 defer。
        var f = globalThis.document.createDocumentFragment();
        var t = this.toString();
        if (t) f.appendChild(globalThis.document.createTextNode(t));
        return f;
      },
      getBoundingClientRect: function () { return { top: 0, left: 0, right: 0, bottom: 0, width: 0, height: 0, x: 0, y: 0 }; },
      getClientRects: function () { return []; }
    };
  }

  // Selection 单例工厂。addRange 简化为单 range（多 range 仅 Firefox，主流单 range）。
  function _getSelection() {
    if (_selection) return _selection;
    _selection = {
      _ranges: [],
      get rangeCount() { return this._ranges.length; },
      get isCollapsed() { return this._ranges.length === 0 || this._ranges.every(function (r) { return r.collapsed; }); },
      get anchorNode() { return this._ranges[0] ? this._ranges[0].startContainer : null; },
      get anchorOffset() { return this._ranges[0] ? this._ranges[0].startOffset : 0; },
      get focusNode() { return this._ranges[0] ? this._ranges[0].endContainer : null; },
      get focusOffset() { return this._ranges[0] ? this._ranges[0].endOffset : 0; },
      get type() { return this._ranges.length === 0 ? 'None' : (this.isCollapsed ? 'Caret' : 'Range'); },
      toString: function () { return this._ranges.map(function (r) { return r.toString(); }).join(''); },
      getRangeAt: function (i) { return this._ranges[i | 0] || null; },
      removeAllRanges: function () { this._ranges = []; },
      empty: function () { this._ranges = []; },
      removeRange: function (range) { this._ranges = this._ranges.filter(function (r) { return r !== range; }); },
      addRange: function (range) { this._ranges = [range]; /* 多 range（FF）简化为单 */ },
      collapse: function (node, off) {
        if (!node) { this._ranges = []; return; }
        var r = _makeRange(); r.setStart(node, off | 0); r.collapse(true);
        this._ranges = [r];
      },
      collapseToStart: function () { if (this._ranges[0]) { this._ranges[0].collapse(true); } },
      collapseToEnd: function () { if (this._ranges[0]) { this._ranges[0].collapse(false); } },
      extend: function (node, off) { if (this._ranges[0]) { this._ranges[0].setEnd(node, off | 0); } },
      containsNode: function () { return false; } // best-effort（无真选择几何）
    };
    return _selection;
  }
  globalThis.getSelection = _getSelection;
  globalThis.Selection = function Selection() {};
  globalThis.Range = function Range() {};

  // HTML 规范「Window 上的命名属性访问」：带 id 的元素应作为全局变量可访问
  // （`<div id="container">…</div>` → JS `container.appendChild(...)`）。动态 reftest
  // 普遍用裸标识符引用元素（257 个 reftest 文件），缺失则抛 ReferenceError 中断脚本。
  // 仅安装合法标识符 id；不覆盖已存在全局（避免 shadow `document`/`window` 等真实 global）。
  function _installNamedAccess() {
    try {
      var ids = __zw_collect_ids();
      if (!ids) return;
      ids.split('|').forEach(function(id) {
        if (!id || !/^[A-Za-z_$][A-Za-z0-9_$]*$/.test(id)) return;
        if (globalThis[id] !== undefined) return;
        var el = globalThis.document.getElementById(id);
        if (el) globalThis[id] = el;
      });
    } catch (_e) {}
  }
  _installNamedAccess();

  globalThis.__zw_dispatch_event = function(sel, type, detail) {
    var ev;
    if (detail && (detail.key || detail.code)) {
      ev = new KeyboardEvent(type, {
        bubbles: true,
        cancelable: true,
        key: detail.key || '',
        code: detail.code || detail.key || ''
      });
    } else {
      ev = _makeEvent(type, { bubbles: true, cancelable: true });
    }
    var ok = _dispatchWithBubble(_elKey(sel, null), sel, null, ev);
    return ok ? 'ok' : 'prevented';
  };
})();
