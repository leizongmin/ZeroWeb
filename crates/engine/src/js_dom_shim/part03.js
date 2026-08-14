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
    // R40：window 注册打 tgt='win' 标（document/window/html 三合一 key 内槽位区分）。
    _listenerStore[key][t].push({ fn: fn, capture: _optCapture(opts), once: _optOnce(opts), tgt: 'win' });
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
    // R40：window 注册带 tgt='win'，移除也限定同槽位（避免误删 document 同 fn 同 capture 注册）。
    _listenerStore[key][t] = _listenerStore[key][t].filter(function(l) {
      return !(l.fn === fn && l.capture === cap && l.tgt === 'win');
    });
  }

  // R3270：Node/Element/HTMLElement 用 `||` 守卫——native_dom 模式下 native bindings（S5a R3264）已注册
  // 真实 native HTMLElement（FunctionTemplate + 完整 Element/Node 接口 R3268），polyfill 不得覆盖（否则
  // `class X extends HTMLElement` 继承 polyfill stub 无 native slot，upgrade 坏）。native_dom 关闭时
  //（polyfill-only 路径）三者未定义 → polyfill 定义 stub + 建原型链（Node→Element→HTMLElement）供 polyfill
  // 元素 proxy 的 instanceof 校验。HTMLFormElement 等子类经 `Object.create(HTMLElement.prototype)` 继承——
  // native_dom 下继承 native HTMLElement.prototype（含 R3268 接口），polyfill-only 下继承 polyfill stub prototype。
  var _zwBuiltNodeChain = !globalThis.HTMLElement; // polyfill 是否自建三者（native 已注册则 false）
  if (!globalThis.Node) globalThis.Node = function Node() {};
  if (!globalThis.Element) globalThis.Element = function Element() {};
  if (!globalThis.HTMLElement) globalThis.HTMLElement = function HTMLElement() {};
  // prototype 链仅当 polyfill 自建三者时设（native 已注册则不重设——避免破坏 native prototype）。
  if (_zwBuiltNodeChain) {
    globalThis.Node.prototype = {};
    globalThis.Element.prototype = Object.create(globalThis.Node.prototype);
    globalThis.HTMLElement.prototype = Object.create(globalThis.Element.prototype);
  }
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
  // js-dom M4：ProcessingInstruction 构造器占位（spec `dom-processinginstruction`，CharacterData : Node 子类）。
  // createProcessingInstruction 返 polyfill Proxy 节点（非构造器真实例），instanceof 恒 false（与
  // HTMLFormElement 占位同语义）；构造器须以 function 存在，使 `x instanceof ProcessingInstruction` 不抛
  // TypeError。原型挂 Node.prototype（PI instanceof Node 经原型链，polyfill Proxy instanceof 仍 false，记入
  // R8 instanceof 89 块缺口，独立切片）。
  globalThis.ProcessingInstruction = globalThis.ProcessingInstruction || function ProcessingInstruction() {};
  globalThis.ProcessingInstruction.prototype = Object.create(globalThis.Node.prototype);
  // js-dom M4 R11：HTML 元素接口子类构造器（spec HTML 元素接口表，~64 个）——`el instanceof HTMLDivElement`
  // 等（Node-cloneNode 用例 `create_element_and_check` 对每个 tag 查对应接口）。构造器占位（polyfill Proxy 非
  // 真实例，instanceof 经 getPrototypeOf 返对应 prototype 为 true，R10 getPrototypeOf + R11 tag 映射）。
  // 原型链：HTML*Element.prototype → HTMLElement.prototype → Element.prototype → Node.prototype。
  // 已注册（如 HTMLFormElement）跳过，避免覆盖既有 prototype 成员。
  var _zwHtmlElementIfaces = [
    'HTMLAnchorElement','HTMLAreaElement','HTMLAudioElement','HTMLBRElement','HTMLBaseElement',
    'HTMLBodyElement','HTMLButtonElement','HTMLCanvasElement','HTMLDListElement','HTMLDataElement',
    'HTMLDataListElement','HTMLDialogElement','HTMLDirectoryElement','HTMLDivElement','HTMLElement',
    'HTMLEmbedElement','HTMLFieldSetElement','HTMLFontElement','HTMLFrameElement','HTMLFrameSetElement',
    'HTMLHRElement','HTMLHeadElement','HTMLHeadingElement','HTMLHtmlElement','HTMLIFrameElement',
    'HTMLImageElement','HTMLInputElement','HTMLLIElement','HTMLLabelElement','HTMLLegendElement',
    'HTMLLinkElement','HTMLMapElement','HTMLMediaElement','HTMLMenuElement','HTMLMetaElement','HTMLMeterElement',
    'HTMLModElement','HTMLOListElement','HTMLObjectElement','HTMLOptGroupElement','HTMLOptionElement',
    'HTMLOutputElement','HTMLParagraphElement','HTMLParamElement','HTMLPictureElement','HTMLPreElement',
    'HTMLProgressElement','HTMLQuoteElement','HTMLScriptElement','HTMLSelectElement','HTMLSlotElement',
    'HTMLSourceElement','HTMLSpanElement','HTMLStyleElement','HTMLTableCaptionElement','HTMLTableCellElement',
    'HTMLTableColElement','HTMLTableElement','HTMLTableRowElement','HTMLTableSectionElement','HTMLTemplateElement',
    'HTMLTextAreaElement','HTMLTimeElement','HTMLTitleElement','HTMLTrackElement','HTMLUListElement',
    'HTMLUnknownElement','HTMLVideoElement',
  ];
  for (var _zi = 0; _zi < _zwHtmlElementIfaces.length; _zi++) {
    var _zn = _zwHtmlElementIfaces[_zi];
    if (!globalThis[_zn]) {
      globalThis[_zn] = new Function('return function ' + _zn + '() {}')();
      // HTMLElement 自身 prototype 已建（_zwBuiltNodeChain）；其余子类 → HTMLElement.prototype。
      globalThis[_zn].prototype = (_zn === 'HTMLElement')
        ? globalThis.HTMLElement.prototype
        : Object.create(globalThis.HTMLElement.prototype);
    }
  }
  // spec tag → HTML 元素接口名映射（HTMLElement 接口表，html.spec.whatwg.org#toc-named-given）。
  // getPrototypeOf（part05）按元素 tag 查此表返对应子类 prototype；未知/自定义元素 → HTMLUnknownElement。
  // R11：使 `document.createElement('div') instanceof HTMLDivElement` 为 true（Node-cloneNode 用例）。
  globalThis.__zwHtmlTagIface = {
    a: 'HTMLAnchorElement', abbr: 'HTMLElement', acronym: 'HTMLElement', address: 'HTMLElement',
    area: 'HTMLAreaElement', article: 'HTMLElement', aside: 'HTMLElement', audio: 'HTMLAudioElement',
    b: 'HTMLElement', base: 'HTMLBaseElement', bdi: 'HTMLElement', bdo: 'HTMLElement', bgsound: 'HTMLElement',
    big: 'HTMLElement', blockquote: 'HTMLQuoteElement', body: 'HTMLBodyElement', br: 'HTMLBRElement',
    button: 'HTMLButtonElement', canvas: 'HTMLCanvasElement', caption: 'HTMLTableCaptionElement',
    center: 'HTMLElement', cite: 'HTMLElement', code: 'HTMLElement', col: 'HTMLTableColElement',
    colgroup: 'HTMLTableColElement', data: 'HTMLDataElement', datalist: 'HTMLDataListElement',
    dd: 'HTMLElement', del: 'HTMLModElement', details: 'HTMLElement', dfn: 'HTMLElement',
    dialog: 'HTMLDialogElement', dir: 'HTMLDirectoryElement', div: 'HTMLDivElement', dl: 'HTMLDListElement',
    dt: 'HTMLElement', em: 'HTMLElement', embed: 'HTMLEmbedElement', fieldset: 'HTMLFieldSetElement',
    figcaption: 'HTMLElement', figure: 'HTMLElement', font: 'HTMLFontElement', footer: 'HTMLElement',
    form: 'HTMLFormElement', frame: 'HTMLFrameElement', frameset: 'HTMLFrameSetElement',
    h1: 'HTMLHeadingElement', h2: 'HTMLHeadingElement', h3: 'HTMLHeadingElement', h4: 'HTMLHeadingElement',
    h5: 'HTMLHeadingElement', h6: 'HTMLHeadingElement', head: 'HTMLHeadElement', header: 'HTMLElement',
    hgroup: 'HTMLElement', hr: 'HTMLHRElement', html: 'HTMLHtmlElement', i: 'HTMLElement',
    iframe: 'HTMLIFrameElement', img: 'HTMLImageElement', input: 'HTMLInputElement', ins: 'HTMLModElement',
    kbd: 'HTMLElement', label: 'HTMLLabelElement', legend: 'HTMLLegendElement', li: 'HTMLLIElement',
    link: 'HTMLLinkElement', listing: 'HTMLPreElement', main: 'HTMLElement', map: 'HTMLMapElement',
    mark: 'HTMLElement', marquee: 'HTMLElement', menu: 'HTMLMenuElement', meta: 'HTMLMetaElement',
    meter: 'HTMLMeterElement', nav: 'HTMLElement', nobr: 'HTMLElement', noembed: 'HTMLElement',
    noframes: 'HTMLElement', noscript: 'HTMLElement', object: 'HTMLObjectElement', ol: 'HTMLOListElement',
    optgroup: 'HTMLOptGroupElement', option: 'HTMLOptionElement', output: 'HTMLOutputElement',
    p: 'HTMLParagraphElement', param: 'HTMLParamElement', picture: 'HTMLPictureElement', plaintext: 'HTMLElement',
    pre: 'HTMLPreElement', progress: 'HTMLProgressElement', q: 'HTMLQuoteElement', rp: 'HTMLElement',
    rt: 'HTMLElement', ruby: 'HTMLElement', s: 'HTMLElement', samp: 'HTMLElement', script: 'HTMLScriptElement',
    section: 'HTMLElement', select: 'HTMLSelectElement', slot: 'HTMLSlotElement', small: 'HTMLElement',
    source: 'HTMLSourceElement', span: 'HTMLSpanElement', strike: 'HTMLElement', strong: 'HTMLElement',
    style: 'HTMLStyleElement', sub: 'HTMLElement', summary: 'HTMLElement', sup: 'HTMLElement',
    table: 'HTMLTableElement', tbody: 'HTMLTableSectionElement', td: 'HTMLTableCellElement',
    template: 'HTMLTemplateElement', textarea: 'HTMLTextAreaElement', tfoot: 'HTMLTableSectionElement',
    th: 'HTMLTableCellElement', thead: 'HTMLTableSectionElement', time: 'HTMLTimeElement',
    title: 'HTMLTitleElement', tr: 'HTMLTableRowElement', track: 'HTMLTrackElement', tt: 'HTMLElement',
    u: 'HTMLElement', ul: 'HTMLUListElement', var: 'HTMLElement', video: 'HTMLVideoElement',
    wbr: 'HTMLElement', xmp: 'HTMLPreElement',
  };
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
  // DOM 原型方法设为不可枚举（spec：WebIDL 操作默认 enumerable:false）——R10 getPrototypeOf 让 polyfill
  // Proxy 原型链含 HTMLElement/Element.prototype，若这些方法可枚举会污染 for...in（expando 枚举回归）。
  function _zwDefProtoMethod(proto, name, fn) {
    Object.defineProperty(proto, name, { value: fn, writable: true, configurable: true, enumerable: false });
  }
  _zwDefProtoMethod(globalThis.Element.prototype, 'cloneNode', function (deep) {
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
  });
  // Node.DOCUMENT_POSITION_* 静态常量（compareDocumentPosition bitmask，R2815）——库常读 Node.DOCUMENT_POSITION_FOLLOWING 等。
  globalThis.Node.DOCUMENT_POSITION_DISCONNECTED = 1;
  globalThis.Node.DOCUMENT_POSITION_PRECEDING = 2;
  globalThis.Node.DOCUMENT_POSITION_FOLLOWING = 4;
  globalThis.Node.DOCUMENT_POSITION_CONTAINS = 8;
  globalThis.Node.DOCUMENT_POSITION_CONTAINED_BY = 16;
  globalThis.Node.DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC = 32;
  _zwDefProtoMethod(globalThis.Element.prototype, 'addEventListener', function(type, fn, opts) {
    _globalAddEventListener(type, fn, opts);
  });
  _zwDefProtoMethod(globalThis.Element.prototype, 'removeEventListener', function(type, fn, opts) {
    _globalRemoveEventListener(type, fn, opts);
  });

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
    // upgrade(root)（R3269）：spec `custom-elements-upgrade`——遍历 root 子树（含 root 自身），对每个 tag
    // 命中已注册 custom element 名的元素，`Object.setPrototypeOf(el, ctor.prototype)` 升级为 custom 实例
    //（保留 NodeId + 子树 + 属性，**不重建元素**——R3269 PoC 验证 setPrototypeOf 对 native 元素可行：instanceof
    // registeredCtor 成立 + slot[0]=NodeId 保留 + nodeType accessor 仍可读）。升级后若元素已连入 document，
    // 触发 connectedCallback（经 ctor.prototype.connectedCallback，this=el）。
    // **native_dom 模式**：parser 建的 `<my-el>`（define 前已存在）是普通 native 元素，define 后 upgrade 升级。
    // **polyfill 模式**：元素为 generic Proxy，setPrototypeOf 改 Proxy 的 prototype（非 spec 严格语义，但
    // best-effort 让 instanceof 成立 + prototype 方法可达）。spec 返 undefined。
    upgrade: function (root) {
      if (!root) return;
      try {
        _ceUpgradeSubtree(root);
      } catch (_e) {}
    },
  };

  // R3269 upgrade 子树遍历：DFS（firstChild → nextSibling），对每个 Element 节点，tag 命中 registry 则
  // setPrototypeOf 升级 + 已连入 document 触发 connectedCallback。Text/Comment 跳过（无 tag）。
  function _ceUpgradeNode(el) {
    var tag = _elTagName(el);
    if (tag) {
      var entry = _ce_registry[tag.toLowerCase()];
      if (entry && entry.ctor) {
        try { Object.setPrototypeOf(el, entry.ctor.prototype); } catch (_e) {}
        // R3274：升级时对 ctor.observedAttributes 派发初始 attributeChangedCallback（name, null, 当前值）。
        // 元素升级前可能已设属性（parser 建 / createElement + setAttribute 未注册时），升级后组件须能响应
        // 这些既有属性（lit/stencil 等框架依赖此初始化路径）。spec `custom-elements-upgrades`「upgrade a
        // custom element」enqueue step。在 connectedCallback 前派发（spec：attr change 先于 connected）。
        _ceFireInitialAttrChanges(el, entry.ctor);
        // 升级后若已连入 document，触发 connectedCallback（spec：upgrade 已 connected 的元素触发回调）。
        if (_elConnected(el)) {
          var ccb = entry.ctor.prototype && entry.ctor.prototype.connectedCallback;
          if (typeof ccb === 'function') { try { ccb.call(el); } catch (_e) {} }
        }
      }
    }
    // 递归子节点（firstChild → nextSibling 链）。
    var child = el.firstChild;
    while (child) {
      var next = child.nextSibling;
      _ceUpgradeNode(child);
      child = next;
    }
  }
  function _ceUpgradeSubtree(root) {
    _ceUpgradeNode(root);
  }
  // R3274：升级时对 ctor.observedAttributes 派发初始 attributeChangedCallback（name, null, 当前值）。
  // observedAttributes 缺失（ctor 无静态 getter）/ 空 → 无回调。getAttribute 经元素自身方法读（native
  // getter R3268 或 polyfill Proxy trap）；属性不存在 → value=null（spec 仍 enqueue，best-effort 派发 null
  // 让组件可初始化）。attributeChangedCallback 缺失 → 不派发（仅 observed 列表存在时）。
  function _ceFireInitialAttrChanges(el, ctor) {
    var observed;
    try {
      observed = ctor.observedAttributes;
    } catch (_e) { return; }
    if (!observed || typeof observed.length !== 'number' || observed.length === 0) return;
    var proto = ctor.prototype;
    var acb = proto && proto.attributeChangedCallback;
    if (typeof acb !== 'function') return;
    for (var i = 0; i < observed.length; i++) {
      var name = String(observed[i]);
      var value = null;
      try {
        if (typeof el.getAttribute === 'function') {
          var v = el.getAttribute(name);
          value = (v === null || v === undefined) ? null : String(v);
        }
      } catch (_e) { value = null; }
      try { acb.call(el, name, null, value); } catch (_e) {}
    }
  }
  // 读元素 tagName（小写 local name）——native 元素经 tagName getter（R3268）；polyfill Proxy 经 _realTag。
  function _elTagName(el) {
    try {
      var t = el.tagName;
      return t ? String(t).toLowerCase() : '';
    } catch (_e) { return ''; }
  }
  // 元素是否连入 document——parent 链中存在 nodeType===9（DOCUMENT_NODE）= connected（spec isConnected 近似）。
  // native 元素经 parentNode getter（R3268）；detached 元素 parent 链无 document → false。
  function _elConnected(el) {
    var cur = el;
    var guard = 0;
    while (cur && guard < 10000) {
      try {
        if (cur.nodeType === 9) return true; // 到达 document 节点
      } catch (_e) { return false; }
      var p;
      try { p = cur.parentNode; } catch (_e) { return false; }
      if (p === null || p === undefined) return false; // detached 子树根（无 document 祖先）
      cur = p;
      guard++;
    }
    return false;
  }

  // P1b S5b（R3265）：customElements registry 反查 hook——供 native_dom 路径 `document.createElement(tag)`
  //（Rust `native_create_element_invoke`）在 host 建元素后反查是否为已注册 custom element。命中返 ctor
  //（Rust 侧 `new_instance` 触发 super() → native HTMLElement ctor 复用 host NodeId 填 slot[0]，产出 native
  // custom 实例），未命中返 null。native_dom 关闭时此函数定义但无人调用（零开销）。registry 在 polyfill 闭包
  // 内，故经本全局函数暴露只读查询（不暴露内部 Map/对象引用）。
  globalThis.__zw_native_ce_lookup = globalThis.__zw_native_ce_lookup || function (tag) {
    var entry = _ce_registry[tag];
    return entry ? entry.ctor : null;
  };

  // P1b S5c（R3266）：custom element 连接态 lifecycle 派发 hook——native_dom 路径 appendChild/
  // insertBefore/removeChild（Rust 直接改 DOM）绕过本 polyfill 的 `_ceApplyConn`（基于 sel/handle），
  // 故 connectedCallback/disconnectedCallback 不触发。Rust `custom_elements` 模块追踪连接态（树逻辑），
  // 状态真转时调本 hook：对每个 native 实例按 tag 查 `_ce_registry` + 调 ctor.prototype 回调（this=native
  // 实例）。JS 负责「调 ctor.prototype 回调」（有 ctor/prototype），Rust 负责「什么变了、连没连」（有 DOM 树）。
  // instances[i] / tags[i] 并列配对；tags 为小写 tag 名（registry 键）。回调异常 try/catch 吞（不中断脚本）。
  // native_dom 关闭时此函数定义但无人调用（零开销）。S5c 切片：连接态 callback；attributeChangedCallback
  // 经 setAttribute polyfill trap 已就绪（native_dom 下 setAttribute 走 Rust，attr 派发为后续）。
  globalThis.__zw_native_ce_notify_connect =
    globalThis.__zw_native_ce_notify_connect ||
    function (instances, connected, tags) {
      if (!instances || !tags || instances.length !== tags.length) return;
      for (var i = 0; i < instances.length; i++) {
        var entry = _ce_registry[tags[i]];
        if (!entry || !entry.ctor) continue;
        var proto = entry.ctor.prototype;
        if (!proto) continue;
        var cb = connected ? proto.connectedCallback : proto.disconnectedCallback;
        if (typeof cb === 'function') {
          try { cb.call(instances[i]); } catch (_e) {}
        }
      }
    };

  // P1b S5d（R3267）：custom element attributeChangedCallback 派发 hook——native_dom 路径
  // setAttribute/removeAttribute/toggleAttribute（Rust 直接改 DOM）绕过本 polyfill 的 setAttribute trap
  //（含 `_ce_dispatchAttrChange`），故 attributeChangedCallback 不触发。Rust `custom_elements` 模块在
  // mutation 前读 oldVal + tag，mutation 后调本 hook：按 tag 查 `_ce_registry` 取 entry → 调既有
  // `_ce_dispatchAttrChange`（observedAttributes 过滤 + 值真变判定 + 调 ctor.prototype.attributeChangedCallback，
  // this=native 实例）。Rust 负责「读 old/new、判 tag」，JS 负责「observedAttributes 过滤 + 调回调」（复用
  // `_ce_dispatchAttrChange`）。native_dom 关闭时此函数定义但无人调用（零开销）。
  globalThis.__zw_native_ce_notify_attr_change =
    globalThis.__zw_native_ce_notify_attr_change ||
    function (instance, name, oldVal, newVal, tag) {
      if (!instance || !tag) return;
      var entry = _ce_registry[String(tag).toLowerCase()];
      if (!entry) return; // 非 custom 元素
      _ce_dispatchAttrChange(entry, instance, name, oldVal, newVal);
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
  // present → 值串。经 has-attr 判存在（handle 元素用 __zw_has_attr_handle，sel 用 __zw_has_attr_lw latest-wins），
  // 区别于 get-attr 对 absent 返 ''（无法区分 absent 与空串值）。
  // R3205：sel 路径 latest-wins（`__zw_has_attr_lw`/`__zw_get_attr_lw`）反映同批 setAttribute——parsed-CE 同批
  // 连续 setAttribute 时第 N 次回调的 old 须读第 N-1 次设值（旧纯快照读 stale 致 old=null）。handle 路径本就
  // latest-wins（`__zw_*_handle` 读 mutations）。闭合 `__zw_has_attr(sel)` stale 审计最后一项。
  function _ce_attrValue(sel, handle, name) {
    var present = false;
    if (handle && typeof __zw_has_attr_handle === 'function') {
      try { present = __zw_has_attr_handle(handle, name) === '1'; } catch (_e) { present = false; }
    } else if (sel && typeof __zw_has_attr_lw === 'function') {
      try { present = __zw_has_attr_lw(sel, name) === '1'; } catch (_e) { present = false; }
    } else if (sel && typeof __zw_has_attr === 'function') {
      try { present = __zw_has_attr(sel, name) === '1'; } catch (_e) { present = false; }
    }
    if (!present) return null;
    try {
      if (handle) return __zw_get_attr_handle(handle, name);
      if (typeof __zw_get_attr_lw === 'function') return __zw_get_attr_lw(sel, name);
      return __zw_get_attr(sel, name);
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

  // Constraint Validation ValidityState（R2825）。customError 由 setCustomValidity 跟踪；tooLong/tooShort
  // 仅对宿主标记的用户编辑值计算，脚本 `.value=` 不触发。其余原生约束仍为 permissive false。
  // https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#the-constraint-validation-api
  function _validityState(key, sel, handle) {
    var hasCustom = _customValidity[key] != null && _customValidity[key] !== '';
    var tooLong = false, tooShort = false;
    if (_userEdited[key] && _isTextControl(sel, handle)) {
      var value = _controlValue(sel, handle, key);
      var readLength = function(name) {
        var raw = '';
        try {
          raw = handle
            ? __zw_get_attr_handle(handle, name)
            : (typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(sel, name) : __zw_get_attr(sel, name));
        } catch (_e) {}
        if (raw == null || raw === '') return null;
        var parsed = Number(raw);
        return Number.isInteger(parsed) && parsed >= 0 ? parsed : null;
      };
      var maxLength = readLength('maxlength');
      var minLength = readLength('minlength');
      tooLong = maxLength !== null && value.length > maxLength;
      tooShort = value.length > 0 && minLength !== null && value.length < minLength;
    }
    return {
      valueMissing: false, typeMismatch: false, patternMismatch: false,
      tooLong: tooLong, tooShort: tooShort, rangeUnderflow: false, rangeOverflow: false,
      stepMismatch: false, badInput: false, customError: hasCustom,
      valid: !hasCustom && !tooLong && !tooShort,
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
    // R3067：入 per-element 动画注册表（elKey 复用 _elKey(sel,handle)），供 Element/Document.getAnimations() 查询。
    var _akey = _elKey(sel, handle);
    (_elementAnimations[_akey] || (_elementAnimations[_akey] = [])).push(anim);
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

  // R3032：`classList` 完整 DOMTokenList。旧实现仅 add/remove/toggle/contains，缺 `toggle(token,force)`
  //（force 参被忽略——常见 `classList.toggle('x', cond)` 模式失效）、`replace`、`item`、`length`、indexed 访问、
  // `forEach`、`toString`/`value`、variadic add/remove、`Symbol.iterator`。modern 框架/库高频用 length/indexed/
  // forEach 迭代 + replace + toggle(cond) 条件切换。Proxy 暴露动态 length + indexed 访问（每次读 live 列表）。
  function _classListProxy(sel, handle) {
    var key = _elKey(sel, handle);
    // R19：per-element 缓存——spec classList accessor 每次返回同一 DOMTokenList 对象（WPT identity 断言）。
    if (_clsProxyCache[key]) return _clsProxyCache[key];
    // 当前列表（缓存优先，反映同脚本内累积操作，非 stale snapshot）。
    // spec DOMTokenList：token 集合为**有序去重**（首个出现位置保留，后续重复丢弃）+ 按 ASCII 空白分隔。
    // `"a a a"` → `["a"]`（length 1）；`"\t\n\f\r a\t\n\f\r b\t\n\f\r "` → `["a","b"]`（R13 classlist 去重）。
    var cur = function () {
      var raw = _readClass(key, sel, handle).split(/\s+/).filter(Boolean);
      var seen = Object.create(null);
      var out = [];
      for (var i = 0; i < raw.length; i++) {
        var t = raw[i];
        if (!seen[t]) { seen[t] = 1; out.push(t); }
      }
      return out;
    };
    var write = function (arr, force) {
      // spec DOMTokenList runUpdate：比较「新 token 集合序列化（单空格分隔）」与「原 attribute 原始值」，
      // 不同才 setAttribute（避免无谓 mutation；MutationObserver 检查依赖此）。add/remove 总经此
      //（即使 token 集合不变，原值含尾空格/重复时仍规范化重写，WPT checkAdd("a b c ",["a","a"],"a b c")）。
      // toggle 的 force 分支无变化时直接 return 不调 write（spec toggle no-op，WPT checkToggle 保持原样）。
      // **replace 例外**（R19）：spec `dom-domtokenlist-replace` 返 true 时**必触发 mutation**（即使规范化后
      // attribute 值未变——WPT checkReplace("a","a","a",true,"a") 期望 mutation），故 replace 调 `write(p, true)`
      // 强制 setAttribute + notify，绕过 runUpdate 的「值相同 return」。返 false（oldT 不存在）时不 write。
      var v = arr.join(' ');
      if (!force && v === _readClass(key, sel, handle)) return;
      // js-dom M4 R45：classList write 的 attributeOldValue——写入前捕获（同 IDL setter 模式）。
      var _clsMoId = _mo_id(handle, sel);
      var _clsOld = (_clsMoId != null && _mo_any_wants_attr_old(_clsMoId, 'class'))
        ? _mo_read_attr(sel, handle, 'class') : null;
      _classCache[key] = v;
      if (handle) __zw_set_attr_handle(handle, 'class', v);
      else __zw_set_attr(sel, 'class', v);
      _mo_notify(sel, handle, { type: 'attributes', attributeName: 'class', oldValue: _clsOld });
    };
    // DOMTokenList token 校验（spec `dom-domtokenlist-validation`）：空串 → SyntaxError
    // DOMException（code 12）；含 ASCII 空白 → InvalidCharacterError DOMException（code 5）。
    // 用 `globalThis.DOMException`（native_dom=true 叠加路径下 = 原生 DOMException；纯 polyfill
    // 下 = part01b 的）——保证抛的异常 `e.constructor === self.DOMException`（WPT assert_throws_dom
    // "wrong global" 要求）。若用词法作用域的 part01b DOMException，叠加路径下 shim 实例
    // constructor（part01b）≠ 全局 native DOMException → wrong global（R6 定位修复）。
    var check = function (t) {
      var s = String(t);
      var DOMEx = globalThis.DOMException;
      if (s === '') {
        throw new DOMEx("An invalid or illegal string was specified.", "SyntaxError");
      }
      if (/\s/.test(s)) {
        throw new DOMEx("An invalid or illegal string was specified.", "InvalidCharacterError");
      }
    };
    var target = {
      add: function () {
        var p = cur();
        for (var i = 0; i < arguments.length; i++) {
          var c = String(arguments[i]);
          check(c);
          if (p.indexOf(c) < 0) p.push(c);
        }
        write(p);
      },
      remove: function () {
        var p = cur();
        for (var j = 0; j < arguments.length; j++) {
          var r = String(arguments[j]);
          check(r);
          p = p.filter(function (x) { return x !== r; });
        }
        write(p);
      },
      contains: function (c) {
        c = String(c);
        // spec `dom-domtokenlist-contains`（R13）：空串或含 ASCII 空白的 token → 返 false（**不抛**，
        // 区别于 add/remove/toggle/replace 的 check 抛 SyntaxError/InvalidCharacterError）。WPT
        // Element-classlist checkContains(null,["a","","  "],false) + checkContains("a",["a\t",...],false)。
        if (c === '' || /\s/.test(c)) return false;
        return cur().indexOf(c) >= 0;
      },
      toggle: function (c, force) {
        c = String(c);
        check(c);
        var p = cur();
        var i = p.indexOf(c);
        var on;
        // force≠undefined：force true→加、false→移除（不切换）；force undefined→切换。
        if (force !== undefined) {
          on = !!force;
          // spec toggle(token, force)：force 与现状一致（on 且已在 / off 且不在）→ **no-op，不触发 update**
          //（WPT checkToggle("a a a  b","a",true)→保持原样 "a a a  b"，不规范化）。仅状态冲突时修改 + write。
          if (on && i < 0) p.push(c);
          else if (!on && i >= 0) p.splice(i, 1);
          else return on; // 无变化 no-op（不 write，保持 attribute 原样）
        } else if (i >= 0) {
          p.splice(i, 1);
          on = false;
        } else {
          p.push(c);
          on = true;
        }
        write(p);
        return on;
      },
      replace: function (oldT, newT) {
        oldT = String(oldT);
        newT = String(newT);
        // spec `dom-domtokenlist-replace` 校验顺序**特殊**（区别 add/remove 的逐参先空后空白）：
        // 先校验**两个** token 的空串（SyntaxError），再校验**两个**的 ASCII 空白（InvalidCharacterError）。
        // 故 `replace(" ","")` → newT="" 先抛 SyntaxError（非 oldT=" " 的 InvalidCharacterError）。
        // WPT checkReplace(null," ","",...,"SyntaxError")。用 globalThis.DOMException 保 identity（R6）。
        var DOMEx = globalThis.DOMException;
        if (oldT === '' || newT === '') {
          throw new DOMEx("An invalid or illegal string was specified.", "SyntaxError");
        }
        if (/\s/.test(oldT) || /\s/.test(newT)) {
          throw new DOMEx("An invalid or illegal string was specified.", "InvalidCharacterError");
        }
        // spec：oldT===newT 时若 oldT 存在返 true 且 runUpdate（规范化 attribute）。R19：返 true 必触发
        // mutation（即使规范化后值未变，WPT checkReplace("a","a","a",true,"a") 期望 mutation）→ write(cur(), true)。
        if (oldT === newT) {
          var c0 = cur();
          if (c0.indexOf(oldT) < 0) return false;
          write(c0, true); // runUpdate + 强制 mutation
          return true;
        }
        var p = cur();
        var i = p.indexOf(oldT);
        if (i < 0) return false;
        // spec `dom-domtokenlist-replace`：在 oldT 首位置替换为 newT，结果保持有序去重（首个出现位置保留）。
        // WPT checkReplace：
        //   "a b c","c","a" → "a b"（c@2 换 a → [a,b,a] 去重保首个 a@0 → [a,b]）
        //   "c b a","c","a" → "a b"（c@0 换 a → [a,b,a] 去重保首个 a@0 → [a,b]）
        //   "a b c","b","d" → "a d c"（b@1 换 d，d 不在 → [a,d,c]）
        // 算法：splice 替换 oldT 为 newT，然后全局有序去重（与 cur() 同款 seen 表，首个保留）。
        var p = cur();
        var i = p.indexOf(oldT);
        if (i < 0) return false;
        p.splice(i, 1, newT);
        // 全局有序去重（保留每个 token 首次出现位置）。涵盖 newT 替换 oldT 后与其他位置的重复。
        var seen = Object.create(null);
        var out = [];
        for (var k = 0; k < p.length; k++) {
          if (!seen[p[k]]) { seen[p[k]] = 1; out.push(p[k]); }
        }
        // R19：返 true 必触发 mutation（强制 write，绕过 runUpdate「值相同 return」）。
        write(out, true);
        return true;
      },
      item: function (i) {
        var p = cur();
        i = i | 0;
        return i < 0 || i >= p.length ? null : p[i];
      },
      forEach: function (cb, thisArg) {
        var p = cur();
        for (var k = 0; k < p.length; k++) cb.call(thisArg, p[k], k, proxy);
      },
      toString: function () { return _readClass(key, sel, handle); },
      entries: function () { var p = cur(); var n = 0; return { next: function () { return n < p.length ? { value: [n, p[n++]], done: false } : { value: undefined, done: true }; } }; },
      keys: function () { var p = cur(); var n = 0; return { next: function () { return n < p.length ? { value: n++, done: false } : { value: undefined, done: true }; } }; },
      values: function () { var p = cur(); var n = 0; return { next: function () { return n < p.length ? { value: p[n++], done: false } : { value: undefined, done: true }; } }; },
    };
    // Proxy：length + indexed 访问动态读 live 列表；value/nodeValue 返当前 class 串；Symbol iterable。
    var proxy = new Proxy(target, {
      get: function (_t, prop) {
        if (prop === 'length') return cur().length;
        if (prop === 'value' || prop === 'nodeValue') return _readClass(key, sel, handle);
        if (typeof prop !== 'string') return target[prop];
        if (/^\d+$/.test(prop)) {
          var p = cur();
          var idx = +prop;
          return idx < p.length ? p[idx] : undefined;
        }
        return target[prop];
      },
    });
    target[Symbol.iterator] = target.values;
    _clsProxyCache[key] = proxy; // R19：缓存，同元素 get 始终返同一 DOMTokenList（spec identity）
    return proxy;
  }

  // 派发某元素 key 上的事件 listener。`phase`：`'all'`（target 阶段，capture+非 capture，默认）、
  // `'capture'`（仅 capture listener，捕获期祖先用）、`'bubble'`（仅非 capture，冒泡期祖先用）。
  // `thisObj`：handler 内 `this` 与 `event.currentTarget`（默认 event.target）。`stopImmediatePropagation`
  // 中断当前节点内后续 listener。`once` listener（`{once:true}` 注册）派发后自动移除——用快照迭代，
  // 派发完一次性从原 list 滤除已触发的 once 条目（不扰动迭代；reentrancy 下按对象引用滤除安全）。
  function _dispatchToListeners(key, event, phase, thisObj, slot) {
    var listeners = _listenerStore[key];
    if (!listeners || !listeners[event.type]) return !event._defaultPrevented;
    var list = listeners[event.type];
    // js-dom M4 R40：槽位过滤——document/window listener 与 html 元素 listener 共存于 _elKey('html') key
    //（三合一，postMessage/onerror/inline-handler 依赖）。entry.tgt 标记注册目标（'doc'/'win'/undefined=html
    // 元素注册）。派发虚站（document 站只触发 tgt==='doc'，window 站只触发 tgt==='win'，html 站只触发
    // 无标记的）保证 currentTarget 身份与注册目标一致（spec：listener 在其注册的节点上触发）。
    // slot 为 undefined → 无过滤（旧行为：全部触发），兼容既存 12 个直接派发点（lifecycle/postMessage 等
    // 「任一注册都触发」语义）。
    if (slot !== undefined) {
      if (slot === null) {
        // null slot = html 元素站：只触发无标记（html 元素 addEventListener）注册。
        list = list.filter(function(e) { return e.tgt === undefined; });
      } else {
        list = list.filter(function(e) { return e.tgt === slot; });
      }
    }
    var ctx = thisObj || event.target;
    event.currentTarget = ctx;
    // js-dom M4 R35：spec `concept-event-dispatch`——派发期 event.eventPhase 反映当前阶段：capture 祖先→
    // CAPTURING_PHASE(1)、target（'all'）→ AT_TARGET(2)、bubble 祖先→ BUBBLING_PHASE(3)。target 阶段的 capture
    // 与 non-capture listener 都为 AT_TARGET（WPT Event-dispatch-order-at-target）。dispatch 后由调用方
    // （_dispatchWithBubble finally 或 _dispatchToListeners 末尾）复位为 NONE(0)。
    event.eventPhase = phase === 'capture' ? 1 : (phase === 'bubble' ? 3 : 2);
    var snap = list.slice();
    var firedOnce = null;
    // js-dom M4 R27：spec `EventListener` invoke——listener 是**函数**直接 call；是**对象**则每次派发
    // Get 其 `handleEvent` 属性再调用（this=对象本身，支持 getter：WPT "performs Get every time event
    // is dispatched"）。旧实现恒 `entry.fn.call(...)`，对象 listener 抛（对象无 call）→ WPT
    // EventListener-handleEvent fail。
    var fire = function(entry) {
      var fn = entry.fn;
      var callable = fn;
      if (typeof fn !== 'function') {
        // 对象 listener：Get handleEvent（每次派发都 Get，spec invoke 步骤）。非对象/null handleEvent → 跳过
        //（spec：callable 为 undefined/null 则不抛不调）。
        callable = fn && fn.handleEvent;
      }
      if (typeof callable === 'function') {
        // 函数 listener: this=currentTarget；对象 listener: this=对象本身（spec EventListener invoke）。
        callable.call(typeof fn !== 'function' ? fn : ctx, event);
      }
      if (entry.once) {
        if (!firedOnce) firedOnce = [];
        firedOnce.push(entry);
      }
    };
    // js-dom M4 R34：stop propagation flag 检查。polyfill Event（part05 `_makeEvent`）的 stopPropagation
    // 设 `_propagationStopped`；ZW_NATIVE_DOM=1 叠加路径下 `new MouseEvent` 走 native Event 构造器，其
    // stopPropagation（dom_bindings event_target.rs `native_stop_propagation_invoke`）设 `__zw_stop`——但 dispatch
    // 仍走 polyfill `_dispatchWithBubble`（用例侧 document=shim，未解问题 #9）。故须同时认两个 flag 才能在两路径
    // 下一致止上溯/止同节点后续 listener。immediate flag 同理（`_immediateStopped` polyfill / `__zw_stop_immediate` native）。
    var stopped = function() { return event._propagationStopped || event.__zw_stop === true; };
    var immediateStopped = function() { return event._immediateStopped || event.__zw_stop_immediate === true; };
    if (phase !== 'bubble') {
      for (var i = 0; i < snap.length; i++) {
        if (snap[i].capture) {
          fire(snap[i]);
          if (immediateStopped()) break;
        }
      }
    }
    if (phase !== 'capture' && !immediateStopped()) {
      // js-dom M4 R34：spec `concept-event-dispatch`——AT_TARGET（phase 'all'）时 capture-listener 先于
      // non-capture-listener 触发；若 capture listener 调 stopPropagation（设 _propagationStopped / __zw_stop），
      // 同节点的 non-capture listener **不再触发**（WPT Event-stopPropagation-cancel-bubbling：capture 内
      // stopPropagation 止同元素 bubble handler）。'bubble' phase（祖先冒泡）正常 stopped()=false 不受影响。
      // stopImmediatePropagation 止当前节点剩余 + 后续节点（更强，已在每 listener 后检查）。
      if (stopped()) {
        if (firedOnce) {
          listeners[event.type] = (listeners[event.type] || []).filter(function(e) { return firedOnce.indexOf(e) < 0; });
        }
        return !event._defaultPrevented;
      }
      for (var j = 0; j < snap.length; j++) {
        if (!snap[j].capture) {
          fire(snap[j]);
          if (immediateStopped()) break;
        }
      }
    }
    if (firedOnce) {
      // R40：once 移除须从原始 store 数组过滤（slot 过滤下 list 可能是子集副本，直接写回会丢
      // 其他槽位的注册）。firedOnce 引用原始 entry 对象，indexOf 匹配安全。
      listeners[event.type] = (listeners[event.type] || []).filter(function(e) { return firedOnce.indexOf(e) < 0; });
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
  function _dispatchWithBubble(targetKey, targetSel, targetHandle, event, targetSlot) {
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

    // js-dom M4 R40：document/window 入派发链（spec 结构 html → document → window）。
    // 元素祖先链止于 html（host `__zw_parent('html')` 返空）；此处追加两个**虚派发站**：
    //   - 仅当 target 连入文档（__zw_contains('html', targetSel)，与 composedPath 同判定）时追加；
    //   - capture 反序：window → document → (html 元素链反序)；
    //   - bubble 正序：(html 元素链正序) → document → window。
    // 虚站经 `_dispatchToListeners(htmlKey, …, slot)` 槽位过滤：document 站只触发 document.addEventListener
    // 注册（entry.tgt==='doc'），window 站只触发 window/on* 注册（entry.tgt==='win'）；html 元素站只触发
    // 无标记注册（html 元素 addEventListener 不打标）。listener 在其注册的节点身份上触发（currentTarget
    // = document/window 本体），三合一 key 存储不变（postMessage/onerror/inline-handler 依赖）。
    // handle-only（detached createElement 容器）targetSel 为 null → 不追加（detached 不经 document/window）。
    // targetSlot（R40）：document.dispatchEvent / window.dispatchEvent 以 document/window 为 target——
    // target 阶段用对应槽位（'doc'/'win'），且不再向 document/window 虚站冒泡（target 已是它们；spec：
    // document.dispatchEvent 的 path = [document, window]，window 是 document 的唯一祖先）。
    // https://dom.spec.whatwg.org/#concept-event-dispatch（event path 结构）
    var htmlKey = _elKey('html', null);
    var isDocTarget = targetSlot === 'doc';
    var isWinTarget = targetSlot === 'win';
    var inDoc = false;
    if (targetSel && typeof __zw_contains === 'function') {
      try { inDoc = __zw_contains('html', targetSel) === '1'; } catch (_e) {}
    }
    // document 为 target：path = [document, window]（window 是 document 祖先）；元素 target：连入文档才追加。
    var docObj = globalThis.document
      ? (isDocTarget ? globalThis.document : (inDoc ? globalThis.document : null))
      : null;
    var winObj = globalThis.window ? globalThis.window : null;
    // capture/bubble 是否经过 document/window 虚站：元素 target 连入文档 → 经过（capture 反序 win→doc，
    // bubble 正序 doc→win）；document target → path = [document, window]，doc 已是 target 不再入虚站
    //（capture 无更早节点，bubble 仅 window）；window target → path = [window]，无虚站。detached 元素
    //（handle-only / 不在 html 子树）不经虚站（spec：path 止于其 root）。
    var passDoc = !isWinTarget && !isDocTarget && inDoc;
    var passWin = !isWinTarget && (isDocTarget || inDoc);
    // target 是 document/window 时，元素链（targetSel='html' 的祖先链）不应派发——document.dispatchEvent
    // 的 path 不含 html 元素站。用空元素链实现。
    var elemChain = (isDocTarget || isWinTarget) ? [] : chain;

    // composedPath（R3244，DOM §4.3）：dispatch 期事件路径 = [target, ...祖先链, (document, window 若连入文档)]。
    // 祖先链经 _wrapSelector 转 proxy；连入文档（target 在 html 子树，__zw_contains 判）→ 追加 document + window
    //（spec 路径末端；detached 元素链止于其 root，不追加）。dispatch 结束 finally 清空（spec：非 dispatch 返 []）。
    // R40：document/window 为 target 时 path[0] 是 document/window 本体（非 html proxy——targetSlot 场景
    // _makeProxy('html') 只是占位，真正 target 在 target 阶段覆盖，composedPath 同步用本体）。
    var cpTarget = isDocTarget ? docObj : (isWinTarget ? winObj : target);
    if (!cpTarget) cpTarget = target;
    var cpPath = [cpTarget];
    for (var cpi = 0; cpi < elemChain.length; cpi++) cpPath.push(_wrapSelector(elemChain[cpi]));
    // R40：composedPath 与派发虚站一致——passDoc/passWin 控制 document/window 追加（document target 的
    // path = [document, window]；window target = [window]；元素连入文档 = [..., document, window]）。
    if (passDoc && docObj) cpPath.push(docObj);
    if (passWin && winObj) cpPath.push(winObj);
    event._composedPath = cpPath;

    // js-dom M4 R33：`Window.event`（HTML `current event`）——dispatch 前 save 外层 event、set 当前 event。
    // 嵌套 dispatch（redispatch）时内层 finally 恢复外层（spec innermost-first，外层结束后其 event 仍可见）。
    // finally 统一 restore，与 _composedPath/_propagationStopped 清理同处。prevEvent 用局部变量保 dispatch 栈。
    var prevEvent = globalThis.event;
    globalThis.event = event;
    // js-dom M4 R34：stop propagation flag 兼容——polyfill Event 设 `_propagationStopped`，native Event
    // （ZW_NATIVE_DOM=1 叠加，dispatch 仍走此 polyfill 路径，未解问题 #9）设 `__zw_stop`。两 flag 都须认。
    var bubbleStopped = function() { return event._propagationStopped || event.__zw_stop === true; };
    try {
      // js-dom M4 R39：spec `concept-event-dispatch` 步骤 2——dispatch 开始时若 stop propagation flag
      // **已设**（dispatch 前外部调 stopPropagation()/stopImmediatePropagation()/设 cancelBubble=true，
      // R29 setter 等同 stopPropagation），跳过全部 listener 触发（capture/target/bubble 三阶段全不进），
      // 仅保留 target/composedPath/Window.event 等 dispatch 期赋值（finally 正常清理）。
      // WPT Event-dispatch-propagation-stopped（dispatch 前 stopPropagation → 期望零触发）+
      // Event-dispatch-bubble-canceled（dispatch 前 cancelBubble=true → 同零触发）。
      // 旧实现各阶段循环先派发后才查 flag → html capture 先触发 2 次才止（wrong）。
      // https://dom.spec.whatwg.org/#concept-event-dispatch
      if (bubbleStopped()) return !event._defaultPrevented;

      // ① capture 阶段：root→target 方向（window → document → chain 反序），祖先派发 capture-only。
      if (!globalThis.__zw_no_capture) {
        if (passWin && winObj) {
          _dispatchToListeners(htmlKey, event, 'capture', winObj, 'win');
          if (bubbleStopped()) return !event._defaultPrevented;
        }
        if (passDoc && docObj) {
          _dispatchToListeners(htmlKey, event, 'capture', docObj, 'doc');
          if (bubbleStopped()) return !event._defaultPrevented;
        }
        if (elemChain.length > 0) {
          for (var i = elemChain.length - 1; i >= 0; i--) {
            var capKey = _elKey(elemChain[i], null);
            _ensureInlineHandler(capKey, elemChain[i], null, event.type); // R2935 祖先 inline on* handler 触发
            var capAnc = _wrapSelector(elemChain[i]);
            _dispatchToListeners(capKey, event, 'capture', capAnc, elemChain[i] === 'html' ? null : undefined);
            if (bubbleStopped()) return !event._defaultPrevented;
          }
        }
      }

      // ② target 阶段：capture + 非 capture（AT_TARGET，保旧行为）。R40：document/window 为 target 时
      // 用对应槽位（只触发 document/window 注册）；currentTarget/target 用 document/window 本体（非
      // html proxy——event.target 已在函数开头经 _makeProxy('html') 设为 html proxy，此处覆盖）。
      if (isDocTarget || isWinTarget) {
        var tgtObj = isDocTarget ? globalThis.document : globalThis;
        event.target = tgtObj;
        target = tgtObj;
      }
      event.currentTarget = target;
      if (!isDocTarget && !isWinTarget) {
        _ensureInlineHandler(targetKey, targetSel, targetHandle, event.type); // R2934 inline on* handler 触发
      }
      // R40：html 元素为 target（host lifecycle `__zw_dispatch_event('html', …)`）时 target 站也只触发
      // html 元素槽位（null slot）——doc/win 注册留给后续 doc/win 虚站（否则 target 站全触发 + 虚站再
      // 触发 = 双 fire，renderer R2941/R2943 回归）。其他元素 target 无共存槽位问题（slot undefined 全触发，
      // 旧行为）。
      var tgtSlotFilter = targetSlot !== undefined ? targetSlot : (targetSel === 'html' ? null : undefined);
      _dispatchToListeners(targetKey, event, 'all', target, tgtSlotFilter);
      if (bubbleStopped()) return !event._defaultPrevented;

      // ③ bubble 阶段：target→root 方向（chain 正序 → document → window），祖先派发非 capture（仅 event.bubbles）。
      if (event.bubbles && !globalThis.__zw_no_bubble) {
        if (elemChain.length > 0) {
          for (var k = 0; k < elemChain.length; k++) {
            var bKey = _elKey(elemChain[k], null);
            _ensureInlineHandler(bKey, elemChain[k], null, event.type); // R2935 祖先 inline on* handler 冒泡触发
            var bAnc = _wrapSelector(elemChain[k]);
            _dispatchToListeners(bKey, event, 'bubble', bAnc, elemChain[k] === 'html' ? null : undefined);
            if (bubbleStopped()) break;
          }
        }
        // R40：document / window 虚站冒泡。元素链 html 站被 stopPropagation 止住时（上面 break），
        // spec 语义后续节点（document/window）也不再触发——用同一 bubbleStopped 检查统一处理。
        // document 为 target 时链空但 window 仍冒泡（path = [document, window]）；window 为 target 无更高站。
        if (!bubbleStopped()) {
          if (passDoc && docObj) {
            _dispatchToListeners(htmlKey, event, 'bubble', docObj, 'doc');
          }
          if (!bubbleStopped() && passWin && winObj) {
            _dispatchToListeners(htmlKey, event, 'bubble', winObj, 'win');
          }
        }
      }
      return !event._defaultPrevented;
    } finally {
      event._composedPath = null;
      // js-dom M4 R35：spec `concept-event-dispatch` 末尾——dispatch 结束 eventPhase→NONE(0)、currentTarget→null
      //（WPT Event-dispatch-order-at-target 等读 dispatch 后 eventPhase；event-global "currentTarget null after dispatch"）。
      event.eventPhase = 0;
      event.currentTarget = null;
      // js-dom M4 R33：dispatch 结束 restore 外层 event（嵌套 dispatch 正确）；顶层 dispatch 后回 undefined
      //（WPT event-global "undefined after dispatch"）。须先于 _propagationStopped 重置，保证 restore 与 set 配对。
      globalThis.event = prevEvent;
      // js-dom M4 R29：spec `concept-event-dispatch` 步骤14——dispatch 结束 unset stop propagation flag
      //（+ 步骤清其他 dispatch flags）。reset 后 cancelBubble getter（后端 _propagationStopped）返 false
      //（WPT Event-cancelBubble "cancelBubble must be false after an event has been dispatched"）。
      // 仅清 dispatch 内设的 flag；监听器外显式 stopPropagation（未 dispatch）的 flag 保留至 initEvent 重置。
      event._propagationStopped = false;
      // js-dom M4 R34：同步清 native Event 的 stop flag（叠加路径下 `new MouseEvent` 是 native 对象，dispatch
      // 走此 polyfill 但 native dispatch_event_impl 未跑故不自清；同 event 重派发需 fresh，与 _propagationStopped 同语义）。
      if (event.__zw_stop === true) event.__zw_stop = false;
      if (event.__zw_stop_immediate === true) event.__zw_stop_immediate = false;
    }
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
      // srcElement（js-dom M4 R32，spec `dom-event-srcelement`）：Event.target 的 legacy IE 别名（IDL
      // [LegacyLenientThis] getter 返 target）。初始 null（dispatch 前 target 未设），dispatch 期返 target。
      // 与 defaultPrevented/cancelBubble 同款「公开镜像」——此处占位 null，真正的 getter 经下方
      // defineProperty 定义（读 this.target，dispatch 时 target 更新即反映）。
      srcElement: null,
      timeStamp: typeof __zw_performance_now === 'function'
        ? Number(__zw_performance_now())
        : (typeof Date.now === 'function' ? Date.now() : 0),
      detail: options.detail, // CustomEvent 用；Event 读得 undefined（spec 一致）
      defaultPrevented: false, // 公开镜像（dispatch 读 _defaultPrevented，勿删私字段）
      _defaultPrevented: false,
      _propagationStopped: false,
      _immediateStopped: false,
      // cancelBubble（js-dom M4 R26/R29，spec `dom-event-cancelbubble`）：stop propagation flag 的公开别名
      //（legacy IE）。R26 为普通 data 属性（值镜像，stopPropagation 同步设 true），但 setter 无副作用——
      // 外部 `ev.cancelBubble = true` 不止上溯。R29 改 defineProperty getter/setter 直接以 _propagationStopped
      //（= stop propagation flag）为后端：getter 返 flag；setter 设 true→置 flag（等同 stopPropagation，dispatch
      // bubble/capture 循环读 flag 止上溯），设 false→no-op（spec：flag 一旦设除非 initEvent 否则不可清）。
      // _dispatchWithBubble finally 重置 flag（spec concept-event-dispatch 步骤14，dispatch 后 cancelBubble=false）。
      // 与 R28 returnValue 同款「defineProperty getter/setter + 私 flag 后端」模式。
      // composedPath（R3244）：DOM §4.3——dispatch 期间返事件路径（target→祖先→document→window），
      // 非 dispatch（前后）返 []。`_composedPath` 由 _dispatchWithBubble / globalThis.dispatchEvent 在派发期
      // 填充、finally 清空（spec：dispatch flag unset 时返空）。事件委托（e.composedPath()[0] === target）
      // + 祖先匹配（path.includes(ancestor)）高频。
      _composedPath: null,
      composedPath: function() {
        return this._composedPath ? this._composedPath.slice() : [];
      },
      preventDefault: function() { if (this.cancelable) { this.defaultPrevented = true; this._defaultPrevented = true; } },
      stopPropagation: function() { this._propagationStopped = true; },
      stopImmediatePropagation: function() {
        this._immediateStopped = true;
        this._propagationStopped = true;
      }
    };
    // js-dom M4 R28：`Event.returnValue`（spec `dom-event-returnvalue`，legacy IE 别名 = !canceled flag）。
    // getter 返 `!_defaultPrevented`（canceled flag 的反向镜像）；setter：设 false 仅当 cancelable 时触发
    // preventDefault（设 canceled），cancelable=false 或已 canceled 后设 true 均 no-op。WPT Event-returnValue：
    // 初始 true / preventDefault(cancelable)→false / returnValue=false(cancelable)→prevent / initEvent 重置 /
    // returnValue=true 已 canceled 后 no-op。用 defineProperty（getter/setter，非普通 data 属性——setter 需
    // 触发 prevent 副作用）。
    Object.defineProperty(ev, 'returnValue', {
      enumerable: false,
      configurable: true,
      get: function() { return !this._defaultPrevented; },
      set: function(v) {
        // 仅 cancelable 且设 false 时触发 preventDefault（设 canceled flag）。设 true 永远 no-op（spec：canceled
        // flag 一旦设不可清）。cancelable=false 时任何设值 no-op（WPT "no effect if cancelable is false"）。
        if (!v && this.cancelable) { this.defaultPrevented = true; this._defaultPrevented = true; }
      }
    });
    // js-dom M4 R29：`Event.cancelBubble` setter dispatch 副作用（spec `dom-event-cancelbubble`）。R26 用普通 data
    // 属性镜像 stopPropagation 设值，但 `ev.cancelBubble = true` 无副作用不止上溯。R29 改 defineProperty，后端直
    // 接复用 stop propagation flag `_propagationStopped`：getter 返 flag；setter 设 true→置 flag（等同 stopPropagation，
    // _dispatchWithBubble capture/target/bubble 三循环均读此 flag 止上溯），设 false→no-op（spec：stop propagation
    // flag 一旦设除非 initEvent 重新初始化否则不可清——WPT "cancelBubble=false must have no effect"）。WPT 覆盖：
    // 初始 false / stopPropagation→true / cancelBubble=false no-op / dispatch bubble 循环止上溯 / dispatch 后 flag
    // 清（_dispatchWithBubble finally 重置 _propagationStopped）→ cancelBubble=false。
    Object.defineProperty(ev, 'cancelBubble', {
      enumerable: false,
      configurable: true,
      get: function() { return this._propagationStopped; },
      set: function(v) {
        // 设 true → 置 stop propagation flag（spec cancelBubble setter：true 时「set this's stop propagation flag」，
        // 等同 stopPropagation）。设 false → no-op（flag 不可被 setter 清，只能 initEvent 重置）。
        if (v) { this._propagationStopped = true; }
      }
    });
    // js-dom M4 R32：`Event.srcElement`（spec `dom-event-srcelement`，legacy IE 别名 = target）。IDL getter
    // [LegacyLenientThis] 返 target——dispatch 前 target=null 故 srcElement=null；dispatch 期 target 已设故
    // srcElement=target。用 defineProperty getter 读 this.target（dispatch 时 target 更新即反映，与普通 data 属性
    // 占位 null 不同——占位会在 dispatch 后读 null 而非 target）。WPT event-src-element-nullable：new Event 读
    // null + dispatch 后 listener 读非 null。setter 不定义（spec 只读，赋值静默丢弃，JS 默认行为）。
    Object.defineProperty(ev, 'srcElement', {
      enumerable: false,
      configurable: true,
      get: function() { return this.target; }
    });
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

  // js-dom M4 createElementNS（spec `dom-document-createelementns`，R18）：大小写敏感的 qualifiedName 解析
  // 辅助。createElementNS spec **不**小写 localName，且保留原 prefix——故 `_realTag` 的强制大写不适用。
  // 这三个 helper 从原 qualifiedName 取 `prefix:local` 各段，原样返（不经 toUpperCase/toLowerCase）。
  // - `_nsLocal("svg:rect")` → "rect"；`_nsLocal("rect")` → "rect"（大小写敏感）
  // - `_nsPrefix("svg:rect")` → "svg"；`_nsPrefix("rect")` → null（无 prefix）
  // - `_nsQualified` 直接返原值（tagName/nodeName 对 createElementNS = 大小写敏感 qualifiedName）
  function _nsLocal(q) {
    var c = q.indexOf(':');
    return c >= 0 ? q.slice(c + 1) : q;
  }
  function _nsPrefix(q) {
    var c = q.indexOf(':');
    return c >= 0 ? q.slice(0, c) : null;
  }
  function _nsQualified(q) {
    return q;
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
  // R3317 HTML 日期/时间 value 串解析 → UTC Date 对象（valueAsDate getter 用）。
  // 按 HTML §4.10.5.18：date(YYYY-MM-DD)→当日 00:00:00 UTC；time(HH:MM[:SS[.fff]][,秒小数] 可带 'Z'/±HH:MM)
  // →1970-01-01 当日 UTC；month(YYYY-MM)→当月 1 日 00:00:00 UTC；week(YYYY-Www)→该年 ISO 周一 00:00:00 UTC。
  // 无效/不匹配→null（spec：valueAsDate 对无效 value 返 null）。所有分量 Date.UTC（UTC，规避本地时区）。
  function _parseHtmlDateValue(v, type) {
    v = String(v).trim();
    if (type === 'date') {
      // spec 严格 YYYY-MM-DD（月/日补零，范围校验：月 1-12、日 1-31）。
      var m = v.match(/^(\d{4,})-(\d{2})-(\d{2})$/);
      if (!m) return null;
      var y = +m[1], mo = +m[2], d = +m[3];
      if (mo < 1 || mo > 12 || d < 1 || d > 31) return null;
      return new Date(Date.UTC(y, mo - 1, d));
    }
    if (type === 'month') {
      var mm = v.match(/^(\d{4,})-(\d{2})$/);
      if (!mm) return null;
      var my = +mm[1], mmo = +mm[2];
      if (mmo < 1 || mmo > 12) return null;
      return new Date(Date.UTC(my, mmo - 1, 1));
    }
    if (type === 'time') {
      // HH:MM[:SS[.fff]]，可选时区后缀（'Z' 或 ±HH:MM）。解析为 1970-01-01 当日（UTC 基准）。
      var tm = v.match(/^(\d{2}):(\d{2})(?::(\d{2})(?:\.(\d{1,3}))?)?(Z|[+-]\d{2}:\d{2})?$/);
      if (!tm) return null;
      var H = +tm[1], M = +tm[2], S = tm[3] ? +tm[3] : 0, ms = tm[4] ? +tm[4].padEnd(3, '0') : 0;
      if (H > 23 || M > 59 || S > 59) return null;
      var base = Date.UTC(1970, 0, 1, H, M, S, ms);
      if (tm[5] && tm[5] !== 'Z') {
        var sm = tm[5].match(/^([+-])(\d{2}):(\d{2})$/);
        if (sm) {
          var off = (+sm[2]) * 3600000 + (+sm[3]) * 60000;
          base += (sm[1] === '-') ? off : -off; // 本地时间→UTC：东区减、西区加
        }
      }
      return new Date(base);
    }
    if (type === 'week') {
      // YYYY-Www：该年第 ww 个 ISO 周（周一为首日）的周一 00:00:00 UTC。ISO 8601 周日期。
      var wm = v.match(/^(\d{4,})-W(\d{2})$/);
      if (!wm) return null;
      var wy = +wm[1], w = +wm[2];
      if (w < 1 || w > 53) return null;
      // ISO 周：该年 1 月 4 日必在第 1 周内；第 1 周一 = 1 月 4 日 - ((day-1) 天)，day: 周日=0..周六=6。
      var jan4 = Date.UTC(wy, 0, 4);
      var jan4Day = new Date(jan4).getUTCDay(); // 0=Sun..6=Sat
      var week1Mon = jan4 - (jan4Day === 0 ? 6 : jan4Day - 1) * 86400000;
      return new Date(week1Mon + (w - 1) * 7 * 86400000);
    }
    return null;
  }
  // R3317 Date → HTML 日期/时间 value 串（valueAsDate setter 用）。无效/非 Date→''（setter 后续判 '' 视为清空）。
  function _formatHtmlDateValue(d, type) {
    if (!(d instanceof Date) || isNaN(d.getTime())) return '';
    var Y = d.getUTCFullYear(), Mo = d.getUTCMonth() + 1, D = d.getUTCDate();
    var H = d.getUTCHours(), Mi = d.getUTCMinutes(), S = d.getUTCSeconds();
    var p2 = function (n) { return (n < 10 ? '0' : '') + n; };
    var p4 = function (n) { return (n < 0 ? '-' : '') + (Math.abs(n) < 1000 ? String(Math.abs(n)).padStart(4, '0') : String(Math.abs(n))); };
    if (type === 'date') return p4(Y) + '-' + p2(Mo) + '-' + p2(D);
    if (type === 'month') return p4(Y) + '-' + p2(Mo);
    if (type === 'time') {
      // time 仅取时分秒（1970-01-01 当日），忽略毫秒若为 0
      var t = p2(H) + ':' + p2(Mi);
      if (S > 0 || d.getUTCMilliseconds() > 0) t += ':' + p2(S);
      return t;
    }
    return '';
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
  // R34xx：本地已移除元素标记（sel → true）——同步脚本内 `el.remove()` 后 parentNode
  // 须立即返 null（host mutation 异步应用，快照仍含该元素；2d.shadow.attributes.
  // shadowColor.current.removed：remove 后 currentColor 解析为黑）。
  var _zwRemovedSels = {};
  function _zwMarkRemoved(sel) { if (sel) _zwRemovedSels[sel] = true; }
  function _zwUnmarkRemoved(sel) { if (sel) delete _zwRemovedSels[sel]; }
  function _zwIsRemoved(sel) { return !!(sel && _zwRemovedSels[sel]); }

  function _parentNodeFor(sel, handle) {
    // R34xx：本地移除标记优先——remove() 后（mutation 未应用）parentNode 返 null。
    if (_zwIsRemoved(sel)) return null;
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

  // R3243：表格修改 API（insertRow/deleteRow/insertCell/deleteCell）内部用的行/单元格集合读取助函数。
  // 经 proxy.rows/.cells 读取（table+section 有 .rows，tr 有 .cells），失败/空/非数组 → []。
  // 与读侧 getter（part03 .rows/.cells）一致：detached handle-only 元素无 sel → []（无 DOM 可查）。
  function _rowList(elProxy) {
    try { var r = elProxy.rows; return (r && r.length) ? r : []; } catch (_e) { return []; }
  }
  function _cellList(elProxy) {
    try { var c = elProxy.cells; return (c && c.length) ? c : []; } catch (_e) { return []; }
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
      // R34xx：id 含特殊字符（点号等——canvas WPT 的 id="green.png"）时 '#'+id 选择器
      // 解析错误（点号被当类）→ 改用属性选择器（[id="..."] 精确匹配）。
      getElementById: function (id) { return queryOne('[id="' + String(id).replace(/"/g, '\\"') + '"]'); },
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
      // R34xx：id 含特殊字符（点号等——canvas WPT 的 id="green.png"）时 '#'+id 选择器
      // 解析错误（点号被当类）→ 改用属性选择器（[id="..."] 精确匹配）。
      getElementById: function (id) { return queryOne('[id="' + String(id).replace(/"/g, '\\"') + '"]'); },
      getElementsByTagName: function (tag) { return queryAll(String(tag)); },
      getElementsByClassName: function (cls) { return queryAll('.' + String(cls)); },
      // R3018：createElement/createTextNode 返完整可变节点（_zwMEl/_zwMText），非 hollow stub。
      // DOMPurify / 模板引擎经 createElement 建替换节点后 insertBefore/appendChild 入树，须支持 parentNode/
      // sibling/childNodes/setAttribute/序列化全套语义。HTML 文档 tagName 大写、localName 小写。
      createElement: function (t) { return _zwMEl({ tag: String(t).toLowerCase() }, null); },
      createTextNode: function (t) { return _zwMText(String(t), null); },
      // R15：detached doc 的 implementation（用例 doTest(doc,...) 经 doc.implementation.createDocumentType）。
      // ownerDocument 指向此 detached doc（spec：doctype.ownerDocument === 创建它的 document）。
      implementation: {
        hasFeature: function () { return true; },
        createDocumentType: function (qualifiedName, publicId, systemId) {
          return {
            nodeType: 10,
            name: String(qualifiedName == null ? '' : qualifiedName),
            nodeName: String(qualifiedName == null ? '' : qualifiedName),
            publicId: String(publicId == null ? '' : publicId),
            systemId: String(systemId == null ? '' : systemId),
            ownerDocument: doc,
            nodeValue: null,
            textContent: null,
          };
        },
      },
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
  // R3198：handle 经 `__zw_attr_names_handle`（属性名仅来自 mutations，无快照基底）——旧 handle 元素 NamedNodeMap
  // 恒空（length 0 / item·getNamedItem 返 null / iterator 空）。setNamedItem/removeNamedItem 真 mutation（R3022，
  // 委托元素 setAttribute/removeAttribute host 路径，返旧/移除 Attr），非只读 no-op。
  function _attributesProxy(sel, handle) {
    var readNames = function() {
      // R3198：handle 经 `__zw_attr_names_handle`，sel 经 `__zw_attr_names`（latest-wins）。各方法
      //（length/item/getNamedItem/iterator）均经此，故 handle NamedNodeMap 旧全空。
      try {
        var n = handle
          ? (typeof __zw_attr_names_handle === 'function' ? __zw_attr_names_handle(handle) : '')
          : (typeof __zw_attr_names === 'function' ? __zw_attr_names(sel) : '');
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
        // js-dom M4 R44：named getter（spec supported property names）——`attrs.id` 返对应
        // Attr 节点（与 ownKeys/getOwnPropertyDescriptor 枚举一致；WPT namednodemap +
        // attrs.id.value 访问模式）。
        if (typeof p === 'string' && names.indexOf(p) >= 0) {
          return attrObj(p);
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
      },
      // js-dom M4 R44：spec NamedNodeMap supported property names（`dom-namednodemap-supported-property-names`）
      // ——own keys = 数值索引（"0","1",…）+ 属性名（id/class/…）。WPT namednodemap-supported-property-names
      // 断言 `Object.getOwnPropertyNames(el.attributes)` === [indices..., names...]。旧实现 Proxy 落
      // target（{}）→ 恒 []。ownKeys 须与 getOwnPropertyDescriptor 一致（invariant：ownKeys 列出的
      // 键须存在可描述）——descriptor 返 {enumerable, configurable: true} 数据属性近似。
      ownKeys: function() {
        var names = readNames();
        var keys = [];
        for (var i = 0; i < names.length; i++) keys.push(String(i));
        for (var j = 0; j < names.length; j++) keys.push(names[j]);
        return keys;
      },
      getOwnPropertyDescriptor: function(_t, p) {
        var names = readNames();
        if (p === 'length') {
          return { value: names.length, writable: false, enumerable: false, configurable: true };
        }
        var idx = parseInt(p, 10);
        if (!isNaN(idx) && String(idx) === String(p) && idx >= 0 && idx < names.length) {
          return { value: attrObj(names[idx]), writable: false, enumerable: true, configurable: true };
        }
        if (names.indexOf(String(p)) >= 0) {
          return { value: attrObj(String(p)), writable: false, enumerable: true, configurable: true };
        }
        return undefined;
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
  // R3193：拆分 style 声明值为 {value, priority}——spec `dom-cssstyledeclaration-getpropertyvalue` 返值
  // **不含** !important，`getPropertyPriority` 返 "important"/""。`!important` 在末尾（CSS 语法允许 `!` 与
  // important 间空白）。供 readProp/getPropertyPriority 复用。
  function _styleValueAndPriority(decl) {
    var s = String(decl).trim();
    var stripped = s.replace(/\s*!\s*important\s*$/i, '');
    if (stripped !== s) return { value: stripped.trim(), priority: 'important' };
    return { value: s, priority: '' };
  }

  function _styleProxy(sel, handle) {
    var readRaw = function() {
      // R3194：sel 走 latest-wins（`__zw_get_style_lw` replay pending style mutation），闭合 sync set→read
      // stale（R3193 已知限制①）。R3199：handle 走 `__zw_get_style_lw_handle`（正序 replay SetStyleOnHandle/
      // RemoveStyleOnHandle/SetAttrOnHandle(style)/RemoveAttrOnHandle(style)，无快照基底），闭合 R3194 已知
      // 限制①（handle style sync set→read stale）。无 lw 回调 → fallback 纯快照 `__zw_get_attr_handle`。
      if (handle) {
        if (typeof __zw_get_style_lw_handle === 'function') return __zw_get_style_lw_handle(handle) || '';
        return __zw_get_attr_handle(handle, 'style') || '';
      }
      if (typeof __zw_get_style_lw === 'function') return __zw_get_style_lw(sel) || '';
      return __zw_get_attr(sel, 'style') || '';
    };
    // R3193：读取属性声明的**原始值串**（含 !important），按首个 ':' 切分（旧 `split(':')` 致 url() 等含
    // 冒号值截断）。命中返原始值串，未命中返 null。供 readProp（值）/ getPropertyPriority（优先级）复用。
    // R3213：取**末次非空**匹配（spec getPropertyValue/getPropertyPriority 返 LAST 声明——CSSOM「get a
    // CSS declaration」末值胜，与 native parse_style dedup 末值胜对称）。跳过空值匹配（与 R3212 parse 丢空值
    // 声明对称——`color:red;color:` 应返 "red" 非 ""）。非 duplicate 场景仅一匹配，首=末，行为不变。
    var readDeclValue = function(name) {
      var raw = readRaw();
      if (!raw) return null;
      var want = _stylePropName(name).toLowerCase();
      var parts = raw.split(';');
      var found = null;
      for (var i = 0; i < parts.length; i++) {
        var decl = parts[i];
        var colon = decl.indexOf(':');
        if (colon < 0) continue;
        if (decl.slice(0, colon).trim().toLowerCase() === want) {
          var val = decl.slice(colon + 1).trim();
          if (val) found = val; // 末次非空匹配覆盖
        }
      }
      return found;
    };
    // getPropertyValue 返值**不含** !important（spec；旧返 "red !important"）。
    var readProp = function(name) {
      var v = readDeclValue(name);
      return v == null ? '' : _styleValueAndPriority(v).value;
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
        .map(function(s) { var c = s.indexOf(':'); return (c < 0 ? s : s.slice(0, c)).trim(); })
        .filter(Boolean);
    };
    return new Proxy({}, {
      get: function(_t, p) {
        var ps = String(p);
        if (ps === 'cssText') return readRaw();
        if (ps === 'length') return propNames().length;
        if (ps === 'getPropertyValue') return function(name) { return readProp(name); };
        // R3193：getPropertyPriority 返 "important"（声明带 !important）/ ""（spec；旧 stub 恒 ""）。
        if (ps === 'getPropertyPriority') {
          return function(name) {
            var v = readDeclValue(name);
            return v == null ? '' : _styleValueAndPriority(v).priority;
          };
        }
        // R3193：setProperty 第三参 priority——spec "important"（ci）→ 追加 !important；余 → 无优先级。
        // 先剥离 value 末尾既存 !important（priority arg 显式控制优先级，非 value 字符串）。
        if (ps === 'setProperty') {
          return function(name, value, priority) {
            var v = String(value).replace(/\s*!\s*important\s*$/i, '').trim();
            var hasImp = priority !== undefined && String(priority).trim().toLowerCase() === 'important';
            if (hasImp) v = v + ' !important';
            setProp(name, v);
            return undefined;
          };
        }
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
        // `input.valueAsDate`（HTMLInputElement，R3317）——date/month/week/time 输入值↔Date 转换（日期选择器
        // 读 Date 算星期/比较/计算、表单库读 null 判空）。type=date/month/week/time：按 HTML §4.10.5.18 解析
        // value 串为 UTC Date（date=当日 00:00:00 UTC、time=1970-01-01 当日 UTC、month=当月 1 日 UTC、
        // week=该年 ISO 周一 00:00:00 UTC）；空/无效 value→null；非 date/time type→null。仅 INPUT。
        // https://html.spec.whatwg.org/multipage/input.html#dom-input-valueasdate
        if (prop === 'valueAsDate' && _realTag(sel, handle) === 'INPUT') {
          try {
            var vadT = (handle ? __zw_get_attr_handle(handle, 'type') : __zw_get_attr(sel, 'type')) || '';
            vadT = vadT.toLowerCase();
            if (vadT !== 'date' && vadT !== 'month' && vadT !== 'week' && vadT !== 'time') return null;
            var vadV = _inputValues[key];
            if (vadV == null) vadV = (handle ? __zw_get_attr_handle(handle, 'value') : __zw_get_attr(sel, 'value')) || '';
            if (vadV === '') return null;
            return _parseHtmlDateValue(vadV, vadT); // null = 无效 / Date 对象 = 有效
          } catch (_e) { return null; }
        }
        // text-control 选区 getter（R2844）：selectionStart / selectionEnd / selectionDirection。
        // 仅 text control（_isTextControl gate）。默认 {0, 0, 'forward'}（Chromium 150 oracle 锚定）。
        // 文本编辑器 / 自动选择 / Range 算法读选区状态高频。非 text input 按规范返回 null。
        // getter 不污染 _textSelection（纯读）。
        if (prop === 'selectionStart' || prop === 'selectionEnd' || prop === 'selectionDirection') {
          if (_isTextControl(sel, handle)) {
            var gs = _textSelection[key] || { start: 0, end: 0, direction: 'forward' };
            if (prop === 'selectionStart') return gs.start;
            if (prop === 'selectionEnd') return gs.end;
            return gs.direction;
          }
          if (_realTag(sel, handle) === 'INPUT') return null;
        }
        // `el.setSelectionRange(start, end, direction?)`（HTMLInputElement.textarea，R2844）——设选区。
        // Chromium 150 oracle 锚定：start/end clamp [0, len]；end<start → start 折叠到 end（setSR(4,2)→{2,2}）；
        // direction 缺省 'forward'，否则取给定值（'backward'/'none'，其他归 'forward'）。仅 text control。
        if (prop === 'setSelectionRange') {
          if (_isTextControl(sel, handle)) {
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
          if (_realTag(sel, handle) === 'INPUT') {
            return function() { _throwDom('InvalidStateError', 'input type does not support text selection'); };
          }
        }
        // `el.setRangeText(replacement [, start [, end [, selectionMode]]])`（HTMLInputElement/textarea，
        // R3245）——替换 value 中 [start,end) 子串为 replacement 并按 selectionMode 重定选区。
        // https://html.spec.whatwg.org/multipage/input.html#dom-textarea/input-setrangetext（§4.10.5.23）。
        // start/end 缺省取 selectionStart/End；start>end 抛 IndexSizeError；selectionMode ∈ select/start/end/preserve
        //（缺省 preserve）。复用既有原语：`_controlValue` 读、`this.value=` setter 写（textarea 走 text-content、
        // input 走 attr + dirty 跟踪）、`_selObj` 选区。文本编辑库（auto-format / mask / undo 补全）高频。
        if (prop === 'setRangeText') {
          if (!_isTextControl(sel, handle)) {
            if (_realTag(sel, handle) === 'INPUT') {
              return function() { _throwDom('InvalidStateError', 'input type does not support text selection'); };
            }
          } else {
            return function(replacement, start, end, selectionMode) {
            var so = _selObj(key);
            if (arguments.length < 2) start = so.start;
            if (arguments.length < 3) end = so.end;
            start = Number(start); end = Number(end);
            if (isNaN(start)) start = 0;
            if (isNaN(end)) end = 0;
            // spec：start > end 抛 IndexSizeError（边界校验先于 clamp）。
            if (start > end) _throwDom('IndexSizeError', 'setRangeText: start greater than end');
            replacement = String(replacement == null ? '' : replacement);
            if (selectionMode !== 'select' && selectionMode !== 'start' && selectionMode !== 'end') selectionMode = 'preserve';
            var val = _controlValue(sel, handle, key);
            var cs = _clampSelOffset(start, val.length);
            var ce = _clampSelOffset(end, val.length);
            var oldStart = so.start, oldEnd = so.end;
            // 替换 [cs,ce) 子串；经 proxy value setter 持久化（区分 textarea/input + dirty 跟踪）。
            this.value = val.slice(0, cs) + replacement + val.slice(ce);
            // 按 selectionMode 重定选区（spec §4.10.5.23）。delta = 替换后净长度变化。
            var delta = replacement.length - (ce - cs);
            if (selectionMode === 'select') {
              so.start = cs; so.end = cs + replacement.length;
            } else if (selectionMode === 'start') {
              so.start = cs; so.end = cs;
            } else if (selectionMode === 'end') {
              so.start = cs + replacement.length; so.end = cs + replacement.length;
            } else {
              // preserve：选区在编辑区之前不动；之后按 delta 平移；跨编辑区则折叠到 cs（近似，保 selection 合法）。
              so.start = (oldStart <= cs) ? oldStart : (oldStart >= ce ? oldStart + delta : cs);
              so.end = (oldEnd <= cs) ? oldEnd : (oldEnd >= ce ? oldEnd + delta : cs);
            }
              return undefined;
            };
          }
        }
        // `input.stepUp(n)` / `input.stepDown(n)`（HTMLInputElement，R3317）——number/range 按步进增减
        // （数量调节器、范围滑块步进 UI 高频）。n 缺省 1。按 step 属性（缺省 1）×n 改 value，clamp [min,max]
        //（spec：超界抛 InvalidStateError；headless 近似 clamp 不抛——保守合法化）。空/无效 value→按 min（缺 0）起算。
        // 仅 type=number/range 暴露（与 valueAsNumber 同域）；非 number/range 的 INPUT 该方法 typeof=undefined（调用抛
        // TypeError，real-browser 亦不暴露）。复用 value setter 持久化路径。
        // https://html.spec.whatwg.org/multipage/input.html#dom-input-stepup
        if ((prop === 'stepUp' || prop === 'stepDown') && _realTag(sel, handle) === 'INPUT') {
          var suT = (handle ? __zw_get_attr_handle(handle, 'type') : __zw_get_attr(sel, 'type')) || '';
          if (suT.toLowerCase() !== 'number' && suT.toLowerCase() !== 'range') return undefined;
          return function(n) {
            n = (n === undefined) ? 1 : Number(n);
            if (isNaN(n)) n = 1;
            var step = parseFloat(handle ? __zw_get_attr_handle(handle, 'step') : __zw_get_attr(sel, 'step'));
            if (isNaN(step) || step <= 0) step = 1; // 'any'/缺省/无效 → 1（spec 'any' 抛，近似 1）
            var min = parseFloat(handle ? __zw_get_attr_handle(handle, 'min') : __zw_get_attr(sel, 'min'));
            if (isNaN(min)) min = 0;
            var max = parseFloat(handle ? __zw_get_attr_handle(handle, 'max') : __zw_get_attr(sel, 'max'));
            var hasMax = !isNaN(max);
            var cur = parseFloat(_controlValue(sel, handle, key));
            if (isNaN(cur)) cur = min; // 空/无效 → 从 min 起
            var delta = (prop === 'stepUp' ? 1 : -1) * step * n;
            var next = cur + delta;
            if (hasMax && next > max) next = max;
            if (next < min) next = min;
            this.value = String(next);
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
        // 组件 setter（a.pathname='/x'）经 R3070 set-trap 分支接通 `__zw_set_url_part` 重算 href 写回属性
        //（闭合 R2838 旧限制「组件 setter 误设 spurious 属性」）。origin 为只读（无 setter）。
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
        // R3202：sel 路径改走 latest-wins（`__zw_get_attr_lw`）反映同批 `form.method=`/`setAttribute`（旧纯快照
        // `__zw_get_attr` → `f.method='POST'; f.method` 读 stale 快照返 default 'get'，同 R3190/R3195 stale 模式）。
        if (_realTag(sel, handle) === 'FORM' &&
            (prop === 'action' || prop === 'method' || prop === 'enctype' || prop === 'target')) {
          var fv = handle
            ? __zw_get_attr_handle(handle, prop)
            : (typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(sel, prop) : __zw_get_attr(sel, prop));
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
        // `.form`（form-associated 控件 INPUT/SELECT/TEXTAREA/BUTTON/OUTPUT，R2841）——返所属 <form> 元素
        // （form owner）。form 校验 / 序列化库读 input.form 找 owner form 上下文高频。**spec 顺序**：
        // ① `form` 属性关联优先（`<input form="id">` → getElementById(id)，即使无 ancestor form）；
        // ② 否则最近 ancestor <form>（经 `_ancestorChain` 上行）。handle-only detached / 无 owner → null。
        // https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#association-of-controls-and-forms
        if (prop === 'form') {
          var fcTag = _realTag(sel, handle);
          if (fcTag === 'INPUT' || fcTag === 'SELECT' || fcTag === 'TEXTAREA' || fcTag === 'BUTTON' || fcTag === 'OUTPUT') {
            try {
              var hasFormAttr = false;
              if (handle && typeof __zw_has_attr_handle === 'function') {
                hasFormAttr = __zw_has_attr_handle(handle, 'form') === '1';
              } else if (sel && typeof __zw_has_attr_lw === 'function') {
                hasFormAttr = __zw_has_attr_lw(sel, 'form') === '1';
              } else if (sel && typeof __zw_has_attr === 'function') {
                hasFormAttr = __zw_has_attr(sel, 'form') === '1';
              }
              var formAttr = handle ? __zw_get_attr_handle(handle, 'form') : (sel ? __zw_get_attr(sel, 'form') : '');
              if (hasFormAttr) {
                if (formAttr && globalThis.document && globalThis.document.getElementById) {
                  var byId = globalThis.document.getElementById(formAttr);
                  if (byId && String(byId.tagName || '').toUpperCase() === 'FORM') return byId;
                }
                return null;
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
        // `<tr>`.cells（HTMLTableRowElement，R3243）——行内全部 td+th 单元格（document order，混计）。
        // 与 cellIndex（R2842）同源 :is(td, th) 查询，返真数组（length/索引/迭代）。表格修改库
        // （insertCell/deleteCell 定位 + DataTables 等读列）高频。仅 TR。
        if (prop === 'cells' && _realTag(sel, handle) === 'TR') {
          if (!sel) return [];
          try { return _wrapSelector(sel).querySelectorAll(':is(td, th)'); } catch (_e) { return []; }
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
        // ── HTMLTableElement / HTMLTableSectionElement / HTMLTableRowElement 修改 API（R3243）——
        // WHATWG HTML §4.9.1 表格写侧族：insertRow/deleteRow（table + thead/tbody/tfoot section）、
        // insertCell/deleteCell（tr）。读侧 rows/tBodies/caption/tHead/tFoot/rowIndex/cellIndex 已就绪
        // （R2842-R2849），此片补写侧。全部经既有 createElement + appendChild/insertBefore/removeChild
        // 原语（无新 host 回调）。index 语义：省略/-1 = 末尾追加；index < -1 或越界抛 IndexSizeError
        // （Chromium oracle 一致，经 `_throwDom`）；table.insertRow 在无 tr 且无 section 子时自动建 tbody
        // 挂载新行（spec「no tr/tbody/thead/tfoot children」分支）。
        if (prop === 'insertRow') {
          var _irTag = _realTag(sel, handle);
          // https://html.spec.whatwg.org/multipage/tables.html#dom-table-insertrow
          if (_irTag === 'TABLE') {
            return function(index) {
              if (arguments.length === 0) index = -1;
              else { index = Number(index); if (isNaN(index)) index = 0; }
              if (index < -1) _throwDom('IndexSizeError', 'index is negative and not -1');
              var tr = globalThis.document.createElement('tr');
              var rowsArr = _rowList(this);
              var firstTbody = null;
              try { var _tbs = this.tBodies; firstTbody = (_tbs && _tbs.length) ? _tbs[0] : null; } catch (_e) {}
              var hasSection = firstTbody || this.tHead || this.tFoot;
              // 无 tr 且无 section → 建 tbody 挂新行（spec「no tr/tbody/thead/tfoot children」分支）
              if (rowsArr.length === 0 && !hasSection) {
                var tb = globalThis.document.createElement('tbody');
                this.appendChild(tb);
                tb.appendChild(tr);
                return tr;
              }
              if (index === -1 || index === rowsArr.length) {
                if (rowsArr.length === 0) {
                  // section 已存在（非上分支）→ 入首个 tbody（无 tbody 则直挂 table）
                  if (firstTbody) firstTbody.appendChild(tr); else this.appendChild(tr);
                } else {
                  rowsArr[rowsArr.length - 1].parentNode.appendChild(tr);
                }
              } else if (index >= 0 && index < rowsArr.length) {
                var ref = rowsArr[index];
                ref.parentNode.insertBefore(tr, ref);
              } else {
                _throwDom('IndexSizeError', 'index out of range');
              }
              return tr;
            };
          }
          // https://html.spec.whatwg.org/multipage/tables.html#dom-tbody-insertrow
          if (_irTag === 'THEAD' || _irTag === 'TBODY' || _irTag === 'TFOOT') {
            return function(index) {
              if (arguments.length === 0) index = -1;
              else { index = Number(index); if (isNaN(index)) index = 0; }
              if (index < -1) _throwDom('IndexSizeError', 'index is negative and not -1');
              var tr = globalThis.document.createElement('tr');
              var rowsArr = _rowList(this);
              if (index === -1 || index === rowsArr.length) {
                this.appendChild(tr);
              } else if (index >= 0 && index < rowsArr.length) {
                this.insertBefore(tr, rowsArr[index]);
              } else {
                _throwDom('IndexSizeError', 'index out of range');
              }
              return tr;
            };
          }
        }
        // https://html.spec.whatwg.org/multipage/tables.html#dom-table-deleterow（table + section 共用）
        if (prop === 'deleteRow') {
          var _drTag = _realTag(sel, handle);
          if (_drTag === 'TABLE' || _drTag === 'THEAD' || _drTag === 'TBODY' || _drTag === 'TFOOT') {
            return function(index) {
              if (arguments.length === 0) index = -1;
              else { index = Number(index); if (isNaN(index)) index = 0; }
              var rowsArr = _rowList(this);
              if (index < -1 || index >= rowsArr.length) {
                _throwDom('IndexSizeError', 'index out of range');
              }
              // 经 victim.remove()（self 级，sel-based 走 __zw_remove / handle 走 __zw_remove_handle），
              // 非 parentNode.removeChild——后者要求 child.__zwHandle，而 querySelectorAll 返的 sel-based
              // victim 无 handle（no-op）。remove() 对两种身份都正确记录 Remove mutation。
              var victim = (index === -1) ? rowsArr[rowsArr.length - 1] : rowsArr[index];
              if (victim && typeof victim.remove === 'function') victim.remove();
            };
          }
        }
        // https://html.spec.whatwg.org/multipage/tables.html#dom-tr-insertcell
        if (prop === 'insertCell' && _realTag(sel, handle) === 'TR') {
          return function(index) {
            if (arguments.length === 0) index = -1;
            else { index = Number(index); if (isNaN(index)) index = 0; }
            if (index < -1) _throwDom('IndexSizeError', 'index is negative and not -1');
            var td = globalThis.document.createElement('td');
            var cellsArr = _cellList(this);
            if (index === -1 || index === cellsArr.length) {
              this.appendChild(td);
            } else if (index >= 0 && index < cellsArr.length) {
              this.insertBefore(td, cellsArr[index]);
            } else {
              _throwDom('IndexSizeError', 'index out of range');
            }
            return td;
          };
        }
        // https://html.spec.whatwg.org/multipage/tables.html#dom-tr-deletecell
        if (prop === 'deleteCell' && _realTag(sel, handle) === 'TR') {
          return function(index) {
            if (arguments.length === 0) index = -1;
            else { index = Number(index); if (isNaN(index)) index = 0; }
            var cellsArr = _cellList(this);
            if (index < -1 || index >= cellsArr.length) {
              _throwDom('IndexSizeError', 'index out of range');
            }
            var victim = (index === -1) ? cellsArr[cellsArr.length - 1] : cellsArr[index];
            if (victim && typeof victim.remove === 'function') victim.remove();
          };
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
        // R3049：textarea.defaultValue（闭合 R3048 限制①）——textarea 无 value 属性，default = 初值 textContent。
        // 惰性捕获（首读时 = 当前 text，未被 value= 改过即初值；value setter 首写前亦捕获保初值不丢）。
        if (prop === 'defaultValue' && _realTag(sel, handle) === 'TEXTAREA') {
          if (_textareaDefault[key] == null) {
            _textareaDefault[key] = handle ? (__zw_get_text_handle(handle) || '') : (__zw_get_text(sel) || '');
          }
          return _textareaDefault[key];
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
        // reflected 字符串属性（title/lang）——get 反射同名 attribute（无 → ''）；同步 set→get 优先读
        // _reflectedAttrs 缓存（__zw_set_attr 异步入队，无缓存则 set 后 get 读 stale 快照）。
        if (prop === 'title' || prop === 'lang') {
          var rc = _reflectedAttrs[key];
          if (rc && Object.prototype.hasOwnProperty.call(rc, prop)) return rc[prop];
          return (handle ? __zw_get_attr_handle(handle, prop) : __zw_get_attr(sel, prop)) || '';
        }
        // `el.dir`——spec enumerated attribute（https://html.spec.whatwg.org/multipage/dom.html#the-dir-attribute）：
        // 关键字 ltr/rtl/auto（ASCII case-insensitive）→ 返规范小写；missing/invalid（含 "null"/"foo"/"" 等，
        // spec missing & invalid value default 均空串）→ 空串。区别 title/lang 的 plain DOMString 反射（直读）。
        // 同步 set→get 优先读缓存。闭合 R3185 已知限制①。
        if (prop === 'dir') {
          var drc = _reflectedAttrs[key];
          var dval = (drc && Object.prototype.hasOwnProperty.call(drc, 'dir'))
            ? drc['dir']
            : ((handle ? __zw_get_attr_handle(handle, 'dir') : __zw_get_attr(sel, 'dir')) || '');
          var dlo = String(dval).toLowerCase();
          return (dlo === 'ltr' || dlo === 'rtl' || dlo === 'auto') ? dlo : '';
        }
        // https://html.spec.whatwg.org/multipage/interaction.html#dom-tabindex
        // `el.tabIndex` reflects explicit tabindex; natively focusable elements default to 0.
        if (prop === 'tabIndex') {
          var rtc = _reflectedAttrs[key];
          if (rtc && Object.prototype.hasOwnProperty.call(rtc, 'tabindex')) return rtc['tabindex'];
          var tiraw = handle ? __zw_get_attr_handle(handle, 'tabindex') : __zw_get_attr(sel, 'tabindex');
          var tin = parseInt(tiraw, 10);
          if (!isNaN(tin)) return tin;
          var titag = _realTag(sel, handle);
          if (titag === 'BUTTON' || titag === 'INPUT' || titag === 'SELECT' || titag === 'TEXTAREA' || titag === 'SUMMARY') return 0;
          if (titag === 'A' || titag === 'AREA') {
            var tihref = handle ? __zw_has_attr_handle(handle, 'href') : __zw_has_attr(sel, 'href');
            if (tihref === '1') return 0;
          }
          var ticeHas = handle ? __zw_has_attr_handle(handle, 'contenteditable') : __zw_has_attr(sel, 'contenteditable');
          if (ticeHas === '1') {
            var tice = handle ? __zw_get_attr_handle(handle, 'contenteditable') : __zw_get_attr(sel, 'contenteditable');
            if (tice === '' || String(tice).toLowerCase() === 'true') return 0;
          }
          return -1;
        }
        // `el.contentEditable`——spec HTML 枚举属性反射，规范化状态：空串/case-insensitive "true"→"true"、
        // "false"→"false"、缺省/非法→"inherit"。经 [`_contentEditableState`]（R3187，has_attr 区分缺省与空串
        // keyword）。旧实现直读缓存/host 原值返 "foo"/"TRUE"/""（R3187 闭合）。
        if (prop === 'contentEditable') {
          return _contentEditableState(key, sel, handle);
        }
        // `el.isContentEditable`——计算 bool：contentEditable 处 true 状态（空串 / case-insensitive "true"）→
        // true，余（含 false / inherit / 缺省）→ false。**简化**：不沿祖先链解析 'inherit'（spec：inherit 时看
        // 最近可编辑祖先）——本沙箱无渲染期可编辑态，元素自身 true 状态即 true。经 [`_contentEditableState`]。
        if (prop === 'isContentEditable') {
          return _contentEditableState(key, sel, handle) === 'true';
        }
        // `el.accessKey`——反射 accesskey 属性（无 → ''）；同步 set→get 优先读缓存。
        if (prop === 'accessKey') {
          var akc = _reflectedAttrs[key];
          if (akc && Object.prototype.hasOwnProperty.call(akc, 'accesskey')) return akc['accesskey'];
          return (handle ? __zw_get_attr_handle(handle, 'accesskey') : __zw_get_attr(sel, 'accesskey')) || '';
