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

  // ── 浏览器运行时桩（定时器、navigator、location 等）──
  var _timerId = 1;
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
      var rec = {};
      for (var k in baseRecord) rec[k] = baseRecord[k];
      rec.target = obs._targetProxies[id];
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
  // 读 target/root 的 rect（复用 gBCR）；sel 空 / handler 未注册 / 未命中 → 零 rect。
  function _io_rectFromSel(sel) {
    if (sel && typeof __zw_getBoundingClientRect === 'function') {
      try {
        var s = __zw_getBoundingClientRect(sel);
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
    var targetRect = _io_rectFromSel(sel);
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
        var r = _io_rectFromSel(t.proxy.__zwSelector);
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
    // 经 __zw_query_match 解析为 canonical stable selector（与 querySelector 同 identity），
    // 使派发的 input 事件命中 querySelector 注册的 listener（host 传入的 selector 与
    // querySelector 返回的 __zwSelector 须统一）。
    var resolved = typeof __zw_query_match === 'function' ? __zw_query_match(sel) : sel;
    if (!resolved) return;
    // 真实 tag（host `__zw_get_tag` 查 DOM；shim `_tagFromSel`/el.tagName 对 id-only 选择器仅启发式）。
    var tag = typeof __zw_get_tag === 'function' ? __zw_get_tag(resolved) : '';
    tag = (tag || '').toUpperCase();
    if (tag !== 'INPUT' && tag !== 'TEXTAREA') return;
    var el = _wrapSelector(resolved);
    if (!el) return;
    el.value = (el.value || '') + ch;
    var ev = _makeEvent('input', { bubbles: true, cancelable: true });
    el.dispatchEvent(ev);
  };
  // P1a form input：导航（URL 变化）时清 value 缓存——防跨页同选择器 stale value。
  globalThis.__zw_reset_form_state = function() { _inputValues = {}; };

  // 现代动态 reftest 常用模式：`requestAnimationFrame(() => requestAnimationFrame(() => { …setup…; takeScreenshot(); }))`
  // 把 DOM setup 延迟到「布局/绘制后」。harness 在脚本+load 派发后才截图，故 rAF
  // 同步立即执行回调即可让 setup mutation 被记录并应用到二次渲染（镜像 setTimeout 的 microtask 语义，
  // 但同步以保证回调在 sandbox 生命周期内必然执行）。
  globalThis.requestAnimationFrame = function(fn) {
    if (typeof fn === 'function' && _rafBudget > 0) {
      _rafBudget--;
      try { fn(0); } catch (_e) {}
    }
    return _timerId++;
  };
  globalThis.cancelAnimationFrame = function(_id) {};
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
  // （区别于 `offsetHeight` 等属性访问返回 undefined 不抛、仅值错，作 reflow 触发器
  // 无害）。JS 在渲染前执行，无真实 computed 值可返；返回空 CSSStyleDeclaration 桩
  // （任意属性访问/getPropertyValue 返 ''）不抛，让后续视觉 mutation 正常执行。
  // 返 '' 对 `if (cs.display === 'none') …` 类条件可能取错分支，但脚本本会整体中断，
  // stub 严格不劣于中断且对无条件 mutation（主流 reflow-触发模式）净正向。
  globalThis.getComputedStyle = function(_elt, _pseudo) {
    var empty = '';
    return new Proxy({}, {
      get: function(_t, prop) {
        var p = String(prop);
        if (p === 'getPropertyValue' || p === 'getPropertyPriority' || p === 'item') {
          return function() { return empty; };
        }
        if (p === 'length') return 0;
        if (p === 'parentRule') return null;
        if (p === 'cssText') return empty;
        // 其余任意 CSS 属性访问（.height/.display/.textAlign…）返空串；
        // Symbol 属性（toPrimitive/iterator 等）返 undefined 避免误触发协议。
        return typeof prop === 'string' ? empty : undefined;
      }
    });
  };

  function _emptyCollection() {
    return { length: 0, item: function() { return null; }, namedItem: function() { return null; } };
  }

  function _parseLocation(href) {
    var m = String(href || 'about:blank').match(/^([^:]+):\/\/([^\/]*)(\/[^?#]*)?(\?[^#]*)?(#.*)?$/);
    if (!m) {
      return { href: href || 'about:blank', protocol: 'about:', host: '', hostname: '', pathname: '/', search: '', hash: '', origin: 'null' };
    }
    var host = m[2] || '';
    var hostname = host.split(':')[0] || '';
    return {
      href: href,
      protocol: m[1] + ':',
      host: host,
      hostname: hostname,
      pathname: m[3] || '/',
      search: m[4] || '',
      hash: m[5] || '',
      origin: host ? m[1] + '://' + host : 'null'
    };
  }

  function _makeLocation() {
    function href() {
      return typeof __zw_get_page_url === 'function' ? __zw_get_page_url() : 'about:blank';
    }
    return {
      get href() { return href(); },
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
      toString: function() { return href(); }
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

  globalThis.history = {
    length: 1,
    state: null,
    back: function() {},
    forward: function() {},
    go: function() {},
    pushState: function() {},
    replaceState: function() {}
  };

  globalThis.Worker = function() {
  };

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
    taintEnabled: function() { return false; }
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

  globalThis.Image = function() {
    return { src: '', width: 0, height: 0, onload: null, onerror: null, onabort: null };
  };

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

  function _globalAddEventListener(type, fn, opts) {
    var key = _elKey('html', null);
    var t = String(type);
    if (!_listenerStore[key]) _listenerStore[key] = {};
    if (!_listenerStore[key][t]) _listenerStore[key][t] = [];
    _listenerStore[key][t].push({ fn: fn, capture: !!(opts && opts.capture) });
  }

  function _globalRemoveEventListener(type, fn) {
    var key = _elKey('html', null);
    var t = String(type);
    if (!_listenerStore[key] || !_listenerStore[key][t]) return;
    _listenerStore[key][t] = _listenerStore[key][t].filter(function(l) { return l.fn !== fn; });
  }

  globalThis.Node = function Node() {};
  globalThis.Element = function Element() {};
  globalThis.HTMLElement = function HTMLElement() {};
  globalThis.Node.prototype = {};
  globalThis.Element.prototype = Object.create(globalThis.Node.prototype);
  globalThis.HTMLElement.prototype = Object.create(globalThis.Element.prototype);
  globalThis.Element.prototype.addEventListener = function(type, fn, opts) {
    _globalAddEventListener(type, fn, opts);
  };
  globalThis.Element.prototype.removeEventListener = function(type, fn) {
    _globalRemoveEventListener(type, fn);
  };

  function _elKey(sel, handle) {
    return handle ? ('@' + handle) : sel;
  }

  function _classListProxy(sel, handle) {
    return {
      add: function(c) {
        var cur = handle ? __zw_get_attr_handle(handle, 'class') : __zw_get_attr(sel, 'class');
        var parts = (cur || '').split(/\s+/).filter(Boolean);
        if (parts.indexOf(c) < 0) parts.push(c);
        var v = parts.join(' ');
        if (handle) __zw_set_attr_handle(handle, 'class', v);
        else __zw_set_attr(sel, 'class', v);
      },
      remove: function(c) {
        var cur = handle ? __zw_get_attr_handle(handle, 'class') : __zw_get_attr(sel, 'class');
        var parts = (cur || '').split(/\s+/).filter(Boolean).filter(function(x) { return x !== c; });
        var v = parts.join(' ');
        if (handle) __zw_set_attr_handle(handle, 'class', v);
        else __zw_set_attr(sel, 'class', v);
      },
      toggle: function(c) {
        var cur = handle ? __zw_get_attr_handle(handle, 'class') : __zw_get_attr(sel, 'class');
        var parts = (cur || '').split(/\s+/).filter(Boolean);
        var i = parts.indexOf(c);
        if (i >= 0) parts.splice(i, 1);
        else parts.push(c);
        var v = parts.join(' ');
        if (handle) __zw_set_attr_handle(handle, 'class', v);
        else __zw_set_attr(sel, 'class', v);
      },
      contains: function(c) {
        var cur = handle ? __zw_get_attr_handle(handle, 'class') : __zw_get_attr(sel, 'class');
        return (cur || '').split(/\s+/).indexOf(c) >= 0;
      }
    };
  }

  function _dispatchToListeners(key, event) {
    var listeners = _listenerStore[key];
    if (!listeners || !listeners[event.type]) return true;
    var list = listeners[event.type];
    for (var i = 0; i < list.length; i++) {
      if (list[i].capture) {
        list[i].fn.call(event.target, event);
        if (event._immediateStopped) return !event._defaultPrevented;
      }
    }
    for (var j = 0; j < list.length; j++) {
      if (!list[j].capture) {
        list[j].fn.call(event.target, event);
        if (event._immediateStopped) return !event._defaultPrevented;
      }
    }
    return !event._defaultPrevented;
  }

  function _makeEvent(type, options) {
    var ev = {
      type: type,
      bubbles: !!(options && options.bubbles),
      cancelable: !!(options && options.cancelable),
      detail: options && options.detail,
      target: null,
      currentTarget: null,
      _defaultPrevented: false,
      _propagationStopped: false,
      _immediateStopped: false,
      preventDefault: function() { if (this.cancelable) this._defaultPrevented = true; },
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

  function _parentNodeFor(sel, handle) {
    if (sel === 'html') return null;
    if (sel === 'body' || sel === 'head') return _wrapSelector('html');
    return _wrapSelector('body');
  }

  function _liveQueryCollection(sel) {
    return new Proxy({ length: 0 }, {
      get: function(_t, prop) {
        var list = globalThis.document.querySelectorAll(sel);
        if (prop === 'length') return list.length;
        if (prop === 'item') return function(i) { return list[i] || null; };
        var idx = parseInt(prop, 10);
        if (!isNaN(idx) && String(idx) === String(prop)) return list[idx];
        return list[prop];
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
          // P1a form input：value 属性 get——per-element 缓存，lazy-init 自 value 属性。
          if (_inputValues[key] == null) {
            var va = handle ? __zw_get_attr_handle(handle, 'value') : __zw_get_attr(sel, 'value');
            _inputValues[key] = (va == null) ? '' : va;
          }
          return _inputValues[key];
        }
        if (prop === 'style') {
          return new Proxy({}, {
            set: function(_s, p, v) {
              if (handle) __zw_set_style_handle(handle, String(p), String(v));
              else __zw_set_style(sel, String(p), String(v));
              _mo_notify(sel, handle, { type: 'attributes', attributeName: 'style' });
              return true;
            },
            get: function(_s, p) {
              var raw = handle ? __zw_get_attr_handle(handle, 'style') : __zw_get_attr(sel, 'style');
              if (!raw) return '';
              var parts = raw.split(';');
              var pstr = String(p);
              for (var i = 0; i < parts.length; i++) {
                var kv = parts[i].split(':');
                if (kv[0] && kv[0].trim().toLowerCase() === pstr.toLowerCase()) {
                  return (kv[1] || '').trim();
                }
              }
              return '';
            }
          });
        }
        if (prop === 'classList') return _classListProxy(sel, handle);
        if (prop === 'className') {
          return handle ? __zw_get_attr_handle(handle, 'class') : __zw_get_attr(sel, 'class');
        }
        if (prop === 'id') {
          return handle ? __zw_get_attr_handle(handle, 'id') : __zw_get_attr(sel, 'id');
        }
        if (prop === 'textContent') {
          return handle ? __zw_get_text_handle(handle) : __zw_get_text(sel);
        }
        if (prop === 'innerHTML') {
          return handle ? __zw_get_inner_html_handle(handle) : __zw_get_inner_html(sel);
        }
        if (prop === 'parentNode' || prop === 'parentElement') {
          return _parentNodeFor(sel, handle);
        }
        if (prop === 'tagName') {
          return _tagFromSel(sel);
        }
        if (prop === 'nodeName') {
          return _tagFromSel(sel);
        }
        if (prop === 'nodeType') {
          return 1;
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
            if (handle) __zw_set_attr_handle(handle, name, String(value));
            else __zw_set_attr(sel, name, String(value));
            _mo_notify(sel, handle, { type: 'attributes', attributeName: String(name) });
          };
        }
        if (prop === 'removeAttribute') {
          return function(name) {
            if (handle) __zw_set_attr_handle(handle, name, '');
            else __zw_set_attr(sel, name, '');
            _mo_notify(sel, handle, { type: 'attributes', attributeName: String(name) });
          };
        }
        if (prop === 'addEventListener') {
          return function(type, fn, opts) {
            if (!_listenerStore[key]) _listenerStore[key] = {};
            if (!_listenerStore[key][type]) _listenerStore[key][type] = [];
            _listenerStore[key][type].push({ fn: fn, capture: !!(opts && opts.capture) });
          };
        }
        if (prop === 'removeEventListener') {
          return function(type, fn) {
            if (!_listenerStore[key] || !_listenerStore[key][type]) return;
            _listenerStore[key][type] = _listenerStore[key][type].filter(function(l) { return l.fn !== fn; });
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
            event.target = _makeProxy(sel, handle);
            return _dispatchToListeners(key, event);
          };
        }
        if (prop === 'click') {
          return function() {
            var ev = _makeEvent('click', { bubbles: true, cancelable: true });
            ev.target = _makeProxy(sel, handle);
            return _dispatchToListeners(key, ev);
          };
        }
        if (prop === 'appendChild') {
          return function(child) {
            if (child && child.__zwHandle) {
              if (handle) __zw_append_child_handle(handle, child.__zwHandle);
              else __zw_append_child(sel, child.__zwHandle);
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
              // `insertBefore(node, null)` 等价于 appendChild。
              if (refNode == null) {
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
        if (prop === 'remove') {
          return function() {
            if (handle) __zw_remove_handle(handle);
            else __zw_remove(sel);
          };
        }
        // `Element.append(...nodesOrStrings)`（现代 API，区别于 appendChild）：
        // 追加多个节点/字符串，字符串自动包成 Text 节点。复用既有 appendChild +
        // createTextNode 回调，无需新增 Rust 端 callback。
        if (prop === 'append') {
          return function() {
            var added = [];
            for (var i = 0; i < arguments.length; i++) {
              var item = arguments[i];
              if (item == null) continue;
              if (typeof item === 'object' && item.__zwHandle) {
                if (handle) __zw_append_child_handle(handle, item.__zwHandle);
                else __zw_append_child(sel, item.__zwHandle);
                added.push(item);
              } else {
                var txt = String(item);
                var tn = __zw_create_text(txt);
                if (handle) __zw_append_child_handle(handle, tn);
                else __zw_append_child(sel, tn);
                added.push({ __zwHandle: tn, __zwSelector: '' });
              }
            }
            if (added.length > 0) {
              _mo_notify(sel, handle, { type: 'childList', addedNodes: added, removedNodes: [] });
            }
            return undefined;
          };
        }
        if (prop === 'querySelector') {
          return function(q) {
            var hit = __zw_query_match(q);
            return hit ? _wrapSelector(hit) : null;
          };
        }
        if (prop === 'querySelectorAll') {
          return function(q) {
            var all = __zw_query_all(q);
            if (!all) return [];
            return all.split('|').filter(Boolean).map(_wrapSelector);
          };
        }
        // 布局测量 API：`el.getBoundingClientRect()` 返真实 DOMRect（P1a gBCR path C）。
        // selector-identity 元素（querySelector/getElementById，sel=stable_selector）→ host
        // `__zw_getBoundingClientRect(sel)` 解析 dom_html→NodeId→layout-rect snapshot 返 "x,y,w,h"。
        // host 未注册 / 未命中 / handle-identity（createElement，sel 为空）→ 零 rect（= 旧行为，零回归；
        // 作 reflow 触发器语义仍正确——返回值多被丢弃）。注：rect 反映「上次 render」（stale-but-non-zero），
        // 改样式后同脚本内即读见 pre-change rect（force-reflow-on-demand 为 follow-up）。
        // offsetWidth/offsetHeight 等是属性访问返回 undefined 不抛，作 reflow 触发器无害，不特例化。
        if (prop === 'getBoundingClientRect') {
          return function() {
            if (sel && typeof __zw_getBoundingClientRect === 'function') {
              try {
                var s = __zw_getBoundingClientRect(sel);
                if (s && s.indexOf(',') >= 0) {
                  var p = s.split(',');
                  var x = +p[0], y = +p[1], w = +p[2], h = +p[3];
                  return { x: x, y: y, top: y, left: x, right: x + w, bottom: y + h, width: w, height: h, toJSON: function() { return this; } };
                }
              } catch (_e) {}
            }
            return { x: 0, y: 0, top: 0, left: 0, right: 0, bottom: 0, width: 0, height: 0, toJSON: function() { return this; } };
          };
        }
        if (prop === 'getClientRects') {
          return function() { return []; };
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
        } else if (p === 'className') {
          if (handle) __zw_set_attr_handle(handle, 'class', String(value));
          else __zw_set_attr(sel, 'class', String(value));
          moAttr = 'class';
        } else if (p === 'id') {
          if (handle) __zw_set_attr_handle(handle, 'id', String(value));
          else __zw_set_attr(sel, 'id', String(value));
          moAttr = 'id';
        } else if (p === 'value') {
          // P1a form input：value 属性 set——更新缓存 + 记 value 属性 mutation（供 render）。
          _inputValues[key] = String(value);
          if (handle) __zw_set_attr_handle(handle, 'value', String(value));
          else __zw_set_attr(sel, 'value', String(value));
          moAttr = 'value';
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

  globalThis.CustomEvent = function(type, options) {
    return _makeEvent(type, options);
  };

  globalThis.Event = function(type, options) {
    return _makeEvent(type, options);
  };

  globalThis.KeyboardEvent = function(type, options) {
    var ev = _makeEvent(type, options);
    if (options) {
      ev.key = options.key || '';
      ev.code = options.code || options.key || '';
    }
    return ev;
  };

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
      var handle = __zw_create_element(String(tag));
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
      return _wrapHandle(handle);
    },
    documentElement: _wrapSelector('html'),
    body: _wrapSelector('body'),
    head: _wrapSelector('head'),
    compatMode: 'CSS1Compat',
    characterSet: 'UTF-8',
    charset: 'UTF-8',
    readyState: 'complete',
    styleSheets: _emptyCollection(),
    forms: _liveQueryCollection('form'),
    images: _liveQueryCollection('img'),
    scripts: _liveQueryCollection('script'),
    links: _liveQueryCollection('a'),
    addEventListener: function(type, fn, opts) {
      _makeProxy('html', null).addEventListener(type, fn, opts);
    },
    removeEventListener: function(type, fn) {
      _makeProxy('html', null).removeEventListener(type, fn);
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
    var key = _elKey(sel, null);
    var el = _wrapSelector(sel);
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
    ev.target = el;
    var ok = _dispatchToListeners(key, ev);
    return ok ? 'ok' : 'prevented';
  };
})();
