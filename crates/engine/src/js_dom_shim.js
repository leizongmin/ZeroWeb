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
  // P1a DocumentFragment：已创建的 fragment handle 集合（nodeType=11 标识 + appendChild 时
  // flatten 检测）。fragment 为 create 句柄，无 selector，故用此 set 区别于普通元素句柄。
  var _fragmentHandles = {};
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
  globalThis.__zw_reset_form_state = function() { _inputValues = {}; _classCache = {}; };

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

  // performance.now()——DOMHighResTimeStamp（ms，自 time origin 起，单调）。host `__zw_performance_now`
  // 返 elapsed ms（子毫秒）；未注册（polyfill/reftest 路径）走 Date.now() 兜底（仍单调非负）。
  globalThis.performance = globalThis.performance || {
    now: function() {
      return typeof __zw_performance_now === 'function'
        ? Number(__zw_performance_now())
        : (typeof Date.now === 'function' ? Date.now() : 0);
    }
  };

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
  globalThis.URLSearchParams = globalThis.URLSearchParams || function URLSearchParams(init) {
    if (!(this instanceof URLSearchParams)) return new URLSearchParams(init);
    this._p = [];
    if (init == null) return;
    if (typeof init === 'string') {
      var s = init;
      if (s.charAt(0) === '?') s = s.slice(1);
      if (s) {
        var parts = s.split('&');
        for (var i = 0; i < parts.length; i++) {
          var p = parts[i];
          if (p === '') continue;
          var eq = p.indexOf('=');
          var k = eq < 0 ? p : p.slice(0, eq);
          var v = eq < 0 ? '' : p.slice(eq + 1);
          this._p.push([decodeURIComponent(k.replace(/\+/g, ' ')), decodeURIComponent(v.replace(/\+/g, ' '))]);
        }
      }
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
    append: function (n, v) { this._p.push([String(n), String(v)]); },
    delete: function (n, v) {
      n = String(n);
      if (arguments.length >= 2) {
        v = String(v);
        this._p = this._p.filter(function (p) { return !(p[0] === n && p[1] === v); });
      } else {
        this._p = this._p.filter(function (p) { return p[0] !== n; });
      }
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
    },
    sort: function () { this._p.sort(function (a, b) { return a[0] < b[0] ? -1 : a[0] > b[0] ? 1 : 0; }); },
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

  // URL——WHATWG URL 解析（protocol/host/hostname/port/pathname/search/hash/origin/href/searchParams）。
  // location.href 操纵 / fetch 相对 URL / 链接解析高频。委托 host `__zw_parse_url(url, base)`（spec-correct
  // via `url` crate：base 解析 / percent-encoding / IDNA / 默认端口归一），失败抛 TypeError（spec）；
  // 未注册（纯 sandbox 无 host，如 reftest 无 JS 回调路径）抛 TypeError。**已知限制**：组件无 setter
  // （只读属性；set 须重新构造，defer）；searchParams 与属性不双向同步（修改 searchParams 不更新 search）。
  function URL(url, base) {
    if (typeof __zw_parse_url !== 'function') {
      throw new TypeError('URL constructor requires a URL parser (__zw_parse_url not registered)');
    }
    if (!(this instanceof URL)) return new URL(url, base); // 允许无 new
    var raw = __zw_parse_url(String(url), base !== undefined ? String(base) : '');
    var p = raw ? JSON.parse(raw) : null;
    if (!p) throw new TypeError('Invalid URL: ' + url);
    this.protocol = p.protocol;
    this.username = p.username;
    this.password = p.password;
    this.hostname = p.hostname;
    this.host = p.host;
    this.port = p.port;
    this.origin = p.origin;
    this.pathname = p.pathname;
    this.search = p.search;
    this.hash = p.hash;
    this.href = p.href;
    // searchParams 复用 URLSearchParams（R2772），从 search 去前导 '?'。
    var sp = p.search.charAt(0) === '?' ? p.search.slice(1) : p.search;
    this.searchParams = new URLSearchParams(sp);
  }
  URL.prototype = Object.create(Object.prototype);
  URL.prototype.constructor = URL;
  URL.prototype.toString = function () { return this.href; };
  URL.prototype.toJSON = function () { return this.href; };
  // canParse 静态——解析成功 true / 失败 false（不抛）。
  URL.canParse = function (url, base) {
    if (typeof __zw_parse_url !== 'function') return false;
    return !!__zw_parse_url(String(url), base !== undefined ? String(base) : '');
  };
  globalThis.URL = globalThis.URL || URL;

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
  globalThis.Element.prototype.addEventListener = function(type, fn, opts) {
    _globalAddEventListener(type, fn, opts);
  };
  globalThis.Element.prototype.removeEventListener = function(type, fn, opts) {
    _globalRemoveEventListener(type, fn, opts);
  };

  function _elKey(sel, handle) {
    return handle ? ('@' + handle) : sel;
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
          // P1a select option：selected 属性存在性（boolean，经 host `__zw_has_attr`）。
          if (typeof __zw_has_attr === 'function') {
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
        // DocumentFragment handle（nodeType 11 / nodeName '#document-fragment'）。
        var isFrag = handle && _fragmentHandles[handle];
        if (prop === 'tagName') {
          return isFrag ? undefined : _realTag(sel, handle);
        }
        if (prop === 'nodeName') {
          return isFrag ? '#document-fragment' : _realTag(sel, handle);
        }
        if (prop === 'nodeType') {
          return isFrag ? 11 : 1;
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
        // `el.cloneNode(deep)`——克隆元素（返新 handle proxy，detached）。复用既有回调组合：
        // create(tag) + 逐属性 set_attr_handle + (deep) set_inner_html_handle。sel-based 源完整；
        // handle 源 tag/attrs 受限（无 get_tag/attr_names handle 变体，best-effort）。
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
            var added = [];
            for (var i = 0; i < arguments.length; i++) {
              var item = arguments[i];
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
            var id = sel || handle;
            if (id && typeof __zw_getBoundingClientRect === 'function') {
              try {
                var s = __zw_getBoundingClientRect(id);
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
        } else if (p === 'value') {
          // P1a select：编程设 `<select>.value = value` → 记 SelectOption mutation（apply 时
          // mark 匹配 option selected + deselect 兄弟）。匹配浏览器：编程设值不自动派 change。
          if (!handle && sel && typeof __zw_select_option === 'function' && _isTag(sel, 'SELECT')) {
            __zw_select_option(sel, String(value));
            // SelectOption 改的是子 option 的 selected 属性，非 select 元素自身的属性 mutation；
            // 不发 select 的 attributes MO 通知（语义正确）。
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

  // dataset 键转换：camelCase ↔ data-kebab-case（fooBar ↔ data-foo-bar）。
  function _camelToKebab(s) {
    return s.replace(/[A-Z]/g, function(m) { return '-' + m.toLowerCase(); });
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

  globalThis.KeyboardEvent = function KeyboardEvent(type, options) {
    var ev = _makeEvent(type, options);
    Object.setPrototypeOf(ev, globalThis.KeyboardEvent.prototype);
    ev.key = (options && options.key) || '';
    ev.code = (options && (options.code || options.key)) || '';
    return ev;
  };
  globalThis.KeyboardEvent.prototype = Object.create(globalThis.Event.prototype);
  globalThis.KeyboardEvent.prototype.constructor = globalThis.KeyboardEvent;

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
    // `document.createDocumentFragment()`：DocumentFragment（nodeType 11，轻量容器）。
    // 建 fragment（append 子节点经既有 append_child_handle）+ 标记 handle 到 _fragmentHandles
    //（供 nodeType=11 与 append 时 flatten 检测）。
    createDocumentFragment: function() {
      if (typeof __zw_create_document_fragment !== 'function') return _wrapHandle('');
      var handle = __zw_create_document_fragment();
      if (handle) _fragmentHandles[handle] = true;
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
