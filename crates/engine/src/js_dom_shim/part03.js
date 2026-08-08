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
    if (t === 'pageshow') _maybeFirePageShow(); // R2931：首次 pageshow listener → _defer 派发一次
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
  // R3019：DOM 接口构造器占位——库（DOMPurify 等）常做 `x instanceof HTMLFormElement` /
  // `el.attributes instanceof NamedNodeMap` / `node.content instanceof DocumentFragment` 校验。
  // 这些构造器须以 function 存在（否则 `instanceof undefined` 抛 TypeError 中断 sanitize）。本桥接的
  // 元素为 proxy 对象非真实例，instanceof 恒返 false（正确：DOMPurify 仅借此识别 form/template 特殊处理）。
  // 原型链挂到对应基类（DocumentFragment→Node、HTML*→HTMLElement）仅为语义一致，instanceof 不依赖实例身份。
  globalThis.HTMLFormElement = globalThis.HTMLFormElement || function HTMLFormElement() {};
  globalThis.HTMLFormElement.prototype = Object.create(globalThis.HTMLElement.prototype);
  globalThis.NamedNodeMap = globalThis.NamedNodeMap || function NamedNodeMap() {};
  // R3024：Attr 构造器占位——_zwMakeAttr 经 Object.create(Attr.prototype) 建真实例，使 `attr instanceof Attr`
  // 为 true（闭合 R3023 限制①；消费者按 nodeType===2 / instanceof Attr 校验属性节点）。
  globalThis.Attr = globalThis.Attr || function Attr() {};
  globalThis.DocumentFragment = globalThis.DocumentFragment || function DocumentFragment() {};
  globalThis.DocumentFragment.prototype = Object.create(globalThis.Node.prototype);
  // R3019：Element.prototype 成员补全——DOMPurify 等库加载时经 lookupGetter(ElementPrototype, 'parentNode'/
  // 'remove'/'cloneNode'/'nextSibling'/'childNodes') 固化原型链成员（unapply 后以节点为 this 调用）。旧 shim
  // 原型空壳 → lookup 全落 fallback（恒返 null）→ _forceRemove 的 getParentNode(node).removeChild(node) 抛
  // TypeError → catch 走 remove(node)（fallback 空函数）静默失败：removed 数组记录了但节点从未真正移除
  // （真实 DOMPurify sanitize 保留 iframe，探针实证）。补真实成员：getter 优先 own property（_zwMEl 节点
  // own parentNode/childNodes 等），无 own → null（页面 proxy 同旧 fallback 语义，零回归）。
  function _zwProtoOwnGetter(name) {
    return function () { return Object.prototype.hasOwnProperty.call(this, name) ? this[name] : null; };
  }
  Object.defineProperty(globalThis.Element.prototype, 'parentNode', { get: _zwProtoOwnGetter('parentNode'), configurable: true });
  Object.defineProperty(globalThis.Element.prototype, 'childNodes', { get: _zwProtoOwnGetter('childNodes'), configurable: true });
  Object.defineProperty(globalThis.Element.prototype, 'nextSibling', { get: _zwProtoOwnGetter('nextSibling'), configurable: true });
  // remove()（Node 方法）：DOMPurify _forceRemove catch 分支。_zwMEl 节点无 own remove → 走原型。
  Object.defineProperty(globalThis.Element.prototype, 'remove', { value: function () {
    var p = this.parentNode;
    if (p && typeof p.removeChild === 'function') p.removeChild(this);
  }, configurable: true });
  // cloneNode(deep)（Node 方法）：DOMPurify keep-content 路径（clone 子节点插回 parentNode）。deep 克隆
  // 复制 nodeType/attrs/子树（含文本/注释），parentNode=null 由 insertBefore relink。内联深克隆（不依赖子
  // 节点有 cloneNode 方法——_zwMText/_zwMComment 为 plain object 无原型方法）。
  globalThis.Element.prototype.cloneNode = function (deep) {
    function deepClone(n) {
      if (!n || typeof n !== 'object' || n.nodeType === undefined) return null;
      var o;
      if (n.nodeType === 3 || n.nodeType === 8) {
        o = { nodeType: n.nodeType, nodeName: n.nodeName, nodeValue: n.nodeValue, data: n.nodeValue, textContent: n.nodeValue, childNodes: [], children: [] };
      } else if (n.nodeType === 1) {
        o = { nodeType: 1, nodeName: n.nodeName, tagName: n.tagName, localName: n.localName, attributes: [], childNodes: [], children: [] };
        var as = n.attributes;
        if (as) for (var i = 0; i < as.length; i++) o.attributes.push({ name: as[i].name, value: as[i].value });
        var cs = n.childNodes;
        if (cs) for (var j = 0; j < cs.length; j++) {
          var cc = deepClone(cs[j]);
          if (cc) { cc.parentNode = o; o.childNodes.push(cc); }
        }
      } else {
        return null;
      }
      return o;
    }
    return deepClone(this);
  };
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

  // ── custom element lifecycle slice（R2992）：attributeChangedCallback 分派 ──────────
  // element 实例为 generic Proxy 非 ctor 实例（upgrade/ctor 调用 defer），故本 slice 仅落地「属性变更」回调——
  // 这是 CE 最常用的可观察行为（lit/@property / 各 CE 库 react-to-attr 模式）。setAttribute/removeAttribute
  // 命中 observedAttributes 时，取 ctor.prototype.attributeChangedCallback 以 element proxy 为 this 调用，
  // old/new 值经 getAttribute 前读（首次 set old=null，remove new=null）。
  // https://html.spec.whatwg.org/multipage/custom-elements.html#attr-associated (observedAttributes 过滤 +
  // 值真变才入队)。**仍 defer**：connectedCallback/disconnectedCallback（需元素连接追踪）、adoptedCallback、
  // upgrade / ctor 实例化、IDL 反射 setter（className=/id= 等，不走 setAttribute 函数）——独立 slice。
  // tag 不可变 → 每 element 首次属性变更时算一次 registry 查询并缓存（避 setAttribute 热路径每次 host 调用）。
  var _ceEntryByKey = {}; // element key → registry entry | false（false 哨兵 = 非 custom，避重查）
  function _ceEntryFor(key, sel, handle) {
    if (Object.prototype.hasOwnProperty.call(_ceEntryByKey, key)) {
      var cached = _ceEntryByKey[key];
      return cached || null;
    }
    var entry = _ce_registry[_realTag(sel, handle).toLowerCase()] || null;
    _ceEntryByKey[key] = entry || false;
    return entry;
  }
  // 分派 attributeChangedCallback：仅当 attr ∈ ctor.observedAttributes 且值真变时（spec set/remove 同值无 change）。
  function _ce_dispatchAttrChange(entry, proxy, name, oldVal, newVal) {
    var ctor = entry.ctor;
    var obs;
    try { obs = ctor && ctor.observedAttributes; } catch (_e) { return; }
    if (!obs) return;
    var nl = String(name).toLowerCase();
    var matched = false;
    for (var i = 0; i < obs.length; i++) {
      if (String(obs[i]).toLowerCase() === nl) { matched = true; break; }
    }
    if (!matched) return;
    // 值真变判定（absent/null 归一为 '' 比较仅用于 gate；回调收 raw oldVal/newVal）。
    var o = oldVal == null ? '' : String(oldVal);
    var nv = newVal == null ? '' : String(newVal);
    if (o === nv) return;
    var cb = ctor.prototype && ctor.prototype.attributeChangedCallback;
    if (typeof cb === 'function') {
      try { cb.call(proxy, String(name), oldVal, newVal); } catch (_e) {}
    }
  }
  // 读元素属性值用于 CE old-value：absent → null（spec attributeChangedCallback old/new 为 null 表 absent），
  // present → 值串。经 has-attr 判存在（handle 元素用 __zw_has_attr_handle，sel 用 __zw_has_attr），
  // 区别于 get-attr 对 absent 返 ''（无法区分 absent 与空串值）。
  function _ce_attrValue(sel, handle, name) {
    var present = false;
    if (handle && typeof __zw_has_attr_handle === 'function') {
      try { present = __zw_has_attr_handle(handle, name) === '1'; } catch (_e) { present = false; }
    } else if (sel && typeof __zw_has_attr === 'function') {
      try { present = __zw_has_attr(sel, name) === '1'; } catch (_e) { present = false; }
    }
    if (!present) return null;
    try {
      return handle ? __zw_get_attr_handle(handle, name) : __zw_get_attr(sel, name);
    } catch (_e) { return null; }
  }

  function _elKey(sel, handle) {
    return handle ? ('@' + handle) : sel;
  }

  // ── custom element lifecycle slice（R2994）：connectedCallback / disconnectedCallback ──────────
  // spec HTML §4.13：custom element 连入 document 树时调 connectedCallback，断开时调 disconnectedCallback
  //（双向，可重复触发——再连再调）。element 为 generic Proxy 非 ctor 实例（upgrade/ctor 调用仍 defer），
  // 故以 element proxy 为 this 分派 ctor.prototype 上的回调。连入态由 JS 端追踪（host 快照不实时反映
  // handle 元素挂载），appendChild/insertBefore/removeChild/remove 等插入/移除点单点 hook。
  // https://html.spec.whatwg.org/multipage/custom-elements.html#custom-element-reactions (connected/disconnected)
  // **已知限制（既存，非本切片引入）**：① parsed 元素（HTML 源中的 <my-el>）不经 JS appendChild → 初始
  // connectedCallback 不触发（ createElement 路径才触发，框架主流用法）；② insertAdjacentHTML 解析生成
  // 的节点无 handle proxy → 不触发；③ upgrade/ctor 实例化仍 defer。
  var _ceConn = {}; // element key → true（custom element 当前已连入 document 树；非 custom handle 元素亦入，作 detached container 传播连接态供其后代判定）

  // 判定「插入操作的父」是否已连入 document：sel-based 父 → host `__zw_contains('html', sel)`（documentElement
  // 子树判定，权威，含 html/body/head 自身）；handle-based 父（detached createElement 容器 / shadow root /
  // fragment）→ _ceConn 追踪（其先前挂载时由 _ceApplyConn 传播标记）。
  function _ceParentConnected(parentSel, parentHandle) {
    if (parentSel) {
      if (typeof __zw_contains === 'function') {
        try { return __zw_contains('html', parentSel) === '1'; } catch (_e) { return true; }
      }
      return true; // 无 host 回调（polyfill/WebView 路径）→ sel-based 视为已连入（parsed 即在树内）
    }
    if (parentHandle) return !!_ceConn[_elKey(null, parentHandle)];
    return false;
  }

  // 对子树（rootProxy 及其 handle-registry 后代，pre-order tree order）应用连接态变更。
  // connected=true：未连→连（custom element 分派 connectedCallback）；connected=false：已连→断（disconnectedCallback）。
  // 非 custom handle 元素仅传播连接态（供其作父/容器时后代判定）；sel-based 非 custom 元素连接态由 host 权威，不追踪。
  // 仅 custom 元素在状态真转时调回调（再连再调、同态跳过）。回调异常 try/catch 吞（不中断脚本）。
  function _ceApplyConn(rootProxy, connected) {
    var stack = [rootProxy];
    while (stack.length) {
      var node = stack.shift();
      if (!node) continue;
      var ns = node.__zwSelector || null;
      var nh = node.__zwHandle || null;
      if (!ns && !nh) continue;
      var key = _elKey(ns, nh);
      var entry = _ceEntryFor(key, ns, nh);
      if (entry) {
        var was = !!_ceConn[key];
        if (connected && !was) {
          _ceConn[key] = true;
          var ccb = entry.ctor && entry.ctor.prototype && entry.ctor.prototype.connectedCallback;
          if (typeof ccb === 'function') { try { ccb.call(node); } catch (_e) {} }
        } else if (!connected && was) {
          delete _ceConn[key];
          var dcb = entry.ctor && entry.ctor.prototype && entry.ctor.prototype.disconnectedCallback;
          if (typeof dcb === 'function') { try { dcb.call(node); } catch (_e) {} }
        }
      } else if (nh && !ns) {
        // 非 custom 纯 handle 元素：追踪连接态作传播（detached container 场景）。
        if (connected) _ceConn[key] = true; else delete _ceConn[key];
      }
      // 递归 handle registry 后代（R2927/R2928 维护的容器子树，pre-order：先 shift 自身再压子）。
      if (nh) {
        var kids = _handleChildren[nh];
        if (kids) for (var i = 0; i < kids.length; i++) stack.push(kids[i]);
      }
    }
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

  // Web Animations Animation 对象（el.animate，R2827 stub → R2965 真关键帧应用）。headless 无真时间轴
  // → 动画「瞬间完成」：创建即 playState='running'，execute 末 _defer microtask 后 playState='finished' +
  // finished Promise resolve + onfinish 触发（除非 cancel）。R2965：finish 时若 fill ∈ {forwards, both}，
  // 把末关键帧（offset 1 或数组末项）的 CSS 属性写入元素 inline style（经 `__zw_set_style[_handle]`），
  // 使动画末态经样式→布局→渲染管线可见（headless 截图反映动画终态）。`commitStyles()` 显式提交当前态
  //（headless 瞬间完成 = 末态）到 inline style，不依赖 fill。modern 动画库（Framer Motion / GSAP / Lottie）
  // feature-detect + 链式 + 终态可见全通。fill: none（默认）/ auto 不自动持久化（spec：finish 后无 effect）。
  // keyframe 解析：数组每项为属性 dict + 可选 meta 键 offset/easing/composite；末态取 offset===1 项，无则末项。
  function _applyKeyframeProps(props, sel, handle) {
    // 把单关键帧 dict 的 CSS 属性写入元素 inline style。跳过 meta 键；camelCase→kebab（复用 _stylePropName）。
    if (!props || typeof props !== 'object') return;
    for (var propName in props) {
      if (!Object.prototype.hasOwnProperty.call(props, propName)) continue;
      if (propName === 'offset' || propName === 'easing' || propName === 'composite') continue;
      var cssProp = _stylePropName(propName);
      var val = String(props[propName]);
      if (handle && typeof __zw_set_style_handle === 'function') {
        try { __zw_set_style_handle(handle, cssProp, val); } catch (_e) {}
      } else if (sel && typeof __zw_set_style === 'function') {
        try { __zw_set_style(sel, cssProp, val); } catch (_e) {}
      }
    }
    _mo_notify(sel, handle, { type: 'attributes', attributeName: 'style' });
  }

  function _endStateFromKeyframes(keyframes) {
    // 末态关键帧：优先 offset===1 项；无显式 offset 则取数组末项。空数组 / 非数组 → null（无末态可应用）。
    if (!Array.isArray(keyframes) || keyframes.length === 0) return null;
    for (var i = 0; i < keyframes.length; i++) {
      var kf = keyframes[i];
      if (kf && typeof kf === 'object' && kf.offset === 1) return kf;
    }
    var last = keyframes[keyframes.length - 1];
    return (last && typeof last === 'object') ? last : null;
  }

  function _makeAnimation(keyframes, options, sel, handle) {
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
      _committed: false,
      play: function () { anim.playState = 'running'; },
      pause: function () { anim.playState = 'paused'; },
      cancel: function () { anim._cancelled = true; anim.playState = 'idle'; },
      finish: function () { anim.playState = 'finished'; },
      reverse: function () { anim.playbackRate = -anim.playbackRate; return anim; },
      updatePlaybackRate: function (rate) { anim.playbackRate = rate; },
      // commitStyles()：显式把当前态（headless 瞬间完成 = 末态）写入 inline style。spec 不依赖 fill——
      // 调用即提交，用于动画移除前固化终态。多关键帧属性经 _applyKeyframeProps camelCase→kebab。
      commitStyles: function () {
        if (!anim._committed && anim._endState) {
          _applyKeyframeProps(anim._endState, sel, handle);
          anim._committed = true;
        }
      },
      persist: function () {},
      addEventListener: function () {},
      removeEventListener: function () {},
      dispatchEvent: function () { return true; },
    };
    // options：number=duration(ms) / object={duration,id,fill,...}。提取 duration（finish 后 currentTime 用）+ id + fill。
    var fill = 'auto';
    if (options != null) {
      if (typeof options === 'number') anim.duration = options;
      else {
        if (typeof options.duration === 'number') anim.duration = options.duration;
        if (options.id != null) anim.id = String(options.id);
        if (options.fill != null) fill = String(options.fill);
      }
    }
    // 末态关键帧（finish/commitStyles 时应用）。fill ∈ {forwards, both} 时 finish 自动持久化（spec）；
    // none / auto（auto 解析为 none，无父 group）不自动持久化——元素回归 underlying 值。
    anim._endState = _endStateFromKeyframes(keyframes);
    anim._persist = (fill === 'forwards' || fill === 'both');
    var resolveFinish;
    anim._finishedP = new Promise(function (r) { resolveFinish = r; });
    Object.defineProperty(anim, 'finished', { get: function () { return anim._finishedP; } });
    // headless 瞬间完成（无真时间轴）—— microtask 后 finished + onfinish（cancel 则 idle 不完成）。
    // persist 时把末态写入 inline style（经样式管线可见）。已 commitStyles 则跳过重复写。
    _defer(function () {
      if (!anim._cancelled) {
        anim.playState = 'finished';
        anim.currentTime = anim.duration;
        if (anim._persist && !anim._committed && anim._endState) {
          _applyKeyframeProps(anim._endState, sel, handle);
          anim._committed = true;
        }
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

  // R2934 inline HTML event handler 编译（`<button onclick="...">`）：on* getter 无 JS 设值时回落编译
  // 元素的 on* 属性串为函数（spec 近似：`new Function('event', 'with(document){with(this){ <code> }}')`，
  // this=元素），缓存 + 注册进 _listenerStore[key]（使 dispatchEvent/click 触发）。JS 设值优先（set trap 覆盖），
  // =null 移除（不再回落重编译，spec onclick=null 清除）。缓存 false 标记「已查无 inline」避重复 host 查询。
  // 由 on* getter（返回编译 fn）+ _dispatchWithBubble target 阶段（点击触发）调用。
  function _ensureInlineHandler(key, sel, handle, type) {
    if (_onHandlers[key] && _onHandlers[key][type] !== undefined) return; // 已编译 fn / 已查无 false / JS 设值
    var attr = 'on' + type;
    var code = null;
    try {
      code = handle ? __zw_get_attr_handle(handle, attr) : (sel ? __zw_get_attr(sel, attr) : null);
    } catch (_e) {}
    if (!_onHandlers[key]) _onHandlers[key] = {};
    if (!code) { _onHandlers[key][type] = false; return; } // 查无 → 缓存 false（getter 返 null）
    var fn = null;
    try { fn = new Function('event', 'with(document) { with(this) { ' + code + ' } }'); }
    catch (_e) { _onHandlers[key][type] = false; return; } // 编译失败（语法错）→ 视为无 handler
    _onHandlers[key][type] = fn;
    if (!_listenerStore[key]) _listenerStore[key] = {};
    if (!_listenerStore[key][type]) _listenerStore[key][type] = [];
    _listenerStore[key][type].push({ fn: fn, capture: false, once: false });
  }

  // R2946 <body on*> 内容属性 → window.on* 反射（HTML spec：body 元素的 WindowEventHandlers 内容属性
  // 反射为 window 的事件处理器 IDL 属性）。`<body onload="init()">` 经此把 init 编译为 window.onload，
  // 使既有 lifecycle 'load' 派发到 'html'（window listener 键 _elKey('html', null)）触发——与
  // window.addEventListener('load') / window.onload = fn 路径合一。element 级 inline handler
  //（_ensureInlineHandler，dispatch 到元素本身触发）互补：body onload spec 派发到 window 非 body。
  // **每页一次**：按 page URL 去重（_bodyReflectUrl），导航后 URL 变 → 重新反射。**JS 优先**：仅当
  // window.on<type> 未被 JS 设值（typeof === 'function'）时反射——页面脚本 `window.onload = custom`
  // 不被覆盖（spec：IDL 设值优先于内容属性反射）。编译同 _ensureInlineHandler（new Function + with 双 scope），
  // this=window（window.onload 经 R2932 accessor 注册，dispatch 时 fn.call(globalThis)）。
  // 调用点：① __zw_begin_script（每脚本执行前，覆盖有脚本页 + 让脚本可读到反射后的 window.onload）；
  // ② host lifecycle 派发前（__zw_reflect_body_handlers，覆盖无脚本页）。两者幂等。
  var _bodyReflectUrl = null;
  var _BODY_WIN_HANDLER_TYPES = [
    'load', 'error', 'resize', 'scroll', 'hashchange', 'pageshow', 'pagehide', 'beforeunload',
    'unload', 'message', 'online', 'offline', 'popstate', 'storage', 'focus', 'blur',
  ];
  function _zw_reflect_body_window_handlers() {
    // kill-switch：reftest harness 自有更完整的 <body>/<frameset>/<html> onload 处理（直接执行 handler 体
    // + 派发 'load'），启用本反射会与其重复触发 body onload（双 fire → 重复 mutation → apply 失败）。
    // harness 在 shim init 后置 `globalThis.__zw_no_body_reflect = true`；生产路径（browser/renderer）不禁用。
    if (globalThis.__zw_no_body_reflect) return;
    var url = (typeof __zw_get_page_url === 'function') ? __zw_get_page_url() : '';
    if (_bodyReflectUrl === url) return; // 每页一次
    _bodyReflectUrl = url;
    for (var i = 0; i < _BODY_WIN_HANDLER_TYPES.length; i++) {
      var t = _BODY_WIN_HANDLER_TYPES[i];
      if (typeof globalThis['on' + t] === 'function') continue; // JS 已设值，不覆盖
      var code = null;
      try { code = __zw_get_attr('body', 'on' + t); } catch (_e) {}
      if (code) {
        try {
          globalThis['on' + t] = new Function('event', 'with(document){with(this){' + code + '}}');
        } catch (_e) {}
      }
    }
  }
  globalThis.__zw_reflect_body_handlers = _zw_reflect_body_window_handlers;

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
        var capKey = _elKey(chain[i], null);
        _ensureInlineHandler(capKey, chain[i], null, event.type); // R2935 祖先 inline on* handler 触发
        var capAnc = _wrapSelector(chain[i]);
        _dispatchToListeners(capKey, event, 'capture', capAnc);
        if (event._propagationStopped) return !event._defaultPrevented;
      }
    }

    // ② target 阶段：capture + 非 capture（AT_TARGET，保旧行为）。
    event.currentTarget = target;
    _ensureInlineHandler(targetKey, targetSel, targetHandle, event.type); // R2934 inline on* handler 触发
    _dispatchToListeners(targetKey, event, 'all', target);
    if (event._propagationStopped) return !event._defaultPrevented;

    // ③ bubble 阶段：target→root 方向（chain 正序），祖先派发非 capture（仅 event.bubbles）。
    if (propagate && event.bubbles && !globalThis.__zw_no_bubble) {
      for (var k = 0; k < chain.length; k++) {
        var bKey = _elKey(chain[k], null);
        _ensureInlineHandler(bKey, chain[k], null, event.type); // R2935 祖先 inline on* handler 冒泡触发
        var bAnc = _wrapSelector(chain[k]);
        _dispatchToListeners(bKey, event, 'bubble', bAnc);
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
  // R2996 input.defaultValue 独立追踪 helper。spec：`.value=` 改 dirty 当前态、不改 defaultValue（=初始 value
  // 属性）。shim 的 `.value=` 仍写 value 属性供 render，故属性被污染——首次 `.value=`（或 valueAsNumber=）前
  // 捕获当前 value 属性（=真默认值，latest-wins 反映已 setAttribute 的值），之后 defaultValue 读捕获值；
  // setAttribute('value')/defaultValue=/removeAttribute('value') 重同步（清 dirty → getter 回落属性）。
  // 读源同 defaultValue getter 非_dirty 分支：sel-based 用 `__zw_get_attr_lw`（latest-wins），handle 用
  // `__zw_get_attr_handle`，无回调回落 `__zw_get_attr` 快照。仅 INPUT（textarea 的 defaultValue=text 内容，另案）。
  function _captureInputDefault(key, sel, handle) {
    if (_inputDefaultDirty[key]) return;
    var d = '';
    if (handle && typeof __zw_get_attr_handle === 'function') {
      try { d = __zw_get_attr_handle(handle, 'value') || ''; } catch (_e) {}
    } else if (sel && typeof __zw_get_attr_lw === 'function') {
      try { d = __zw_get_attr_lw(sel, 'value') || ''; } catch (_e) {}
    } else if (sel && typeof __zw_get_attr === 'function') {
      try { d = __zw_get_attr(sel, 'value') || ''; } catch (_e) {}
    }
    _inputDefault[key] = d;
    _inputDefaultDirty[key] = true;
  }
  function _clearInputDefault(key) { _inputDefaultDirty[key] = false; }
  // R2998 布尔默认态独立追踪 helper（checked→defaultChecked, selected→defaultSelected）。capture：dirty 未设时
  // 读当前属性存在性（=真默认，latest-wins 反映已 setAttribute 的值）存 `_boolDefault[ck]`（boolean）并置
  // dirty=true——**仅首次**（不重捕被 .checked=/.selected= 污染的属性）。读源同 default* getter 非_dirty 分支：
  // handle→`__zw_has_attr_handle`，sel→`__zw_has_attr_lw`（latest-wins），无 `_lw` 回落 `__zw_has_attr` 快照。
  // clear：dirty=false（setAttribute/removeAttribute/default*= 重同步，getter 回落属性 latest-wins）。
  function _captureBoolDefault(key, attr, sel, handle) {
    var ck = key + ':' + attr;
    if (_boolDefaultDirty[ck]) return;
    var present = false;
    if (handle && typeof __zw_has_attr_handle === 'function') {
      try { present = __zw_has_attr_handle(handle, attr) === '1'; } catch (_e) {}
    } else if (sel && typeof __zw_has_attr_lw === 'function') {
      try { present = __zw_has_attr_lw(sel, attr) === '1'; } catch (_e) {}
    } else if (sel && typeof __zw_has_attr === 'function') {
      try { present = __zw_has_attr(sel, attr) === '1'; } catch (_e) {}
    }
    _boolDefault[ck] = present;
    _boolDefaultDirty[ck] = true;
  }
  function _clearBoolDefault(key, attr) { _boolDefaultDirty[key + ':' + attr] = false; }

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

  // detached Document（供 document.implementation.createDocument/createHTMLDocument，R2815；R3013 queryable；
  // R3016 traversable）。
  // R3013：body 经 __zw_parse_html_query 支持可写可查——body.innerHTML setter 存解析源，querySelector 族
  //（包 _zwParseEl 只读 element-proxy）查解析树。满足 jQuery `$.parseHTML` / DOMPurify feature-detect /
  // 模板引擎「detached 解析后查询」模式（旧 hollow querySelector 恒 null）。host 未注册（reftest/纯 sandbox）
  // → query 返 null/空（no-throw，零回归）。createElement/createTextNode 保留（node 工厂 + feature-detect）。
  // R3016：body.childNodes 递归遍历——__zw_parse_html_child_nodes + _zwDetachedEl/Text/Comment 递归 node-proxy，
  // 解锁 DOMPurify.sanitize 设 dom.body.innerHTML 后递归 walk 清洗（旧 body.childNodes 恒空）。
  // **已知限制（记录）**：① querySelectorAll('*') 含 html/head/body（html5ever 包裹）——按具体 tag/id/class 查；
  //   ② innerHTML setter 后树重建（_tree 失效）；③ 树上节点 setAttribute/removeAttribute 仅改树内 attrs（R3018），
  //      不回注 host DOM（detached 容器无需持久化，DOMPurify 等库清洗后读 body.innerHTML 取结果）。
  // R3017：detached mutable tree（cached JS 树）——DOMPurify.sanitize `node.parentNode.removeChild(node)` + 读
  // body.innerHTML 核心须 mutation + 序列化。lazy-snapshot（R3016 只读）→ cached mutable tree（建一次、mutate relink、
  // 无 selector 重算/失效）。元素属性经 __zw_parse_html_query 快照、子经 __zw_parse_html_child_nodes 递归建。
  // R3018：attribute mutation（setAttribute/removeAttribute 入 attrs 数组，序列化反映 + id/class IDL 反射同步）+
  // sibling 导航（previousSibling/nextSibling 经 parentNode 动态求值）+ insertBefore/replaceChild/lastChild 收尾
  // DOMPurify 清洗所需 mutation 面（移禁元素 + 去 on*/style 属性 + 重定位）。
  var _ZW_VOID_TAGS = { area: 1, base: 1, br: 1, col: 1, embed: 1, hr: 1, img: 1, input: 1, link: 1, meta: 1, param: 1, source: 1, track: 1, wbr: 1 };
  function _zwMEscapeText(s) { return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;'); }
  function _zwMEscapeAttr(s) { return String(s).replace(/&/g, '&amp;').replace(/"/g, '&quot;'); }
  // 节点 → HTML 串（元素含属性 + 子树；文本转义；注释包裹）。供 innerHTML/outerHTML 序列化（反映 mutation）。
  function _zwMSerialize(node) {
    if (!node) return '';
    if (node.nodeType === 3) return _zwMEscapeText(node.nodeValue);
    if (node.nodeType === 8) return '<!--' + node.nodeValue + '-->';
    if (node.nodeType !== 1) return '';
    var tag = node.localName || (node.tagName || '').toLowerCase();
    var attrStr = '';
    for (var i = 0; i < node.attributes.length; i++) attrStr += ' ' + node.attributes[i].name + '="' + _zwMEscapeAttr(node.attributes[i].value) + '"';
    if (_ZW_VOID_TAGS[tag]) return '<' + tag + attrStr + '>';
    var inner = '';
    for (var j = 0; j < node.childNodes.length; j++) inner += _zwMSerialize(node.childNodes[j]);
    return '<' + tag + attrStr + '>' + inner + '</' + tag + '>';
  }
  // R3018：兄弟导航 getter（previousSibling/nextSibling）经 parentNode.childNodes indexOf 自身动态求值。
  // removeChild/appendChild/insertBefore/replaceChild relink 已维护 parentNode，故兄弟关系始终一致。
  // 元素/文本/注释节点共用（DOMPurify 等库 walk 时对任意节点类型取兄弟）。
  function _zwMDefineSiblings(node) {
    Object.defineProperty(node, 'previousSibling', { get: function () {
      var p = node.parentNode;
      if (!p) return null;
      var i = p.childNodes.indexOf(node);
      return i > 0 ? p.childNodes[i - 1] : null;
    }, configurable: true });
    Object.defineProperty(node, 'nextSibling', { get: function () {
      var p = node.parentNode;
      if (!p) return null;
      var i = p.childNodes.indexOf(node);
      return i >= 0 && i < p.childNodes.length - 1 ? p.childNodes[i + 1] : null;
    }, configurable: true });
  }
  // 可变元素节点：parentNode/childNodes/removeChild/appendChild（relink，无 selector 重算）+ 惰性 textContent/innerHTML/outerHTML 序列化。
  function _zwMEl(snap, parent) {
    snap = snap || {};
    var tag = snap.tag || '';
    var attrs = [];
    var sa = snap.attrs || {};
    for (var k in sa) { if (Object.prototype.hasOwnProperty.call(sa, k)) attrs.push({ name: k, value: sa[k] }); }
    var node = {
      nodeType: 1,
      tagName: tag.toUpperCase(),
      nodeName: tag.toUpperCase(),
      localName: tag,
      id: snap.id || '',
      className: snap.cls || '',
      attributes: attrs,
      childNodes: [],
      parentNode: parent || null
    };
    node.getAttribute = function (n) { n = String(n); for (var i = 0; i < attrs.length; i++) if (attrs[i].name === n) return attrs[i].value; return null; };
    node.hasAttribute = function (n) { return node.getAttribute(n) !== null; };
    // R3019：hasChildNodes（DOMPurify _sanitizeElements mXSS 检查调 currentNode.hasChildNodes()）。
    node.hasChildNodes = function () { return node.childNodes.length > 0; };
    // R3018：属性 mutation 入树（setAttribute/removeAttribute 改 attrs 数组，序列化反映）。
    // setAttribute 已存在则更新值（latest-wins），否则追加；id/class 同步 IDL 反射字段。
    node.setAttribute = function (n, v) {
      n = String(n); var sv = v == null ? '' : String(v);
      for (var i = 0; i < attrs.length; i++) { if (attrs[i].name === n) { attrs[i].value = sv; _zwMReflectIdl(node, n); return; } }
      attrs.push({ name: n, value: sv });
      _zwMReflectIdl(node, n);
    };
    // removeAttribute 移除全部同名属性（去重保险），同步清 IDL 反射字段。
    node.removeAttribute = function (n) {
      n = String(n);
      for (var i = attrs.length - 1; i >= 0; i--) { if (attrs[i].name === n) attrs.splice(i, 1); }
      _zwMReflectIdl(node, n);
    };
    node.removeChild = function (c) { var i = node.childNodes.indexOf(c); if (i >= 0) { node.childNodes.splice(i, 1); c.parentNode = null; } return c; };
    node.appendChild = function (c) { if (c && c.parentNode) c.parentNode.removeChild(c); node.childNodes.push(c); c.parentNode = node; return c; };
    // R3018：insertBefore/replaceChild（DOMPurify 重定位节点、替换用）。ref=null 等价 append。
    node.insertBefore = function (c, ref) {
      if (c && c.parentNode) c.parentNode.removeChild(c);
      if (ref == null) { node.childNodes.push(c); }
      else { var i = node.childNodes.indexOf(ref); if (i < 0) node.childNodes.push(c); else node.childNodes.splice(i, 0, c); }
      c.parentNode = node; return c;
    };
    node.replaceChild = function (n, o) {
      // spec 顺序：先 adopt（从原父移除 newChild），再定位 oldChild 当前 index（移除可能前移 oldChild）。
      if (n && n.parentNode) n.parentNode.removeChild(n);
      var i = node.childNodes.indexOf(o);
      if (i < 0) return o;
      node.childNodes[i] = n; n.parentNode = node; o.parentNode = null;
      return o;
    };
    Object.defineProperty(node, 'textContent', { get: function () { var t = ''; for (var i = 0; i < node.childNodes.length; i++) { var c = node.childNodes[i]; if (c.nodeType === 3) t += c.nodeValue; else if (c.nodeType === 1) t += c.textContent; } return t; }, configurable: true });
    Object.defineProperty(node, 'innerHTML', { get: function () { var s = ''; for (var i = 0; i < node.childNodes.length; i++) s += _zwMSerialize(node.childNodes[i]); return s; }, configurable: true });
    Object.defineProperty(node, 'outerHTML', { get: function () { return _zwMSerialize(node); }, configurable: true });
    Object.defineProperty(node, 'children', { get: function () { return node.childNodes.filter(function (c) { return c.nodeType === 1; }); }, configurable: true });
    Object.defineProperty(node, 'firstChild', { get: function () { return node.childNodes.length ? node.childNodes[0] : null; }, configurable: true });
    Object.defineProperty(node, 'lastChild', { get: function () { return node.childNodes.length ? node.childNodes[node.childNodes.length - 1] : null; }, configurable: true });
    _zwMDefineSiblings(node);
    attrs.item = function (i) { return attrs[i] ? _zwMakeAttr(attrs[i].name, attrs[i].value, node) : null; };
    attrs.getNamedItem = function (n) { var v = node.getAttribute(n); return v === null ? null : _zwMakeAttr(n, v, node); };
    // R3022：NamedNodeMap 真 mutation——setNamedItem(attr) 等价 setAttribute(attr.name, attr.value)，
    // 返旧 Attr（或 null）；removeNamedItem(name) 等价 removeAttribute，返移除 Attr（缺失返 null，best-effort 不抛）。
    // R3023：返值用 _zwMakeAttr（真 Attr 实例，nodeType 2 + 全字段），非 plain {name,value}。
    attrs.setNamedItem = function (attr) {
      if (!attr || attr.name == null) return null;
      var n = String(attr.name);
      var old = node.getAttribute(n);
      node.setAttribute(n, attr.value != null ? String(attr.value) : '');
      return old !== null ? _zwMakeAttr(n, old, node) : null;
    };
    attrs.removeNamedItem = function (n) {
      n = String(n);
      var old = node.getAttribute(n);
      if (old === null) return null; // spec NOT_FOUND_ERR；best-effort 返 null（不抛，避中断库枚举）
      node.removeAttribute(n);
      return _zwMakeAttr(n, old, node);
    };
    return node;
  }
  // R3018：id/class 属性 ↔ IDL 字段（node.id/node.className）同步，setAttribute/removeAttribute 后保持一致。
  function _zwMReflectIdl(node, attrName) {
    if (attrName === 'id') node.id = node.getAttribute('id') || '';
    else if (attrName === 'class') node.className = node.getAttribute('class') || '';
  }
  function _zwMText(v, parent) { var t = String(v); var n = { nodeType: 3, nodeName: '#text', nodeValue: t, textContent: t, data: t, childNodes: [], children: [], hasChildNodes: function () { return false; }, parentNode: parent || null }; _zwMDefineSiblings(n); return n; }
  function _zwMComment(v, parent) { var t = String(v); var n = { nodeType: 8, nodeName: '#comment', nodeValue: t, textContent: t, data: t, childNodes: [], children: [], hasChildNodes: function () { return false; }, parentNode: parent || null }; _zwMDefineSiblings(n); return n; }
  // 递归建子树：entry = {k:'E',s:sel}/{k:'T',v}/{k:'C',v}（__zw_parse_html_child_nodes）。元素取快照 + 递归子。
  function _zwMBuildNode(html, entry, parent) {
    if (entry.k === 'T') return _zwMText(entry.v, parent);
    if (entry.k === 'C') return _zwMComment(entry.v, parent);
    var snap = {};
    if (typeof __zw_parse_html_query === 'function') {
      try { var a = JSON.parse(__zw_parse_html_query(html, entry.s, '0')); if (a.length) snap = a[0]; } catch (_e) {}
    }
    var node = _zwMEl(snap, parent);
    if (typeof __zw_parse_html_child_nodes === 'function') {
      try {
        var arr = JSON.parse(__zw_parse_html_child_nodes(html, entry.s));
        for (var i = 0; i < arr.length; i++) if (arr[i]) node.childNodes.push(_zwMBuildNode(html, arr[i], node));
      } catch (_e) {}
    }
    return node;
  }
  // 建 body 元素节点树（root，parentNode=null）：从 <body>innerHtml</body> 取 body 子 entries 递归建。
  function _zwMBuildBodyTree(innerHtml) {
    var html = '<body>' + innerHtml + '</body>';
    var body = _zwMEl({ tag: 'body' }, null);
    if (typeof __zw_parse_html_child_nodes === 'function') {
      try {
        var arr = JSON.parse(__zw_parse_html_child_nodes(html, 'body'));
        for (var i = 0; i < arr.length; i++) if (arr[i]) body.childNodes.push(_zwMBuildNode(html, arr[i], body));
      } catch (_e) {}
    }
    return body;
  }
  // R3031：parse-based addedNodes——`innerHTML`/`outerHTML`/`insertAdjacentHTML` 整体替换/插入时，新子经
  // host fragment 解析生成，shim 无同步枚举（snapshot apply 前不可见）→ 旧 MO childList 记录 addedNodes 恒 []。
  // 复用 [`_zwMBuildBodyTree`]（host `__zw_parse_html_child_nodes` 二次 parse）建 `_zwMEl` 代理树，取
  // `.childNodes` 作 addedNodes：节点支持 nodeType/tagName/nodeName/getAttribute/hasAttribute/querySelector(All)/
  // childNodes 等 introspection，满足框架/库（React/Vue）observe 后递归观测新子树的典型消费。代理为解析快照
  //（非 live 文档节点——addedNodes 代表「被加入的结构」，host 未注册 `__zw_parse_html_child_nodes` → 返 [] 旧行为）。
  function _zwFragmentAdded(html) {
    if (typeof _zwMBuildBodyTree !== 'function') return [];
    try { return _zwMBuildBodyTree(String(html == null ? '' : html)).childNodes; } catch (_e) { return []; }
  }
  function _makeDetachedDocument(title) {
    var bodyHtml = '';
    var _tree = null; // R3017：cached mutable body 树（首次 childNodes 访问建，innerHTML setter 失效）
    function ensureTree() { if (!_tree) _tree = _zwMBuildBodyTree(bodyHtml); }
    // detHtml 反映 live 树（_tree 已建则序列化，否则原始 bodyHtml）→ querySelector 与 mutation 一致。
    function detHtml() { return '<body>' + (_tree ? _tree.innerHTML : bodyHtml) + '</body>'; }
    function queryBody(sel, all) {
      if (typeof __zw_parse_html_query !== 'function') return [];
      try {
        return JSON.parse(__zw_parse_html_query(detHtml(), String(sel), all ? '1' : '0'));
      } catch (_e) { return []; }
    }
    function queryOne(sel) { var a = queryBody(sel, false); return a.length ? new _zwParseEl(a[0]) : null; }
    function queryAll(sel) { var a = queryBody(sel, true); var out = []; for (var i = 0; i < a.length; i++) out.push(new _zwParseEl(a[i])); return out; }
    var body = {
      nodeType: 1,
      tagName: 'BODY',
      nodeName: 'BODY',
      localName: 'body',
      parentNode: null, // R3017：detached root，parentNode=null（DOMPurify 经 node.parentNode 取父）
      get innerHTML() { return _tree ? _tree.innerHTML : bodyHtml; },
      set innerHTML(v) { bodyHtml = v == null ? '' : String(v); _tree = null; },
      querySelector: function (sel) { return queryOne(sel); },
      querySelectorAll: function (sel) { return queryAll(sel); },
      getElementById: function (id) { return queryOne('#' + String(id)); },
      getElementsByTagName: function (tag) { return queryAll(String(tag)); },
      getElementsByClassName: function (cls) { return queryAll('.' + String(cls)); },
      // R3016/R3017：body.childNodes 递归遍历（cached mutable tree）。DOMPurify.sanitize walk 入口。
      get childNodes() { ensureTree(); return _tree.childNodes; },
      get children() { ensureTree(); return _tree.childNodes.filter(function (c) { return c.nodeType === 1; }); },
      get firstChild() { ensureTree(); return _tree.childNodes.length ? _tree.childNodes[0] : null; },
      removeChild: function (c) { ensureTree(); return _tree.removeChild(c); },
      appendChild: function (c) { ensureTree(); return _tree.appendChild(c); }
    };
    var doc = {
      nodeType: 9,
      nodeName: '#document',
      documentElement: { nodeType: 1, tagName: 'HTML', nodeName: 'HTML', childNodes: [] },
      head: { nodeType: 1, tagName: 'HEAD', nodeName: 'HEAD', childNodes: [] },
      body: body,
      title: title != null ? String(title) : '',
      querySelector: function (sel) { return queryOne(sel); },
      querySelectorAll: function (sel) { return queryAll(sel); },
      getElementById: function (id) { return queryOne('#' + String(id)); },
      getElementsByTagName: function (tag) { return queryAll(String(tag)); },
      getElementsByClassName: function (cls) { return queryAll('.' + String(cls)); },
      // R3018：createElement/createTextNode 返完整可变节点（_zwMEl/_zwMText），非 hollow stub。
      // DOMPurify / 模板引擎经 createElement 建替换节点后 insertBefore/appendChild 入树，须支持 parentNode/
      // sibling/childNodes/setAttribute/序列化全套语义。HTML 文档 tagName 大写、localName 小写。
      createElement: function (t) { return _zwMEl({ tag: String(t).toLowerCase() }, null); },
      createTextNode: function (t) { return _zwMText(String(t), null); }
    };
    body.ownerDocument = doc;
    return doc;
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
  // R3023：构造真 Attr 节点（nodeType 2，全字段：name/nodeName/value/nodeValue/localName/namespaceURI=
  // null/prefix=null/specified/ownerElement）。供 NamedNodeMap 各方法返值 + document.createAttribute 用，
  // 闭合 R3022 限制①（返 plain {name,value}）。ownerEl 为 null 时为游离 Attr（createAttribute / detached）。
  function _zwMakeAttr(name, value, ownerEl) {
    var n = String(name);
    var v = value != null ? String(value) : '';
    // R3024：经 Object.create(Attr.prototype) 建真实例（`instanceof Attr` true），非 plain object。
    var a = Object.create(globalThis.Attr.prototype);
    a.nodeType = 2;
    a.name = n;
    a.nodeName = n;
    a.value = v;
    a.nodeValue = v;
    a.localName = n;
    a.prefix = null;
    a.namespaceURI = null;
    a.specified = true;
    a.ownerElement = ownerEl || null;
    return a;
  }
  // `el.attributes`（NamedNodeMap）：length / item(i) / getNamedItem(name) / 数值索引 /
  // Symbol.iterator，每项 Attr-like {name,value,localName,...}。经 `__zw_attr_names`+`__zw_get_attr`。
  // handle-only（无 attr_names 变体）→ 空集；R3022：setNamedItem/removeNamedItem 真 mutation（委托元素
  // setAttribute/removeAttribute host 路径，返旧/移除 Attr），不再只读 no-op。
  function _attributesProxy(sel, handle) {
    var readNames = function() {
      if (!sel || typeof __zw_attr_names !== 'function') return [];
      try {
        var n = __zw_attr_names(sel);
        return n ? n.split('|').filter(Boolean) : [];
      } catch (_e) { return []; }
    };
    var attrObj = function(name) {
      // R3003：sel 用 latest-wins（`__zw_get_attr_lw`）反映同批 setAttribute（旧 `__zw_get_attr` 纯快照 → Attr.value
      // stale）；handle 用 `__zw_get_attr_handle`（latest-wins from mutations）。
      // R3023：经 _zwMakeAttr 返真 Attr 实例（nodeType 2 + nodeName/nodeValue 全字段），非 plain object。
      var v;
      if (handle) v = __zw_get_attr_handle(handle, name);
      else if (typeof __zw_get_attr_lw === 'function') v = __zw_get_attr_lw(sel, name);
      else v = __zw_get_attr(sel, name);
      return _zwMakeAttr(name, v || '', _makeProxy(sel, handle));
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
        if (p === 'setNamedItem') {
          // R3022：真 mutation——setNamedItem(attr) 等价 setAttribute(attr.name, attr.value)（经元素 host 路径），
          // 返旧 Attr（或 null）。lib 经 element.attributes.setNamedItem(attr) 改属性（与 setAttribute 等价路径）。
          // 返值用 setAttribute 前捕获的 old（attrObj 会 latest-wins 重读到新值）。
          return function (attr) {
            if (!attr || attr.name == null) return null;
            var n = String(attr.name);
            var el = _makeProxy(sel, handle);
            var old = null;
            try { old = el.getAttribute(n); } catch (_e) {}
            try { el.setAttribute(n, attr.value != null ? String(attr.value) : ''); } catch (_e) {}
            // R3023：返真 Attr 实例（_zwMakeAttr，ownerElement=元素 proxy），非 plain {name,value}。
            return old != null && old !== '' ? _zwMakeAttr(n, old, el) : null;
          };
        }
        if (p === 'removeNamedItem') {
          // R3022：真 mutation——removeNamedItem(name) 等价 removeAttribute（经元素 host 路径），返移除 Attr（缺失返 null）。
          return function (name) {
            var n = String(name);
            var el = _makeProxy(sel, handle);
            var existed = null;
            try { existed = el.getAttribute(n); } catch (_e) {}
            try { el.removeAttribute(n); } catch (_e) {}
            return existed != null && existed !== '' ? _zwMakeAttr(n, existed, el) : null;
          };
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
  // vendor 前缀特例（CSSOM §CSSStyleDeclaration）：IDL 属性 `webkitXxx` → CSS 属性 `-webkit-xxx`
  //（`webkitLineClamp` → `-webkit-line-clamp`）——通用 `_camelToKebab` 产 `webkit-line-clamp`
  //（丢前导 `-`）→ CSS parser 不认 → 渲染静默失效。仅 webkit 前缀用无连字符 camelCase 暴露
  //（moz/ms/o 前缀 IDL 为 `MozXxx`/`msXxx`/`Oxxx`，罕见，保持通用路径）。
  function _stylePropName(name) {
    var s = String(name).trim();
    if (s === 'cssFloat') return 'float';
    if (s.charAt(0) === '-' && s.charAt(1) === '-') return s;
    var m = /^webkit([A-Z])(.*)$/.exec(s);
    if (m) return '-webkit-' + m[1].toLowerCase() + _camelToKebab(m[2]);
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
        // QuickJS Proxy ToPrimitive 差异（2026-08-08）：V8 对 get(Symbol.toPrimitive)
        // 返回 undefined 时回退默认 valueOf/toString；QuickJS 直接抛 TypeError: toPrimitive
        //（createElement handle proxy 被隐式字符串化——appendChild/observer id 等——
        // 在 QuickJS 下中断脚本）。显式返回字符串化函数（有 sel 用 sel，否则 handle），
        // 保证 v8/quickjs 接口行为一致。
        if (prop === Symbol.toPrimitive) {
          return function() { return sel ? sel : String(handle); };
        }
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
          // boolean reflected property（checked/hidden/disabled）——属性存在性。R2997：sel-based 改 latest-wins
          // （`__zw_has_attr_lw`）反映同批 SetAttr/RemoveAttr / `.checked=` setter 推的 mutation（旧读纯快照
          // `__zw_has_attr` → removeAttribute / .checked=false / .hidden=true 后 stale）。typeof guard 回落纯快照。
          // handle 经 `__zw_has_attr_handle`（latest-wins from mutations，R2993/R2995 已无 stale）。
          if (handle && typeof __zw_has_attr_handle === 'function') {
            try { return __zw_has_attr_handle(handle, String(prop)) === '1'; } catch (_e) {}
          }
          if (!handle && sel && typeof __zw_has_attr_lw === 'function') {
            try { return __zw_has_attr_lw(sel, String(prop)) === '1'; } catch (_e) {}
          }
          if (!handle && sel && typeof __zw_has_attr === 'function') {
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
          // P1a select option：selected 当前态属性存在性（boolean）。R2999：sel-based 改 latest-wins
          // （`__zw_has_attr_lw`）反映同批 SetAttr/RemoveAttr / `.selected=` setter 推的 mutation（旧读纯快照
          // `__zw_has_attr` → removeAttribute / .selected= / setAttribute 后 stale，R2997 限制 ②）。R3000：优先
          // `__zw_option_selected`（额外感知 SelectOption——`select.value=` 编程选中后 option.selected 反映），
          // 它内部已 consult SetAttr/RemoveAttr latest-wins + SelectOption + 快照；无该回调回落 `_lw`/快照链。
          // handle 经 `__zw_has_attr_handle`（latest-wins from mutations；`new Option()` 不在 select DOM，无 SelectOption）。
          if (handle && typeof __zw_has_attr_handle === 'function') {
            try { return __zw_has_attr_handle(handle, 'selected') === '1'; } catch (_e) {}
          }
          if (!handle && sel && typeof __zw_option_selected === 'function') {
            try { return __zw_option_selected(sel) === '1'; } catch (_e) {}
          }
          if (!handle && sel && typeof __zw_has_attr_lw === 'function') {
            try { return __zw_has_attr_lw(sel, 'selected') === '1'; } catch (_e) {}
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
        // `input.defaultValue`（HTMLInputElement，R2840）——反射 `value` 属性（初始值；区别 `.value` 当前态；
        // form reset 逻辑 / 校验库读 defaultValue 判「值是否改过」）。R2996：spec `.value=` 不改 defaultValue，
        // 但 shim `.value=` 仍写 value 属性供 render → 属性被污染；故 dirty 时（`.value=`/valueAsNumber= 后）返
        // 捕获的 `_inputDefault[key]`，非 dirty 时回落属性（latest-wins 反映 setAttribute('value')/defaultValue=）。
        if (prop === 'defaultValue' && _realTag(sel, handle) === 'INPUT') {
          if (_inputDefaultDirty[key]) return _inputDefault[key] || '';
          if (handle) return (__zw_get_attr_handle(handle, 'value') || '');
          if (typeof __zw_get_attr_lw === 'function') return __zw_get_attr_lw(sel, 'value') || '';
          return (__zw_get_attr(sel, 'value') || '');
        }
        // `input.defaultChecked`（HTMLInputElement，R2840）——反射 `checked` 属性存在性（初始选中态，区别
        // `.checked` 当前态；复选框 reset 逻辑读）。R2998：spec `.checked=` 不改 defaultChecked，但 shim
        // `.checked=` 仍写属性供 render → 属性被污染；故 dirty 时（`.checked=` 后）返捕获的 `_boolDefault`，
        // 非 dirty 时回落属性 latest-wins（反映 setAttribute('checked')/defaultChecked=/removeAttribute('checked')）。
        if (prop === 'defaultChecked' && _realTag(sel, handle) === 'INPUT') {
          var _dcKey = key + ':checked';
          if (_boolDefaultDirty[_dcKey]) return _boolDefault[_dcKey] === true;
          if (handle && typeof __zw_has_attr_handle === 'function') {
            try { return __zw_has_attr_handle(handle, 'checked') === '1'; } catch (_e) {}
          }
          if (!handle && sel && typeof __zw_has_attr_lw === 'function') {
            try { return __zw_has_attr_lw(sel, 'checked') === '1'; } catch (_e) {}
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
          // text = 显示文本（= textContent）。R3028：sel-based latest-wins，闭合 textContent= 后 stale。
          if (handle) return __zw_get_text_handle(handle);
          return typeof __zw_get_text_lw === 'function' ? __zw_get_text_lw(sel) : __zw_get_text(sel);
        }
        if (prop === 'label' && _realTag(sel, handle) === 'OPTION') {
          // label 属性；缺省回落 text。R3028：sel-based latest-wins 回落（同 text）。
          var lab = handle ? __zw_get_attr_handle(handle, 'label') : __zw_get_attr(sel, 'label');
          var _lt = handle ? __zw_get_text_handle(handle) : (typeof __zw_get_text_lw === 'function' ? __zw_get_text_lw(sel) : __zw_get_text(sel));
          return lab || _lt || '';
        }
        if (prop === 'defaultSelected' && _realTag(sel, handle) === 'OPTION') {
          // defaultSelected = 'selected' 属性存在性（boolean reflected，初始选中态）。R2998：spec `.selected=`
          // 不改 defaultSelected，但 shim `.selected=` 仍写属性 → 属性被污染；dirty 时（`.selected=` 后）返捕获的
          // `_boolDefault`，非 dirty 回落属性 latest-wins（反映 setAttribute/removeAttribute('selected')）。
          // handle-based（`new Option()` 创建）经 `__zw_has_attr_handle`（句柄不在快照）。
          var _dsKey = key + ':selected';
          if (_boolDefaultDirty[_dsKey]) return _boolDefault[_dsKey] === true;
          if (handle && typeof __zw_has_attr_handle === 'function') {
            try { return __zw_has_attr_handle(handle, 'selected') === '1'; } catch (_e) {}
          }
          if (!handle && sel && typeof __zw_has_attr_lw === 'function') {
            try { return __zw_has_attr_lw(sel, 'selected') === '1'; } catch (_e) {}
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
