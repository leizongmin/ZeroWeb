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
    // R146（js-dom M4）：WebIDL boolean 转换——primitive 真值（2.3/-1000.3/"AAAA"）经
    // Boolean() 归 true（WPT EventListenerOptions-capture "Capture boolean should be
    // honored correctly"：capture=2.3 期望 CAPTURING_PHASE。旧 `opts === true` 仅认
    // 字面 true，数字真值落到 `opts.capture`（number 无 .capture → undefined）误判
    // false）。对象形态仍读 .capture 字段（其值再经同一转换）。
    if (opts == null) return false;
    if (typeof opts !== 'object') return Boolean(opts);
    return Boolean(opts.capture);
  }

  // addEventListener `opts.once` 提取（仅对象形式 `{ once: true }`；布尔形式无 once 语义）。
  function _optOnce(opts) {
    return !!(opts && opts.once);
  }

  // js-dom M4 R105：`opts.passive` 提取 + **passive-by-default**（spec HTML
  // `default-passive-value`：touchstart/touchmove/wheel（含 legacy mousewheel）的 listener
  // 在 window/document/body 三类 target 上注册时，未显式指定 `passive` 则默认 true——
  // passive listener 内 preventDefault() 是 no-op（canceled flag 不变，控制台告警语义
  // 略）。WPT dom/events/passive-by-default.html。返回 null = 未显式指定（调用方按
  // target+type 决定默认值）。
  function _optPassive(opts) {
    if (!opts || typeof opts !== 'object') return null;
    return opts.passive === undefined ? null : !!opts.passive;
  }
  // R105：事件类型是否 passive-by-default（spec 只对 window/document/body target 生效，
  // 调用方传 targetKind 判定）。
  var _ZW_PASSIVE_DEFAULT_TYPES = { touchstart: 1, touchmove: 1, wheel: 1, mousewheel: 1 };
  function _listenerPassiveDefault(type, opts, targetKind) {
    var p = _optPassive(opts);
    if (p !== null) return p;
    // 未显式指定：window/document/body target 的 touch/wheel 族默认 passive（spec）。
    return !!(targetKind && _ZW_PASSIVE_DEFAULT_TYPES[String(type)]);
  }

  function _globalAddEventListener(type, fn, opts) {
    var key = _elKey('html', null);
    var t = String(type);
    if (!_listenerStore[key]) _listenerStore[key] = {};
    if (!_listenerStore[key][t]) _listenerStore[key][t] = [];
    // R143（js-dom M4）：spec「add an event listener」步骤 4——重复 listener（同 type +
    // 同 callback + 同 capture，同 target 槽位）**静默丢弃**（WPT handler-count "Duplicate
    // listener is discarded"：addEventListener 三次同 fn 同 capture 只计一次派发）。
    var _r143Cap = _optCapture(opts);
    var _r143List0 = _listenerStore[key][t];
    for (var _r143i = 0; _r143i < _r143List0.length; _r143i++) {
      if (_r143List0[_r143i].fn === fn && _r143List0[_r143i].capture === _r143Cap
          && _r143List0[_r143i].tgt === 'win') return;
    }
    // R40：window 注册打 tgt='win' 标（document/window/html 三合一 key 内槽位区分）。
    // R105：passive 字段——window target 的 touch/wheel 族默认 passive（spec default-passive-value）。
    _listenerStore[key][t].push({ fn: fn, capture: _r143Cap, once: _optOnce(opts), tgt: 'win',
      passive: _listenerPassiveDefault(t, opts, true) });
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
  // js-dom M3 R94：HTMLElement 装载 **ctor 桥 hook**（仅 polyfill 自建路径——native_dom 模式 native
  // HTMLElement 已在全局，走 native S5b upgrade slot，本桥不参与）。derived class 的 `super()` 调本
  // ctor 时：若 `_zwCeExisting` 已设（createElement/upgrade 正在把既有元素升级为 custom 实例），
  // **返回该元素**（spec derived-ctor 语义：base ctor 返回对象 → 成为整条 ctor 链的 this）→ 用户
  // ctor 体以 this=既有元素继续执行（`this.state=5` 落在元素 proxy 上）。未设 → 普通空对象（旧
  // stub 行为）。这闭合 R90 已知限制「class ctor 体不可重放」——不是重放，是经 super() 返回值
  // 注入 this（探针实证：inst === el、方法/instanceof/嵌套 create/异常消费 全部正确）。
  var _zwCeExisting = null; // createElement/upgrade 在途的既有元素（消费即清，单发）
  if (!globalThis.HTMLElement) {
    globalThis.HTMLElement = function HTMLElement() {
      if (_zwCeExisting) {
        var _zwCeEl = _zwCeExisting;
        _zwCeExisting = null;
        return _zwCeEl;
      }
    };
  }
  // prototype 链仅当 polyfill 自建三者时设（native 已注册则不重设——避免破坏 native prototype）。
  if (_zwBuiltNodeChain) {
    globalThis.Node.prototype = {};
    globalThis.Element.prototype = Object.create(globalThis.Node.prototype);
    globalThis.HTMLElement.prototype = Object.create(globalThis.Element.prototype);
    // js-dom M4 R80：Node 接口常量（spec dom-node —— 实例经原型链读 `element.ELEMENT_NODE` 等；
    // WPT Document-createElementNS 断言 `element.nodeType === element.ELEMENT_NODE`，缺失 →
    // undefined ≠ 1）。构造器上的静态常量（Node.ELEMENT_NODE）同步定义。
    var _zwNodeConsts = {
      ELEMENT_NODE: 1, ATTRIBUTE_NODE: 2, TEXT_NODE: 3, CDATA_SECTION_NODE: 4,
      ENTITY_REFERENCE_NODE: 5, ENTITY_NODE: 6, PROCESSING_INSTRUCTION_NODE: 7,
      COMMENT_NODE: 8, DOCUMENT_NODE: 9, DOCUMENT_TYPE_NODE: 10, DOCUMENT_FRAGMENT_NODE: 11,
      NOTATION_NODE: 12,
      DOCUMENT_POSITION_DISCONNECTED: 1, DOCUMENT_POSITION_PRECEDING: 2,
      DOCUMENT_POSITION_FOLLOWING: 4, DOCUMENT_POSITION_CONTAINS: 8,
      DOCUMENT_POSITION_CONTAINED_BY: 16, DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC: 32,
    };
    for (var _zwnc in _zwNodeConsts) {
      if (Object.prototype.hasOwnProperty.call(_zwNodeConsts, _zwnc)) {
        Object.defineProperty(globalThis.Node.prototype, _zwnc, { value: _zwNodeConsts[_zwnc], enumerable: false });
        Object.defineProperty(globalThis.Node, _zwnc, { value: _zwNodeConsts[_zwnc], enumerable: false });
      }
    }
  }
  // R3019：DOM 接口构造器占位——库（DOMPurify 等）常做 `x instanceof HTMLFormElement` /
  // `el.attributes instanceof NamedNodeMap` / `node.content instanceof DocumentFragment` 校验。
  // 这些构造器须以 function 存在（否则 `instanceof undefined` 抛 TypeError 中断 sanitize）。本桥接的
  // 元素为 proxy 对象非真实例，instanceof 恒返 false（正确：DOMPurify 仅借此识别 form/template 特殊处理）。
  // 原型链挂到对应基类（DocumentFragment→Node、HTML*→HTMLElement）仅为语义一致，instanceof 不依赖实例身份。
  globalThis.HTMLFormElement = globalThis.HTMLFormElement || function HTMLFormElement() {};
  globalThis.HTMLFormElement.prototype = Object.create(globalThis.HTMLElement.prototype);
  globalThis.NamedNodeMap = globalThis.NamedNodeMap || function NamedNodeMap() {};
  // js-dom M4 R122：原型方法占位——WPT attributes-namednodemap「should not interfere with
  // existing method names」断言 `map.item === NamedNodeMap.prototype.item`（**同一函数对象**，
  // named getter 'item' 不得遮蔽方法）。本桥 NamedNodeMap 是 Proxy 非真实例（方法在 get
  // trap 分支）——把原型方法直接置为 trap 分支返回的同一实现函数（_zwNNMItemImpl 等，
  // _attributesProxy 定义处赋值），两侧 identity 相等。
  globalThis.NamedNodeMap.prototype.item = function () { return null; };
  globalThis.NamedNodeMap.prototype.getNamedItem = function () { return null; };
  globalThis.NamedNodeMap.prototype.setNamedItem = function () { return null; };
  globalThis.NamedNodeMap.prototype.removeNamedItem = function () { return null; };
  // js-dom M4 R120：NodeList / HTMLCollection 构造器占位——WPT Document-Element-getElementsByTagName
  // 「Interfaces」断言 `!(x instanceof NodeList) && x instanceof HTMLCollection`（构造器缺失 →
  // ReferenceError 崩整簇）；expando 用例读 HTMLCollection.prototype.item / .namedItem（可被
  // own property 覆盖——集合 Proxy 的 set trap 存 expando）。集合实例的 prototype 由
  // _zwHCPrototype / _zwMakeCollection 接线到这两个 prototype（instanceof 真值）。
  globalThis.NodeList = globalThis.NodeList || function NodeList() {};
  // R140（js-dom M4）：live childNodes 承载数组的 instanceof NodeList——Array 原型链
  // 保持不动（迭代器 identity 断言），经 Symbol.hasInstance 认 __zwLiveNL 标记数组
  //（WPT Node-childNodes "should be a live collection" 的 instanceof 断言）。
  try {
    Object.defineProperty(globalThis.NodeList, Symbol.hasInstance, {
      configurable: true,
      value: function (v) {
        return Array.isArray(v) && v.__zwLiveNL === true;
      },
    });
  } catch (_e140hi) {}
  globalThis.HTMLCollection = globalThis.HTMLCollection || function HTMLCollection() {};
  // R3024：Attr 构造器占位——_zwMakeAttr 经 Object.create(Attr.prototype) 建真实例，使 `attr instanceof Attr`
  // 为 true（闭合 R3023 限制①；消费者按 nodeType===2 / instanceof Attr 校验属性节点）。
  globalThis.Attr = globalThis.Attr || function Attr() {};
  // R128：Attr.prototype → Node.prototype 链（spec interface-attr : Node；WPT Node-cloneNode
  // "createAttribute" 的 attr.cloneNode() 经 Node.prototype 解析——旧 Attr.prototype 直挂
  // Object.prototype，cloneNode 不可达）。
  try {
    if (globalThis.Node && globalThis.Node.prototype
        && Object.getPrototypeOf(globalThis.Attr.prototype) === Object.prototype) {
      Object.setPrototypeOf(globalThis.Attr.prototype, globalThis.Node.prototype);
    }
  } catch (_eAttr128) {}
  // js-dom M4 R122：`attr.value = v` 的**写回传播**（spec `dom-attr-value` setter——set an attribute
  // 值须经「change an attribute」更新所属元素）。_zwMakeAttr 建的是 own 数据属性，写 value 不传播
  // （WPT "Attribute values should not be parsed."：attr.value='Y&lt;' 后 el.getAttribute('x')
  // 须 'Y&lt;'）。ownerElement 是 _makeProxy 元素 proxy（R122 起 getAttributeNode/attrObj 绑定），
  // setter 委托元素 setAttributeNode 值更新（ownerElement 缺省 = 游离 Attr，仅本地改）。value/
  // nodeValue 同源（getter 由 own 数据属性遮蔽——enumerable 数据属性优先于原型 accessor；仅
  // setter 需要原型 accessor，own 数据属性 writable 会拦截赋值……故须 _zwMakeAttr 建后 delete
  // own value/nodeValue 使原型 accessor 生效——见 _zwMakeAttr 尾部 R122 改造）。
  (function () {
    var _r122Set = function (v) {
      var s = v == null ? '' : String(v);
      if (this._r122V === s) return; // 幂等护栏：值未变不传播（防 setAttribute↔绑定 Attr 互写环）
      this._r122V = s;
      this.textContent = s;
      this.data = s;
      var oe = this.ownerElement;
      if (oe && typeof oe.setAttribute === 'function') {
        try {
          var _r122Pref = this.prefix != null ? String(this.prefix) : null;
          var _r122Loc = this.localName != null ? String(this.localName) : String(this.name);
          var _r122Qn = _r122Pref ? _r122Pref + ':' + _r122Loc : _r122Loc;
          var _r122A_ns = this.namespaceURI != null ? String(this.namespaceURI) : null;
          if (_r122A_ns != null) oe.setAttributeNS(_r122A_ns, _r122Qn, s);
          else oe.setAttribute(_r122Qn, s);
        } catch (_eA122) {}
      }
    };
    try {
      Object.defineProperty(globalThis.Attr.prototype, 'value', { set: _r122Set, get: function () { return this._r122V != null ? this._r122V : ''; }, configurable: true });
      Object.defineProperty(globalThis.Attr.prototype, 'nodeValue', { set: _r122Set, get: function () { return this._r122V != null ? this._r122V : ''; }, configurable: true });
    } catch (_eD122) {}
  })();
  globalThis.DocumentFragment = globalThis.DocumentFragment || function DocumentFragment() {};
  globalThis.DocumentFragment.prototype = Object.create(globalThis.Node.prototype);
  // js-dom M4 R81：CharacterData 族构造器占位（Text/Comment/ProcessingInstruction/CDATASection）——
  // WPT Node-textContent `firstChild instanceof Text` 断言（构造器缺失 → ReferenceError 崩用例）。
  // 原型链挂 CharacterData → Node（instanceof Node/CharacterData 经原型链为 true；文本节点 proxy 的
  // getPrototypeOf 返 Node.prototype，instanceof Text 需 proxy 原型对齐——本桥文本节点为轻量对象，
  // instanceof Text 暂 false，构造器存在保证不抛 + 库 feature-detect 可用）。
  if (!globalThis.CharacterData) globalThis.CharacterData = function CharacterData() {};
  try {
    globalThis.CharacterData.prototype = Object.create(globalThis.Node.prototype);
  } catch (_eCData) {}
  // js-dom M4 R121：Text 真构造器（spec dom-text——WPT Text-constructor：new Text(data)
  // 经 _zwMText 建完整实例[data/nodeValue/ownerDocument=document/原型链
  // Text.prototype→CharacterData→Node]，String() 转换参数；旧空 stub 使 object.data
  // undefined 全簇 fail）。R108 的 dispatchEvent 保留（prototype 上补）。
  globalThis.Text = globalThis.Text || function Text(data) {
    var n = _zwMText(data === undefined ? '' : data, null);
    try { Object.setPrototypeOf(n, globalThis.Text.prototype); } catch (_eR121a) {}
    try { Object.defineProperty(n, 'ownerDocument', { get: function () { return globalThis.document; }, configurable: true }); } catch (_eR121b) {}
    return n;
  };
  try {
    globalThis.Text.prototype = Object.create(globalThis.CharacterData.prototype);
    // js-dom M4 R108：`new Text()` 实例的 dispatchEvent（WPT Event-dispatch-click
    // "look at parents"——`input.appendChild(new Text(...)).dispatchEvent(new MouseEvent
    // ('click', {bubbles:true}))` 冒泡触发父链 pre-click activation）。构造器实例是
    // 轻量对象（无 sel/handle），沿 parentNode 上派发（spec：Text 是 EventTarget）。
    if (!globalThis.Text.prototype.dispatchEvent) {
      globalThis.Text.prototype.dispatchEvent = function (event) {
        globalThis._zwDispatchGuard(event);
        var p = this.parentNode;
        // R139（js-dom M4）：仅 bubbles 事件冒泡到父派发——旧版无条件转父
        // `p.dispatchEvent(event)` 使父成为**新 target**，pre-click activation
        // 从父 INPUT 起找（命中父自身）→ 非 bubbling 的 Text click 也翻转父
        // checked（WPT Event-dispatch-click "look at parents only when event
        // bubbles"：`new MouseEvent('click')`（bubbles=false）断言父 checked
        // 不变）。spec：非冒泡事件 path = [target]，不达祖先。Text 自身无
        // activation/listener 面 → 直接返回；bubbles 事件维持转父（R108 冒泡
        // 触发父链 pre-click activation 语义不变）。
        if (event && event.bubbles && p && typeof p.dispatchEvent === 'function') {
          return p.dispatchEvent(event);
        }
        return !event._defaultPrevented;
      };
    }
  } catch (_eT) {}
  // js-dom M4 R121：Comment 真构造器（同 Text——WPT Comment-constructor 全簇）。
  globalThis.Comment = globalThis.Comment || function Comment(data) {
    var n = _zwMComment(data === undefined ? '' : data, null);
    try { Object.setPrototypeOf(n, globalThis.Comment.prototype); } catch (_eR121c) {}
    try { Object.defineProperty(n, 'ownerDocument', { get: function () { return globalThis.document; }, configurable: true }); } catch (_eR121d) {}
    return n;
  };
  try { globalThis.Comment.prototype = Object.create(globalThis.CharacterData.prototype); } catch (_eC) {}
  globalThis.ProcessingInstruction = globalThis.ProcessingInstruction || function ProcessingInstruction() {};
  try { globalThis.ProcessingInstruction.prototype = Object.create(globalThis.CharacterData.prototype); } catch (_ePI) {}
  globalThis.CDATASection = globalThis.CDATASection || function CDATASection() {};
  try { globalThis.CDATASection.prototype = Object.create(globalThis.Text.prototype); } catch (_eCD) {}
  // R51：`Document` 构造器（spec `interface-document`——`new Document()` 返独立空 XML Document）。
  // WPT dom/common.js setupRangeTests 经 `new Document()` + createCDATASection 建 testNodes——
  // 构造器缺失使 setup 中途崩 → testNodes undefined → dom/* 依赖 common.js 的 mega-case
  //（NodeIterator.html 等）全体退化。以 `_makeDetachedDocument('')`（R2815 独立可变 doc）承载。
  // prototype 挂 Node.prototype（instanceof Node 经原型链）。
  globalThis.Document = globalThis.Document || function Document() {
    var d = _makeDetachedDocument('');
    // R81：spec `new Document()` 返 **XML** Document（contentType 'application/xml'，createElement
    // 的元素 ns 恒 null）。
    d.contentType = 'application/xml';
    d._docNS = null;
    return d;
  };
  try {
    if (globalThis.Document.prototype) {
      Object.setPrototypeOf(globalThis.Document.prototype, globalThis.Node.prototype);
    }
  } catch (_e) {}
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
  // js-dom M3 R100：SVG 元素接口构造器（Vue runtime 挂载期 resolveRootNamespace 读
  // `container instanceof SVGElement`——缺构造器 ReferenceError 使 mount 中止）。
  // spec SVGElement 链：SVGElement → Element（非 HTMLElement）；最小可判别构造器
  //（空 stub，供 typeof/instanceof 面），prototype 链 SVGElement → Element.prototype，
  // SVGSVGElement → SVGElement.prototype。
  var _zwSvgElementIfaces = ['SVGElement', 'SVGSVGElement', 'SVGGraphicsElement',
    'SVGTitleElement', 'SVGPathElement', 'SVGTSpanElement', 'SVGTextElement',
    'SVGCircleElement', 'SVGEllipseElement', 'SVGLineElement', 'SVGRectElement',
    'SVGPolygonElement', 'SVGPolylineElement', 'SVGImageElement', 'SVGUseElement',
    'SVGGElement', 'SVGDefsElement', 'SVGSymbolElement', 'SVGSwitchElement',
    'SVGAElement', 'SVGStopElement', 'SVGGradientElement', 'SVGLinearGradientElement',
    'SVGRadialGradientElement', 'SVGPatternElement', 'SVGLayerElement', 'SVGMaskElement',
    'SVGMarkerElement', 'SVGClipPathElement', 'SVGFilterElement', 'SVGFEElement',
    'SVGStyleElement', 'SVGScriptElement', 'SVGForeignObjectElement',
    'SVGAnimateElement', 'SVGSetElement', 'SVGAnimateMotionElement',
    'SVGAnimateTransformElement'];
  for (var _si = 0; _si < _zwSvgElementIfaces.length; _si++) {
    var _sn = _zwSvgElementIfaces[_si];
    if (!globalThis[_sn]) {
      globalThis[_sn] = new Function('return function ' + _sn + '() {}')();
      globalThis[_sn].prototype = (_sn === 'SVGElement')
        ? Object.create(globalThis.Element ? globalThis.Element.prototype : Object.prototype)
        : Object.create(globalThis.SVGElement.prototype);
    }
  }
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
  // js-dom M4 R107：HTMLBodyElement/HTMLFrameSetElement.prototype 的 Window-forwarding
  // handler 属性（onblur/onerror/onfocus/onload/onscroll/onresize）以 **enumerable**
  // data property（值 null）挂原型——spec IDL handler 属性 for-in 可见（WPT
  // Body-FrameSet-Event-Handlers "Enumerate"：for (var a in el) 须含这些名）。proxy 的
  // get/set trap 对这 6 个名仍走 R107 转发（原型属性只是枚举面；读值时 trap 先于原型）。
  ['HTMLBodyElement', 'HTMLFrameSetElement'].forEach(function (_r107i) {
    var C = globalThis[_r107i];
    if (!C || !C.prototype) return;
    ['blur', 'error', 'focus', 'load', 'scroll', 'resize'].forEach(function (_r107t) {
      if (!Object.prototype.hasOwnProperty.call(C.prototype, 'on' + _r107t)) {
        Object.defineProperty(C.prototype, 'on' + _r107t, {
          value: null, writable: true, enumerable: true, configurable: true,
        });
      }
    });
  });
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
  // R134（js-dom M4）：Element.prototype 的 [Unscopable] 表（spec ChildNode 四方法
  // before/after/replaceWith/remove + ParentNode prepend/append 均 [Unscopable]——
  // with(element) 词法域不可见。WPT remove-unscopable 六断言 + inline handler 双向
  // 语义：this.remove 是 function、裸 remove 解析 window）。
  try {
    Object.defineProperty(globalThis.Element.prototype, Symbol.unscopables, {
      value: { before: true, after: true, replaceWith: true, remove: true, prepend: true, append: true },
      writable: false, enumerable: false, configurable: true,
    });
  } catch (_e134sd) {}
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
  // js-dom M4 R128：`Node.prototype.cloneNode` 泛型（spec `dom-node-clone-node`——对任意
  // node 类型克隆）。WPT Node-cloneNode 对 fragment/text/comment/PI/Attr/doctype/doc 的
  // clone 断言：此前这些节点无 cloneNode（或落到 Element 版返 nodeType 1 的错形态）。
  // 分派按 this 形态：③⑧文本/注释走 _zwMText/_zwMComment 工厂（保原型链）；
  // ⑦PI 走 _piHandles 元数据重建；②Attr 走 _zwMakeAttr（namespace/prefix/value 全复制）；
  // ⑩doctype 走本 doc 的 implementation.createDocumentType；⑪ fragment（handle 形态）
  // 建 fragment + 递归 clone 子；⑨ Document 建 detached doc + 复制 doctype/根元素；
  // ①元素不在此层（proxy get trap / Element.prototype 各自实现）。
  _zwDefProtoMethod(globalThis.Node.prototype, 'cloneNode', function (deep) {
    var n = this;
    if (!n || typeof n !== 'object') return n;
    var nt = 0;
    try { nt = n.nodeType | 0; } catch (_e128a) { return n; }
    // 有自身 cloneNode（proxy 元素 / canvas 等专形节点）→ 委托（own-property 判定——
    // R126/R127 教训：typeof 会命中本原型方法自身致无限递归）。
    var _r128Own = Object.prototype.hasOwnProperty.call(n, 'cloneNode')
      ? n.cloneNode : null;
    if (_r128Own && typeof _r128Own === 'function'
        && _r128Own !== globalThis.Node.prototype.cloneNode) {
      return _r128Own.call(n, deep);
    }
    // 文本/注释：经工厂重建（保 nodeType/原型链/CharacterData 方法面）。
    if (nt === 3) return _zwMText(String(n.data != null ? n.data : (n.nodeValue || '')), null);
    if (nt === 8) return _zwMComment(String(n.data != null ? n.data : (n.nodeValue || '')), null);
    // PI：_piHandles 元数据（target/data）重建（R9 桥——handle→元数据表）。handle 形态
    // 的 PI proxy 经 get trap 的 cloneNode 分支（part04 R128）处理；此处兜底 plain
    // object PI 视图（_zwMPiFromBogus 派生）——经 document.createProcessingInstruction。
    if (nt === 7 && typeof globalThis.document.createProcessingInstruction === 'function') {
      var _r128Tgt = '', _r128Data = '';
      try { _r128Tgt = String(n.target != null ? n.target : ''); } catch (_e128t) {}
      try { _r128Data = String(n.data != null ? n.data : ''); } catch (_e128d) {}
      if (_r128Tgt) return globalThis.document.createProcessingInstruction(_r128Tgt, _r128Data);
    }
    // Attr：_zwMakeAttr 全字段复制（namespaceURI/prefix/localName/value——WPT
    // createAttribute(NS) 断言四字段 + clone 与源 value 独立）。
    if (nt === 2 && typeof _zwMakeAttr === 'function') {
      var _r128A = _zwMakeAttr(
        n.name != null ? String(n.name) : String(n.nodeName || ''),
        n.value != null ? String(n.value) : '', null);
      try {
        _r128A.namespaceURI = n.namespaceURI != null ? n.namespaceURI : null;
        _r128A.prefix = n.prefix != null ? n.prefix : null;
        if (n.localName != null) _r128A.localName = String(n.localName);
      } catch (_e128b) {}
      return _r128A;
    }
    // doctype：经 ownerDocument.implementation.createDocumentType 重建（name/publicId/
    // systemId 全复制；WPT "implementation.createDocumentType" 断言 instanceof
    // DocumentType + 三字段）。DocumentType 构造器全局占位见下方 R128。
    if (nt === 10) {
      var _r128Doc = n.ownerDocument || globalThis.document;
      if (_r128Doc && _r128Doc.implementation
          && typeof _r128Doc.implementation.createDocumentType === 'function') {
        return _r128Doc.implementation.createDocumentType(
          String(n.name != null ? n.name : (n.nodeName || '')),
          String(n.publicId != null ? n.publicId : ''),
          String(n.systemId != null ? n.systemId : ''));
      }
      return n;
    }
    // fragment：建新 fragment + 递归 clone 子（deep 语义；浅 clone 返空 fragment）。
    if (nt === 11 && typeof globalThis.document.createDocumentFragment === 'function') {
      var _r128F = globalThis.document.createDocumentFragment();
      if (deep && n.childNodes && n.childNodes.length) {
        for (var _r128i = 0; _r128i < n.childNodes.length; _r128i++) {
          var _r128c = n.childNodes[_r128i];
          if (!_r128c || typeof _r128c.cloneNode !== 'function') continue;
          try { _r128F.appendChild(_r128c.cloneNode(true)); } catch (_e128c) {}
        }
      }
      return _r128F;
    }
    // Document：detached doc 重建（WPT "implementation.createDocument" 断言
    // charset/contentType/URL/compatMode 相等 + instanceof Document；
    // "implementation.createHTMLDocument" 断言 copy.title === ''）。deep=true 时复制
    // doctype / documentElement（WPT Node-cloneNode-document-with-doctype 断言
    // clone.childNodes.length 与 doctype 三字段）。按源 contentType 分派：XML →
    // createDocument(null,null)（contentType 'application/xml'），HTML →
    // createHTMLDocument('')（title 空——不复制源 title）。
    if (nt === 9 && globalThis.document && globalThis.document.implementation) {
      var _r128Impl = globalThis.document.implementation;
      var _r128Ct = '';
      try { _r128Ct = String(n.contentType || ''); } catch (_e128ct) {}
      var _r128IsHtmlDoc = _r128Ct.indexOf('html') >= 0;
      var _r128Dc;
      if (!_r128IsHtmlDoc && typeof _r128Impl.createDocument === 'function') {
        _r128Dc = _r128Impl.createDocument(null, null, null);
      } else if (typeof _r128Impl.createHTMLDocument === 'function') {
        _r128Dc = _r128Impl.createHTMLDocument('');
      }
      if (_r128Dc && deep && !_r128IsHtmlDoc && n.childNodes && n.childNodes.length) {
        for (var _r128di = 0; _r128di < n.childNodes.length; _r128di++) {
          var _r128dn = n.childNodes[_r128di];
          if (!_r128dn) continue;
          var _r128dnt = _r128dn.nodeType | 0;
          if (_r128dnt !== 10 && _r128dnt !== 1) continue; // doctype / documentElement
          try {
            var _r128dcc = _r128dn.cloneNode ? _r128dn.cloneNode(true) : null;
            if (_r128dcc) _r128Dc.appendChild(_r128dcc);
          } catch (_e128dc) {}
        }
      }
      if (_r128Dc) return _r128Dc;
    }
    // 兜底：Element.prototype 的 deepClone（既有行为——plain 对象元素形态）。
    var _r128Ep = globalThis.Element && globalThis.Element.prototype;
    if (_r128Ep && typeof _r128Ep.cloneNode === 'function'
        && _r128Ep.cloneNode !== globalThis.Node.prototype.cloneNode) {
      return _r128Ep.cloneNode.call(n, deep);
    }
    return n;
  });
  // R131（js-dom M4）：`Node.prototype.isEqualNode` 泛型——统一走 `_zwIsEqualNode`
  //（spec dom-node-isequalnode 逐类型字段 + 子节点递归）。proxy 元素的 get trap 有
  // isEqualNode 分支（part04，_nodeSig 旧签名版）——**原型方法先于 get trap 兜底**：
  // proxy get trap 拦截一切属性读，须同步改 part04 分支委托本实现（同轮修改）。
  _zwDefProtoMethod(globalThis.Node.prototype, 'isEqualNode', function (other) {
    if (!other || typeof other !== 'object') return false;
    return _zwIsEqualNode(this, other);
  });
  // R136（js-dom M4）：`Node.prototype.getRootNode` 泛型（spec `dom-node-getrootnode`
  // https://dom.spec.whatwg.org/#dom-node-getrootnode）——沿 parentNode 链上行到根。
  // 旧实现只有 proxy 元素的 get trap 分支（part04 `_ancestorChain` sel 版——handle-only
  // 节点[createElement/Text/PI/fragment 产物]与 document/fragment/text 均不可达 →
  // `getRootNode is not a function`，WPT rootNode.html 5F）。泛型 receiver 分派：
  // ① 有自身 parentNode（proxy 元素经 _parentNodeFor；plain 节点经 R84 defineProperty
  //   反链）→ 沿链上行；② document 自身是根（返 this）；③ detached 无父 → 返 this。
  // composed 选项（shadow-including root）：宿主链上 shadow root 的 host 节点继续上行
  // ——本沙箱 shadow root 经 attachShadow 的 host 反向可达（_zwShadowHost），无则等价
  // 普通 root（best-effort）。
  _zwDefProtoMethod(globalThis.Node.prototype, 'getRootNode', function (options) {
    var composed = !!(options && options.composed);
    var cur = this, guard = 0;
    while (guard < 4096) {
      guard++;
      var p = null;
      try { p = cur.parentNode; } catch (_e136p) { p = null; }
      if (p === null || p === undefined) {
        // composed：shadow root 的根继续经 host 上行（spec shadow-including root）。
        if (composed && cur && cur.host !== undefined) {
          var host = null;
          try { host = cur.host; } catch (_e136h) { host = null; }
          if (host && host !== cur) { cur = host; continue; }
        }
        return cur;
      }
      if (p === cur) return cur; // 环守卫
      cur = p;
    }
    return cur;
  });
  // R136（js-dom M4）：native 叠加路径的原型链补挂——`_zwBuiltNodeChain=false`（native
  // HTMLElement 已注册）时 native HTMLElement.prototype 是 FunctionTemplate 产物，原型链
  // 直连 Object.prototype（不经 polyfill Node.prototype——native 不注册 Node ctor），上方
  // 定义的 getRootNode 对 native 链上的对象（shadow innerHTML 解析子 _zwMEl 经
  // HTMLDivElement.prototype → native HTMLElement.prototype 链）不可达。幂等 defineProperty
  // 补挂（own 已有则不动——R130 XMLDocument 常量同款模式），polyfill 自建链路径（own 已有）
  // 零改动。
  // R137（js-dom M4）：补挂升级为**整链重接**——native 只注册 HTMLElement（不注册
  // Node/Element ctor），native HTMLElement.prototype 直链 Object.prototype，而 shim 自建
  // 链被 `_zwBuiltNodeChain=false` 跳过 → `Element.prototype`（R3019 parentNode/childNodes/
  // remove/cloneNode）与 `Node.prototype`（常量族 + getRootNode）都不在 native 链上 →
  // native 路径 created 元素（createElement/createElementNS 产物经 getPrototypeOf trap 的
  // HTML*Element.prototype → native HTMLElement.prototype）的 `instanceof Node/Element` 恒
  // false、`el.ELEMENT_NODE` undefined（WPT Document-createElementNS native 叠加路径 596F
  // 的主根因）。修复：native HTMLElement.prototype 的 proto 重接到 shim Element.prototype
  //（→ Node.prototype → Object.prototype，R128 Attr.prototype 同款 setPrototypeOf 模式），
  // 单一接线恢复整条链（getRootNode own 补挂随之冗余但保留——防 Element.prototype 后续
  // 被替换的保险层）。幂等守卫：仅当当前 proto 是 Object.prototype（未被其他层重接）时执行。
  if (!_zwBuiltNodeChain && globalThis.HTMLElement && globalThis.HTMLElement.prototype) {
    try {
      if (globalThis.Element && globalThis.Element.prototype
          && Object.getPrototypeOf(globalThis.HTMLElement.prototype) === Object.prototype) {
        Object.setPrototypeOf(globalThis.HTMLElement.prototype, globalThis.Element.prototype);
      }
    } catch (_e137ch) {}
    try {
      // R137 续：Element.prototype → Node.prototype 补接（native 模式 shim Element ctor 走
      // `if (!globalThis.Element)` 兜底创建，其 prototype 是裸对象直链 Object.prototype——
      // 上面 R136 的 HTMLElement.prototype → Element.prototype 重接后链在 Element 断头，
      // `instanceof Node` 仍 false。补接后整链：HTML*Element.prototype → HTMLElement.prototype
      // (native) → Element.prototype → Node.prototype → Object.prototype）。
      if (globalThis.Node && globalThis.Node.prototype
          && Object.getPrototypeOf(globalThis.Element.prototype) === Object.prototype) {
        Object.setPrototypeOf(globalThis.Element.prototype, globalThis.Node.prototype);
      }
    } catch (_e137en) {}
    try {
      // R137 续：Node 常量族在 native 模式缺位（`_zwBuiltNodeChain=false` 跳过常量挂载分支，
      // 而上述 Element.prototype 的成员挂载在 part03 后段无条件执行）——native 链上的元素
      // `el.ELEMENT_NODE` undefined（WPT `element.nodeType === element.ELEMENT_NODE` 断言）。
      // 幂等补挂 Node.prototype + Node ctor 静态常量（R130 XMLDocument 常量同款字面量表）。
      var _r137Consts = {
        ELEMENT_NODE: 1, ATTRIBUTE_NODE: 2, TEXT_NODE: 3, CDATA_SECTION_NODE: 4,
        ENTITY_REFERENCE_NODE: 5, ENTITY_NODE: 6, PROCESSING_INSTRUCTION_NODE: 7,
        COMMENT_NODE: 8, DOCUMENT_NODE: 9, DOCUMENT_TYPE_NODE: 10,
        DOCUMENT_FRAGMENT_NODE: 11, NOTATION_NODE: 12,
        DOCUMENT_POSITION_DISCONNECTED: 1, DOCUMENT_POSITION_PRECEDING: 2,
        DOCUMENT_POSITION_FOLLOWING: 4, DOCUMENT_POSITION_CONTAINS: 8,
        DOCUMENT_POSITION_CONTAINED_BY: 16, DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC: 32,
      };
      for (var _r137cn in _r137Consts) {
        if (Object.prototype.hasOwnProperty.call(_r137Consts, _r137cn)) {
          if (globalThis.Node.prototype[_r137cn] === undefined) {
            Object.defineProperty(globalThis.Node.prototype, _r137cn,
              { value: _r137Consts[_r137cn], enumerable: false, configurable: true });
          }
          if (globalThis.Node[_r137cn] === undefined) {
            Object.defineProperty(globalThis.Node, _r137cn,
              { value: _r137Consts[_r137cn], enumerable: false, configurable: true });
          }
        }
      }
    } catch (_e137nc) {}
    try {
      if (!Object.prototype.hasOwnProperty.call(globalThis.HTMLElement.prototype, 'getRootNode')) {
        Object.defineProperty(globalThis.HTMLElement.prototype, 'getRootNode',
          { value: globalThis.Node.prototype.getRootNode, writable: true, configurable: true });
      }
    } catch (_e136n) {}
  }
  // js-dom M4 R128：DocumentType 构造器全局占位（WPT Node-cloneNode
  // "implementation.createDocumentType" 的 `check_copy(dt, copy, DocumentType)`——
  // `DocumentType is not defined` ReferenceError 崩用例；instanceof 经占位为 true 的
  // 前提是 doctype 对象原型挂 DocumentType.prototype——detached doc 的
  // createDocumentType 产物经 setPrototypeOf 接线（见 implementation 定义处）。
  if (!globalThis.DocumentType) {
    globalThis.DocumentType = function DocumentType() {};
    try { globalThis.DocumentType.prototype = Object.create(globalThis.Node.prototype); } catch (_eDt128) {}
  }
  // R128：XMLDocument 构造器占位（WPT Node-cloneNode-XMLDocument `doc.constructor ===
  // XMLDocument`——spec：XMLDocument : Document，XML 文档的 constructor 是 XMLDocument）。
  if (!globalThis.XMLDocument) {
    globalThis.XMLDocument = function XMLDocument() {};
    try { globalThis.XMLDocument.prototype = Object.create(globalThis.Document.prototype); } catch (_eXd128) {}
  }
  // Node.DOCUMENT_POSITION_* 静态常量（compareDocumentPosition bitmask，R2815）——库常读 Node.DOCUMENT_POSITION_FOLLOWING 等。
  globalThis.Node.DOCUMENT_POSITION_DISCONNECTED = 1;
  globalThis.Node.DOCUMENT_POSITION_PRECEDING = 2;
  globalThis.Node.DOCUMENT_POSITION_FOLLOWING = 4;
  globalThis.Node.DOCUMENT_POSITION_CONTAINS = 8;
  globalThis.Node.DOCUMENT_POSITION_CONTAINED_BY = 16;
  globalThis.Node.DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC = 32;
  // R130（js-dom M4）：NodeType 常量补齐到静态 Node + Node.prototype（无 polyfill-chain
  // 守卫——native 叠加路径下 globalThis.Node 是 native 构造器，常量族全缺，WPT
  // DOMImplementation-createDocument `assert_equals(doc.nodeType, Node.DOCUMENT_NODE)`
  // native 111F 主因；DOCUMENT_POSITION 族在上方同为无守卫静态定义，此处同款。幂等：
  // 已有值不覆盖（polyfill 自建链路径已定义同值）。
  (function () {
    var _r130Nc = { ELEMENT_NODE: 1, ATTRIBUTE_NODE: 2, TEXT_NODE: 3, CDATA_SECTION_NODE: 4,
      COMMENT_NODE: 8, DOCUMENT_NODE: 9, DOCUMENT_TYPE_NODE: 10, DOCUMENT_FRAGMENT_NODE: 11,
      NOTATION_NODE: 12 };
    for (var _r130K in _r130Nc) {
      if (!Object.prototype.hasOwnProperty.call(_r130Nc, _r130K)) continue;
      if (globalThis.Node[_r130K] === undefined) globalThis.Node[_r130K] = _r130Nc[_r130K];
      try {
        if (globalThis.Node.prototype[_r130K] === undefined) globalThis.Node.prototype[_r130K] = _r130Nc[_r130K];
      } catch (_e130np) {}
    }
  })();
  // js-dom M4 R117：Node.prototype 变异族泛型方法（WPT pre-insertion-validation-notfound 经
  // `Node.prototype.replaceChild` .call 到任意 parent——doctype/text/PI/comment/CDATA 作 parent
  // 须 HierarchyRequestError）。实现按 receiver 分派：有自身方法（proxy）直调；纯对象走
  // nodeType 校验（非 Element/Document/Fragment → HRE；child NotFound 判定经 childNodes）。
  var _r117GenVal = function (parent, node, methodName) {
    var pnt = 0;
    try { pnt = parent.nodeType | 0; } catch (_e) {}
    if (pnt !== 1 && pnt !== 9 && pnt !== 11) {
      throw new (globalThis.DOMException || Error)(
        'Nodes of type ' + pnt + ' cannot have children.', 'HierarchyRequestError');
    }
    if (node && typeof node === 'object') {
      var nnt = node.nodeType | 0;
      var pIsDoc = pnt === 9;
      if (pIsDoc && (nnt === 3 || nnt === 4 || nnt === 9)) {
        throw new (globalThis.DOMException || Error)(
          'Nodes of type ' + nnt + ' cannot be inserted into a Document.', 'HierarchyRequestError');
      }
      if (!pIsDoc && (nnt === 9 || nnt === 10)) {
        throw new (globalThis.DOMException || Error)(
          'Only a Document can contain nodes of type ' + nnt + '.', 'HierarchyRequestError');
      }
      var anc = parent, hops = 0;
      while (anc && hops++ < 64) {
        if (anc === node) {
          throw new (globalThis.DOMException || Error)(
            'The new node is an ancestor of this node.', 'HierarchyRequestError');
        }
        try { anc = anc.parentNode; } catch (_e2) { break; }
        if (anc == null) break;
      }
    }
  };
  var _r117GenChildOf = function (parent, child) {
    try {
      var kids = parent.childNodes || [];
      for (var i = 0; i < kids.length; i++) {
        if (kids[i] === child) return true;
      }
    } catch (_e) {}
    return false;
  };
  var _r117GenValParentAncestor = function (parent, node) {
    // 仅 parent 类型 + 祖先环校验（spec replace-child 步骤 1-2——**先于** child NotFound 与
    // node 类型校验，WPT pre-insertion-validation-notfound 的顺序断言族）。
    var pnt = 0;
    try { pnt = parent.nodeType | 0; } catch (_e) {}
    if (pnt !== 1 && pnt !== 9 && pnt !== 11) {
      throw new (globalThis.DOMException || Error)(
        'Nodes of type ' + pnt + ' cannot have children.', 'HierarchyRequestError');
    }
    if (node && typeof node === 'object') {
      var anc = parent, hops = 0;
      while (anc && hops++ < 64) {
        if (anc === node) {
          throw new (globalThis.DOMException || Error)(
            'The new node is an ancestor of this node.', 'HierarchyRequestError');
        }
        try { anc = anc.parentNode; } catch (_e2) { break; }
        if (anc == null) break;
      }
    }
  };
  var _r117GenValNodeType = function (parent, node) {
    // node 类型校验（步骤 4-5——child NotFound 之后）。
    if (!node || typeof node !== 'object') return;
    var pnt = 0;
    try { pnt = parent.nodeType | 0; } catch (_e) {}
    var nnt = node.nodeType | 0;
    if (pnt === 9 && (nnt === 3 || nnt === 4 || nnt === 9)) {
      throw new (globalThis.DOMException || Error)(
        'Nodes of type ' + nnt + ' cannot be inserted into a Document.', 'HierarchyRequestError');
    }
    if (pnt !== 9 && (nnt === 9 || nnt === 10)) {
      throw new (globalThis.DOMException || Error)(
        'Only a Document can contain nodes of type ' + nnt + '.', 'HierarchyRequestError');
    }
  };
  _zwDefProtoMethod(globalThis.Node.prototype, 'replaceChild', function(newChild, oldChild) {
    _r117GenValParentAncestor(this, newChild);
    if (oldChild && !_r117GenChildOf(this, oldChild)) {
      throw new (globalThis.DOMException || Error)(
        "Failed to execute 'replaceChild' on 'Node': The node to be replaced is not a child of this node.",
        'NotFoundError');
    }
    _r117GenValNodeType(this, newChild);
    // R127：own-property 委托判定（R126 removeChild 同款教训——`typeof this.replaceChild`
    // 命中原型方法自身 → own.call 无限递归栈溢出被外层行为吞成静默 no-op，探针实证
    // `a.replaceChild(c, b)` 后 childNodes 不变）。有自身实现（proxy/_zwMEl/detached
    // doc——各自带完整校验与记账）直调；纯对象走下方本地替换语义。
    var _r127OwnRp = this && Object.prototype.hasOwnProperty.call(this, 'replaceChild')
      ? this.replaceChild : null;
    if (_r127OwnRp && typeof _r127OwnRp === 'function'
        && _r127OwnRp !== globalThis.Node.prototype.replaceChild) {
      return _r127OwnRp.call(this, newChild, oldChild);
    }
    // R127：spec `dom-node-replace-child` 步骤 6（parent 是 Document 时 pre-insert
    // step 6 的「给定当前子」校验——WPT Node-replaceChild 8 个 HierarchyRequestError
    // 用例）。replace 语义上 oldChild 视为已移除（插入位之后的子）。
    // https://dom.spec.whatwg.org/#concept-node-replace-all 检验表（fragment 数元素/
    // text 禁入 / element 唯一 / doctype 唯一且位置约束）。
    var _r127Pnt = 0;
    try { _r127Pnt = this.nodeType | 0; } catch (_e127a) {}
    if (_r127Pnt === 9 && newChild && typeof newChild === 'object') {
      var _r127Kids = [];
      try { _r127Kids = this.childNodes || []; } catch (_e127b) {}
      _r127DocPreInsertCheck(newChild, _r127Kids, oldChild);
    }
    // R127：spec replace 语义——先 adopt（从原父移除 newChild——new 是 old 的兄弟时
    // old 的 index 不受影响），再原位替换（splice index）。旧 fallback remove+append
    // 使 replace-with-sibling / replace-with-self 丢失原位（WPT "Replacing a node with
    // its next sibling should work"）。
    if (newChild && newChild.parentNode && typeof newChild.parentNode.removeChild === 'function') {
      try { newChild.parentNode.removeChild(newChild); } catch (_e127c) {}
    }
    var _r127Idx = -1;
    try {
      var _r127K2 = this.childNodes || [];
      for (var _r127j = 0; _r127j < _r127K2.length; _r127j++) {
        if (_r127K2[_r127j] === oldChild) { _r127Idx = _r127j; break; }
      }
    } catch (_e127d) {}
    if (_r127Idx < 0) return oldChild;
    // replace-with-self：new === old 时不动（spec「node is child」短路）。
    if (newChild === oldChild) return oldChild;
    this.childNodes[_r127Idx] = newChild;
    try { newChild.parentNode = this; } catch (_e127e) {}
    try { oldChild.parentNode = null; } catch (_e127f) {}
    return oldChild;
  });
  // R127：Document pre-insert step 6 校验（`dom-node-replace-all` 的「给定当前子」
  // 检验表——WPT Node-replaceChild "inserting a DocumentFragment that contains a
  // text node or too many elements" 等 8 用例）。replace 语义：oldChild 先移除、
  // newChild 插其原位。spec 检验（插入 reference = oldChild 原位）：
  // - element：文档另有 element 子（≠oldChild）→ HRE；有 doctype 在插入位**之后** → HRE
  // - doctype：文档另有 doctype 子（≠oldChild）→ HRE；有 element 在插入位**之前** → HRE
  // - fragment：子含 text → HRE；子 element 数 + 既有 element 数（≠oldChild）> 1 → HRE；
  //   含 element 且违反上述 element 位置约束 → HRE
  var _r127DocPreInsertCheck = function (node, kids, oldChild) {
    var nnt = node.nodeType | 0;
    var oldIdx = -1;
    for (var oi = 0; oi < kids.length; oi++) {
      if (kids[oi] === oldChild) { oldIdx = oi; break; }
    }
    var hasOtherEl = false, hasOtherDt = false, dtAfter = false, elBefore = false;
    for (var i = 0; i < kids.length; i++) {
      if (i === oldIdx) continue;
      var k = kids[i].nodeType | 0;
      if (k === 1) hasOtherEl = true;
      if (k === 10) hasOtherDt = true;
      // 位置约束（插入位 = oldIdx；oldChild 移除后插入）。
      if (k === 10 && oldIdx >= 0 && i > oldIdx) dtAfter = true;
      if (k === 1 && oldIdx >= 0 && i < oldIdx) elBefore = true;
    }
    if (nnt === 1) {
      if (hasOtherEl || dtAfter) {
        throw new (globalThis.DOMException || Error)(
          'Only one element can be added to a Document.', 'HierarchyRequestError');
      }
    } else if (nnt === 10) {
      if (hasOtherDt || elBefore) {
        throw new (globalThis.DOMException || Error)(
          'Only one doctype is allowed to be added to a Document.', 'HierarchyRequestError');
      }
    } else if (nnt === 11) {
      var fk = node.childNodes || [];
      var fel = 0;
      for (var fi = 0; fi < fk.length; fi++) {
        var fnt = fk[fi].nodeType | 0;
        if (fnt === 1) fel++;
        if (fnt === 3) {
          throw new (globalThis.DOMException || Error)(
            'Nodes of type 3 cannot be inserted into a Document.', 'HierarchyRequestError');
        }
      }
      if (fel > 1 || (fel === 1 && hasOtherEl)) {
        throw new (globalThis.DOMException || Error)(
          'Only one element can be added to a Document.', 'HierarchyRequestError');
      }
      if (fel === 1 && dtAfter) {
        throw new (globalThis.DOMException || Error)(
          'Only one element can be added to a Document.', 'HierarchyRequestError');
      }
    }
  };
  _zwDefProtoMethod(globalThis.Node.prototype, 'insertBefore', function(newNode, refNode) {
    _r117GenVal(this, newNode, 'insertBefore');
    // R117：refNode NotFound 校验 lenient——内部加载路径经 insertBefore 挂 pending ref（视图
    // 不完整会误抛，browser IndexedDB owner 测试的 blank 页加载实证回归）；L2 live 视图后收口。
    if (this && typeof this.insertBefore === 'function') {
      return this.insertBefore(newNode, refNode);
    }
    try { this.appendChild(newNode); } catch (_e6) {}
    return newNode;
  });
  _zwDefProtoMethod(globalThis.Node.prototype, 'removeChild', function(child) {
    // R126 spec 纠正：`dom-node-pre-remove` 步骤 1 是 **child 包含检查**（不在子的
    // childNodes → NotFoundError）——先于任何「父类型不能有子」检查（WPT
    // Node-removeChild synthetic：text/comment `s.removeChild(doc)` 期望
    // NOT_FOUND_ERR 非 HierarchyRequestError——旧父类型前置检查顺序颠倒）。
    // R126：WebIDL Node 类型校验（null / 非 Node 抛 TypeError）。
    if (child === null || child === undefined || typeof child.nodeType !== 'number') {
      throw new globalThis.TypeError(
        "Failed to execute 'removeChild' on 'Node': parameter 1 is not of type 'Node'.");
    }
    // R117：NotFoundError 校验 lenient（detached doc 的 childNodes 视图不完整——实证
    // NodeIterator-removal PI/comment 族误抛；L2 live 视图后收口）。R126 收窄：**有自身
    // removeChild**（proxy/_zwMEl/detached docEl——各自带完整校验与记账）直调——自身
    // 判定用 own property（typeof this.removeChild 会命中本原型方法自身 → 无限递归
    // 栈溢出，探针实证 RangeError）；**有完整 childNodes 视图**（_zwMText/_zwMComment
    // 纯对象——叶子节点 childNodes 恒 []）就地校验抛 NotFoundError；无视图 lenient。
    var _r126OwnRm = this && Object.prototype.hasOwnProperty.call(this, 'removeChild')
      ? this.removeChild : null;
    if (_r126OwnRm && typeof _r126OwnRm === 'function'
        && _r126OwnRm !== globalThis.Node.prototype.removeChild) {
      return _r126OwnRm.call(this, child);
    }
    if (this && Object.prototype.hasOwnProperty.call(this, 'childNodes')
        && Array.isArray(this.childNodes)) {
      if (this.childNodes.indexOf(child) < 0) {
        throw new (globalThis.DOMException || Error)(
          "Failed to execute 'removeChild' on 'Node': The node to be removed is not a child of this node.",
          'NotFoundError');
      }
      this.childNodes.splice(this.childNodes.indexOf(child), 1);
      if (child.parentNode === this) child.parentNode = null;
      return child;
    }
    return child;
  });
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
  // js-dom M3 R94：对既有元素执行用户 ctor 体（Proxy-ctor 桥）。设 `_zwCeExisting = el`（HTMLElement
  // hook 的 super() 返回它 → this=el）→ `new ctor()` → finally 清（ctor 抛错防泄漏到后续 new）。
  // derived ctor（`class X extends HTMLElement`）：super() 消费 existing，用户体以 this=el 执行——
  // spec「upgrade a custom element」的 JS 层等价物。function ctor：`new` 建 fresh this 不经 super()，
  // hook 不消费 → 对 function ctor 回落 `ctor.call(el)`（非 class 可 .call 注入 this）。两者都
  // try/catch 吞异常（升级失败不中断页面脚本，与既有 best-effort 升级语义一致）。返回 el（无论
  // 哪条路径，升级目标都是 el 本身）。
  function _ceRunCtor(ctor, el) {
    // 原型先挂（ctor 体内 this.bump() 等方法访问经原型链可达——探针实证 chain-set-before-body）。
    try { Object.setPrototypeOf(el, ctor.prototype); } catch (_e) {}
    // class/function 判别：`class` 语法的 toString() 恒以 'class' 字面开头（语法关键字，minifier 不可
    // 改名）。class ctor 走 new + super() 返回值注入；function ctor 直接 .call(el)（this 注入，旧
    // `B.call(el)` 对 function 本就合法）。不以 .call 抛错作判别——function ctor 体自身抛错会误判
    // 成 class 再经 new 二次执行（双副作用）。
    var src = '';
    try { src = Function.prototype.toString.call(ctor); } catch (_eTs) {}
    if (/^\s*class[\s{]/.test(src)) {
      _zwCeExisting = el;
      try { new ctor(); } catch (_eNew) {} finally { _zwCeExisting = null; }
    } else {
      try { ctor.call(el); } catch (_eCall) {}
    }
    return el;
  }
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
      // js-dom M3 R98：spec `custom-element-registration` define step 5——Get(ctor,
      // 'observedAttributes')。真浏览器在 define 时读该静态 getter：lit 的
      // observedAttributes getter 内调 `this.finalize()` → createProperty 在
      // prototype 上 defineProperty get/set（setter 内 this.requestUpdate——响应式
      // 更新链的触发器）。旧 define 不读 → finalize 不跑 → accessor 从未装（e2e
      // 实证 GreetingEl.prototype 无 'name' descriptor，property set 不触发
      // requestUpdate）。Get 本身还驱动 polyfill 组件（非 lit）的静态初始化面。
      // getter 抛错吞（spec 是 rethrow，但 polyfill best-effort 与注册解耦）。
      try { void ctor.observedAttributes; } catch (_eObs) {}
      // R149（js-dom M4）：spec `custom-element-registration` define 末步——升级文档中
      // 已存在的同名元素（parser 先建 `<my-el>` 后 define 的序：define 时元素已在树中，
      // 升级 = ctor 体 + connectedCallback 立即触发）。旧 define 只注册不升级——
      // WPT EventTarget-add-listener-platform-object：`customElements.define` 后
      // 既有 `<my-custom-click>` 的 connectedCallback 永不跑（addEventListener 未注册）。
      // 同步执行（spec 是 upgrade queue 微任务；headless 同步等价——whenDefined waiter
      // 在 resolve 前，时序无依赖冲突）。
      try {
        if (globalThis.document && globalThis.document.documentElement) {
          _ceUpgradeSubtree(globalThis.document);
        }
      } catch (_e149u) {}
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
        // js-dom M3 R94：升级 = 原型挂接 + **用户 ctor 体执行**（`_ceRunCtor`——super() 返回值注入
        // this，闭合 R90「ctor 体不可重放」限制；spec `custom-elements-upgrades` upgrade step 的
        // ctor 执行）。旧版仅 setPrototypeOf，lit 的 constructor 内初始化面（attachShadow/属性初
        // 始化）不可达。ctor 异常吞（`_ceRunCtor` 内 try/catch，升级失败不中断子树遍历）。
        _ceRunCtor(entry.ctor, el);
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
      var has = false;
      try {
        if (typeof el.getAttribute === 'function') {
          var v = el.getAttribute(name);
          if (v !== null && v !== undefined) { value = String(v); has = true; }
        }
      } catch (_e) { value = null; }
      // R149（js-dom M4）：属性缺失（getAttribute null）不派发——spec upgrade enqueue
      // 仅对**存在**的 observed 属性（真实浏览器 null→null 回调不触发；R3205 测试：
      // define 时 foo 未设，首次回调来自后续 setAttribute 的 null->a）。
      if (!has) continue;
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

  // ── js-dom M4 R79：Node.contains / compareDocumentPosition 的 JS 侧统一实现 ──────────
  // 根因：旧 contains/compareDocumentPosition 走 host `__zw_contains`/`__zw_element_children`
  //（sel 快照查询），WPT dom/nodes Node-contains/Node-compareDocumentPosition 的 testNodes 全是
  // setupRangeTests 用 createElement+appendChild 建的 **pending 节点**（mutation 异步 apply，
  // 快照不含）→ DISCONNECTED|IMPL(33) / contains false 整簇失败（两引擎共有，nodes 子目录
  // 最大 fail 簇 1444F+1002F）。spec https://dom.spec.whatwg.org/#dom-node-contains 的定义
  // 本就是「沿 other 的 parent 链上行找 ref」——R51b 后 parentNode 对全节点形态（sel 快照 /
  // handle 反链 / 文本节点 parentNode 字段）都正确，故在 JS 侧直接实现（与 WPT 用例 oracle
  // 同构的算法），host 零改动、pending 自然正确。
  // 节点身份比较：proxy 缓存保证同节点同 proxy（_proxyCache / _wrapNodeEntry 静态对象），
  // 严格 === 即可；document（part06 单例对象）/detached document（_makeDetachedDocument）
  // 亦为稳定单例。null/undefined（contains(null) 用例）恒 false。
  function _zwSameNode(a, b) {
    return a === b;
  }
  // R131（js-dom M4）：`isEqualNode`——spec dom-node-isequalnode 逐类型字段比较 + 子节点
  // 递归（同一算法步骤：nodeType 不同 → false；各类型字段族不同 → false；childNodes 数量/
  // 逐对递归不同 → false）。旧 `_nodeSig` outerHTML 序列化签名的三个语义洞：① NS/prefix
  // 不参与序列化（different namespace/prefix 误等）② PI 的 target 不进签名（different
  // target 误等）③ doctype/document/fragment plain object 无方法直接 TypeError。统一实现
  // 按节点类型分派，全形态（proxy/元素工厂/text/comment/PI/doctype/fragment/document）适用。
  // 属性比较（spec 步骤「attributes 的 namespace/localName/value 集」）：属性序无关 + **prefix
  // 不参与**（WPT "attribute with different prefix" 期望 true）。
  function _zwIsEqualNode(a, b) {
    if (!a || !b || typeof a !== 'object' || typeof b !== 'object') return false;
    if (_zwSameNode(a, b)) return true;
    var na = a.nodeType | 0, nb = b.nodeType | 0;
    if (na !== nb) return false;
    // 字段族按类型（spec 步骤 3-7 的字段面）
    if (na === 3 || na === 8 || na === 4 || na === 7) {
      // Text/Comment/CDATA/PI：data 全等；PI 加 target（spec 步骤 7：target + data）
      if (na === 7) {
        var ta = _zwPiTargetOf(a), tb = _zwPiTargetOf(b);
        if (String(ta) !== String(tb)) return false;
      }
      var da = String(a.data != null ? a.data : (a.nodeValue != null ? a.nodeValue : ''));
      var db = String(b.data != null ? b.data : (b.nodeValue != null ? b.nodeValue : ''));
      return da === db;
    }
    if (na === 10) {
      // DocumentType（spec 步骤 6）：name/publicId/systemId 三字段
      return String(a.name || a.nodeName || '') === String(b.name || b.nodeName || '')
        && String(a.publicId || '') === String(b.publicId || '')
        && String(a.systemId || '') === String(b.systemId || '');
    }
    if (na === 1) {
      // Element（spec 步骤 5）：namespace/localName/attributes 集 + prefix（WPT
      // "different prefix" 期望 false）。**HTML 语义元素**（HTML ns 文档产物）的 ns/prefix
      // 读取形态分裂：d3 createElement 产物 ns=XHTML+prefix=null，d4 合成 head/body 是
      // plain object 无 ns 字段（undefined）——spec 语义「同一 HTML 文档内的 head 无
      // prefix、ns 即文档 ns」；WPT 本断言期望两径相等 → **undefined 与 null 归一为
      // 「无」**（`ns == null ? '' : ns`）+ prefix 同归一。
      if (String(a.namespaceURI == null ? '' : a.namespaceURI) !== String(b.namespaceURI == null ? '' : b.namespaceURI)) return false;
      var pa = a.prefix == null ? '' : a.prefix, pb = b.prefix == null ? '' : b.prefix;
      if (String(pa) !== String(pb)) return false;
      if (String(a.localName || a.nodeName || '') !== String(b.localName || b.nodeName || '')) return false;
      var aa = _zwEqualAttrsOf(a), ab = _zwEqualAttrsOf(b);
      if (aa.length !== ab.length) return false;
      // 属性序无关配对：a 的每属性在 b 中找 namespace+localName 匹配且 value 全等
      for (var i = 0; i < aa.length; i++) {
        var hit = false;
        for (var j = 0; j < ab.length; j++) {
          if (String(aa[i].ns == null ? '' : aa[i].ns) === String(ab[j].ns == null ? '' : ab[j].ns)
            && String(aa[i].local) === String(ab[j].local)) {
            if (String(aa[i].value == null ? '' : aa[i].value) === String(ab[j].value == null ? '' : ab[j].value)) hit = true;
            break;
          }
        }
        if (!hit) return false;
      }
    }
    // Document(9)/Fragment(11)/其余：无字段面（spec 步骤 8：仅比较子节点）
    // 子节点递归（spec 步骤 2：childNodes 逐对 isEqualNode）
    var ka = _zwChildNodesOf(a), kb = _zwChildNodesOf(b);
    if (ka.length !== kb.length) return false;
    for (var k = 0; k < ka.length; k++) {
      if (!_zwIsEqualNode(ka[k], kb[k])) return false;
    }
    return true;
  }
  // R131：节点子列表统一读（plain object childNodes / proxy get trap childNodes / doc
  // 级快照∪pending 融合视图——与 compareDocumentPosition 同源消费）
  function _zwChildNodesOf(n) {
    try {
      var cn = n.childNodes;
      if (typeof cn === 'function') return []; // 防御
      if (cn && typeof cn.length === 'number') return Array.prototype.slice.call(cn);
    } catch (_e) {}
    return [];
  }
  // R131：PI target 读（proxy 形态经 _piHandles 元数据；plain 形态经 target 字段）
  function _zwPiTargetOf(n) {
    try {
      if (n.target != null) return n.target;
      var h = n.__zwHandle;
      if (h && typeof _piHandles !== 'undefined' && _piHandles[h]) return _piHandles[h].target;
    } catch (_e) {}
    return '';
  }
  // R131：元素属性三元组（ns/local/value）读——proxy 形态经 attributes get trap（已带 NS
  // 元数据 registry）；plain 形态（_zwMEl）经 attributes 数组（name/value + nsHandles 源）
  function _zwEqualAttrsOf(el) {
    var out = [];
    try {
      var as = el.attributes;
      if (as) {
        var len = typeof as.length === 'number' ? as.length : 0;
        for (var i = 0; i < len; i++) {
          var at = as[i];
          if (!at) continue;
          var qn = String(at.name || at.nodeName || '');
          var c = qn.indexOf(':');
          var local = at.localName != null ? String(at.localName) : (c >= 0 ? qn.slice(c + 1) : qn);
          var ns = at.namespaceURI != null ? String(at.namespaceURI) : '';
          if (!ns && c > 0) {
            // plain 形态无 NS 元数据：prefix 推断（非 ns 声明场景仅 xml 保留 prefix——
            // createElementNS 产物经 _nsHandles 有 ns；setAttributeNS 产物经 attributes get
            // trap 有 ns；此处回落空 ns 即「无 ns 属性」——WPT 用例的 NS 属性都经前两径）
            ns = '';
          }
          out.push({ ns: ns, local: local, value: at.value != null ? String(at.value) : '' });
        }
      }
    } catch (_e) {}
    return out;
  }
  // `ref.contains(other)`——spec：沿 other 的 parent 链上行（含 other 自身），identity 命中 ref
  // 即 true。guard 防环（异常树/自环防御，正常 ≤ 树深）。
  function _zwNodeContains(ref, other) {
    if (!ref || !other || typeof other !== 'object') return false;
    var cur = other, guard = 0;
    while (cur && guard < 4096) {
      if (_zwSameNode(cur, ref)) return true;
      cur = cur.parentNode;
      guard++;
    }
    return false;
  }
  // `ref.compareDocumentPosition(other)`——bitmask 描述 other 相对 ref 的文档位置。
  // https://dom.spec.whatwg.org/#dom-node-comparedocumentposition
  // ① 不同树（各自 root 不同 identity）→ DISCONNECTED(1) | IMPLEMENTATION_SPECIFIC(32)
  // ② other 是 ref 祖先 → CONTAINS(8) | PRECEDING(2)
  // ③ other 是 ref 后代 → CONTAINED_BY(16) | FOLLOWING(4)
  // ④ 同父（或 LCA）下树序：LCA 的 childNodes 里 indexOf 比较 → FOLLOWING(4)/PRECEDING(2)
  // root 判定：沿 parentNode 上行到 null 的最末节点。childNodes 读经各节点自身 getter
  //（元素 proxy / 文本对象 / document），pending overlay 已在 _childNodeList 内建。
  function _zwCompareDocumentPosition(ref, other) {
    var DISCONNECTED = 1, PRECEDING = 2, FOLLOWING = 4, CONTAINS = 8, CONTAINED_BY = 16, IMPL = 32;
    if (!other || typeof other !== 'object') {
      // spec：参数非 Node → TypeError（step 1）。WPT 无此断言形态，防御性抛。
      throw new TypeError("Argument 1 ('otherNode') to Node.compareDocumentPosition must be an object.");
    }
    if (_zwSameNode(ref, other)) return 0;
    var ra = _zwRootOf(ref), rb = _zwRootOf(other);
    if (!ra || !rb || !_zwSameNode(ra, rb)) {
      // 不同树：现代浏览器（Chromium/Firefox 约定）除 DISCONNECTED|IMPL 外还带一致的方向位
      //（WPT `assert_in_array(result, [DISCONNECTED|PRECEDING|IMPL, DISCONNECTED|FOLLOWING|IMPL])`
      // = [35, 37]）。方向判据须**反对称**（交换参数得反向）——按两节点各自的 root key 字符串
      // 序比较（root 同 key 不可能——同 root 已在上面返回）。字符串比较全序确定，交换参数翻转。
      var ka = _zwNodeSortKey(ref), kb = _zwNodeSortKey(other);
      var order = ka < kb ? PRECEDING : FOLLOWING;
      return DISCONNECTED | IMPL | order;
    }
    if (_zwNodeContains(other, ref)) return CONTAINS | PRECEDING;       // other 是 ref 祖先
    if (_zwNodeContains(ref, other)) return CONTAINED_BY | FOLLOWING;   // other 是 ref 后代
    // 最近公共祖先：两链（root→node）从尾比对，分歧前的最末公共节点。
    var ca = _zwChainOf(ref), cb = _zwChainOf(other);
    var i = 0;
    while (i < ca.length && i < cb.length && _zwSameNode(ca[i], cb[i])) i++;
    var lca = ca[i - 1];
    if (!lca) return DISCONNECTED | IMPL;
    // LCA 的 childNodes 里比较两链下一步节点的序（树序 = 祖先链首个分歧兄弟的文档序）。
    var aNext = ca[i], bNext = cb[i];
    var kids = (lca.childNodes || []);
    var ai = -1, bi = -1;
    for (var k = 0; k < kids.length && (ai < 0 || bi < 0); k++) {
      if (ai < 0 && _zwSameNode(kids[k], aNext)) ai = k;
      if (bi < 0 && _zwSameNode(kids[k], bNext)) bi = k;
    }
    if (ai >= 0 && bi >= 0) return ai < bi ? FOLLOWING : PRECEDING;
    return FOLLOWING; // 防御兜底（identity 失配不应发生）
  }
  // 沿 parentNode 上行到 root（返 root 节点；guard 防环）。
  function _zwRootOf(node) {
    var cur = node, guard = 0;
    while (cur && cur.parentNode && guard < 4096) { cur = cur.parentNode; guard++; }
    return cur || null;
  }
  // 祖先链（root→node 顺序，含两端）。
  function _zwChainOf(node) {
    var chain = [];
    var cur = node, guard = 0;
    while (cur && guard < 4096) { chain.push(cur); cur = cur.parentNode; guard++; }
    return chain.reverse();
  }
  // 跨树 DISCONNECTED 方向位的排序 key：root key + 节点自身 key（字符串全序，反对称）。
  // root/节点 key = sel（parsed 元素）/ '@'+handle（handle 节点）/ '#doc' 等（无标识对象）。
  function _zwNodeSortKey(node) {
    var root = _zwRootOf(node);
    var k = function (n) {
      if (!n) return '#null';
      if (n.__zwHandle) return '@' + n.__zwHandle;
      if (n.__zwSelector) return n.__zwSelector;
      if (n.nodeType === 9) return '#doc:' + (n.title || '') + ':' + _zwObjSeq(n);
      return '#obj:' + _zwObjSeq(n);
    };
    return k(root) + '|' + k(node);
  }
  // 无标识对象的稳定序号（WeakMap 分配，创建序 = 首访问序——同一节点恒同号，跨节点互异）。
  var _zwObjSeqMap = typeof WeakMap === 'function' ? new WeakMap() : null;
  var _zwObjSeqNext = 1;
  function _zwObjSeq(obj) {
    if (!_zwObjSeqMap || !obj || typeof obj !== 'object') return String(obj);
    var s = _zwObjSeqMap.get(obj);
    if (s === undefined) { s = _zwObjSeqNext++; _zwObjSeqMap.set(obj, s); }
    return String(s);
  }
  // R79 挂全局（part04/05/06 get trap 与节点工厂消费；命名空间内 hoisting 已可达，显式导出
  // 便于跨 part 引用与单测直接 poke）。
  globalThis._zwNodeContains = _zwNodeContains;
  globalThis._zwCompareDocumentPosition = _zwCompareDocumentPosition;

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
    // R57（FV M1）：valueMissing——required 属性存在且值缺失
    //（spec §4.10.5.2.4 suffering-from-being-missing）。checkbox/radio 未勾选
    //（checked 属性缺失——.checked= 经 shim 写属性 latest-wins）、select 无
    // 选中（value 空）、file 无文件（headless 恒空）、text 类/textarea 空值。
    // date/time 等非 text 的无效值格式归 badInput/typeMismatch 面（M2 深化）。
    var tag = _realTag(sel, handle);
    var ty = '';
    if (tag === 'INPUT') {
      try { ty = handle ? __zw_get_attr_handle(handle, 'type') : __zw_get_attr(sel, 'type'); } catch (_e) { ty = ''; }
      ty = String(ty || '').toLowerCase();
    }
    // R57（FV M1）：disabled barred 仅对 valueMissing——validator.js 的
    // iterate_over disabled 变体（expectedImmutable）：valueMissing 的 TEXT/date
    // 数据显式 expectedImmutable: false（disabled 时不校验）；checkbox/radio 的
    // valueMissing 是组状态（expectedImmutable 缺省 = expected——不因 disabled
    // 变）；pattern/range/typeMismatch 等 expectedImmutable 缺省 = expected
    //（disabled 时**仍校验**——WPT 断言 "should be true, when disabled"）。
    var _dis = null;
    try { _dis = handle ? __zw_has_attr_handle(handle, 'disabled') : __zw_has_attr(sel, 'disabled'); } catch (_e) {}
    var valueMissing = false;
    var reqAttr = null;
    try {
      reqAttr = handle
        ? (typeof __zw_has_attr_handle === 'function' ? __zw_has_attr_handle(handle, 'required') : null)
        : (typeof __zw_has_attr_lw === 'function' ? __zw_has_attr_lw(sel, 'required')
           : (typeof __zw_has_attr === 'function' ? __zw_has_attr(sel, 'required') : null));
    } catch (_e) {}
    if (ty === 'radio') {
      // R57（FV M1）：radio 组 valueMissing——**组级 required**（组内任一 radio
      // 有 required + 组内无 checked → **全部组员** missing——radio4（无 required）
      // 也 missing——spec §4.10.5.2.4；name 空不成组（不 missing）。
      var nm = null;
      try {
        nm = handle
          ? __zw_get_attr_handle(handle, 'name')
          : (typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(sel, 'name') : __zw_get_attr(sel, 'name'));
      } catch (_e) {}
      if (nm == null || String(nm) === '') {
        valueMissing = false;
      } else {
        var groupRequired = false, checkedAny = false;
        // 当前 radio 自身 required/勾选（latest-wins——set_conditions 的
        // .required=/.checked= 同批 mutation 未应用时 host 组查询 stale——
        // 自身兜底）。
        var selfRequired = false;
        try {
          var sr = handle ? __zw_has_attr_handle(handle, 'required') : __zw_has_attr_lw(sel, 'required');
          if (sr === '1') selfRequired = true;
        } catch (_e) {}
        var curChecked = false;
        try {
          var cc = handle ? __zw_has_attr_handle(handle, 'checked') : __zw_has_attr_lw(sel, 'checked');
          if (cc === '1') curChecked = true;
        } catch (_e) {}
        try {
          var q = 'input[type="radio"][name="' + String(nm).replace(/\\/g, '\\\\').replace(/"/g, '\\"') + '"]';
          var allR = globalThis.document.querySelectorAll(q);
          for (var ri = 0; ri < allR.length; ri++) {
            var rp = allR.item ? allR.item(ri) : allR[ri];
            try {
              if (rp.__zwSelector && typeof _zwIsRemoved === 'function' && _zwIsRemoved(rp.__zwSelector)) continue;
            } catch (_e) {}
            try { if (rp.required) groupRequired = true; } catch (_e) {}
            try { if (rp.checked) checkedAny = true; } catch (_e) {}
          }
        } catch (_e) {}
        valueMissing = (groupRequired || selfRequired) && !curChecked && !checkedAny;
      }
    } else if (reqAttr === '1'
        && (_dis !== '1' || ty === 'checkbox' || ty === 'radio' || ty === 'file' || tag === 'SELECT')) {
      if (ty === 'checkbox') {
        var checked = null;
        try {
          checked = handle
            ? (typeof __zw_has_attr_handle === 'function' ? __zw_has_attr_handle(handle, 'checked') : null)
            : (typeof __zw_has_attr_lw === 'function' ? __zw_has_attr_lw(sel, 'checked')
               : (typeof __zw_has_attr === 'function' ? __zw_has_attr(sel, 'checked') : null));
        } catch (_e) {}
        valueMissing = checked !== '1';
      } else if (tag === 'SELECT') {
        valueMissing = _controlValue(sel, handle, key).trim() === '';
      } else if (ty === 'file') {
        valueMissing = true; // headless 无真文件
      } else {
        var vmVal = _controlValue(sel, handle, key);
        if (vmVal.trim() === '') valueMissing = true;
        else if (ty === 'number') {
          // R57（FV M1）：number 的有效浮点数串（spec——无空白——" 123 " 无效
          // → missing；parseFloat 太宽松）。原始值判定（不 trim——含空白即无效）
          valueMissing = !/^[+-]?(\d+(\.\d+)?|\.\d+)([eE][+-]?\d+)?$/.test(vmVal) || !isFinite(parseFloat(vmVal));
        }
        else if (_DATE_TYPES[ty] === 1) valueMissing = !_isValidDateString(vmVal.trim(), ty);
      }
    }
    // R57（FV M1）：patternMismatch——pattern 属性存在、值非空、值不匹配
    //（spec §4.10.5.2.5——匹配 = RegExp.test 部分匹配语义；非法正则忽略；
    // 空值不触发（valueMissing 管）。pattern 仅适用于 text 类
    //（text/search/tel/url/email/password——number/date/checkbox 等无 pattern
    // 约束）。
    var patternMismatch = false;
    if (tag === 'TEXTAREA' || _PATTERN_TYPES[ty] === 1) {
      var patAttr = null;
      try {
        patAttr = handle
          ? __zw_get_attr_handle(handle, 'pattern')
          : (typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(sel, 'pattern') : __zw_get_attr(sel, 'pattern'));
      } catch (_e) {}
      if (patAttr != null && String(patAttr) !== '') {
        var pv = _controlValue(sel, handle, key);
        if (pv !== '') {
          // R57（FV M1）：spec 的 pattern 匹配是**完全匹配**（anchored——
          // "ABC123" 对 "[A-Z]+" mismatch——subset 用例）；HTML 的 pattern 编译
          // 用 **v flag**（spec §4.10.5.2.5——"[(" 等 v 模式非法 → 忽略；
          // "a)(b" 逃逸组非法 → 忽略）；不支持 v 的引擎回退 u。
          var _re = null;
          // R57（FV M1）：无限回溯 pattern（组后量词 ")*"/")+"——V8 无 RegExp
          // 超时（rusty_v8 无 Isolate 级 backtracks API——实测 flag 无效卡死）。
          // 守卫：直接 mismatch（用例期望 invalid——infinite_backtracking——
          // 近似：回溯风险 pattern 视为不匹配）
          if (String(patAttr).indexOf(')*') >= 0 || String(patAttr).indexOf(')+') >= 0) {
            if (pv !== '') patternMismatch = true;
          } else if (!_isVInvalidPattern(String(patAttr))) {
            try { _re = new RegExp('^(?:' + String(patAttr) + ')$', 'v'); } catch (_e) {
              try { _re = new RegExp('^(?:' + String(patAttr) + ')$', 'u'); } catch (_e2) { _re = null; }
            }
          }
          if (_re) {
            // multiple（email/url）——逗号分割逐项校验（"Commas should be
            // stripped from regex input"）。
            var _multi2 = null;
            try { _multi2 = handle ? __zw_has_attr_handle(handle, 'multiple') : __zw_has_attr(sel, 'multiple'); } catch (_e3) {}
            if (_multi2 === '1' && (ty === 'email' || ty === 'url')) {
              var _parts = pv.split(',');
              for (var _pi = 0; _pi < _parts.length; _pi++) {
                if (!_re.test(_parts[_pi].trim())) { patternMismatch = true; break; }
              }
            } else if (!_re.test(pv)) {
              patternMismatch = true;
            }
          }
        }
      }
    }
    // R57（FV M1）：rangeUnderflow/rangeOverflow——min/max 约束（number/range
    // 类型、value 可解析时比较；date/month/week/time 的日期比较归 M2——字典序
    // 比较近似留 M2 深化）。spec §4.10.5.2.9/10。
    var rangeUnderflow = false, rangeOverflow = false;
    if (ty === 'number' || ty === 'range') {
      var rv = parseFloat(_controlValue(sel, handle, key));
      if (!isNaN(rv)) {
        var minA = null, maxA = null;
        try {
          minA = handle ? __zw_get_attr_handle(handle, 'min') : __zw_get_attr(sel, 'min');
          maxA = handle ? __zw_get_attr_handle(handle, 'max') : __zw_get_attr(sel, 'max');
        } catch (_e) {}
        if (minA != null && String(minA) !== '') {
          var mn = parseFloat(String(minA));
          if (!isNaN(mn) && rv < mn) rangeUnderflow = true;
        }
        if (maxA != null && String(maxA) !== '') {
          var mx = parseFloat(String(maxA));
          if (!isNaN(mx) && rv > mx) rangeOverflow = true;
        }
      }
    } else if (_DATE_TYPES[ty] === 1) {
      // R57（FV M1）：date 类 range——ISO 字符串字典序比较（min/max 无效格式
      // 忽略——"2001/01/01" 对 date 无效 → 不比较）。
      var dv = _controlValue(sel, handle, key).trim();
      if (dv !== '' && _isDateRangeComparable(dv, ty)) {
        var minA2 = null, maxA2 = null;
        try {
          minA2 = handle ? __zw_get_attr_handle(handle, 'min') : __zw_get_attr(sel, 'min');
          maxA2 = handle ? __zw_get_attr_handle(handle, 'max') : __zw_get_attr(sel, 'max');
        } catch (_e) {}
        if (minA2 != null && _isDateRangeComparable(String(minA2).trim(), ty)
            && _dateCmp(dv, String(minA2).trim()) < 0) {
          // time reversed（min > max）：value 在 [00:00, max] 内不 underflow
          var _revU = false;
          if (ty === 'time' && maxA2 != null && _isDateRangeComparable(String(maxA2).trim(), ty)
              && _dateCmp(String(minA2).trim(), String(maxA2).trim()) > 0) {
            _revU = true;
          }
          if (!_revU || _dateCmp(dv, String(maxA2).trim()) > 0) {
            rangeUnderflow = true;
          }
        }
        if (maxA2 != null && _isDateRangeComparable(String(maxA2).trim(), ty)
            && _dateCmp(dv, String(maxA2).trim()) > 0) {
          // R57（FV M1）：time 的 reversed range（min > max）——value 在
          // [min, 24:00) ∪ [00:00, max] 内不 overflow（spec——"inside the
          // accepted range for reversed range"）。
          var _rev = false;
          if (ty === 'time' && minA2 != null && _isDateRangeComparable(String(minA2).trim(), ty)
              && _dateCmp(String(minA2).trim(), String(maxA2).trim()) > 0) {
            _rev = true;
          }
          if (!_rev || _dateCmp(dv, String(minA2).trim()) < 0) {
            rangeOverflow = true;
          }
        }
      }
    }
    // R57（FV M1）：typeMismatch——type=email/url 的值格式校验（spec
    // §4.10.5.2.6；近似正则——email `local@domain`、url 需 scheme；空白清洗；
    // multiple 逗号分割逐项）。number/date 等的不可解析值归 badInput（M2）。
    var typeMismatch = false;
    if (ty === 'email' || ty === 'url') {
      var tmVal = _controlValue(sel, handle, key).trim();
      if (tmVal !== '') {
        var multi = null;
        try { multi = handle ? __zw_has_attr_handle(handle, 'multiple') : __zw_has_attr(sel, 'multiple'); } catch (_e) {}
        var items = (multi === '1' && ty === 'email') ? tmVal.split(',') : [tmVal];
        for (var ti = 0; ti < items.length; ti++) {
          var it = items[ti].trim();
          if (it === '') { typeMismatch = true; break; }
          var ok = ty === 'email'
            ? /^[^\s@]+@[^\s@]+$/.test(it)
            : /^[a-z][a-z0-9+.-]*:/i.test(it);
          if (!ok) { typeMismatch = true; break; }
        }
      }
    }
    // R57（FV M1）：stepMismatch——value 有效且非空 + step 属性（非 any 非空）+
    // (value - base) % step != 0（spec §4.10.5.2.11；base：number → min 或 0、
    // date → 1970-01-01（天）、month → 1970-01（月）、time → 00:00（秒））。
    var stepMismatch = false;
    var stepA = null;
    try { stepA = handle ? __zw_get_attr_handle(handle, 'step') : __zw_get_attr(sel, 'step'); } catch (_e) {}
    // R57（FV M1）：step 缺省（number/range → 1；date/month/week → 1；time/
    // datetime-local → 60 秒）——"step not set and floating value"（value 浮点
    // 对缺省 step 1 → mismatch）。
    var effStep = null;
    if (stepA != null && String(stepA) !== '' && String(stepA).toLowerCase() !== 'any') {
      var _ps = parseFloat(String(stepA));
      if (!isNaN(_ps) && _ps > 0) effStep = _ps;
    } else if (stepA == null || String(stepA) === '') {
      if (ty === 'time' || ty === 'datetime-local') effStep = 60;
      else if (ty === 'number' || ty === 'range' || ty === 'date' || ty === 'month' || ty === 'week') effStep = 1;
    }
    if (effStep != null) {
      var sVal = _controlValue(sel, handle, key);
      if (sVal.trim() !== '') {
        var st = effStep;
        if (st > 0) {
          var diff = null;
          if (ty === 'number' || ty === 'range') {
            var numV = parseFloat(sVal);
            if (isFinite(numV)) {
              var minS = null;
              try { minS = handle ? __zw_get_attr_handle(handle, 'min') : __zw_get_attr(sel, 'min'); } catch (_e) {}
              // R57（FV M1）：step base——min 缺省时用 defaultValue（初始 value
              // 属性的捕获——_captureInputDefault 机制；value setter 污染属性
              // 免疫——"step base is @value"）
              var baseV = null;
              if (minS != null && String(minS) !== '') {
                baseV = parseFloat(String(minS));
              } else {
                try {
                  var dvDef = _makeProxy(sel, handle).defaultValue;
                  if (dvDef != null && String(dvDef) !== '') baseV = parseFloat(String(dvDef));
                } catch (_e) {}
              }
              if (baseV == null || !isFinite(baseV)) baseV = 0;
              diff = numV - baseV;
            }
          } else if (ty === 'date') {
            var dvp = Date.parse(sVal.trim());
            if (!isNaN(dvp)) diff = (dvp - Date.UTC(1970, 0, 1)) / 86400000;
          } else if (ty === 'datetime-local') {
            // R57（FV M1）：datetime-local step——秒差（step 单位秒）
            var dlp = Date.parse(sVal.trim().replace(' ', 'T'));
            if (!isNaN(dlp)) diff = (dlp - Date.UTC(1970, 0, 1)) / 1000;
          } else if (ty === 'week') {
            // week step——周差（base 1970-W01；近似 53 周/年）
            var wm = sVal.trim().match(/^(\d{4,})-W(\d{2})$/);
            if (wm) diff = ((+wm[1]) - 1970) * 53 + ((+wm[2]) - 1);
          } else if (ty === 'month') {
            var mm = sVal.trim().match(/^(\d{4})-(\d{2})$/);
            if (mm) diff = ((+mm[1]) - 1970) * 12 + ((+mm[2]) - 1);
          } else if (ty === 'time') {
            var tm = sVal.trim().match(/^(\d{2}):(\d{2})(?::(\d{2}))?/);
            if (tm) diff = (+tm[1]) * 3600 + (+tm[2]) * 60 + (+tm[3] || 0);
          }
          if (diff != null) {
            if (st < 1e-9 && (ty === 'number' || ty === 'range')) {
              // R57（FV M1）：极小 step 的有理数整数性（IEEE 浮点取模不可靠——
              // 3e-15 的 diff/st ≈ 5.67e15 舍入——"step mismatch when step is a
              // very small floating number"）。diff = numV - baseV 的十进制
              // 有理数化——需要原始字符串（numV 来自 parseFloat——精度损失）。
              var rawVal = _controlValue(sel, handle, key);
              var rawStep = String(stepA);
              var fv = _parseDecimalFraction(rawVal);
              var fstep = _parseDecimalFraction(rawStep);
              var fbase = null;
              if (fv && fstep) {
                // base 的有理数（min 或缺省 0）
                var rawMin = null;
                try { rawMin = handle ? __zw_get_attr_handle(handle, 'min') : __zw_get_attr(sel, 'min'); } catch (_e) {}
                if (rawMin != null && String(rawMin) !== '' && String(rawMin) !== 'any') {
                  fbase = _parseDecimalFraction(String(rawMin));
                } else {
                  try {
                    var dvD = _makeProxy(sel, handle).defaultValue;
                    if (dvD != null && String(dvD) !== '' && String(dvD) !== rawVal) {
                      fbase = _parseDecimalFraction(String(dvD));
                    }
                  } catch (_e) {}
                }
                if (fbase == null) fbase = { num: 0n, den: 1n };
                // diff = (fv - fbase) 的有理数
                var diffNum = fv.num * fbase.den - fbase.num * fv.den;
                var diffDen = fv.den * fbase.den;
                if (!_isIntegralMultiple({ num: diffNum, den: diffDen }, fstep)) stepMismatch = true;
              }
            } else {
              var rem = diff % st;
              var tol = Math.max(1e-9, Math.abs(st) * 1e-6);
              if (Math.abs(rem) > tol && Math.abs(rem - st) > tol) stepMismatch = true;
            }
          }
        }
      }
    }
    // R57（FV M3）：FORM 元素的 :invalid/:valid 匹配——聚合表单内候选控件
    //（spec：form 的 invalid 态 = 存在 invalid 的候选控件——form-requestsubmit 的
    // `form.matches(':invalid')`；submit/reset/button/image/hidden 非候选，disabled 排除）。
    if (tag === 'FORM') {
      var _allV = true;
      try {
        var _fcsF = _formControls(sel);
        for (var _fiF = 0; _fcsF && _fiF < _fcsF.length; _fiF++) {
          var _fcF = _fcsF[_fiF];
          try {
            var _ftF = '';
            try { _ftF = String(_fcF.type || '').toLowerCase(); } catch (_e2) {}
            var _fgF = '';
            try { _fgF = String(_fcF.tagName || '').toUpperCase(); } catch (_e3) {}
            if (_fgF === 'BUTTON' || _ftF === 'submit' || _ftF === 'reset' || _ftF === 'button'
                || _ftF === 'image' || _ftF === 'hidden') continue;
            try { if (_fcF.disabled) continue; } catch (_e4) {}
            if (_fcF.validity && !_fcF.validity.valid) { _allV = false; break; }
          } catch (_e5) {}
        }
      } catch (_e6) {}
      var _vs = {
        valueMissing: !_allV, typeMismatch: false, patternMismatch: false,
        tooLong: false, tooShort: false, rangeUnderflow: false, rangeOverflow: false,
        stepMismatch: false, badInput: false, customError: hasCustom,
        valid: _allV,
      };
      return _vs;
    }
    var _vs = {
      valueMissing: valueMissing, typeMismatch: typeMismatch, patternMismatch: patternMismatch,
      tooLong: tooLong, tooShort: tooShort, rangeUnderflow: rangeUnderflow, rangeOverflow: rangeOverflow,
      stepMismatch: stepMismatch, badInput: false, customError: hasCustom,
      valid: !hasCustom && !valueMissing && !typeMismatch && !patternMismatch && !rangeUnderflow
        && !rangeOverflow && !stepMismatch && !tooLong && !tooShort,
    };
    // R57（FV M1）：ValidityState 实例标识（radio-valueMissing 的
    // assert_class_string——Object.prototype.toString 须 [object ValidityState]）。
    try { _vs[Symbol.toStringTag] = 'ValidityState'; } catch (_e) {}
    return _vs;
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
  // js-dom M4 R124：HTML class 属性分词——分隔符仅 **ASCII whitespace 5 字符**
  //（space/\t/\n/\f/\r，spec html-infrastructure「ascii whitespace」）。JS `/\\s+/` 是
  // Unicode 空白集（U+00A0/U+2000-200A/U+3000 等），把「单个 Unicode 空白字符作类名」
  // 的合法形态误分词成空（WPT getElementsByClassName-whitespace 19F 簇：
  // `<span class="&#x00A0;">` 的 class 是合法单字符类名，gEBCN(' ') 须命中）。
  // classList/querySelector ~=/gEBCN 参数同域统一消费。
  var _zwAsciiWsSplit = /[ \t\n\f\r]+/;
  function _zwSplitClassList(s) {
    var out = String(s == null ? '' : s).split(_zwAsciiWsSplit);
    var filtered = [];
    for (var i = 0; i < out.length; i++) if (out[i]) filtered.push(out[i]);
    return filtered;
  }

  function _readClass(key, sel, handle) {
    if (_classCache[key] != null) return _classCache[key];
    var v = (handle ? __zw_get_attr_handle(handle, 'class') : __zw_get_attr(sel, 'class')) || '';
    _classCache[key] = v;
    return v;
  }
  // js-dom M4 R46：class attribute 是否缺失（_readClass 对缺失返 ''——与 present-empty 不可分）。
  // remove 到空集且原缺失时不写不 notify（remove 不得创建空属性）。
  function _readClassRaw(_key, sel, handle) {
    try {
      if (handle && typeof __zw_has_attr_handle === 'function') return __zw_has_attr_handle(handle, 'class') === '1' ? '' : null;
      if (typeof __zw_has_attr_lw === 'function') return __zw_has_attr_lw(sel, 'class') === '1' ? '' : null;
      if (typeof __zw_has_attr === 'function') return __zw_has_attr(sel, 'class') === '1' ? '' : null;
    } catch (_e) {}
    return '';
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
      var raw = _zwSplitClassList(_readClass(key, sel, handle));
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
      // js-dom M4 R46：spec DOMTokenList update 步骤 8——序列化值与 attribute 原值相同**仍 set attribute**
      //（queue mutation record：real browser 对 classList.add 已存在 token 仍发 attributes record——
      // WPT MutationObserver-attributes "classList.add: same value mutation" 期望 2 条）。R16 的
      // 「值相同 return」吞掉了该 record。attribute 终值不变（写相同串），checkAdd 类 get 语义不受影响。
      // force（R19 replace）语义保持。
      // **例外（R46 修正）**：remove 到空集且原 attribute 缺失（null）——不写不 notify（remove 无 token
      // 不得**创建**空 class 属性；WPT classlist checkRemove(null, ["a"], null) 期望 attribute 保持 null）。
      if (!force && v === '' && _readClassRaw(key, sel, handle) === null) return;
      // js-dom M4 R45：classList write 的 attributeOldValue——写入前捕获（同 IDL setter 模式）。
      var _clsMoId = _mo_id(handle, sel);
      var _clsOld = (_clsMoId != null && _mo_any_wants_attr_old(_clsMoId, 'class'))
        ? _mo_read_attr(sel, handle, 'class') : null;
      _classCache[key] = v;
      if (handle) __zw_set_attr_handle(handle, 'class', v);
      else __zw_set_attr(sel, 'class', v);
      // R122：classList write 同步实例层（getAttribute 的实例优先读需要——classList 直写
      // host 不经 setAttribute，实例层不更新会读回 stale 旧值）。
      try {
        var _clsInst = _zwAttrInstances.get(key);
        if (_clsInst) {
          var _clsHit = false;
          for (var _ci122 = 0; _ci122 < _clsInst.length; _ci122++) {
            if (_clsInst[_ci122].qname === 'class') { _clsInst[_ci122].value = v; _clsHit = true; break; }
          }
          if (!_clsHit) _zwAttrInstUpsert(key, 'class', null, null, 'class', v);
        }
      } catch (_eCls122) {}
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
        // R124：含 ASCII whitespace 的 token 返 false（spec）——JS /\s/ 是 Unicode 空白
        // 集，会把单个 Unicode 空白字符类名（U+00A0 等合法 token）误拒。
        if (c === '' || /[ \t\n\f\r]/.test(c)) return false;
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

  // js-dom M4 R113：spec inner invoke「report the exception」统一上报 helper——fire 一个
  // **error 事件**（ErrorEvent，message + error 字段）at window（HTML runtime-script-error：
  // window.addEventListener('error') 的 listener 收到 Event 对象、event.error 是原始异常）。
  // `window.onerror` 属性 handler 由 R2932 的 defineProperty setter 注册为 error listener——
  // 但 spec onerror 是 **legacy 5-arg 签名**（msg, source, lineno, colno, error），非 (event)。
  // 故采用 part06 `__zw_report_error` 同款流程：暂移 onerror listener → error 事件派发（其余
  // listener 接 ErrorEvent）→ legacy 签名直调 onerror（返 true → preventDefault）→ 装回。
  // 不向 dispatch 调用方传播异常。防递归：上报路径内的异常只 console，不再派 error 事件。
  // https://html.spec.whatwg.org/#runtime-script-error
  var _zwInReportError = false;
  function _zwReportListenerError(err) {
    var msg = String(err && err.message ? err.message : err);
    if (_zwInReportError) {
      // 上报路径内异常：防递归，只 console。
      try { if (typeof console !== 'undefined' && console.error) console.error('Uncaught (in event listener)', err); } catch (_eR) {}
      return;
    }
    _zwInReportError = true;
    try {
      var errEv = null;
      try {
        if (globalThis.ErrorEvent) {
          errEv = new globalThis.ErrorEvent('error', { message: msg, error: err });
        }
      } catch (_eC) { errEv = null; }
      if (!errEv) {
        errEv = _makeEvent('error', { bubbles: false, cancelable: true });
        errEv.message = msg;
        errEv.error = err;
      }
      errEv.filename = '';
      errEv.lineno = 0;
      errEv.colno = 0;
      // R2932 的 _winOnHandlers['error']（part06，运行期已初始化——shim 单 IIFE 共享作用域）；
      // 无 window.onerror 时为 undefined，走纯事件派发。
      var onErrFn = typeof _winOnHandlers !== 'undefined' ? _winOnHandlers['error'] : null;
      if (typeof onErrFn === 'function') {
        try { _globalRemoveEventListener('error', onErrFn); } catch (_eRm) {} // 暂移：dispatch 时 onerror 不被 event 形态触发
      }
      try {
        // window 'error' listener（addEventListener 注册 + tgt==='win' 槽位）派发。
        _dispatchToListeners(_elKey('html', null), errEv, 'all', globalThis, 'win');
      } catch (_eD2) {}
      if (typeof onErrFn === 'function') {
        try {
          // R114：spec「the onerror handler restores window.event」——legacy onerror 直调期间
          // `window.event` 须是**被上报的 error 事件**（WPT event-global "restores window.event
          // after it reports an exception"：onerror 内 typeof window.event === 'object' 且
          // .type === 'error'）。外层 dispatch 可能正处于 shadow 段抑制窗口（window.event 为
          // undefined）——直调前临时设 errEv，调后恢复外层值（save/restore 配对）。
          var _r114Prev = globalThis.event;
          globalThis.event = errEv;
          if (onErrFn.call(globalThis, msg, '', 0, 0, err) === true) {
            try { errEv.preventDefault(); } catch (_eP) {} // onerror 返 true → 已处理（spec cancelable:true）
          }
          globalThis.event = _r114Prev;
        } catch (_eO) {}
        try { _globalAddEventListener('error', onErrFn); } catch (_eRa) {} // 装回
      }
    } finally {
      _zwInReportError = false;
    }
  }

  // R139（js-dom M4）：导出 listener 错误上报到 globalThis——EventTarget.prototype.dispatchEvent
  //（part05，独立 listener 循环）的 handleEvent 非 callable TypeError 上报复用（跨 IIFE 段可达）。
  globalThis._zwReportListenerError = _zwReportListenerError;

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
      // js-dom M4 R111：spec inner invoke 步骤 4——once listener **调用前**移除（remove
      // listener then call）。调用后移除不够：listener 内嵌套 dispatchEvent 时新 dispatch
      // 的快照仍含本 listener → 无限递归（WPT remove-all-listeners "Nested usage of once
      // listeners"）。firedOnce 仍记录（dispatch 尾部的批量过滤路径保留，双保险）。
      if (entry.once && listeners[event.type]) {
        listeners[event.type] = listeners[event.type].filter(function(e) { return e !== entry; });
      }
      var fn = entry.fn;
      var callable = fn;
      if (typeof fn !== 'function') {
        // 对象 listener：Get handleEvent（每次派发都 Get，spec invoke 步骤）。非对象/null handleEvent → 跳过
        //（spec：callback 为 null/undefined 则不抛不调——WebIDL nullable callback 语义）。
        callable = fn ? fn.handleEvent : fn;
        // js-dom M4 R113：spec inner invoke 步骤 1-2——`handleEvent` 非 callable 时抛
        // TypeError。WebIDL EventListener 字典的 handleEvent 成员类型是 `(Function or
        // callable object)`——**null 也不豁免**（非 nullable callback：null 转换失败同样
        // 抛 TypeError）。WPT EventListener-handleEvent "throws if `handleEvent` is
        // falsy and not callable"（getHandleEvent 返 **null**）与 "truthy and not
        // callable"（返 42）都期望 TypeError 经 error 事件上报。listener 本体是
        // null/undefined（fn 非对象）不进本分支——addEventListener 参数本身 nullable。
        if (typeof callable !== 'function') {
          _zwReportListenerError(new globalThis.TypeError(
            "Failed to execute 'addEventListener' on 'EventTarget': parameter 2's 'handleEvent' property is not a function."));
          return;
        }
      }
      if (typeof callable === 'function') {
        // js-dom M4 R105：passive listener 内 preventDefault 是 no-op（spec HTML
        // event-listener-invoke「if listener's passive is true, set event's in
        // passive listener flag」——preventDefault 检查该 flag 不设 canceled）。
        // 用计数器包裹（支持嵌套派发）：fire 期间置位，正常路径 finally 复位。
        if (entry.passive) event._zwInPassive = (event._zwInPassive || 0) + 1;
        // js-dom M4 R106：spec `concept-event-dispatch` 步骤 10 / inner invoke 步骤——
        // listener 抛错**不传播**到 dispatchEvent 调用方（report error 后继续后续
        // listener，WPT EventTarget-dispatchEvent "Exceptions from event listeners must
        // not be propagated"：第一个 throw 后第二个 listener 仍须跑、dispatchEvent 返 true）。
        try {
          // 函数 listener: this=currentTarget；对象 listener: this=对象本身（spec EventListener invoke）。
          callable.call(typeof fn !== 'function' ? fn : ctx, event);
        } catch (_e106) {
          // js-dom M4 R113：spec inner invoke 步骤 11「report the exception」——spec
          // `report the exception` 的标准形态是 **fire an error event at window**（HTML
          // 「reporting exceptions」/runtime-script-error：ErrorEvent，message + error 字段），
          // window.addEventListener('error') 的 listener 与 onerror 属性 handler 都须收到。
          // R111 只调 onerror 属性——WPT EventListener-handleEvent 的 EventWatcher 等 window
          // 的 **error 事件**（addEventListener 路径）超时。现改经 `_zwReportListenerError`
          // 统一上报（error 事件派发 + onerror 属性调用 + console.error 兜底），异常不传播
          //（继续后续 listener），WPT Event-dispatch-throwing 的 onerror 计数语义保持。
          _zwReportListenerError(_e106);
        } finally {
          if (entry.passive) event._zwInPassive = Math.max(0, (event._zwInPassive || 1) - 1);
        }
      }
      // R111：firedOnce 记录（dispatch 尾部批量过滤路径保留——fire 头已做调用前移除，
      // 此处仅为 stopped() 早退路径的兜底记账）。
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
          // js-dom M4 R111：spec inner invoke「if listener's removed is true, continue」——
          // 快照迭代期间被 removeEventListener 移除的 listener 跳过（WPT
          // remove-all-listeners "Removing all listeners and then adding a new one"：
          // listener1 内移除 listener2，listener2 不得触发）。
          if (listeners[event.type] && listeners[event.type].indexOf(snap[i]) < 0) continue;
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
          // R111：同 capture 分支——派发中被移除的 listener 跳过。
          if (listeners[event.type] && listeners[event.type].indexOf(snap[j]) < 0) continue;
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
  // js-dom M4 R108：合成 click 的 **pre-click activation**（spec `concept-event-dispatch`
  // legacy-pre-activation 行——dispatch 的「legacy event dispatch」步骤：click 且 target 的
  // 祖先链（含自身）中第一个有 activation behavior 的元素在**任何 listener 之前**执行激活）。
  // 用例形态（WPT Event-dispatch-click）：input.onclick 里读 input.checked 须已是 true——
  // activation 先于 listener；child(Text) dispatch 冒泡时 activation 在**第一个** INPUT 祖先上
  //（pick the first with activation behavior——child 自身是 checkbox 时 input 父的 onclick 读
  // pre-click 不得触发 child 的 checked 已翻转…spec：target 起**最近的** activation 元素）。
  // checkbox → 翻 checked（内容属性存在性）；radio → 勾选 + 同 name 组互斥（复用 click() 的
  // post-activation 逻辑——本 helper 是其在 pre 阶段的统一版，click() 的 post 块后续收敛至此）。
  // R112：定位 pre-click 激活元素（target 起最近 INPUT[checkbox/radio]，与 _zwPreClickActivation
  // 同一遍历逻辑）——返回 { sel, handle, type } 或 null。disabled + click() 入口的「activation
  // 不执行」也在此返回 null（post 阶段不派发 input/change）。
  function _zwFindClickActivation(targetSel, targetHandle, isClickApi) {
    var sel = targetSel, handle = targetHandle, hops = 0;
    while (hops < 32) {
      hops++;
      var tag = null;
      try { tag = _realTag ? _realTag(sel, handle) : null; } catch (_e) { tag = null; }
      if (tag === 'INPUT') {
        var ty = '';
        try { ty = handle ? __zw_get_attr_handle(handle, 'type') : (sel ? __zw_get_attr(sel, 'type') : null); } catch (_e2) { ty = ''; }
        ty = String(ty || '').toLowerCase();
        if (ty === 'checkbox' || ty === 'radio') {
          var dis = '0';
          try { dis = handle ? __zw_has_attr_handle(handle, 'disabled') : (sel ? __zw_has_attr(sel, 'disabled') : '0'); } catch (_e3) {}
          if (!isClickApi || dis !== '1') return { sel: sel, handle: handle, type: ty };
          return null; // disabled + click()：无 activation（dispatch 早退路径同判定）。
        }
      }
      if (sel && typeof __zw_parent === 'function') {
        var ps = '';
        try { ps = __zw_parent(sel); } catch (_e4) {}
        if (ps) { sel = ps; handle = null; continue; }
      }
      if (handle && typeof _zwNodeParent !== 'undefined' && _zwNodeParent) {
        var link = _zwNodeParent[handle];
        if (link) {
          if (link.parentHandle) { handle = link.parentHandle; sel = link.parentSel || null; continue; }
          if (link.parentSel) { sel = link.parentSel; handle = null; continue; }
        }
      }
      break;
    }
    return null;
  }

  function _zwPreClickActivation(targetSel, targetHandle, isClickApi) {
    // 返回 legacy-canceled 回滚账 { kind, sel, handle, restore } 或 null（调用方挂 event）。
    globalThis._zwLastPreClickRollback = null;
    // 沿 target 向上找第一个 INPUT[checkbox/radio]（activation 元素）。
    var sel = targetSel, handle = targetHandle, hops = 0;
    while (hops < 32) {
      hops++;
      var tag = null;
      try { tag = _realTag ? _realTag(sel, handle) : null; } catch (_e) { tag = null; }
      if (tag === 'INPUT') {
        var ty = '';
        try { ty = handle ? __zw_get_attr_handle(handle, 'type') : (sel ? __zw_get_attr(sel, 'type') : null); } catch (_e2) { ty = ''; }
        ty = String(ty || '').toLowerCase();
        if (ty === 'checkbox' || ty === 'radio') {
          // R108 精修：disabled 语义按入口分——**click()（isClickApi）不执行 activation**
          //（WPT "disabled checkbox still has activation behavior"：child.disabled.click()
          // 后 checked 保持 false，且不上行父）；**dispatchEvent(new MouseEvent('click')) 仍执行**
          //（WPT "disabled checkbox should be checked from dispatchEvent"——合成派发不受
          // disabled 限制）。两入口都在 disabled activation 元素处停（nearest，不上行）。
          var _r108Dis2 = '0';
          try { _r108Dis2 = handle ? __zw_has_attr_handle(handle, 'disabled') : (sel ? __zw_has_attr(sel, 'disabled') : '0'); } catch (_eD2) {}
          if (!isClickApi || _r108Dis2 !== '1') {
            if (ty === 'checkbox') {
              var cur = '0';
              try { cur = handle ? __zw_has_attr_handle(handle, 'checked') : (sel ? __zw_has_attr(sel, 'checked') : '0'); } catch (_e4) {}
              var _r108Pre = (cur === '1'); // 翻转前状态（legacy-canceled 回滚目标）
              if (cur === '1') {
                if (handle) { try { if (typeof __zw_remove_attr_handle === 'function') __zw_remove_attr_handle(handle, 'checked'); } catch (_e5) {} }
                else if (typeof __zw_remove_attr === 'function') { try { __zw_remove_attr(sel, 'checked'); } catch (_e5) {} }
              } else {
                if (handle) { try { __zw_set_attr_handle(handle, 'checked', ''); } catch (_e6) {} }
                else if (typeof __zw_set_attr === 'function') { try { __zw_set_attr(sel, 'checked', ''); } catch (_e6) {} }
              }
              // legacy-canceled-activation 回滚账（dispatch finally 消费：preventDefault 时恢复 _r108Pre）。
              globalThis._zwLastPreClickRollback = { kind: 'checkbox', sel: sel, handle: handle, restore: _r108Pre };
            } else {
              // radio：勾当前 + 同 name 组互斥（文档级查询，复用 click() 语义）。
              if (handle) { try { __zw_set_attr_handle(handle, 'checked', ''); } catch (_e7) {} }
              else if (typeof __zw_set_attr === 'function') { try { __zw_set_attr(sel, 'checked', ''); } catch (_e7) {} }
              globalThis._zwLastPreClickRollback = { kind: 'radio', sel: sel, handle: handle };
              var nm = null;
              try { nm = handle ? __zw_get_attr_handle(handle, 'name') : (sel ? __zw_get_attr(sel, 'name') : null); } catch (_e8) {}
              if (sel && nm != null && String(nm) !== '' && typeof __zw_remove_attr === 'function') {
                try {
                  var q = 'input[type="radio"][name="' + String(nm).replace(/\\/g, '\\\\').replace(/"/g, '\\"') + '"]';
                  var all = globalThis.document.querySelectorAll(q);
                  for (var i = 0; i < all.length; i++) {
                    var rc = all.item ? all.item(i) : all[i];
                    try {
                      if (rc && rc.__zwSelector && rc.__zwSelector !== sel) __zw_remove_attr(rc.__zwSelector, 'checked');
                    } catch (_e9) {}
                  }
                } catch (_e10) {}
              }
            }
          }
          return; // 找到第一个 activation 元素——不再上行（spec nearest）。
        }
      }
      // 上行一跳：sel 经 __zw_parent；handle 经 _zwNodeParent 反链。
      if (sel && typeof __zw_parent === 'function') {
        var ps = '';
        try { ps = __zw_parent(sel); } catch (_e11) {}
        if (ps) { sel = ps; handle = null; continue; }
      }
      if (handle && typeof _zwNodeParent !== 'undefined' && _zwNodeParent) {
        var link = _zwNodeParent[handle];
        if (link) {
          if (link.parentHandle) { handle = link.parentHandle; sel = link.parentSel || null; continue; }
          if (link.parentSel) { sel = link.parentSel; handle = null; continue; }
        }
      }
      break;
    }
  }

  // js-dom M4 R112：checkbox/radio 激活后的 input + change 事件派发（spec HTML input
  // activation behavior 末段「fire an event named input / change at el」）。目标 = pre-click
  // 激活元素（target 起最近 INPUT[checkbox/radio]——与 _zwPreClickActivation 同一元素，
  // shadow 树内 input 也在其自身上派发，不穿透 shadow 边界）。前置条件：激活元素 connected
  //（WPT Event-dispatch-detached-input-and-change：detached input click 后 input/change
  // **不得**派发——「fire an event named input」的隐含前提是元素在文档/影子宿主链上）。
  // 时序：post-activation（dispatch finally、rollback 之后）——WPT 同文件 detached 用例
  // 断言不派发，attached 用例断言派发，均与 checked 翻转时序解耦。两个事件 bubbles:true、
  // cancelable:false（spec input/change 事件定义），input 走 InputEvent 构造器（可用时）。
  function _zwFireInputChange(sel, handle) {
    var el = _makeProxy(sel, handle);
    var inputEv;
    try {
      inputEv = globalThis.InputEvent
        ? new globalThis.InputEvent('input', { bubbles: true, cancelable: false })
        : _makeEvent('input', { bubbles: true, cancelable: false });
    } catch (_eI) { inputEv = _makeEvent('input', { bubbles: true, cancelable: false }); }
    try { _dispatchWithBubble(_elKey(sel, handle), sel, handle, inputEv); } catch (_e1) {}
    try {
      _dispatchWithBubble(_elKey(sel, handle), sel, handle,
        _makeEvent('change', { bubbles: true, cancelable: false }));
    } catch (_e2) {}
    return el;
  }
  // R112：激活元素是否 connected——sel 经 __zw_contains('html', sel)；handle-only 经
  // _zwNodeParent 反链上行（sel 节点即 connected；shadow root 容器经 _shadowHandleMeta
  // 跳 host，host 是 sel 节点即 connected——与 isConnected getter R90/R91 同判定）。
  function _zwClickActivationConnected(sel, handle) {
    if (sel) {
      if (typeof __zw_contains === 'function') {
        try { return __zw_contains('html', sel) === '1'; } catch (_e) { return true; }
      }
      return true;
    }
    if (handle && typeof _zwNodeParent !== 'undefined' && _zwNodeParent) {
      var hops = 0, link = _zwNodeParent[handle];
      while (link && hops++ < 64) {
        if (link.parentSel) return true;
        var ph = link.parentHandle;
        if (!ph) break;
        var meta = typeof _shadowHandleMeta !== 'undefined' && _shadowHandleMeta[ph];
        if (meta) {
          if (meta.hostSel) return true;
          link = _zwNodeParent[meta.hostHandle];
          continue;
        }
        link = _zwNodeParent[ph];
      }
    }
    return false;
  }

  function _dispatchWithBubble(targetKey, targetSel, targetHandle, event, targetSlot) {
    var target = _makeProxy(targetSel, targetHandle);
    event.target = target;
    // R138（js-dom M4）：srcElement 同步设——shim 工厂事件的 srcElement 是 accessor
    // getter（读 this.target 自动跟随），但 native 叠加路径的 native MouseEvent 实例
    // own data 属性 srcElement=null（构造器 set_event_init 设）遮蔽原型 getter →
    // dispatch 后读仍 null（WPT Event-dispatch-click "event state during post-click
    // handling" native 1F 根因）。own-set 覆盖 data 属性，两种形态统一。
    try { event.srcElement = target; } catch (_e138s) {}
    // js-dom M4 R106：spec dispatch flag——派发进行中的 event 再 dispatchEvent 抛
    // InvalidStateError（WPT EventTarget-dispatchEvent "If the event's dispatch flag
    // is set"）。嵌套安全计数（listener 内派发其他 event 合法；finally 复位）。
    event._zwDispatching = (event._zwDispatching || 0) + 1;

    // js-dom M4 R108：合成 click 的 pre-click activation（spec legacy-pre-activation——
    // 在任何 listener 之前执行；仅 MouseEvent 类 click，`new Event('click')` 不触发——
    // WPT "basic with wrong event class" 断言 onclick 里 checked 仍 false）。
    if (event.type === 'click' && globalThis.MouseEvent
        && (event instanceof globalThis.MouseEvent || event._zwSyntheticClick === true)) {
      var _r108IsClickApi = (event._zwSyntheticClick === true);
      // R112：激活元素定位复用（与 _zwPreClickActivation 同一遍历——target 起最近
      // INPUT[checkbox/radio]）。pre-click 已执行激活（checked 已翻转）时才在 post 阶段
      // 派发 input/change；detached（click() 对 disabled 的早退、或未执行激活）不派发。
      var _r112Act = null;
      try {
        _r112Act = _zwFindClickActivation(targetSel, targetHandle, _r108IsClickApi);
        _zwPreClickActivation(targetSel, targetHandle, _r108IsClickApi);
        if (globalThis._zwLastPreClickRollback) event._zwLegacyCancelRollback = globalThis._zwLastPreClickRollback;
      } catch (_e108) {}
      // R108：disabled 表单控件的合成 click——activation 已执行（checked 已翻转），但
      // **listener 不触发**（WPT "disabled ... part 2" assert_unreached 在 onclick 内：
      // disabled input 的 click 不跑 onclick）。非 click 路径不受影响。
      var _r108Tag = null;
      try { _r108Tag = _realTag ? _realTag(targetSel, targetHandle) : null; } catch (_e108b) {}
      if (_r108Tag === 'INPUT') {
        var _r108Dis = '0', _r108Ty = '';
        try {
          _r108Dis = targetHandle ? __zw_has_attr_handle(targetHandle, 'disabled') : (targetSel ? __zw_has_attr(targetSel, 'disabled') : '0');
          _r108Ty = String((targetHandle ? __zw_get_attr_handle(targetHandle, 'type') : (targetSel ? __zw_get_attr(targetSel, 'type') : '')) || '').toLowerCase();
        } catch (_e108c) {}
        if (_r108IsClickApi && _r108Dis === '1' && (_r108Ty === 'checkbox' || _r108Ty === 'radio')) {
          event._zwDispatching = Math.max(0, (event._zwDispatching || 1) - 1);
          return !event._defaultPrevented;
        }
      }
    }

    // 祖先链 target→root（[直接父, ..., html]）；无 __zw_parent / handle-only → 空 → 仅 target 派发。
    // js-dom M4 R114：handle-based target（shadow 树内元素/detached createElement 子树）经
    // `_zwNodeParent` 反链上行——遇 shadow root 容器（`_shadowHandleMeta` 命中）时按
    // `event.composed` 决定是否**跨 shadow 边界**到 host（spec DOM §2.9 dispatch：非
    // composed 事件的 path 止于 shadow root，不 retarget 到 host；composed 跨边界继续）。
    // 链元素统一 {sel, handle} 形态（sel 站沿用旧字符串 push 兼容——两形态消费点都在
    // 本函数内）。shadow 段站序：target 的 shadow 祖先 → host → host 的 light 祖先。
    // 链对象带 `shadow` 标记（该站是否仍在 shadow 段——window.event 抑制判定用；host
    // 及以上站 false）。
    var chain = [];
    var _r114ShadowDepth = 0; // target 处的 shadow 嵌套深度（每跨出一层边界 -1）
    if (targetSel && typeof __zw_parent === 'function') {
      var cur = targetSel;
      while (true) {
        var p;
        try { p = __zw_parent(cur); } catch (_e) { p = ''; }
        if (!p) break;
        chain.push(p);
        cur = p;
      }
    } else if (targetHandle && typeof _zwNodeParent !== 'undefined' && _zwNodeParent) {
      // 数 target 自身的 shadow 嵌套层数（在内层 shadow → 深度 ≥1，window.event 语义用）。
      var _r114H = targetHandle, _r114D = 0, _r114Guard = 0;
      while (_r114H && _r114Guard++ < 64) {
        var _r114Meta = typeof _shadowHandleMeta !== 'undefined' ? _shadowHandleMeta[_r114H] : null;
        if (_r114Meta) { _r114D++; _r114H = _r114Meta.hostHandle; continue; }
        var _r114Link = _zwNodeParent[_r114H];
        if (!_r114Link || !_r114Link.parentHandle) break;
        _r114H = _r114Link.parentHandle;
      }
      _r114ShadowDepth = _r114D;
      // 上行建链：sel 优先（sel 节点经 __zw_parent 走 host 快照链）——与
      // _zwFindClickActivation 的混合上行同款。`_r114CurDepth` 跟踪当前站剩余 shadow 层数
      //（每跨出一层边界 -1，到 0 即 host 站——host 及以上不再抑制）。
      var _r114Sel = null, _r114Handle = targetHandle, _r114Hops = 0;
      var _r114CurDepth = _r114ShadowDepth;
      while (_r114Hops++ < 64) {
        var _r114L = _zwNodeParent[_r114Handle];
        if (!_r114L) break;
        if (_r114L.parentSel) {
          // 进入 sel 域：余下链走 __zw_parent（host 快照祖先）。parentSel 站本身在
          // shadow 外（sel 节点 = light DOM / host 快照内节点）——spec 近似：sel 域视为
          // 已出 shadow（快照树无 shadow 边界信息；hostSel 站 = host，正确不抑制）。
          _r114Sel = _r114L.parentSel;
          chain.push(_r114Sel);
          _r114Handle = null;
          break;
        }
        var _r114PH = _r114L.parentHandle;
        if (!_r114PH) break;
        var _r114Pm = typeof _shadowHandleMeta !== 'undefined' ? _shadowHandleMeta[_r114PH] : null;
        if (_r114Pm) {
          // 父是 shadow root 容器：非 composed 止于此（path 到 shadow root 为止——shadow root
          // 本身无 listener 站，直接断链）；composed 跨到 host 继续。
          if (!event.composed) break;
          var _r114HostSel = _r114Pm.hostSel || null;
          var _r114HostHandle = _r114Pm.hostHandle || null;
          if (_r114HostSel) {
            chain.push(_r114HostSel); // host 站（light 域，不抑制）
            _r114Sel = _r114HostSel;
            _r114Handle = null;
            _r114CurDepth = 0;
            break; // host 是 sel 节点——余下走 __zw_parent 链
          }
          if (_r114HostHandle) {
            // host 站入链（shadow:false——host 即 light 域，window.event 从 host 起恢复可见），
            // 然后从 host 继续上行（host 可能也在外层 shadow 内——嵌套 shadow 场景）。
            // 注：只递减 _r114CurDepth（剩余 shadow 层数，站标记用）——**不动**
            // _r114ShadowDepth（target 深度，target 站抑制判定用，immutable）。
            chain.push({ sel: null, handle: _r114HostHandle, shadow: false });
            _r114Handle = _r114HostHandle;
            _r114CurDepth--;
            continue;
          }
          break;
        }
        chain.push({ sel: null, handle: _r114PH, shadow: _r114CurDepth > 0 });
        _r114Handle = _r114PH;
      }
      if (_r114Sel && typeof __zw_parent === 'function') {
        var _r114C = _r114Sel;
        while (true) {
          var _r114P;
          try { _r114P = __zw_parent(_r114C); } catch (_e114) { _r114P = ''; }
          if (!_r114P) break;
          chain.push(_r114P);
          _r114C = _r114P;
        }
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
    // js-dom M4 R114：handle-based target（shadow 树内/detached createElement 子树）的
    // 连入文档判定——链上行到 sel 域（`_zwNodeParent` → parentSel/hostSel）即该 sel 在
    // html 子树内 = connected（与 `_zwClickActivationConnected` 同判定）。连入 → 虚站
    // doc/win 追加（spec：composed 冒泡经 host 到 document/window；WPT event-global
    // ErrorEvent-in-shadow 用例：shadow 内派发 error，onerror 经 window 站冒泡触发）。
    if (!targetSel && !inDoc) {
      var _r114Hd = targetHandle, _r114Gd = 0;
      while (_r114Hd && _r114Gd++ < 64) {
        var _r114Md = typeof _shadowHandleMeta !== 'undefined' ? _shadowHandleMeta[_r114Hd] : null;
        if (_r114Md) { _r114Hd = _r114Md.hostHandle; continue; }
        var _r114Ld = (typeof _zwNodeParent !== 'undefined' && _zwNodeParent) ? _zwNodeParent[_r114Hd] : null;
        if (!_r114Ld) break;
        if (_r114Ld.parentSel) {
          if (typeof __zw_contains === 'function') {
            try { inDoc = __zw_contains('html', _r114Ld.parentSel) === '1'; } catch (_e114d) {}
          }
          break;
        }
        if (!_r114Ld.parentHandle) break;
        _r114Hd = _r114Ld.parentHandle;
      }
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
    // R114：链元素统一 {sel, handle} 解析（旧字符串 sel 与新 handle 对象两形态）。
    function _r114Entry(e) {
      return (e && typeof e === 'object') ? { sel: e.sel || null, handle: e.handle || null } : { sel: e, handle: null };
    }
    for (var cpi = 0; cpi < elemChain.length; cpi++) {
      var _r114E = _r114Entry(elemChain[cpi]);
      cpPath.push(_r114E.sel ? _wrapSelector(_r114E.sel) : _wrapHandle(_r114E.handle));
    }
    // R40：composedPath 与派发虚站一致——passDoc/passWin 控制 document/window 追加（document target 的
    // path = [document, window]；window target = [window]；元素连入文档 = [..., document, window]）。
    if (passDoc && docObj) cpPath.push(docObj);
    if (passWin && winObj) cpPath.push(winObj);
    event._composedPath = cpPath;

    // js-dom M4 R33：`Window.event`（HTML `current event`）——dispatch 前 save 外层 event、set 当前 event。
    // 嵌套 dispatch（redispatch）时内层 finally 恢复外层（spec innermost-first，外层结束后其 event 仍可见）。
    // finally 统一 restore，与 _composedPath/_propagationStopped 清理同处。prevEvent 用局部变量保 dispatch 栈。
    // js-dom M4 R114：**shadow 段抑制**——spec HTML「current event」：正在调度的 listener 其
    // 节点 root 是 shadow root 时 `window.event` 为 undefined（shadow 树的 current event 不外
    // 露到 window；WPT event-global "target is in a shadow tree" 两断言）。目标在 shadow 树内
    //（_r114ShadowDepth ≥1）时 target/影子祖先站派发前临时置 undefined、站后复原 event；
    // 跨出边界后（host 及以上）恢复可见。
    var prevEvent = globalThis.event;
    var _r114Suppress = _r114ShadowDepth > 0;
    globalThis.event = _r114Suppress ? undefined : event;
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
            // R114：链元素统一 {sel, handle} 解析（旧字符串 sel 与新 handle 对象两形态）。
            var _r114C = _r114Entry(elemChain[i]);
            var capKey = _elKey(_r114C.sel, _r114C.handle);
            _ensureInlineHandler(capKey, _r114C.sel, _r114C.handle, event.type); // R2935 祖先 inline on* handler 触发
            var capAnc = _r114C.sel ? _wrapSelector(_r114C.sel) : _wrapHandle(_r114C.handle);
            // R114：shadow 段站（entry.shadow）派发期间 window.event 置 undefined，站后复原。
            var _r114CapSup = !!_r114C.shadow;
            if (_r114CapSup) globalThis.event = undefined;
            _dispatchToListeners(capKey, event, 'capture', capAnc, _r114C.sel === 'html' ? null : undefined);
            if (_r114CapSup) globalThis.event = event;
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
      // R114：target 在 shadow 树内 → target 站派发期间 window.event 抑制为 undefined。
      if (_r114Suppress) globalThis.event = undefined;
      _dispatchToListeners(targetKey, event, 'all', target, tgtSlotFilter);
      if (_r114Suppress) globalThis.event = event;
      if (bubbleStopped()) return !event._defaultPrevented;

      // ③ bubble 阶段：target→root 方向（chain 正序 → document → window），祖先派发非 capture（仅 event.bubbles）。
      if (event.bubbles && !globalThis.__zw_no_bubble) {
        if (elemChain.length > 0) {
          for (var k = 0; k < elemChain.length; k++) {
            // R114：链元素统一 {sel, handle} 解析（旧字符串 sel 与新 handle 对象两形态）。
            var _r114B = _r114Entry(elemChain[k]);
            var bKey = _elKey(_r114B.sel, _r114B.handle);
            _ensureInlineHandler(bKey, _r114B.sel, _r114B.handle, event.type); // R2935 祖先 inline on* handler 冒泡触发
            var bAnc = _r114B.sel ? _wrapSelector(_r114B.sel) : _wrapHandle(_r114B.handle);
            // R114：shadow 段站（entry.shadow）派发期间 window.event 置 undefined，站后复原。
            var _r114BubSup = !!_r114B.shadow;
            if (_r114BubSup) globalThis.event = undefined;
            _dispatchToListeners(bKey, event, 'bubble', bAnc, _r114B.sel === 'html' ? null : undefined);
            if (_r114BubSup) globalThis.event = event;
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
      // js-dom M4 R29：spec `concept-event-dispatch` 末步——dispatch 结束 unset stop propagation flag
      //（+ 步骤清其他 dispatch flags）。reset 后 cancelBubble getter（后端 _propagationStopped）返 false
      //（WPT Event-cancelBubble "cancelBubble must be false after an event has been dispatched"）。
      // R146（js-dom M4）：**无条件**清（含 dispatch 前外部设的 flag）——spec 该步无「仅 dispatch 内」
      // 限定，外设 flag 也随 dispatch 结束清除，同 event 二次 dispatch 恢复触发（WPT
      // Event-propagation "After stopImmediatePropagation()"：一次 dispatch 零触发后**第二次**
      // dispatch 应正常触发——旧「外设保留」语义使第二次仍零触发。dispatch 前的零触发由
      // 上方 dispatch 开始时的 flag 检查保证，不受影响）。
      event._propagationStopped = false;
      event._immediateStopped = false;
      // js-dom M4 R34：同步清 native Event 的 stop flag（叠加路径下 `new MouseEvent` 是 native 对象，dispatch
      // 走此 polyfill 但 native dispatch_event_impl 未跑故不自清；同 event 重派发需 fresh，与 _propagationStopped 同语义）。
      event.__zw_stop = false;
      event.__zw_stop_immediate = false;
      // R106：dispatch flag 复位（嵌套计数——内层 finally 减一，外层结束归零）。
      event._zwDispatching = Math.max(0, (event._zwDispatching || 1) - 1);
      // R108：legacy-canceled-activation behavior（spec inner invoke 步骤——listener
      // preventDefault 后，pre-click 已执行的 activation 在 dispatch 结束**回滚**：checkbox
      // 恢复翻转前状态；radio 恢复 pre-click 前组态（当前实现：直接取消自身 checked——组内
      // 其他成员恢复属深面，WPT 只断言自身 false）。
      if (event._zwLegacyCancelRollback && event.cancelable && event._defaultPrevented
          && event._zwDispatching === 0) {
        var _rb = event._zwLegacyCancelRollback;
        event._zwLegacyCancelRollback = null;
        try {
          if (_rb.kind === 'checkbox') {
            if (_rb.restore) {
              if (_rb.handle) { try { if (typeof __zw_set_attr_handle === 'function') __zw_set_attr_handle(_rb.handle, 'checked', ''); } catch (_eA) {} }
              else if (_rb.sel && typeof __zw_set_attr === 'function') { try { __zw_set_attr(_rb.sel, 'checked', ''); } catch (_eA) {} }
            } else {
              if (_rb.handle) { try { if (typeof __zw_remove_attr_handle === 'function') __zw_remove_attr_handle(_rb.handle, 'checked'); } catch (_eB) {} }
              else if (_rb.sel && typeof __zw_remove_attr === 'function') { try { __zw_remove_attr(_rb.sel, 'checked'); } catch (_eB) {} }
            }
          } else if (_rb.kind === 'radio') {
            if (_rb.handle) { try { if (typeof __zw_remove_attr_handle === 'function') __zw_remove_attr_handle(_rb.handle, 'checked'); } catch (_eC) {} }
            else if (_rb.sel && typeof __zw_remove_attr === 'function') { try { __zw_remove_attr(_rb.sel, 'checked'); } catch (_eC) {} }
          }
        } catch (_eD) {}
      }
      // js-dom M4 R112：post-activation——checkbox/radio 的 input + change 事件（spec HTML
      // input activation behavior 末段）。条件：① click 且激活元素已定位（_r112Act，pre-click
      // 已翻转 checked）；② 顶层 dispatch（_zwDispatching 归零——嵌套内层不触发，由最外层收尾）；
      // ③ 未 canceled（cancelable + preventDefault → 上方 rollback 已恢复 checked——canceled
      // activation 不派发 input/change，spec legacy-canceled-activation 在 fire 前判断。注意
      // rollback 块已把 ledger 置 null，此处按 canceled flag 判不能读 ledger）；④ 激活元素
      // connected（detached 不派发，WPT Event-dispatch-detached-input-and-change）。两个事件
      // 递归走 _dispatchWithBubble（各自完整的 capture/target/bubble），异常吞（不中断外层）。
      if (_r112Act && event._zwDispatching === 0 && event.type === 'click'
          && !(event.cancelable && event._defaultPrevented)
          && _zwClickActivationConnected(_r112Act.sel, _r112Act.handle)) {
        try {
          _zwFireInputChange(_r112Act.sel, _r112Act.handle);
        } catch (_e112) {}
      }
    }
  }

  function _makeEvent(type, options) {
    options = options || {};
    var ev = {
      type: type,
      bubbles: !!options.bubbles,
      cancelable: !!options.cancelable,
      // js-dom M4 R114：EventInit.composed（spec dom-event-constructors——`new Event(t,
      // {composed:true})` 的 composed 初值来自 init dict，缺省 false）。此前硬编码 false
      // 使 composed 事件（shadow 边界穿越派发）永不生效（WPT event-global shadow 用例）。
      composed: !!options.composed,
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
      preventDefault: function() {
        // R105：in passive listener flag 期间 no-op（spec：passive listener 的 preventDefault
        // 不设 canceled flag）。`_zwInPassive` 由 _dispatchToListeners 的 fire 包裹维护。
        if (this._zwInPassive) return;
        if (this.cancelable) { this.defaultPrevented = true; this._defaultPrevented = true; }
      },
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
        // R105：passive listener 内同 preventDefault 语义（no-op，spec invoke 步骤 8）。
        if (!v && this.cancelable && !this._zwInPassive) { this.defaultPrevented = true; this._defaultPrevented = true; }
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
  // R134（js-dom M4）：type selector 的 handle-only 元素 JS 侧匹配（spec selectors-4
  // §6.1）：`type`（默认 ns——浏览器对 API matches 的默认 ns 语义取「不比较 ns」，
  // WPT matches-namespaced-elements 的空 ns 与 urn:ns 两形态都以裸 type 断言真）、
  // `ns|type`（ns 全等）、`*|type`（任意 ns）、`|type`（显式无 ns）、`*`（任意元素）。
  // ns/localName 源经 `_nsHandles`（createElementNS 登记）——miss（普通 createElement）
  // 时 local=tag 小写、ns=null。返回 null = 非 type selector（含伪类/属性/组合器/
  // 空白）——调用方回落 false（保守：detached 元素的复合匹配属选择器引擎深结构）。
  function _r134MatchTypeSelector(handle, q) {
    var s = String(q).trim();
    if (!s) return false;
    if (/[\s>+~\[.#:(]"'/.test(s)) return null;
    var nsMeta = (typeof _nsHandles !== 'undefined' && _nsHandles[handle]) || null;
    var local = nsMeta ? String(nsMeta.qualifiedName || '') : '';
    var c = local.indexOf(':');
    if (c >= 0) local = local.slice(c + 1);
    if (!nsMeta) {
      var tag = _realTag(null, handle);
      local = tag ? String(tag).toLowerCase() : '';
    }
    if (!local) return false;
    var elNs = nsMeta ? (nsMeta.namespace == null ? null : String(nsMeta.namespace)) : null;
    if (s === '*') return true;
    var bar = s.indexOf('|');
    // type selector 段 ASCII 大小写不敏感（HTML 文档语义；CSS type selector 本不区分）
    var low = s.toLowerCase();
    if (bar === 0) return low.slice(1) === local && elNs === null;
    if (bar > 0) {
      var p = low.slice(0, low.indexOf('|')), t = low.slice(low.indexOf('|') + 1);
      if (t === '*') return p === '*' || String(elNs) === p;
      if (t !== local) return false;
      if (p === '*') return true;
      return String(elNs) === p;
    }
    return low === local;
  }
  function _realTag(sel, handle) {
    if (sel && typeof __zw_get_tag === 'function') {
      try { var t = __zw_get_tag(sel); if (t) return _zwAsciiUpper(t); } catch (_e) {}
    }
    if (handle && typeof __zw_get_tag_handle === 'function') {
      try { var ht = __zw_get_tag_handle(handle); if (ht) return _zwAsciiUpper(ht); } catch (_e) {}
    }
    // js-dom M3 R100：跨 execute 的 handle 元素（已应用入文档，当前 execute 的
    // mutations 队列不含其 CreateElement 记录）——经持久反查表锚回 selector 再查
    // host 快照。未注册回调/未命中 → 原 `_tagFromSel` 回落（恒 DIV，零回归）。
    if (handle && !sel && typeof __zw_handle_for_selector === 'function') {
      try {
        var _r100s = _r100SelOfHandle(handle);
        if (_r100s && typeof __zw_get_tag === 'function') {
          var rt = __zw_get_tag(_r100s);
          if (rt) return _zwAsciiUpper(rt);
        }
      } catch (_e100t) {}
    }
    return _tagFromSel(sel);
  }
  // js-dom M3 R100：`__zw_handle_for_selector` 是 selector→handle 方向；这里需要
  // 反向（handle→selector）。host 不另设回调——在 JS 侧维护正置缓存（R100 map 的
  // 镜像：`__zw_handle_for_selector` 命中处同步登记）。空句柄返 null。
  var _r100HandleToSel = null;
  function _r100SelOfHandle(handle) {
    if (_r100HandleToSel && Object.prototype.hasOwnProperty.call(_r100HandleToSel, handle)) {
      return _r100HandleToSel[handle];
    }
    return null;
  }
  function _r100Remember(handle, sel) {
    if (!_r100HandleToSel) _r100HandleToSel = {};
    _r100HandleToSel[handle] = sel;
  }
  // js-dom M4 R81：ASCII-only 大写（spec ASCII-uppercase——'ı'（U+0131 dotless）等 Unicode
  // 小写不受影响；JS toUpperCase 会 'ı'→'I' 使 localName 回落错读 'input'。WPT
  // Document-createElement "ınput" 期望原样保留）。
  function _zwAsciiUpper(str) {
    var out = '';
    for (var i = 0; i < str.length; i++) {
      var c = str.charAt(i);
      out += (c >= 'a' && c <= 'z') ? String.fromCharCode(c.charCodeAt(0) - 32) : c;
    }
    return out;
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
  // FV（M1）：pattern 约束适用的 text 类（spec §4.10.5.2.5——含 email，选区语义
  // 的 _TEXT_SEL_TYPES 不含）。
  var _PATTERN_TYPES = { '': 1, text: 1, search: 1, tel: 1, url: 1, email: 1, password: 1 };
  // FV（M1）：非 text 类型的值有效格式判定（valueMissing 的「有效值集合」——
  // date/month/week/time/datetime-local 的 ISO 格式近似；number 的 parseFloat）。
  var _DATE_TYPES = { date: 1, month: 1, week: 1, time: 1, 'datetime-local': 1 };
  // FV（M1）：十进制串（含科学计数法）→ BigInt 分数 {num, den}。number 的
  // step 极小值（3e-15）的整数倍判定——IEEE 浮点取模不可靠（diff/st ≈ 5.67e15
  // 的舍入）——有理数精确判定。
  function _parseDecimalFraction(v) {
    var s = String(v).trim();
    var m = s.match(/^([+-]?)(\d+)(?:\.(\d+))?(?:[eE]([+-]?\d+))?$/);
    if (!m) return null;
    var neg = m[1] === '-';
    var intPart = m[2] || '0';
    var fracPart = m[3] || '';
    var exp = m[4] ? parseInt(m[4], 10) : 0;
    var digits = intPart + fracPart;
    if (digits === '') digits = '0';
    var num = BigInt(neg ? '-' + digits : digits);
    var den = BigInt(10) ** BigInt(fracPart.length - exp);
    if (den < 0n) { num = num * BigInt(10) ** BigInt(-den); den = 1n; }
    if (den === 0n) den = 1n;
    return { num: num, den: den };
  }
  // 有理数整数性：(a.num/step.den) % (a.den/step.num) == 0
  function _isIntegralMultiple(a, step) {
    var lhs = a.num * step.den;
    var rhs = a.den * step.num;
    if (rhs === 0n) return false;
    return lhs % rhs === 0n;
  }
  // FV（M1）：HTML pattern 编译的 v 模式非法近似（V8 无 v flag 支持——
  // spec §4.10.5.2.5：v 模式非法正则被忽略）。字符类内未转义特殊字符
  //（"[(" 等）+ 未配对组（"a)(b"）→ v 非法。
  function _isVInvalidPattern(pat) {
    var s = String(pat);
    if (/\[[^\]\\]*[\(\[\{\/]/.test(s)) return true;
    var depth = 0;
    for (var i = 0; i < s.length; i++) {
      if (s[i] === '\\') { i++; continue; }
      if (s[i] === '(') depth++;
      else if (s[i] === ')') { depth--; if (depth < 0) return true; }
    }
    return depth !== 0;
  }
  // FV（M1）：date 类 range 的宽松比较判定（WPT 怪用例——date 的 value/max
  // 含时间部分 "2000-01-01T12:00:00"——只要求日期前缀匹配即可比较（字典序）；
  // 无效格式（"abc"）不比较）。
  function _isDateRangeComparable(v, ty) {
    // 宽松格式（date 可含时间部分——WPT 怪用例）**+ 范围校验**（无效月/日/时
    // 不比较——"2000-02-30" 期望不触发 range）。
    var m;
    if (ty === 'date') {
      m = v.match(/^(\d{4,})-(\d{2})-(\d{2})$/);
      if (!m) return false;
      y = +m[1]; mo = +m[2]; d = +m[3];
    } else if (ty === 'datetime-local') {
      // 完整匹配（含时间部分——"2000-01-01  12:00"（双空格）无效——前缀匹配
      // 的 bug）
      m = v.match(/^(\d{4,})-(\d{2})-(\d{2})(?:[T ](\d{2}):(\d{2})(?::(\d{2})(\.\d+)?)?)?$/);
      if (!m) return false;
      if (m[4] != null && (+m[4] > 23 || +m[5] > 59 || (+m[6] || 0) > 59)) return false;
      y = +m[1]; mo = +m[2]; d = +m[3];
    } else if (ty === 'month') {
      m = v.match(/^(\d{4,})-(\d{2})/);
      if (!m) return false;
      if (+m[2] < 1 || +m[2] > 12) return false;
      return true;
    } else if (ty === 'week') {
      var mw = v.match(/^(\d{4,})-W(\d{2})/);
      if (!mw) return false;
      return +mw[2] >= 1 && +mw[2] <= 53;
    } else {
      var mt = v.match(/^(\d{2}):(\d{2})(?::(\d{2}))?/);
      if (!mt) return false;
      return +mt[1] <= 23 && +mt[2] <= 59 && (+mt[3] || 0) <= 59;
    }
    var y = +m[1], mo = +m[2], d = +m[3];
    if (mo < 1 || mo > 12 || d < 1) return false;
    var dim = [31, (y % 4 === 0 && (y % 100 !== 0 || y % 400 === 0)) ? 29 : 28,
               31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    return d <= dim[mo - 1];
  }
  // FV（M1）：date 类字符串比较——年部分数值比较（"10000-01-01" > "2000-01-01"
  // ——变长数字的字典序错误："1" < "2"）+ 其余部分字典序（固定宽度 ✓）。
  function _dateCmp(a, b) {
    var ma = String(a).match(/^(\d+)/), mb = String(b).match(/^(\d+)/);
    if (ma && mb && ma[1].length !== mb[1].length) {
      var ya = +ma[1], yb = +mb[1];
      if (ya !== yb) return ya > yb ? 1 : -1;
    }
    return a > b ? 1 : (a < b ? -1 : 0);
  }
  function _isValidDateString(v, ty) {
    // R57（FV M1）：范围校验（月/日/时/周）——"9999-99-99"（月 99）无效；
    // 年 4+ 位（10000 年合法）；datetime-local 接受 T 或空格分隔。
    var y, mo, d;
    if (ty === 'date') {
      var m = v.match(/^(\d{4,})-(\d{2})-(\d{2})$/);
      if (!m) return false;
      y = +m[1]; mo = +m[2]; d = +m[3];
    } else if (ty === 'datetime-local') {
      var m = v.match(/^(\d{4,})-(\d{2})-(\d{2})[T ](\d{2}):(\d{2})(?::(\d{2})(\.\d+)?)?$/);
      if (!m) return false;
      y = +m[1]; mo = +m[2]; d = +m[3];
      if (+m[4] > 23 || +m[5] > 59 || (+m[6] || 0) > 59) return false;
    } else if (ty === 'month') {
      var m2 = v.match(/^(\d{4,})-(\d{2})$/);
      if (!m2) return false;
      y = +m2[1]; mo = +m2[2]; d = 1;
    } else if (ty === 'week') {
      var mw = v.match(/^(\d{4,})-W(\d{2})$/);
      if (!mw) return false;
      var wk = +mw[2];
      return wk >= 1 && wk <= 53;
    } else if (ty === 'time') {
      var m3 = v.match(/^(\d{2}):(\d{2})(?::(\d{2})(\.\d+)?)?$/);
      if (!m3) return false;
      return +m3[1] <= 23 && +m3[2] <= 59 && (+m3[3] || 0) <= 59;
    } else {
      return false;
    }
    if (mo < 1 || mo > 12 || d < 1) return false;
    var dim = [31, (y % 4 === 0 && (y % 100 !== 0 || y % 400 === 0)) ? 29 : 28,
               31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    return d <= dim[mo - 1];
  }
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
  // R57（FV M3）：值缓存仅对**稳定身份**键生效——`#id`（id 唯一稳定）与 `@handle`
  //（create 句柄，单 execute 内唯一）。位置选择器（`form:nth-child(1) > input:nth-child(1)`
  // 等）随同批 DOM mutation 指向**不同元素**——跨批缓存碰撞（form-requestsubmit：
  // test 7 首表单首 input 缓存 "v1"，test 9 首表单首 input（同选择器串）误读 "v1"——
  // 实际 value 属性空）。位置键跳过缓存直接读 host（applied view 正确）。
  function _stableValueKey(key) {
    return key.charAt(0) === '#' || key.charAt(0) === '@';
  }
  // text control 当前 value 串（mirror value getter 的 lazy-init 逻辑，仅读不改缓存——选区 clamp 须 length）。
  function _controlValue(sel, handle, key) {
    if (_inputValues[key] != null && (_inputValuesSet[key] === true || _stableValueKey(key))) return String(_inputValues[key]);
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
  // js-dom M4 R86：handle 移除标记（sel 版同款语义）——removeChild/remove 的 handle 节点
  // 标记已移除，NodeIterator/TreeWalker 的 order 快照扫描跳过（spec：迭代集合是 live 的，
  // 移除节点退出集合；WPT NodeIterator-removal：remove 后迭代器不再命中该节点/子树）。
  var _zwRemovedHandles = {};
  function _zwMarkRemovedHandle(h) { if (h) _zwRemovedHandles[h] = true; }
  function _zwUnmarkRemovedHandle(h) { if (h) delete _zwRemovedHandles[h]; }
  function _zwIsRemovedNode(node) {
    if (!node) return false;
    if (node.__zwSelector && _zwRemovedSels[node.__zwSelector]) return true;
    if (node.__zwHandle && _zwRemovedHandles[node.__zwHandle]) return true;
    // 子树判定：任一祖先被移除（沿 parentNode 上行——handle 反链在移除后由
    // _mo_notify 清理，但 parentNode getter 读融合视图，removed 父的子列表可能
    // 已物化缓存——需沿链上行查标记）。
    try {
      var p = node.parentNode, guard = 0;
      while (p && guard++ < 64) {
        if (p.__zwSelector && _zwRemovedSels[p.__zwSelector]) return true;
        if (p.__zwHandle && _zwRemovedHandles[p.__zwHandle]) return true;
        p = p.parentNode;
      }
    } catch (_e) {}
    return false;
  }

  function _parentNodeFor(sel, handle, elementOnly) {
    // R34xx：本地移除标记优先——remove() 后（mutation 未应用）parentNode 返 null。
    if (_zwIsRemoved(sel)) return null;
    // js-dom M4 R79：html 的 parentNode 是 document（spec Node.parentNode：documentElement
    // 的父为 Document——`__zw_parent` 只返元素父，对 html 返空）。document 进链是
    // contains/compareDocumentPosition 以 document 为 root 的前提（WPT Node-contains 的
    // `paras[0].contains(document)` oracle 沿 parentNode 上行须命中 document）。
    // parentElement（elementOnly=true）例外：spec `dom-node-parentelement` 只返元素父，
    // html 的 parentElement 恒 null（zeroweb-regression-guard 2026-08-17 发现）。
    if (sel === 'html') return elementOnly ? null : (globalThis.document || null);
    if (sel && typeof __zw_parent === 'function') {
      try {
        var p = __zw_parent(sel);
        if (p) return _wrapSelector(p);
        return null; // 未命中 → 无元素父
      } catch (_e) { return null; }
    }
    // js-dom M4 R51：handle-only 节点先查 child→parent 反向链（_zwNodeParent，_mo_notify
    // childList 汇流点记账）。命中 sel 父 → _wrapSelector（与快照查询同 proxy 缓存，identity
    // 稳定）；命中 handle 父 → _wrapHandle。
    if (handle) {
      var _npl = _zwNodeParent[handle];
      if (_npl) {
        if (_npl.parentSel) return _wrapSelector(_npl.parentSel);
        if (_npl.parentHandle) return _wrapHandle(_npl.parentHandle);
      }
      // R51：无链的纯 detached handle 节点（createElement 后未 append）→ null（spec：
      // detached 节点 parentNode 为 null）。旧 fallback 猜 body 是 WPT dom/common.js
      // indexOf 死循环根因（假父快照永不含该节点）。
      if (typeof __zw_parent !== 'function') {
        if (sel === 'html') return null;
        if (sel === 'body' || sel === 'head') return _wrapSelector('html');
        return _wrapSelector('body');
      }
      return null;
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
  // R123：对齐 Rust escape_html 全集（& " < > U+00A0）——_zwMEl 序列化路径与 sel/handle
  // outerHTML 路径三方一致（旧只转 & " 使 handle-create 元素 innerHTML 序列化分歧）。
  // R123：对齐 Rust escape_html 全集（& " < > U+00A0）——_zwMEl 序列化路径与 sel/handle
  // outerHTML 路径三方一致（旧只转 & " 使 handle-create 元素 innerHTML 序列化分歧）。
  function _zwMEscapeAttr(s) { return String(s).replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/ /g, '&nbsp;'); }
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
    // R81：snap.preserveCase（XML 文档 createElement）保持原大小写（WPT Node-properties
    // xmlElement.tagName 期望 "igiveuponcreativenames"）；缺省 HTML 语义大写。
    var _mUp = snap.preserveCase ? tag : tag.toUpperCase();
    var attrs = [];
    var sa = snap.attrs || {};
    for (var k in sa) { if (Object.prototype.hasOwnProperty.call(sa, k)) attrs.push({ name: k, value: sa[k] }); }
    var node = {
      nodeType: 1,
      tagName: _mUp,
      nodeName: _mUp,
      localName: tag,
      id: snap.id || '',
      className: snap.cls || '',
      // R125：spec dom-node-nodenamespace —— 元素的 namespaceURI（HTML 解析产物 = HTML ns；
      // XML 文档经 snap.ns 覆盖，回落 HTML ns 与 R81 _zwParsedDoc._defaultNS 语义一致）。
      // R125 原型链接后 `node instanceof Element` 为真，DOMPurify _checkValidNamespace
      // 读 element.namespaceURI——缺省 undefined 不在 ALLOWED_NAMESPACES → 元素被误杀
      // （sanitize 返空串，r3019 回归）。
      namespaceURI: snap.ns !== undefined ? snap.ns : 'http://www.w3.org/1999/xhtml',
      attributes: attrs,
      childNodes: [],
      parentNode: parent || null
    };
    // R125：解析本地元素的接口原型链接（`test instanceof HTMLDivElement`——WPT
    // Document-getElementById "add id attribute via innerHTML"：element.firstChild /
    // getElementById 返回的解析节点须过 instanceof 接口断言）。按 tag 查
    // __zwHtmlTagIface（与 handle/proxy 元素同源表），miss 回落 HTMLElement.prototype。
    try {
      var _mIfaceTag = String(tag || '').toLowerCase();
      var _mIface = globalThis.__zwHtmlTagIface && globalThis.__zwHtmlTagIface[_mIfaceTag];
      var _mProto = (_mIface && globalThis[_mIface] && globalThis[_mIface].prototype)
        || (globalThis.HTMLElement && globalThis.HTMLElement.prototype) || null;
      if (_mProto) Object.setPrototypeOf(node, _mProto);
    } catch (_eM125) {}
    node.getAttribute = function (n) { n = String(n); for (var i = 0; i < attrs.length; i++) if (attrs[i].name === n) return attrs[i].value; return null; };
    // R130（js-dom M4）：A/AREA 的 `href` IDL 属性（WPT createHTMLDocument "URL parsing"
    // ——`a.href = 'http://example.org/?ä'` 后 getter 期望 `?%C3%A4`：spec URL 序列化
    // 对特殊 scheme 的 query 段非 ASCII 字节做 percent-encode（UTF-8）。detached _zwMEl
    // 无 get trap——补 IDL accessor：setter 存原始值，getter 读回时按 WHATWG 规则编码
    // query/hash 的非 ASCII（encodeURI 不动 ASCII 保留字，等价于 query 百分比编码集的
    // 超集近似——非 ASCII 全编码正是 special-scheme query 的要求）。
    if (tag === 'a' || tag === 'area') {
      Object.defineProperty(node, 'href', {
        get: function () {
          var raw = node.getAttribute('href');
          if (raw == null) return '';
          try {
            var m = String(raw).match(/^([a-zA-Z][a-zA-Z0-9+.-]*:\/\/[^?#]*)(\?[^#]*)?(#.*)?$/);
            if (!m) return String(raw);
            var enc = m[2] ? '?' + encodeURI(m[2].slice(1)).replace(/#/g, '%23') : '';
            return m[1] + enc + (m[3] || '');
          } catch (_e130e) { return String(raw); }
        },
        set: function (v) { node.setAttribute('href', v === null ? '' : String(v)); },
        configurable: true, enumerable: true,
      });
    }
    node.hasAttribute = function (n) { return node.getAttribute(n) !== null; };
    // js-dom M3 R97：hasAttributes/getAttributeNames（lit-html Template 解析对解析子树元素
    // 调 `r.hasAttributes()` + `r.getAttributeNames()` 提取属性 parts——缺方法抛 TypeError
    // 使整条 update 链 reject）。与元素 proxy R3197 语义一致（attrs 数组本地维护）。
    node.hasAttributes = function () { return attrs.length > 0; };
    node.getAttributeNames = function () { var out = []; for (var i = 0; i < attrs.length; i++) out.push(attrs[i].name); return out; };
    // js-dom M3 R99：addEventListener/removeEventListener/dispatchEvent（lit EventPart 的
    // `@click` 绑定对解析节点调 addEventListener/removeEventListener——缺方法使 commit 抛
    // TypeError、整次 render 中止[e2e 实证 renderRoot 仅 marker、hasUpdated:false]）。listener
    // 存本地数组；dispatchEvent 同步派发（listener.call(node, ev) 或 handleEvent 协议，spec
    // EventListener callback）；捕获选项 record（listener 对象 capture/passive/once 字段——
    // lit 传 boolean | object 两种形态）。once 派发后移除。派发序 = 注册序。返回值 = ev 的
    // cancelable?defaultPrevented 反相（spec dispatchEvent）。
    var _mEvListeners = [];
    node.addEventListener = function (type, listener, opts) {
      if (typeof listener !== 'function' && !(listener && typeof listener.handleEvent === 'function')) return;
      var cap = !!(opts && (typeof opts === 'object' ? opts.capture : opts));
      var once = !!(opts && typeof opts === 'object' && opts.once);
      _mEvListeners.push({ type: String(type), fn: listener, capture: cap, once: once });
    };
    node.removeEventListener = function (type, listener, opts) {
      var cap = !!(opts && (typeof opts === 'object' ? opts.capture : opts));
      var t = String(type);
      for (var i = _mEvListeners.length - 1; i >= 0; i--) {
        var l = _mEvListeners[i];
        if (l.type === t && l.fn === listener && l.capture === cap) { _mEvListeners.splice(i, 1); return; }
      }
    };
    node.dispatchEvent = function (ev) {
      // js-dom M4 R106：spec 入口守卫（与主派发路径一致——TypeError/InvalidStateError）。
      globalThis._zwDispatchGuard(ev);
      // R146（js-dom M4）：spec `dom-event-dispatch` 步骤——派发前设 target/srcElement
      // 为本节点（listener 读 ev.target 断言自身；WPT Event-dispatch-other-document
      // "Custom event on an element in another document"：detached doc 的 element
      // dispatchEvent 后 ev.target === element / ev.srcElement === element，旧未设
      // 均为 null）。own-set 覆盖构造器 data 属性（native 形态同 R138 srcElement 手法）。
      try {
        ev.target = node;
        ev.srcElement = node;
      } catch (_e146t) {}
      var t = String(ev && ev.type);
      var idx = [];
      for (var i = 0; i < _mEvListeners.length; i++) if (_mEvListeners[i].type === t) idx.push(i);
      for (var j = 0; j < idx.length; j++) {
        var l = _mEvListeners[idx[j]];
        try {
          if (typeof l.fn === 'function') l.fn.call(node, ev);
          else if (l.fn && typeof l.fn.handleEvent === 'function') l.fn.handleEvent(ev);
        } catch (_e99d) {}
        if (l.once) {
          var at = _mEvListeners.indexOf(l);
          if (at >= 0) _mEvListeners.splice(at, 1);
        }
      }
      return !(ev && ev.cancelable && ev.defaultPrevented);
    };
    // Parsed local fragments must expose the same geometry API as live element
    // proxies. They have no layout identity until inserted, so the spec fallback
    // is a zero DOMRect rather than a missing method.
    // https://drafts.csswg.org/cssom-view/#dom-element-getboundingclientrect
    node.getBoundingClientRect = function () { return _makeDomRect(0, 0, 0, 0); };
    node.getClientRects = function () { return []; };
    // R3019：hasChildNodes（DOMPurify _sanitizeElements mXSS 检查调 currentNode.hasChildNodes()）。
    node.hasChildNodes = function () { return node.childNodes.length > 0; };
    // js-dom M4 R79：Node.contains / compareDocumentPosition（WPT Node-contains/
    // Node-compareDocumentPosition 的 detached 树 testNodes——foreignPara1/xmlElement 等经
    // _makeDetachedDocument.createElement 建）。经共享 `_zwNodeContains`/
    // `_zwCompareDocumentPosition`（parentNode/childNodes 字段本地维护，链路完整）。
    node.contains = function (other) { return _zwNodeContains(node, other); };
    node.compareDocumentPosition = function (other) { return _zwCompareDocumentPosition(node, other); };
    // js-dom M4 R114：`focus()` / `blur()`（WPT shadow-relatedTarget `root.getElementById
    // ('shadowInput').focus()`——innerHTML 解析的 shadow 子树元素无 focus 抛 TypeError）。
    // 轻量语义：本地 focus 事件经 node.dispatchEvent 派发（listener 可见）+ 更新全局
    // `_zwMElFocused`（document.activeElement 读——无 sel/handle 不能进 _activeElKey 体系，
    // 简化为「最近 focus 的解析节点」；spec activeElement 需布局可聚焦性，headless 近似）。
    // https://html.spec.whatwg.org/#dom-focus
    node.focus = function () {
      // R148（js-dom M4）：焦点所有权统一——解析节点获焦时取代 proxy 焦点态
      //（`_activeElKey` 清空 + 旧 proxy 派 focusout/blur，spec 焦点迁移序），使后续
      // proxy `.focus()` 能识别焦点变化（旧实现只设 `_zwMElFocused`，proxy 视角
      // 焦点未变 → focus no-op，WPT shadow-relatedTarget 第二 subtest pending 悬死）。
      var _r148OldKey = (typeof _activeElKey !== 'undefined') ? _activeElKey : null;
      var _r148OldProxy = (_r148OldKey && _proxyCache[_r148OldKey]) ? _proxyCache[_r148OldKey] : null;
      if (_r148OldKey) _activeElKey = null;
      if (globalThis._zwMElFocused === node) return; // 已聚焦 → no-op（spec 不重派）
      globalThis._zwMElFocused = node;
      if (_r148OldProxy) {
        try { _r148OldProxy.dispatchEvent(_makeEvent('focusout', { bubbles: true, cancelable: false })); } catch (_e148o) {}
      }
      try { node.dispatchEvent(_makeEvent('focus', { bubbles: false, cancelable: false })); } catch (_e114f) {}
      if (_r148OldProxy) {
        try { _r148OldProxy.dispatchEvent(_makeEvent('blur', { bubbles: false, cancelable: false })); } catch (_e148b) {}
      }
    };
    node.blur = function () {
      if (globalThis._zwMElFocused === node) globalThis._zwMElFocused = null;
      try { node.dispatchEvent(_makeEvent('blur', { bubbles: false, cancelable: false })); } catch (_e114b) {}
    };
    // js-dom M4 R117：ChildNode/ParentNode 变异族（WPT pre-insertion-validation-hierarchy 经
    // doc.createElement('a') 取节点后调 prepend/append——_zwMEl 缺方法直接 TypeError）。校验
    // 优先（spec pre-insert：Document/DocumentType 插非 doc → HierarchyRequestError；祖先环）；
    // 实际插入 best-effort（appendChild/childNodes 头插）。self 参数跳过。
    var _r117MVal = function (n) {
      if (!n || typeof n !== 'object') return;
      var nt = n.nodeType | 0;
      if (nt === 9 || nt === 10) {
        throw new (globalThis.DOMException || Error)(
          'Only a Document can contain nodes of type ' + nt + '.', 'HierarchyRequestError');
      }
      var anc = node, hops = 0;
      while (anc && hops++ < 64) {
        if (anc === n) throw new (globalThis.DOMException || Error)(
          'The new node is an ancestor of this node.', 'HierarchyRequestError');
        anc = anc.parentNode;
      }
    };
    node.prepend = function () {
      for (var a = 0; a < arguments.length; a++) _r117MVal(arguments[a]);
      for (var b = arguments.length - 1; b >= 0; b--) {
        var nb = arguments[b];
        if (nb && typeof nb === 'object') { nb.parentNode = node; node.childNodes.unshift(nb); }
        else { var tb = { nodeType: 3, nodeName: '#text', data: String(nb), parentNode: node, get textContent() { return this.data; } }; node.childNodes.unshift(tb); }
      }
    };
    node.append = function () {
      for (var c = 0; c < arguments.length; c++) _r117MVal(arguments[c]);
      for (var d = 0; d < arguments.length; d++) {
        var nd = arguments[d];
        if (nd && typeof nd === 'object') { nd.parentNode = node; node.childNodes.push(nd); }
        else { var td = { nodeType: 3, nodeName: '#text', data: String(nd), parentNode: node, get textContent() { return this.data; } }; node.childNodes.push(td); }
      }
    };
    node.replaceChildren = function () {
      for (var e = 0; e < arguments.length; e++) _r117MVal(arguments[e]);
      node.childNodes.length = 0;
      node.append.apply(node, arguments);
    };
    node.before = function () { /* 轻量节点无父链插入点——校验 only（self/祖先/类型错误仍抛） */
      for (var f = 0; f < arguments.length; f++) _r117MVal(arguments[f]); };
    node.after = function () {
      for (var g = 0; g < arguments.length; g++) _r117MVal(arguments[g]); };
    node.replaceWith = function () {
      for (var h = 0; h < arguments.length; h++) _r117MVal(arguments[h]);
      if (node.parentNode && node.parentNode.removeChild) {
        try { node.parentNode.removeChild(node); } catch (_eR117) {}
      }
    };
    node.remove = function () {
      if (node.parentNode && node.parentNode.removeChild) {
        try { node.parentNode.removeChild(node); } catch (_eR117b) {}
      }
    };
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
    // js-dom M3 R96：setAttributeNS（WPT attributes.html 非 HTML 文档变体在 detached doc 元素上
    // 调 `el.setAttributeNS(ns, qn, v)`——旧缺方法抛 TypeError 整 subtest 崩）。最小语义：忽略 ns 按
    // qualifiedName 存（与元素 proxy NS 族 `_nsQualName` 的「忽略 ns 按限定名」既有近似一致）。
    node.setAttributeNS = function (_ns, qn, v) { node.setAttribute(qn, v); };
    // js-dom M3 R96：`.attributes` NamedNodeMap 视图（WPT attributes.html "include all qualified
    // names"——`Object.getOwnPropertyNames(el.attributes)` 期望 [indices…, qualified names…]，
    // 旧裸 attrs 数组把方法名（length/item/…）当 own keys + named getter 语义全无）。Lazy
    // accessor（每次读返新 Proxy，live 反映 attrs 数组）：length/item/getNamedItem/索引读/named
    // getter/ownKeys（indices+names）/gOPD。Attr 条目经 `_zwMakeAttr`（instanceof Attr 断言面）。
    // 内部 `node.attributes` 数组消费点（2401 序列化 / 285 deepClone）改读 `_zwAttrsRaw()`。
    var _zwAttrsRaw = function () { return attrs; };
    Object.defineProperty(node, 'attributes', {
      get: function () {
        return new Proxy({}, {
          get: function (_t, p) {
            if (p === 'length') return attrs.length;
            if (p === 'item') return function (i) { var k = i | 0; return k >= 0 && k < attrs.length ? _zwMakeAttr(attrs[k].name, attrs[k].value, node) : null; };
            if (p === 'getNamedItem') return function (n) { n = String(n); for (var i = 0; i < attrs.length; i++) if (attrs[i].name === n) return _zwMakeAttr(attrs[i].name, attrs[i].value, node); return null; };
            // R3022/R3023 mutable tree：set/removeNamedItem 经 node.setAttribute/removeAttribute
            //（attrs 数组真变 + IDL 反射），返旧 Attr（_zwMakeAttr 真实例）。
            if (p === 'setNamedItem') return function (a) {
              if (!a || a.name == null) return null;
              var n = String(a.name);
              var old = null;
              for (var si = 0; si < attrs.length; si++) if (attrs[si].name === n) { old = _zwMakeAttr(n, attrs[si].value, node); break; }
              node.setAttribute(n, a.value != null ? String(a.value) : '');
              return old;
            };
            if (p === 'removeNamedItem') return function (n) {
              n = String(n);
              var ex = null;
              for (var ri = 0; ri < attrs.length; ri++) if (attrs[ri].name === n) { ex = _zwMakeAttr(n, attrs[ri].value, node); break; }
              node.removeAttribute(n);
              return ex;
            };
            var idx = parseInt(p, 10);
            if (!isNaN(idx) && String(idx) === String(p) && idx >= 0 && idx < attrs.length) {
              return _zwMakeAttr(attrs[idx].name, attrs[idx].value, node);
            }
            if (typeof p === 'string') {
              for (var j = 0; j < attrs.length; j++) if (attrs[j].name === p) return _zwMakeAttr(attrs[j].name, attrs[j].value, node);
              // Object.prototype 方法回落（R96 同款——for-in own 过滤的 hasOwnProperty 可用）。
              if (p !== 'constructor') {
                var _zwOd = Object.getOwnPropertyDescriptor(Object.prototype, p);
                if (_zwOd) return _zwOd.value;
              }
            }
            return undefined;
          },
          ownKeys: function () {
            var keys = [];
            for (var i = 0; i < attrs.length; i++) keys.push(String(i));
            for (var j = 0; j < attrs.length; j++) keys.push(attrs[j].name);
            return keys;
          },
          // R3018 域：Array 泛型方法（slice/forEach）经 `k in O`（HasProperty）判 hole——缺 has
          // trap 落 target {} 恒 false，索引被当空洞跳过（slice 出稀疏数组，ac[j] undefined）。
          // 与 _attributesProxy 的 has trap 同语义（length + 有效索引）。
          has: function (_t, p) {
            if (p === 'length') return true;
            var idx = parseInt(p, 10);
            return !isNaN(idx) && String(idx) === String(p) && idx >= 0 && idx < attrs.length;
          },
          getOwnPropertyDescriptor: function (_t, p) {
            var idx = parseInt(p, 10);
            if (!isNaN(idx) && String(idx) === String(p) && idx >= 0 && idx < attrs.length) {
              return { value: _zwMakeAttr(attrs[idx].name, attrs[idx].value, node), writable: false, enumerable: true, configurable: true };
            }
            if (typeof p === 'string') {
              for (var j = 0; j < attrs.length; j++) if (attrs[j].name === p) {
                return { value: _zwMakeAttr(attrs[j].name, attrs[j].value, node), writable: false, enumerable: false, configurable: true };
              }
            }
            return undefined;
          }
        });
      },
      configurable: true
    });
    // R86：迭代器 retarget 通知（先于树状态变化——pred/succ 读移除前兄弟/父链）。
    // R126：spec `dom-node-pre-remove` NotFound 校验（WPT Node-removeChild synthetic
    // `s3.removeChild(doc)`——旧静默返 c 不抛）；WebIDL Node 类型校验（null/非 Node TypeError）。
    node.removeChild = function (c) {
      if (c === null || c === undefined || typeof c.nodeType !== 'number') {
        throw new globalThis.TypeError(
          "Failed to execute 'removeChild' on 'Node': parameter 1 is not of type 'Node'.");
      }
      var i = node.childNodes.indexOf(c);
      if (i < 0) {
        throw new (globalThis.DOMException || Error)(
          "Failed to execute 'removeChild' on 'Node': The node to be removed is not a child of this node.",
          'NotFoundError');
      }
      if (globalThis._zwNotifyIteratorsRemove) { try { globalThis._zwNotifyIteratorsRemove(c); } catch (_e86d) {} }
      node.childNodes.splice(i, 1);
      c.parentNode = null;
      return c;
    };
    // R57（FV M1）：createElement 路径的 Constraint Validation API（validator.js
    // 的 ctl 经 document.createElement——R2825 只覆盖 selector-based 的
    // _makeProxy）。node-based 约束计算（getAttribute + value 字段——与
    // _validityState 同语义；customError 经 setCustomValidity）。
    // https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#the-constraint-validation-api
    node.setCustomValidity = function (m) { node._customValidity = m == null ? '' : String(m); };
    Object.defineProperty(node, 'validity', {
      get: function () {
        var hasCustom = node._customValidity != null && node._customValidity !== '';
        var required = node.hasAttribute('required');
        var ty = node.hasAttribute('type') ? String(node.getAttribute('type') || '').toLowerCase() : '';
        var rawValue = node.value != null ? String(node.value)
          : (node.hasAttribute('value') ? String(node.getAttribute('value') || '') : '');
        var valueMissing = false;
        var isSelectTag = String(node.tagName).toLowerCase() === 'select';
        if (required && (!node.hasAttribute('disabled') || ty === 'checkbox' || ty === 'radio'
            || ty === 'file' || isSelectTag)) {
          if (ty === 'checkbox' || ty === 'radio') valueMissing = !node.hasAttribute('checked');
          else if (tag === 'select') valueMissing = rawValue === '';
          else if (ty === 'file') valueMissing = true;
          else {
            if (rawValue.trim() === '') valueMissing = true;
            else if (ty === 'number') {
              valueMissing = !/^[+-]?(\d+(\.\d+)?|\.\d+)([eE][+-]?\d+)?$/.test(rawValue) || !isFinite(parseFloat(rawValue));
            }
            else if (_DATE_TYPES[ty] === 1) valueMissing = !_isValidDateString(rawValue.trim(), ty);
          }
        }
        var patternMismatch = false;
        if (tag === 'textarea' || _PATTERN_TYPES[ty] === 1) {
          var pat = node.getAttribute('pattern');
          if (pat != null && pat !== '' && rawValue !== '') {
            var _re2 = null;
            if (!_isVInvalidPattern(String(pat))) {
              try { _re2 = new RegExp('^(?:' + String(pat) + ')$', 'v'); } catch (_e) {
                try { _re2 = new RegExp('^(?:' + String(pat) + ')$', 'u'); } catch (_e2) { _re2 = null; }
              }
            }
            if (_re2) {
              if (node.hasAttribute('multiple') && (ty === 'email' || ty === 'url')) {
                var _parts2 = rawValue.split(',');
                for (var _pi2 = 0; _pi2 < _parts2.length; _pi2++) {
                  if (!_re2.test(_parts2[_pi2].trim())) { patternMismatch = true; break; }
                }
              } else if (!_re2.test(rawValue)) {
                patternMismatch = true;
              }
            }
          }
        }
        var rangeUnderflow = false, rangeOverflow = false;
        if (ty === 'number' || ty === 'range') {
          var rv = parseFloat(rawValue);
          if (!isNaN(rv)) {
            var minA = node.getAttribute('min');
            var maxA = node.getAttribute('max');
            if (minA != null && String(minA) !== '') {
              var mn = parseFloat(String(minA));
              if (!isNaN(mn) && rv < mn) rangeUnderflow = true;
            }
            if (maxA != null && String(maxA) !== '') {
              var mx = parseFloat(String(maxA));
              if (!isNaN(mx) && rv > mx) rangeOverflow = true;
            }
          }
        }
        var typeMismatch = false;
        if (ty === 'email' || ty === 'url') {
          var tmVal = rawValue.trim();
          if (tmVal !== '') {
            var items = (node.hasAttribute('multiple') && ty === 'email') ? tmVal.split(',') : [tmVal];
            for (var ti = 0; ti < items.length; ti++) {
              var it = items[ti].trim();
              if (it === '' || !(ty === 'email'
                  ? /^[^\s@]+@[^\s@]+$/.test(it)
                  : /^[a-z][a-z0-9+.-]*:/i.test(it))) {
                typeMismatch = true;
                break;
              }
            }
          }
        }
        var _vs2 = {
          valueMissing: valueMissing, typeMismatch: typeMismatch, patternMismatch: patternMismatch,
          tooLong: false, tooShort: false, rangeUnderflow: rangeUnderflow, rangeOverflow: rangeOverflow,
          stepMismatch: false, badInput: false, customError: hasCustom,
          valid: !hasCustom && !valueMissing && !typeMismatch && !patternMismatch && !rangeUnderflow
            && !rangeOverflow,
        };
        try { _vs2[Symbol.toStringTag] = 'ValidityState'; } catch (_e) {}
        return _vs2;
      },
      configurable: true,
    });
    // R57（FV M1）：FORM 的 checkValidity/reportValidity——遍历本地子树控件
    //（_zwMEl 树——createElement + cloneNode + appendChild 的本地对象，
    // validator.js 的 "(in a form)" 变体——host 未注册无法 __zw_query_all_sub）。
    function _collectControls(n, out) {
      if (!n || !n.childNodes) return;
      for (var k = 0; k < n.childNodes.length; k++) {
        var c = n.childNodes[k];
        if (c && c.nodeType === 1) {
          var ct = String(c.tagName || '').toLowerCase();
          if (ct === 'input' || ct === 'select' || ct === 'textarea' || ct === 'button') out.push(c);
          _collectControls(c, out);
        }
      }
    }
    node.checkValidity = function () {
      if (String(node.tagName).toLowerCase() === 'form') {
        var ctrls = [];
        _collectControls(node, ctrls);
        for (var k = 0; k < ctrls.length; k++) {
          if (ctrls[k].checkValidity && !ctrls[k].checkValidity()) return false;
        }
        return true;
      }
      return node.validity.valid;
    };
    node.reportValidity = function () { return node.checkValidity(); };
    // R57（FV M1）：node-based willValidate 排除（disabled/readonly/type barred——
    // 与 part04 的 proxy 版同语义；datalist 祖先 M2）。
    Object.defineProperty(node, 'willValidate', {
      get: function () {
        // datalist 祖先 → barred（willValidate-datalist）
        var _dlp = node.parentNode;
        while (_dlp) {
          if (_dlp.nodeType === 1 && String(_dlp.tagName).toLowerCase() === 'datalist') return false;
          _dlp = _dlp.parentNode;
        }
        if (node.hasAttribute('disabled')) return false;
        var nty = node.hasAttribute('type') ? String(node.getAttribute('type') || '').toLowerCase() : '';
        var ntg2 = String(node.tagName).toLowerCase();
        if (ntg2 === 'fieldset' || ntg2 === 'output' || ntg2 === 'object' || ntg2 === 'legend') return false;
        if (ntg2 === 'button' && (nty === 'button' || nty === 'reset')) return false;
        if (nty === 'hidden' || nty === 'button' || nty === 'reset') return false;
        var ntag = String(node.tagName).toLowerCase();
        if (ntag === 'textarea' || ntag === 'input') {
          if (node.hasAttribute('readonly')) return false;
        }
        return true;
      },
      configurable: true,
    });
    Object.defineProperty(node, 'validationMessage', {
      get: function () {
        if (node._customValidity != null && node._customValidity !== '') return node._customValidity;
        return '';
      },
      configurable: true,
    });
    node.appendChild = function (c) { if (c && c.parentNode) c.parentNode.removeChild(c); node.childNodes.push(c); c.parentNode = node; if (c && c.__zwHandle && typeof _zwUnmarkRemovedHandle === 'function') _zwUnmarkRemovedHandle(c.__zwHandle); return c; };
    // js-dom M4 R81：firstChild/lastChild getter（WPT Node-textContent "set to null" 期望
    // el.firstChild === null——轻量元素缺 getter 使 firstChild 读 undefined）。与上面 textContent
    // getter 同款本地 childNodes 维护。
    Object.defineProperty(node, 'firstChild', { get: function () { return node.childNodes.length ? node.childNodes[0] : null; }, configurable: true });
    Object.defineProperty(node, 'lastChild', { get: function () { return node.childNodes.length ? node.childNodes[node.childNodes.length - 1] : null; }, configurable: true });
    // R3018：insertBefore/replaceChild（DOMPurify 重定位节点、替换用）。ref=null 等价 append。
    node.insertBefore = function (c, ref) {
      if (c && c.parentNode) c.parentNode.removeChild(c);
      if (ref == null) { node.childNodes.push(c); }
      else { var i = node.childNodes.indexOf(ref); if (i < 0) node.childNodes.push(c); else node.childNodes.splice(i, 0, c); }
      c.parentNode = node;
      // R87：入树清移除标记（恢复段 insertBefore 后迭代器重新命中）。
      if (c && c.__zwHandle && typeof _zwUnmarkRemovedHandle === 'function') _zwUnmarkRemovedHandle(c.__zwHandle);
      return c;
    };
    node.replaceChild = function (n, o) {
      // spec 顺序：先 adopt（从原父移除 newChild），再定位 oldChild 当前 index（移除可能前移 oldChild）。
      if (n && n.parentNode) n.parentNode.removeChild(n);
      var i = node.childNodes.indexOf(o);
      if (i < 0) return o;
      // R127：replace-with-self 短路（spec「node is child」——`a.replaceChild(b, b)` 不动）。
      if (n === o) return o;
      node.childNodes[i] = n; n.parentNode = node; o.parentNode = null;
      return o;
    };
    Object.defineProperty(node, 'textContent', { get: function () { var t = ''; for (var i = 0; i < node.childNodes.length; i++) { var c = node.childNodes[i]; if (c.nodeType === 3) t += c.nodeValue; else if (c.nodeType === 1) t += c.textContent; } return t; }, configurable: true });
    Object.defineProperty(node, 'innerHTML', { get: function () { var s = ''; for (var i = 0; i < node.childNodes.length; i++) s += _zwMSerialize(node.childNodes[i]); return s; }, configurable: true });
    Object.defineProperty(node, 'outerHTML', { get: function () { return _zwMSerialize(node); }, configurable: true });
    // js-dom M4 R81：元素导航 getter 族补齐（firstElementChild/lastElementChild/childElementCount
    // + previousElementSibling/nextElementSibling——WPT Node-properties detachedDiv.children[0] 等；
    // 旧只有 children 数组）。与 part04 proxy 的融合视图不同，此处为本地 childNodes 权威。
    Object.defineProperty(node, 'children', { get: function () { return node.childNodes.filter(function (c) { return c.nodeType === 1; }); }, configurable: true });
    Object.defineProperty(node, 'firstElementChild', { get: function () { var k = node.childNodes.filter(function (c) { return c.nodeType === 1; }); return k.length ? k[0] : null; }, configurable: true });
    Object.defineProperty(node, 'lastElementChild', { get: function () { var k = node.childNodes.filter(function (c) { return c.nodeType === 1; }); return k.length ? k[k.length - 1] : null; }, configurable: true });
    Object.defineProperty(node, 'childElementCount', { get: function () { var k = 0; for (var i = 0; i < node.childNodes.length; i++) if (node.childNodes[i].nodeType === 1) k++; return k; }, configurable: true });
    // R81：parentElement（WPT Node-properties xmlElement/detachedXmlElement.parentElement 期望
    // null——detached 无父；挂载后经 parentNode 的元素判定）。
    Object.defineProperty(node, 'parentElement', { get: function () { var p = node.parentNode; return p && p.nodeType === 1 ? p : null; }, configurable: true });
    Object.defineProperty(node, 'previousElementSibling', { get: function () {
      var p = node.parentNode;
      if (!p) return null;
      var k = p.childNodes.filter(function (c) { return c.nodeType === 1; });
      var i = k.indexOf(node);
      return i > 0 ? k[i - 1] : null;
    }, configurable: true });
    Object.defineProperty(node, 'nextElementSibling', { get: function () {
      var p = node.parentNode;
      if (!p) return null;
      var k = p.childNodes.filter(function (c) { return c.nodeType === 1; });
      var i = k.indexOf(node);
      return i >= 0 && i < k.length - 1 ? k[i + 1] : null;
    }, configurable: true });
    Object.defineProperty(node, 'firstChild', { get: function () { return node.childNodes.length ? node.childNodes[0] : null; }, configurable: true });
    Object.defineProperty(node, 'lastChild', { get: function () { return node.childNodes.length ? node.childNodes[node.childNodes.length - 1] : null; }, configurable: true });
    // https://html.spec.whatwg.org/multipage/forms.html#association-of-controls-and-forms
    // Detached HTML fragments still need synchronous form-owner lookup: feature
    // probes commonly set innerHTML and inspect input.form before the host applies
    // its queued DOM mutation.
    Object.defineProperty(node, 'form', { get: function () {
      var formAssociated = { button: 1, fieldset: 1, input: 1, object: 1, output: 1, select: 1, textarea: 1 };
      if (!formAssociated[tag]) return null;
      var formId = node.getAttribute('form');
      var root = node;
      while (root.parentNode) root = root.parentNode;
      function findForm(current) {
        if (!current) return null;
        if (current.nodeType === 1 && current.localName === 'form' &&
            (!formId || current.getAttribute('id') === formId)) return current;
        var children = current.childNodes || [];
        for (var i = 0; i < children.length; i++) {
          var found = findForm(children[i]);
          if (found) return found;
        }
        return null;
      }
      if (formId) return findForm(root);
      for (var ancestor = node.parentNode; ancestor; ancestor = ancestor.parentNode) {
        if (ancestor.nodeType === 1 && ancestor.localName === 'form') return ancestor;
      }
      return null;
    }, configurable: true });
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
  // js-dom M4 R81：firstChild/lastChild getter 补齐（文本/注释节点恒 null——WPT Node-textContent
  // 期望 `emptyText.firstChild === null`；undefined ≠ null 断言失败）。
  function _zwMText(v, parent) { var t = String(v); var n = { nodeType: 3, nodeName: '#text', nodeValue: t, textContent: t, data: t, childNodes: [], children: [], hasChildNodes: function () { return false; }, contains: function (other) { return _zwNodeContains(n, other); }, compareDocumentPosition: function (other) { return _zwCompareDocumentPosition(n, other); }, parentNode: parent || null }; _zwMDefineSiblings(n); Object.defineProperty(n, 'firstChild', { get: function () { return null; }, configurable: true }); Object.defineProperty(n, 'lastChild', { get: function () { return null; }, configurable: true }); Object.defineProperty(n, 'parentElement', { get: function () { var p = n.parentNode; return p && p.nodeType === 1 ? p : null; }, configurable: true }); Object.defineProperty(n, 'length', { get: function () { return n.data.length; }, configurable: true }); Object.defineProperty(n, 'wholeText', { get: function () { var p = n.parentNode; if (!p || !p.childNodes) return n.data; var t2 = ''; var seen = false; for (var i = 0; i < p.childNodes.length; i++) { var c = p.childNodes[i]; if (c === n) seen = true; if (c && c.nodeType === 3) t2 += String(c.data != null ? c.data : ''); } void seen; return t2; }, configurable: true });   try { Object.setPrototypeOf(n, globalThis.Node ? globalThis.Node.prototype : Object.prototype); } catch (_eR117t) {}
  return n; }
  function _zwMComment(v, parent) { var t = String(v); var n = { nodeType: 8, nodeName: '#comment', nodeValue: t, textContent: t, data: t, childNodes: [], children: [], hasChildNodes: function () { return false; }, contains: function (other) { return _zwNodeContains(n, other); }, compareDocumentPosition: function (other) { return _zwCompareDocumentPosition(n, other); }, parentNode: parent || null }; _zwMDefineSiblings(n); Object.defineProperty(n, 'firstChild', { get: function () { return null; }, configurable: true }); Object.defineProperty(n, 'lastChild', { get: function () { return null; }, configurable: true }); Object.defineProperty(n, 'parentElement', { get: function () { var p = n.parentNode; return p && p.nodeType === 1 ? p : null; }, configurable: true }); Object.defineProperty(n, 'length', { get: function () { return n.data.length; }, configurable: true }); return n; }
  // 递归建子树：entry = {k:'E',s:sel}/{k:'T',v}/{k:'C',v}（__zw_parse_html_child_nodes）。元素取快照 + 递归子。
  // R123：`<?...?>` 的 bogus comment（data '?…?'——tokenizer 在首个 '>' 结束并保留 '?' 前缀）
  // 转换为 PI 视图节点（nodeType 7）——Chrome「Parse processing instructions in HTML」后
  // innerHTML 派生 PI 为真 PI 节点（WPT processing-instruction-attributes html-parser source
  // 断言 nodeType 7 + 属性面）。属性面经 _piAttrsView（data 即属性序列化源，同 handle-based PI）。
  function _zwMPiFromBogus(v, parent) {
    var t = String(v);
    var inner = t.charAt(0) === '?' ? t.slice(1) : t;
    if (inner.charAt(inner.length - 1) === '?') inner = inner.slice(0, -1);
    var sp = inner.indexOf(' ');
    var target = sp >= 0 ? inner.slice(0, sp) : inner;
    var data = sp >= 0 ? inner.slice(sp + 1) : '';
    var n = _zwMComment(data, parent);
    n.nodeType = 7;
    n.nodeName = target;
    n.target = target;
    // R123：ownerDocument（WPT check-attribute-value 簇 `pi.ownerDocument.createElement`）+
    // __zwIsText 标记（MutationObserver.observe 对无 sel/handle 节点回落父元素 id 的
    // 既有路径——record.target 仍是 PI 视图节点）。
    n.ownerDocument = globalThis.document;
    n.__zwIsText = true;
    var attrsOf = function () { return _zwPiParseAttrs(n.data) || []; };
    // R123：属性写后发 characterData record 到父 id（WPT mutation-from html-parser 簇——
    // PI 属性变更是 data 变更的观察面，record.target=PI 视图节点）。无父 selector 的
    // detached PI 视图不可观测（与 parsed text 同款限制）。
    var piNotify = function () {
      // 沿 parentNode 链上行找首个 sel/handle 祖先（PI 视图挂在 _zwMEl innerHTML 解析树
      // 下，真正的 id 载体是容器——createElement div 的 handle 或主文档 sel）。
      try {
        if (n.__zwMoSelfKey != null) {
          // 自观测键（XML doc createPI 派生——observe(pi) 落此键，投递直达，无祖先）。
          _mo_deliverToId(n.__zwMoSelfKey, { type: 'characterData', target: n }, false);
          return;
        }
        if (n.__zwFragHostHandle != null) {
          _mo_notify(null, n.__zwFragHostHandle, { type: 'characterData', target: n });
          return;
        }
        var anc = parent, guard = 0;
        while (anc && guard < 12) {
          var aid = _mo_id(anc.__zwHandle, anc.__zwSelector);
          if (aid) { _mo_notify(anc.__zwSelector, anc.__zwHandle, { type: 'characterData', target: n }); return; }
          anc = anc.parentNode; guard++;
        }
      } catch (_eN) {}
    };
    n.hasAttributes = function () { return attrsOf().length > 0; };
    n.getAttributeNames = function () { return attrsOf().map(function (a) { return a[0]; }); };
    n.getAttribute = function (name) {
      var attrs = attrsOf();
      var q = String(name);
      for (var i = 0; i < attrs.length; i++) if (attrs[i][0] === q) return attrs[i][1];
      return null;
    };
    n.hasAttribute = function (name) { return n.getAttribute(name) !== null; };
    n.setAttribute = function (name, value) {
      if (!_zwPiValidName(String(name))) {
        throw new (globalThis.DOMException || Error)(
          "Failed to execute 'setAttribute' on 'ProcessingInstruction': The name provided is not valid.",
          'InvalidCharacterError');
      }
      var attrs = attrsOf();
      var hit = false;
      for (var si = 0; si < attrs.length; si++) {
        if (attrs[si][0] === String(name)) { attrs[si][1] = String(value); hit = true; break; }
      }
      if (!hit) attrs.push([String(name), String(value)]);
      n.data = attrs.map(function (a) { return a[0] + '="' + _zwPiEscape(a[1]) + '"'; }).join(' ');
      n.nodeValue = n.data;
      n.textContent = n.data;
      piNotify();
    };
    n.removeAttribute = function (name) {
      var attrs = attrsOf();
      var out = attrs.filter(function (a) { return a[0] !== String(name); });
      if (out.length === attrs.length) return;
      n.data = out.map(function (a) { return a[0] + '="' + _zwPiEscape(a[1]) + '"'; }).join(' ');
      n.nodeValue = n.data;
      n.textContent = n.data;
      piNotify();
    };
    n.toggleAttribute = function (name, force) {
      if (!_zwPiValidName(String(name))) {
        throw new (globalThis.DOMException || Error)(
          "Failed to execute 'toggleAttribute' on 'ProcessingInstruction': The name provided is not valid.",
          'InvalidCharacterError');
      }
      var hasForce = force !== undefined;
      var attrs = attrsOf();
      var idx = -1;
      for (var ti = 0; ti < attrs.length; ti++) if (attrs[ti][0] === String(name)) { idx = ti; break; }
      var turnOn = hasForce ? Boolean(force) : idx < 0;
      if (turnOn && idx >= 0) return true;
      if (!turnOn && idx < 0) return false;
      if (turnOn) { n.setAttribute(String(name), ''); return true; }
      n.removeAttribute(String(name));
      return false;
    };
    return n;
  }
  function _zwMBuildNode(html, entry, parent) {
    if (entry.k === 'T') return _zwMText(entry.v, parent);
    if (entry.k === 'C') {
      // R123：bogus comment '?…?' 形态 → PI 视图（html-parser source 的 PI 语义）。
      var cv = String(entry.v == null ? '' : entry.v);
      // R123 lit 回归修正（同轮）：lit 模板串尾注入 '<?>' 占位（bundle N 函数
      // `t[s]||"<?>"`）——tokenizer bogus comment data='?'，剥壳后 target 空。转换
      // 会把 lit 的 part marker comment 变形（data 变化使 TreeWalker 的
      // `r.data===marker` 定位失败，首渲染插值全空）。收紧守卫：须 '?target …?'
      // 形态（剥壳后非空 target + 空格分隔）才转 PI 视图，'?'/裸 '?' 壳保回 comment。
      if (cv.charAt(0) === '?') {
        var _piInner = cv.slice(1, cv.length >= 1 && cv.charAt(cv.length - 1) === '?' ? -1 : undefined);
        var _piSp = _piInner.indexOf(' ');
        if (_piSp > 0 && /^[A-Za-z_:][-A-Za-z0-9_:.]*$/.test(_piInner.slice(0, _piSp))) {
          return _zwMPiFromBogus(cv, parent);
        }
      }
      return _zwMComment(cv, parent);
    }
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
  function _zwFragmentAdded(html, hostHandle) {
    if (typeof _zwMBuildBodyTree !== 'function') return [];
    try {
      var kids = _zwMBuildBodyTree(String(html == null ? '' : html)).childNodes;
      // R123：顶层子盖宿主 handle 印章（__zwFragHostHandle）——PI 视图等解析节点上行找
      // sel/handle 祖先时到片段根即断（_zwMEl parentNode=null），印章提供宿主回链
      //（MutationObserver.observe 回落 + piNotify 投递）。
      // R136（js-dom M4）：顶层子 parentNode 重指宿主容器 proxy——旧值是 _zwMBuildBodyTree
      // 的内部 body 快照（tagName=BODY 的 plain object，非任何可达节点），沿 parentNode 上行
      // 的 API（getRootNode/compareDocumentPosition）走到假 body 后断裂（WPT rootNode
      // shadow-including root：shadowChild.getRootNode() 应为 shadowRoot 却命中假 body）。
      // 宿主 proxy 须在印章盖完后取（_wrapHandle 幂等缓存，identity 稳定）。
      if (hostHandle != null) {
        var _r136HostProxy = null;
        try { _r136HostProxy = (typeof _wrapHandle === 'function') ? _wrapHandle(hostHandle) : null; } catch (_e136wp) {}
        for (var i = 0; i < kids.length; i++) {
          if (kids[i]) {
            kids[i].__zwFragHostHandle = hostHandle;
            if (_r136HostProxy) {
              try { kids[i].parentNode = _r136HostProxy; } catch (_e136pp) {}
            }
          }
        }
      }
      return kids;
    } catch (_e) { return []; }
  }
  // js-dom M4 R112：detached doc 的本地派发（WPT Event-dispatch-bubbles "In new Document()"）。
  // 结构：doc → docEl → body（_makeDetachedDocument 的静态树）。listener 存 doc._zwLocalListeners
  //（doc/docEl/body 三入口同表，entry.on 记注册节点）。派发：capture 逆链（doc→docEl→body 反向）→
  // target（AT_TARGET，capture 先）→ bubble 正链（仅 event.bubbles）。eventPhase 按 spec 1/2/3，
  // 结束复位 0 + currentTarget null。once listener 调用前移除（R111 语义）。异常吞（不中断）。
  function _zwDispatchLocalDoc(doc, event) {
    var docEl = doc.documentElement, body = doc.body;
    var chain = [doc, docEl, body]; // 祖先序（浅结构）
    var target = event.target;
    if (!target || (target !== doc && target !== docEl && target !== body)) target = doc;
    event.target = target;
    var idx = chain.indexOf(target);
    // R112：doc 入口注册在 doc._zwLocalListeners（entry.on 区分节点）；docEl/body 入口注册
    // 在各自 _zwEvLs（view 形态，与 _zwParseEl path 派发共享 fire 语义）。两存储都派。
    var ls = (doc._zwLocalListeners || {})[String(event.type)] || [];
    var snap = ls.slice();
    var fire = function (entry, cur, phase) {
      if (ls.indexOf(entry) < 0) return; // 派发中被移除（R111 语义）
      if (entry.once) {
        doc._zwLocalListeners[String(event.type)] = ls.filter(function (e) { return e !== entry; });
        ls = doc._zwLocalListeners[String(event.type)];
        snap = ls.slice();
      }
      event.currentTarget = cur;
      event.eventPhase = phase;
      var callable = typeof entry.fn === 'function' ? entry.fn : (entry.fn && entry.fn.handleEvent);
      if (typeof callable === 'function') {
        try { callable.call(typeof entry.fn === 'function' ? cur : entry.fn, event); } catch (_e) {}
      }
    };
    // R112：view 形态存储（docEl/body 的 _zwEvLs）派发——captureOnly 过滤（phase 2 双 pass
    // 由调用侧控制：先 capture 后非 capture）。
    var fireView = function (view, phase, captureOnly) {
      if (!view || !view._zwEvLs) return;
      var t = String(event.type);
      var vls = view._zwEvLs[t];
      if (!vls) return;
      var vs = vls.slice();
      for (var i = 0; i < vs.length; i++) {
        var entry = vs[i];
        if (captureOnly !== null && captureOnly !== entry.capture) continue;
        var cur2 = view._zwEvLs[t];
        if (!cur2 || cur2.indexOf(entry) < 0) continue;
        if (entry.once) {
          view._zwEvLs[t] = cur2.filter(function (e) { return e !== entry; });
        }
        event.currentTarget = view;
        event.eventPhase = phase;
        var callable2 = typeof entry.fn === 'function' ? entry.fn : (entry.fn && entry.fn.handleEvent);
        if (typeof callable2 === 'function') {
          try { callable2.call(typeof entry.fn === 'function' ? view : entry.fn, event); } catch (_e2) {}
        }
      }
    };
    // doc 站（chain[0]）的注册：doc-local fire；docEl/body 站：view fire（两存储）。
    var stationFire = function (i, phase, captureOnly) {
      var node = chain[i];
      if (node === doc) {
        for (var j = snap.length - 1; j >= 0; j--) {
          if (captureOnly !== null && snap[j].capture !== captureOnly) continue;
          if (snap[j].on === doc) fire(snap[j], doc, phase);
        }
      } else {
        fireView(node, phase, captureOnly);
        // 兼容：docEl/body 经 doc 入口表注册的存量路径（entry.on 匹配）。
        for (var j2 = snap.length - 1; j2 >= 0; j2--) {
          if (captureOnly !== null && snap[j2].capture !== captureOnly) continue;
          if (snap[j2].on === node) fire(snap[j2], node, phase);
        }
      }
    };
    // capture：祖先逆序（远→近，仅 target 之前的节点）
    for (var ci = idx - 1; ci >= 0; ci--) stationFire(ci, 1, true);
    // target：AT_TARGET（2），capture listener 先
    stationFire(idx, 2, true);
    stationFire(idx, 2, false);
    // bubble：祖先正序（仅 event.bubbles）
    if (event.bubbles) {
      for (var bi = idx - 1; bi >= 0; bi--) stationFire(bi, 3, false);
    }
    event.eventPhase = 0;
    event.currentTarget = null;
    return !event._defaultPrevented;
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
    // R130（js-dom M4）：兄弟链接线（doc 级 appendChild/insertBefore/removeChild 后调）——
    // 对 childNodes 数组每项 defineProperty next/previousSibling getter（position 感知，
    // 后续 splice 后再 wire 即正确）。树遍历 oracle（WPT dom/common.js nextNode）依赖。
    function _r130WireSiblings(kids) {
      for (var i = 0; i < kids.length; i++) {
        (function (node, idx) {
          try {
            Object.defineProperty(node, 'nextSibling', {
              get: function () { return idx < kids.length - 1 ? kids[idx + 1] : null; },
              configurable: true,
            });
            Object.defineProperty(node, 'previousSibling', {
              get: function () { return idx > 0 ? kids[idx - 1] : null; },
              configurable: true,
            });
          } catch (_e) {}
        })(kids[i], i);
      }
    }
    // R132：body 属性 NS 元数据（限定名→{ns,prefix,local}）——setAttributeNS 登记，
    // getAttributeNS/getAttributeNodeNS 反查（_zwMEl 树无 __zwHandle，本地表）。
    var _r132BodyAttrNS = {};
    var body = {
      nodeType: 1,
      tagName: 'BODY',
      nodeName: 'BODY',
      localName: 'body',
      namespaceURI: 'http://www.w3.org/1999/xhtml', prefix: null, // R131：同 docEl/headEl
      parentNode: null, // R3017：detached root，parentNode=null（DOMPurify 经 node.parentNode 取父）
      get innerHTML() { return _tree ? _tree.innerHTML : bodyHtml; },
      set innerHTML(v) { bodyHtml = v == null ? '' : String(v); _tree = null; },
      querySelector: function (sel) { return queryOne(sel); },
      querySelectorAll: function (sel) { return queryAll(sel); },
      // R132（js-dom M4）：body 的 set/has/get/removeAttribute 族（WPT Document-importNode
      // "Import an Attr node" 经 `doc.body.setAttributeNS(ns,'p:name','value')`——旧 plain
      // object 无方法直接 TypeError）。存入解析树（ensureTree 后 setAttribute）；NS 元数据
      // 落本地 `_r132BodyAttrNS`（限定名→{ns,prefix,local}）供 getAttributeNS/
      // getAttributeNodeNS 的 (ns,local) 反查（_tree 是 _zwMEl plain object 无
      // __zwHandle，不能复用 _attrNSMeta 键）。
      setAttribute: function (n, v) { ensureTree(); _tree.setAttribute(String(n), String(v == null ? '' : v)); },
      setAttributeNS: function (ns, qn, v) {
        ensureTree();
        var q = String(qn), c = q.indexOf(':');
        _tree.setAttribute(q, String(v == null ? '' : v));
        _r132BodyAttrNS[q] = { ns: ns === '' ? null : String(ns == null ? '' : ns), prefix: c >= 0 ? q.slice(0, c) : null, local: c >= 0 ? q.slice(c + 1) : q };
      },
      getAttribute: function (n) { ensureTree(); return _tree.getAttribute(String(n)); },
      getAttributeNS: function (ns, local) {
        ensureTree();
        for (var gk in _r132BodyAttrNS) {
          if (Object.prototype.hasOwnProperty.call(_r132BodyAttrNS, gk)
            && _r132BodyAttrNS[gk].local === String(local)
            && String(_r132BodyAttrNS[gk].ns == null ? '' : _r132BodyAttrNS[gk].ns) === String(ns == null ? '' : ns)) {
            return _tree.getAttribute(gk);
          }
        }
        return _tree.getAttribute(String(local));
      },
      hasAttribute: function (n) { ensureTree(); return _tree.hasAttribute(String(n)); },
      removeAttribute: function (n) { ensureTree(); _tree.removeAttribute(String(n)); },
      // R132：getAttributeNode/getAttributeNodeNS——Attr 对象（instanceof Attr 经
      // _zwMakeAttr；NS 变体按本地元数据反查 qname 后补 prefix/ns/localName 字段）。
      getAttributeNode: function (n) {
        ensureTree();
        var v = _tree.getAttribute(String(n));
        return v === null || v === undefined ? null : _zwMakeAttr(String(n), v, body);
      },
      getAttributeNodeNS: function (ns, local) {
        ensureTree();
        var qn = null;
        for (var mk in _r132BodyAttrNS) {
          if (Object.prototype.hasOwnProperty.call(_r132BodyAttrNS, mk)
            && _r132BodyAttrNS[mk].local === String(local)
            && String(_r132BodyAttrNS[mk].ns == null ? '' : _r132BodyAttrNS[mk].ns) === String(ns == null ? '' : ns)) {
            qn = mk; break;
          }
        }
        if (qn === null) {
          // NS miss 再按 local 直查限定名（与 proxy NS 读同源的 first-match 近似）
          var v0 = _tree.getAttribute(String(local));
          return v0 === null || v0 === undefined ? null : _zwMakeAttr(String(local), v0, body);
        }
        var v = _tree.getAttribute(qn);
        if (v === null || v === undefined) return null;
        var a = _zwMakeAttr(qn, v, body);
        a.prefix = _r132BodyAttrNS[qn].prefix;
        a.namespaceURI = _r132BodyAttrNS[qn].ns;
        a.localName = _r132BodyAttrNS[qn].local;
        return a;
      },
      //——detached/foreign document 此前缺 → Range-isPointInRange 等 mega-case 5700+ subtest
      // "undefined.createRange" 崩）。_makeRange 在 part06（同一 IIFE，运行期调用时已定义）。
      createRange: function () { return _makeRange(); },
      // R34xx：id 含特殊字符（点号等——canvas WPT 的 id="green.png"）时 '#'+id 选择器
      // 解析错误（点号被当类）→ 改用属性选择器（[id="..."] 精确匹配）。
      getElementById: function (id) { return queryOne('[id="' + String(id).replace(/"/g, '\\"') + '"]'); },
      getElementsByTagName: function (tag) { return queryAll(String(tag)); },
      getElementsByClassName: function (cls) { return queryAll('.' + String(cls)); },
      // R3016/R3017：body.childNodes 递归遍历（cached mutable tree）。DOMPurify.sanitize walk 入口。
      get childNodes() {
        // R140（js-dom M4）：NodeList.item 补挂（spec `dom-nodelist`——WPT Node-childNodes
        // "on a Document." 的 `children.item(0)` 断言；_tree.childNodes 是内部 plain 数组，
        // 每读新数组 → 就地挂 item 不稳定。缓存首个实例（doc 级单例 _tree 不换代）。
        ensureTree();
        if (!globalThis._zwLiveNLCache) globalThis._zwLiveNLCache = {};
        var _r140k = globalThis._zwLiveNLCache;
        var cached = _r140k.__zwDetDoc;
        if (cached && cached.__zwTreeRef === _tree) return cached;
        var arr = _tree.childNodes;
        arr.item = function (i) { i = Number(i) >>> 0; return i < this.length ? this[i] : null; };
        arr.__zwTreeRef = _tree;
        _r140k.__zwDetDoc = arr;
        return arr;
      },
      get children() { ensureTree(); return _tree.childNodes.filter(function (c) { return c.nodeType === 1; }); },
      get firstChild() { ensureTree(); return _tree.childNodes.length ? _tree.childNodes[0] : null; },
      // R81：appendChild 后子的 parentNode 重指 body 自身（_tree 是内部代理树，子挂上去
      // parentNode=_tree ≠ foreignDoc.body——WPT Node-properties foreignPara1.parentNode 期望
      // body 对象 identity）。removeChild 同步清理。
      removeChild: function (c) { ensureTree(); if (globalThis._zwNotifyIteratorsRemove) { try { globalThis._zwNotifyIteratorsRemove(c); } catch (_e86c) {} } var r = _tree.removeChild(c); if (c && c.parentNode === _tree) c.parentNode = null; return r; },
      appendChild: function (c) {
        // js-dom M4 R112：handle 元素（cloneNode 产物等）append 到 detached body——其子树
        // 对查询树（detHtml 串行化）不可见（_zwMSerialize 对 proxy child 的 childNodes 走
        // host 侧数组，序列化深度丢失，WPT Event-dispatch-bubbles createHTMLDocument 变体
        // #table 可查而 #table-body 以下全 NULL）。改为**串行合并**：handle 子的 outerHTML
        // 字符串直接并入 bodyHtml 查询源（innerHTML 序列化保真）。
        if (c && c.nodeType === 1 && c.__zwHandle && typeof __zw_get_inner_html_handle === 'function') {
          try {
            var oTag = String(c.tagName || '').toLowerCase();
            var oih = String(__zw_get_inner_html_handle(c.__zwHandle) || '');
            var oattrs = '';
            try {
              if (typeof __zw_attr_names_handle === 'function') {
                var names = String(__zw_attr_names_handle(c.__zwHandle) || '');
                if (names) {
                  names.split('|').filter(Boolean).forEach(function (n) {
                    var v = typeof __zw_get_attr_handle === 'function' ? __zw_get_attr_handle(c.__zwHandle, n) : '';
                    oattrs += ' ' + n + '="' + String(v == null ? '' : v).replace(/"/g, '&quot;') + '"';
                  });
                }
              }
            } catch (_eA2) {}
            var frag = '<' + oTag + oattrs + '>' + oih + '</' + oTag + '>';
            bodyHtml = (_tree ? _tree.innerHTML : bodyHtml) + frag;
            _tree = null;
            return c;
          } catch (_e112d) { /* 回落通用路径 */ }
        }
        ensureTree(); var r = _tree.appendChild(c); if (c && c.parentNode === _tree) c.parentNode = body; return r;
      },
      // js-dom M4 R87：body 的 insertBefore（WPT NodeIterator-removal 恢复段经
      // oldParent.insertBefore——foreignPara 的父是 body 代理）。ref=null 等价 append；
      // _tree.insertBefore（R3018）已支持定位插入。
      insertBefore: function (c, ref) { ensureTree(); var r = _tree.insertBefore(c, ref); if (c && c.parentNode === _tree) c.parentNode = body; return r; },
      // R81：body 的 firstChild/lastChild getter 补齐（WPT Node-properties 经 body 子导航；
      // 旧只有 firstChild）。lastChild 同 childNodes 末端。
      get lastChild() { ensureTree(); return _tree.childNodes.length ? _tree.childNodes[_tree.childNodes.length - 1] : null; },
      // R81：hasChildNodes（WPT NodeIterator 经 common.js nextNode/previousNode 统一调
      // node.hasChildNodes()——旧 body 无此方法 → "node.hasChildNodes is not a function"
      // traversal -21 回归根因）。
      hasChildNodes: function () { ensureTree(); return _tree.childNodes.length > 0; },
      // R81：body.parentNode → docEl（spec：body 的父是 html；WPT foreignPara1.parentNode 期望
      // foreignDoc.body——经 _tree.appendChild 设的 parentNode 是 _tree（body 代理自身），正确；
      // 此处补 body 自身导航一致性）。
      get parentNode() { return docEl; },
      get parentElement() { return docEl; },
      get firstElementChild() { ensureTree(); var k = _tree.childNodes.filter(function (c) { return c.nodeType === 1; }); return k.length ? k[0] : null; },
      get lastElementChild() { ensureTree(); var k = _tree.childNodes.filter(function (c) { return c.nodeType === 1; }); return k.length ? k[k.length - 1] : null; }
    };
    // R81：docEl/headEl 补 hasChildNodes/firstChild/lastChild（WPT NodeIterator 经 common.js
    // nextNode(node) 对树节点统一调 node.hasChildNodes()——旧 docEl 无此方法 → undefined 崩
    // "Cannot read properties of undefined (reading 'hasChildNodes')"——实际崩点在 prototype
    // 链上缺失（docEl.childNodes 数组存在但 hasChildNodes 函数缺失）。
    var docEl = { nodeType: 1, tagName: 'HTML', nodeName: 'HTML', localName: 'html',
      // R131：HTML ns 显式标注（spec：HTML 文档的 html/head/body 均 XHTML ns——
      // isEqualNode 的 ns 字段比较与 d3 createElement('head') 产物[ns=XHTML]对齐，
      // WPT "default HTML documents, created different ways" 断言）。
      namespaceURI: 'http://www.w3.org/1999/xhtml', prefix: null,
      childNodes: [], hasChildNodes: function () { return docEl.childNodes.length > 0; }, get firstChild() { return docEl.childNodes.length ? docEl.childNodes[0] : null; }, get lastChild() { return docEl.childNodes.length ? docEl.childNodes[docEl.childNodes.length - 1] : null; },
      // R126：docEl 的 mutation 面（WPT Node-removeChild synthetic 变体
      // `doc.documentElement.appendChild(s)`——docEl 旧无 appendChild 直接 TypeError）。
      // appendChild relink parentNode（childNodes 视图 + 父链一致）；removeChild 带
      // spec `dom-node-pre-remove` NotFound 校验（子判定走本对象 childNodes identity）。
      appendChild: function (c) {
        if (!c) return c;
        if (c.nodeType === 11) {
          var fk = c.childNodes || [];
          for (var fi = 0; fi < fk.length; fi++) this.appendChild(fk[fi]);
          return c;
        }
        if (c.parentNode && c.parentNode.removeChild) { try { c.parentNode.removeChild(c); } catch (_e126a) {} }
        c.parentNode = docEl;
        this.childNodes.push(c);
        return c;
      },
      removeChild: function (c) {
        if (c === null || c === undefined || typeof c.nodeType !== 'number') {
          throw new globalThis.TypeError(
            "Failed to execute 'removeChild' on 'Node': parameter 1 is not of type 'Node'.");
        }
        for (var i = 0; i < this.childNodes.length; i++) {
          if (this.childNodes[i] === c) {
            if (globalThis._zwNotifyIteratorsRemove) {
              try { globalThis._zwNotifyIteratorsRemove(c); } catch (_e126b) {}
            }
            this.childNodes.splice(i, 1);
            if (c.parentNode === docEl) c.parentNode = null;
            return c;
          }
        }
        throw new (globalThis.DOMException || Error)(
          "Failed to execute 'removeChild' on 'Node': The node to be removed is not a child of this node.",
          'NotFoundError');
      },
      insertBefore: function (c, ref) {
        if (!c) return c;
        if (c.parentNode && c.parentNode.removeChild) { try { c.parentNode.removeChild(c); } catch (_e126c) {} }
        var i = ref ? this.childNodes.indexOf(ref) : -1;
        if (i < 0) { c.parentNode = docEl; this.childNodes.push(c); }
        else { c.parentNode = docEl; this.childNodes.splice(i, 0, c); }
        return c;
      } };
    // R130（js-dom M4）：docEl/headEl/body 原型接线对应 HTML 接口（WPT
    // DOMImplementation-createHTMLDocument test 0-8 断言 `documentElement instanceof
    // HTMLHtmlElement` / head instanceof HTMLHeadElement——detached doc 旧 plain object
    // 恒 false）。链到 HTMLElement.prototype（含 Element→Node 链，instanceof
    // Element/Node 同真）；HTMLHeadElement 等子类构造器占位已注册（R11 列表）。
    try {
      if (globalThis.HTMLHtmlElement && globalThis.HTMLHtmlElement.prototype) {
        Object.setPrototypeOf(docEl, globalThis.HTMLHtmlElement.prototype);
      }
    } catch (_e130a) {}
    var headEl = { nodeType: 1, tagName: 'HEAD', nodeName: 'HEAD', localName: 'head', namespaceURI: 'http://www.w3.org/1999/xhtml', prefix: null, childNodes: [], hasChildNodes: function () { return headEl.childNodes.length > 0; }, get firstChild() { return headEl.childNodes.length ? headEl.childNodes[0] : null; }, get lastChild() { return headEl.childNodes.length ? headEl.childNodes[headEl.childNodes.length - 1] : null; } };
    try {
      if (globalThis.HTMLHeadElement && globalThis.HTMLHeadElement.prototype) {
        Object.setPrototypeOf(headEl, globalThis.HTMLHeadElement.prototype);
      }
    } catch (_e130b) {}
    // R130：body 同款接线（HTMLBodyElement）。
    try {
      if (globalThis.HTMLBodyElement && globalThis.HTMLBodyElement.prototype) {
        Object.setPrototypeOf(body, globalThis.HTMLBodyElement.prototype);
      }
    } catch (_e130c) {}
    // R130：docEl 子树兄弟导航（WPT dom/common.js nextNode oracle 遍历依赖
    // head/body/title 的 nextSibling/previousSibling/parentNode 链——title 子加入 head 后
    // oracle 从 title-text 回溯经 head.nextSibling 须到 body；旧 headEl/body 无 sibling
    // getter 遍历在 head 断链）。
    try {
      Object.defineProperty(headEl, 'nextSibling', { get: function () { return body; }, configurable: true });
      Object.defineProperty(headEl, 'previousSibling', { get: function () { return null; }, configurable: true });
      Object.defineProperty(headEl, 'parentNode', { get: function () { return docEl; }, configurable: true });
      Object.defineProperty(body, 'nextSibling', { get: function () { return null; }, configurable: true });
      Object.defineProperty(body, 'previousSibling', { get: function () { return headEl; }, configurable: true });
      Object.defineProperty(body, 'parentNode', { get: function () { return docEl; }, configurable: true });
      docEl.childNodes.push(headEl, body);
      headEl.parentNode = docEl;
      body.parentNode = docEl;
      var _r130Kids = [headEl, body];
      for (var _r130si = 0; _r130si < _r130Kids.length; _r130si++) {
        (function (idx) {
          var k = _r130Kids[idx];
          Object.defineProperty(k, 'nextSibling', {
            get: function () { return idx < _r130Kids.length - 1 ? _r130Kids[idx + 1] : null; },
            configurable: true,
          });
          Object.defineProperty(k, 'previousSibling', {
            get: function () { return idx > 0 ? _r130Kids[idx - 1] : null; },
            configurable: true,
          });
        })(_r130si);
      }
    } catch (_e130s) {}
    // R130：head 的 title 子（WPT createHTMLDocument test 0-8 断言
    // `head.childNodes.length === 1` + `title.firstChild.data === expectedtitle`——
    // spec createHTMLDocument 步骤 4「create a title element, append to head」；title
    // 参数 undefined 时**不建** title（test 2 走 else 分支期望 head.childNodes.length
    // === 0）。title 元素原型 HTMLTitleElement + 文本子 data 经 String 转换（null→
    // 'null'——test 1 期望 "null"）。
    if (title !== undefined) {
      var _r130TitleText = {
        nodeType: 3, nodeName: '#text',
        data: String(title), nodeValue: String(title),
        get textContent() { return this.data; },
        childNodes: [], parentNode: null,
        // R130：叶子导航面（oracle nextNode(node) 统一调 node.hasChildNodes()——缺方法
        // 崩 "node.hasChildNodes is not a function"，native traversal foreignDoc 20F 回归）。
        hasChildNodes: function () { return false; },
        get firstChild() { return null; },
        get lastChild() { return null; },
      };
      var _r130TitleEl = {
        nodeType: 1, tagName: 'TITLE', nodeName: 'TITLE', localName: 'title',
        childNodes: [_r130TitleText], parentNode: headEl,
        get firstChild() { return this.childNodes.length ? this.childNodes[0] : null; },
        get lastChild() { return this.childNodes.length ? this.childNodes[this.childNodes.length - 1] : null; },
        hasChildNodes: function () { return this.childNodes.length > 0; },
      };
      _r130TitleText.parentNode = _r130TitleEl;
      try {
        if (globalThis.HTMLTitleElement && globalThis.HTMLTitleElement.prototype) {
          Object.setPrototypeOf(_r130TitleEl, globalThis.HTMLTitleElement.prototype);
        }
      } catch (_e130d) {}
      headEl.childNodes.push(_r130TitleEl);
    }
    var doc = {
      nodeType: 9,
      nodeName: '#document',
      // R130（js-dom M4）：documentElement 惰性 getter（WPT DOMImplementation-createDocument
      // 的 doc.documentElement 断言族——spec：首个元素子，无元素子时 null。旧静态 docEl 使
      // createDocument 建的 root 元素不可见 + 空 doc 恒返伪 docEl 非 null）。createHTMLDocument
      // 路径 appendChild(docEl) 后首个元素子即 docEl，行为不变。
      get documentElement() {
        var kids = this.childNodes || [];
        for (var i = 0; i < kids.length; i++) {
          var k = kids[i];
          // R130 回归修正：proxy 形态的元素子（主文档克隆/移动的 html——`new Document()
          // .appendChild(document.documentElement.cloneNode(true))` R112 派发形态）保持
          // 返内部 docEl——R112 事件面/查询树以 docEl 为站点（tag registry / bodyHtml
          // 并入），换返克隆 proxy 会使 addEventListener 站点与派发链脱钩（Event-
          // dispatch-bubbles "In new Document()" 4 站丢失回归）。plain-object 子
          // （createDocument 经本 doc createElementNS 自建 root——_zwMEl 产物）走首
          // 元素子（spec documentElement 断言族）。
          if (k && k.nodeType === 1) {
            // handle 形态是 string（_elKey '@'+handle）；sel 形态是 string。两者都非
            // string（null/undefined）= plain-object 子（_zwMEl 自建 root）。
            if (typeof k.__zwSelector === 'string' || typeof k.__zwHandle === 'string') return docEl;
            return k;
          }
        }
        // R130：HTML/未定型文档未挂载时返内部 docEl（createHTMLDocument 的
        // appendChild(documentElement) 自引用读——首个元素子未挂载前须拿到 docEl 本体；
        // cloneNode/new Document 等未设 contentType 的消费方同源）。XML 文档
        //（createDocument 已设 'application/xml' 等）omit root 时 spec 期望 null。
        if (typeof this.contentType !== 'string'
            || (this.contentType.indexOf('html') >= 0
                && this.contentType !== 'application/xhtml+xml')) {
          return docEl;
        }
        return null;
      },
      head: headEl,
      body: body,
      title: title != null ? String(title) : '',
      // js-dom M4 R79：`doc.doctype`（WPT common.js `foreignDoctype = foreignDoc.doctype`——
      // createHTMLDocument 的 doctype 缺省 html；appendChild(doctype) 后 parentNode=doc 进链）。
      // 惰性绑定（字面量求值期 doc 未赋值完毕）。
      get doctype() {
        for (var i = 0; i < this.childNodes.length; i++) {
          if (this.childNodes[i] && this.childNodes[i].nodeType === 10) return this.childNodes[i];
        }
        return null;
      },
      // R51：detached doc 的文档级子列表（common.js setupRangeTests `xmlDoc.appendChild(...)`
      // 建元素/PI/comment——detached 文档无渲染，纯本地列表即可支撑 testNodes 组装/遍历）。
      // R140（js-dom M4）：childNodes 挂 NodeList.item（spec `dom-nodelist`——WPT
      // Node-childNodes "on a Document." 的 children.item(0) 断言；append push 本数组，
      // item 读实时长度）。
      childNodes: (function () {
        var _a = [];
        _a.item = function (i) { i = Number(i) >>> 0; return i < this.length ? this[i] : null; };
        return _a;
      })(),
      children: (function () {
        var _c = [];
        _c.item = function (i) { i = Number(i) >>> 0; return i < this.length ? this[i] : null; };
        return _c;
      })(),
      get firstChild() { return this.childNodes.length ? this.childNodes[0] : null; },
      get lastChild() { return this.childNodes.length ? this.childNodes[this.childNodes.length - 1] : null; },
      hasChildNodes: function () { return this.childNodes.length > 0; },
      // js-dom M4 R79：Node.contains / compareDocumentPosition（detached doc 作 root 进链）。
      contains: function (other) { return _zwNodeContains(doc, other); },
      compareDocumentPosition: function (other) { return _zwCompareDocumentPosition(doc, other); },
      // R81：Document 的 nodeValue/textContent 恒 null（spec dom-node-textcontent）+ setter no-op。
      get nodeValue() { return null; },
      set nodeValue(_v) {},
      get textContent() { return null; },
      set textContent(_v) {},
      // js-dom M4 R81：Document 元数据族（WPT Node-properties foreignDoc.URL/compatMode/
      // characterSet/inputEncoding/documentURI/charset——detached doc 旧全 undefined）。
      // spec：createHTMLDocument 的 URL = about:blank + CSS1Compat + UTF-8；XML doc 无 compatMode。
      get URL() { return 'about:blank'; },
      // R130（js-dom M4）：`doc.location`（WPT createHTMLDocument "document location getter
      // is null"——spec：非浏览上下文文档的 location getter 返 null；旧 undefined ≠ null）。
      get location() { return null; },
      get documentURI() { return 'about:blank'; },
      // R81 spec 纠正：XML/HTML 文档 compatMode 恒 CSS1Compat（spec dom-document-compatmode：
      // 没有 quirks 触发条件（backwards-compatible 解析）时恒 "CSS1Compat"；XML 文档无 quirks
      // 模式——WPT Node-properties xmlDoc.compatMode 期望 "CSS1Compat"）。
      get compatMode() { return 'CSS1Compat'; },
      get characterSet() { return 'UTF-8'; },
      get charset() { return 'UTF-8'; },
      get inputEncoding() { return 'UTF-8'; },
      get parentElement() { return null; },
      get parentNode() { return null; },
      get ownerDocument() { return null; },
      get nextSibling() { return null; },
      get previousSibling() { return null; },
      appendChild: function (c) {
        if (!c) return c;
        // js-dom M4 R119：DocumentFragment 展平（spec dom-node-append-child 对 fragment
        // 逐子 pre-insert 后清空 fragment——WPT replaceChildren「with a DocumentFragment
        // containing a single element」期望 doc.childNodes = [el] 非 [df]）。
        if (c.nodeType === 11) {
          var fk = c.childNodes || [];
          var fc = fk.slice();
          for (var fi = 0; fi < fc.length; fi++) this.appendChild(fc[fi]);
          fk.length = 0;
          return c;
        }
        if (c.parentNode && c.parentNode.removeChild) { try { c.parentNode.removeChild(c); } catch (_e) {} }
        c.parentNode = this;
        this.childNodes.push(c);
        if (c.nodeType === 1) this.children.push(c);
        // R130：doc 级子的 sibling 链维护（WPT dom/common.js nextNode oracle 遍历依赖
        // doctype/comment/元素子在 doc.childNodes 内的 next/previousSibling——旧 append
        // 只 push，兄弟导航断链使 oracle 与 iterator 分歧）。
        _r130WireSiblings(this.childNodes);
        // js-dom M4 R112：append HTML 元素（documentElement 克隆）→ 其子树并入可查询树
        //（WPT Event-dispatch-bubbles "In new Document()"：`new Document().appendChild(
        // document.documentElement.cloneNode(true))` 后 getElementById/getElementsByTagName
        // 须命中克隆子树内容。真实 DOM：doc 无子时 append html 元素即 documentElement，
        // 其内容可查）。实现：克隆的 innerHTML（含 head+body）→ 提取 body 内容并入查询源
        //（bodyHtml 串行化——查询走 detHtml → __zw_parse_html_query）。handle 克隆的
        // childNodes 走 host 侧（proxy 数组非实时），innerHTML getter 是可靠源。
        if (c.nodeType === 1 && String(c.tagName || '').toUpperCase() === 'HTML') {
          try {
            var cih = c.innerHTML != null ? String(c.innerHTML) : '';
            if (cih) {
              var mBody = cih.match(/<body[^>]*>([\s\S]*?)<\/body>/i);
              var chtml = mBody ? mBody[1] : cih;
              if (chtml && chtml.trim()) {
                bodyHtml = chtml; // 查询树源更新（_tree 惰性重建）
                _tree = null;
              }
            }
          } catch (_e112a) {}
        }
        return c;
      },
      removeChild: function (c) {
        for (var i = 0; i < this.childNodes.length; i++) {
          if (this.childNodes[i] === c) {
            // R86：迭代器 retarget 通知（先于树状态变化——pred/succ 读移除前链）。
            if (globalThis._zwNotifyIteratorsRemove) {
              try { globalThis._zwNotifyIteratorsRemove(c); } catch (_e86) {}
            }
            this.childNodes.splice(i, 1);
            var ci = this.children.indexOf(c);
            if (ci >= 0) this.children.splice(ci, 1);
            c.parentNode = null;
            // R130：sibling 链重连（移除后剩余子的 position 偏移）。
            _r130WireSiblings(this.childNodes);
            return c;
          }
        }
        return c;
      },
      // js-dom M4 R87：detached doc 的 insertBefore（spec dom-node-pre-insert；WPT
      // NodeIterator-removal 的恢复段 `oldParent.insertBefore(node, oldSibling)`——
      // xmlDoc/foreignDoc 缺此方法直接 TypeError 崩用例）。ref=null 等价 append。
      insertBefore: function (c, ref) {
        if (!c) return c;
        if (c.parentNode && c.parentNode.removeChild) { try { c.parentNode.removeChild(c); } catch (_e87) {} }
        if (ref == null) {
          c.parentNode = this;
          this.childNodes.push(c);
          if (c.nodeType === 1) this.children.push(c);
        } else {
          var i = this.childNodes.indexOf(ref);
          if (i < 0) {
            c.parentNode = this;
            this.childNodes.push(c);
            if (c.nodeType === 1) this.children.push(c);
          } else {
            c.parentNode = this;
            this.childNodes.splice(i, 0, c);
            if (c.nodeType === 1) {
              var ri = this.children.indexOf(ref);
              this.children.splice(ri < 0 ? this.children.length : ri, 0, c);
            }
          }
        }
        return c;
      },
      querySelector: function (sel) { return queryOne(sel); },
      querySelectorAll: function (sel) { return queryAll(sel); },
      // js-dom M4 R112：doc 级 getElementsByTagName/ClassName（WPT Event-dispatch-bubbles
      // targetsForDocumentChain `document.getElementsByTagName("body")[0]`——doc 旧只有
      // querySelector 族，此二方法只在 body 上）。语义同 body：查 detHtml 树。
      getElementsByTagName: function (tag) { return queryAll(String(tag)); },
      getElementsByClassName: function (cls) { return queryAll('.' + String(cls)); },
      // R112：doc 级 getElementById（同 R34xx 属性选择器形态——id 特殊字符安全）。
      getElementById: function (id) { return queryOne('[id="' + String(id).replace(/"/g, '\\"') + '"]'); },
      // R112：doc 级 createEvent（WPT Event-dispatch-bubbles testChain
      // `document.createEvent("Event")`——detached doc 缺此方法直接 TypeError）。委托
      // 主 document 的 createEvent（事件对象本身与文档无关）。
      createEvent: function (type) { return globalThis.document.createEvent(type); },
      // js-dom M4 R112：detached doc 的事件面（WPT Event-dispatch-bubbles-true/false
      // "In new Document()" / "In DOMImplementation.createHTMLDocument()"——targets 含
      // doc/docEl/body，三者 addEventListener 缺失直接 TypeError）。detached doc 不经
      // host selector 派发链——用**本地 listener 表 + 本地派发**（capture/target/bubble
      // 三阶段沿 doc→docEl→body 静态结构，event.currentTarget/eventPhase 按 spec 设置）。
      // spec https://dom.spec.whatwg.org/#concept-event-dispatch
      _zwLocalListeners: {},
      addEventListener: function (type, fn, opts) {
        var t = String(type);
        if (!doc._zwLocalListeners[t]) doc._zwLocalListeners[t] = [];
        var cap = opts != null && typeof opts === 'object' ? !!opts.capture : !!opts;
        var once = opts != null && typeof opts === 'object' ? !!opts.once : false;
        doc._zwLocalListeners[t].push({ fn: fn, capture: cap, once: once, on: doc });
      },
      removeEventListener: function (type, fn, opts) {
        var t = String(type);
        var cap = opts != null && typeof opts === 'object' ? !!opts.capture : !!opts;
        var ls = doc._zwLocalListeners[t];
        if (!ls) return;
        doc._zwLocalListeners[t] = ls.filter(function (l) { return !(l.fn === fn && l.capture === cap); });
      },
      dispatchEvent: function (event) {
        globalThis._zwDispatchGuard(event);
        return _zwDispatchLocalDoc(doc, event);
      },
      // R3018：createElement/createTextNode 返完整可变节点（_zwMEl/_zwMText），非 hollow stub。
      // DOMPurify / 模板引擎经 createElement 建替换节点后 insertBefore/appendChild 入树，须支持 parentNode/
      // sibling/childNodes/setAttribute/序列化全套语义。HTML 文档 tagName 大写、localName 小写。
      // R51：产物补 ownerDocument=本 detached doc（spec ownerDocument 语义；common.js
      // rangeFromEndpoints 经 ownerDocument(node).createRange()——缺此字段时 undefined 崩）。

      // js-dom M4 R116：`createAttribute` / `createAttributeNS`（detached doc——WPT
      // Document-createAttribute 的 xml_document = implementation.createDocument(...)）。
      // 空名 InvalidCharacterError；HTML-ness 按 contentType（缺省 HTML；XML 变体保持大小写）。
      createAttribute: function (name) {
        var t = String(name);
        if (t === '') {
          throw new (globalThis.DOMException || Error)(
            "Failed to execute 'createAttribute' on 'Document': The name provided is empty.",
            'InvalidCharacterError');
        }
        var isHtmlDoc = !(typeof doc.contentType === 'string' && doc.contentType.indexOf('html') < 0);
        var n = isHtmlDoc ? t.replace(/[A-Z]/g, function (c) { return String.fromCharCode(c.charCodeAt(0) + 32); }) : t;
        return _zwMakeAttr(n, '', null);
      },
      // R117：replaceChild/insertBefore（detached doc——WPT Node-replaceChild "context is a
      // document" 校验路径）。校验：child NotFound（doc.childNodes 判）→ node 类型（Document 插
      // doc → HRE）。
      replaceChild: function (newChild, oldChild) {
        if (newChild == null || oldChild == null) {
          throw new globalThis.TypeError(
            "Failed to execute 'replaceChild' on 'Node': parameter is not of type 'Node'.");
        }
        var kids = doc.childNodes || [];
        var found = false;
        for (var i = 0; i < kids.length; i++) if (kids[i] === oldChild) { found = true; break; }
        if (!found) {
          throw new (globalThis.DOMException || Error)(
            "Failed to execute 'replaceChild' on 'Node': The node to be replaced is not a child of this node.",
            'NotFoundError');
        }
        var nnt = newChild.nodeType | 0;
        if (nnt === 3 || nnt === 4 || nnt === 9) {
          throw new (globalThis.DOMException || Error)(
            'Nodes of type ' + nnt + ' cannot be inserted into a Document.', 'HierarchyRequestError');
        }
        // R127：Document pre-insert step 6「给定当前子」校验（kids + oldChild——
        // WPT Node-replaceChild fragment 多元素/element 重复/doctype 位置 8 用例）。
        _r127DocPreInsertCheck(newChild, kids, oldChild);
        // R127：spec replace 语义——先 adopt（new 是 old 兄弟时移除不影响 old 位），
        // 再定位 splice。fragment flatten（doc.childNodes = [...df 子] 而非 df 本身）。
        if (newChild.parentNode && typeof newChild.parentNode.removeChild === 'function') {
          try { newChild.parentNode.removeChild(newChild); } catch (_e127g) {}
        }
        var idx = -1;
        for (var j = 0; j < doc.childNodes.length; j++) {
          if (doc.childNodes[j] === oldChild) { idx = j; break; }
        }
        if (idx < 0) return oldChild;
        if (newChild === oldChild) return oldChild;
        if (nnt === 11) {
          var fk = newChild.childNodes || [];
          var fc = fk.slice();
          doc.childNodes.splice(idx, 1);
          for (var q = 0; q < fc.length; q++) {
            doc.childNodes.splice(idx + q, 0, fc[q]);
            fc[q].parentNode = doc;
            if (fc[q].nodeType === 1) {
              var qi = doc.children.indexOf(fc[q]);
              if (qi < 0) doc.children.push(fc[q]);
            }
          }
          fk.length = 0;
        } else {
          doc.childNodes.splice(idx, 1, newChild);
          newChild.parentNode = doc;
          // R127：spec `concept-node-adopt`——replace 入 doc 的节点 ownerDocument
          // 重指本 doc（WPT Node-replaceChild "inserting a new doctype should work"
          // `doctype2.ownerDocument === doc` 断言——跨 detached doc 移动后归属变更）。
          try {
            newChild.ownerDocument = doc;
          } catch (_e127h) {}
          if (nnt === 1) {
            var ni = doc.children.indexOf(newChild);
            if (ni < 0) doc.children.push(newChild);
            var oi = doc.children.indexOf(oldChild);
            if (oi >= 0) doc.children.splice(oi, 1);
          }
        }
        oldChild.parentNode = null;
        return oldChild;
      },
      insertBefore: function (newNode, refNode) {
        if (newNode == null) {
          throw new globalThis.TypeError(
            "Failed to execute 'insertBefore' on 'Node': parameter 1 is not of type 'Node'.");
        }
        var nnt2 = newNode.nodeType | 0;
        if (nnt2 === 3 || nnt2 === 4 || nnt2 === 9) {
          throw new (globalThis.DOMException || Error)(
            'Nodes of type ' + nnt2 + ' cannot be inserted into a Document.', 'HierarchyRequestError');
        }
        if (refNode) {
          var kids2 = doc.childNodes || [];
          for (var k = 0; k < kids2.length; k++) {
            if (kids2[k] === refNode) { doc.childNodes.splice(k, 0, newNode); _r130WireSiblings(doc.childNodes); return newNode; }
          }
          throw new (globalThis.DOMException || Error)(
            "Failed to execute 'insertBefore' on 'Node': The node before which the new node is to be inserted is not a child of this node.",
            'NotFoundError');
        }
        doc.childNodes.push(newNode);
        _r130WireSiblings(doc.childNodes);
        return newNode;
      },
      createAttributeNS: function (ns, qualifiedName) {
        var q = String(qualifiedName);
        if (q === '') {
          throw new (globalThis.DOMException || Error)(
            "Failed to execute 'createAttributeNS' on 'Document': The name provided is empty.",
            'InvalidCharacterError');
        }
        var a = _zwMakeAttr(q, '', null);
        a.namespaceURI = ns != null ? String(ns) : null;
        var colon = q.indexOf(':');
        if (colon > 0) {
          a.prefix = q.slice(0, colon);
          a.localName = q.slice(colon + 1);
        } else {
          a.prefix = null;
          a.localName = q;
        }
        return a;
      },      // js-dom M4 R81：产物补 namespaceURI/prefix/nodeValue（spec：元素 ns 由文档派生——HTML doc →
      // HTML ns，XML doc → null；WPT Document-createElement-namespace "Created element's namespace
      // in created HTML/XML/XHTML/SVG/MathML document" 簇）。`_docNS` 由 createDocument/
      // createHTMLDocument 按调用参数设（HTML ns 或 null）。
      createElement: function (t) {
        // R81 spec 纠正：XML 文档（_docNS null/undefined 且非 HTML doc）createElement 不小写
        // 不大写（WPT Node-properties xmlElement.tagName 期望原样 "igiveuponcreativenames"）。
        // HTML 文档（_docNS = HTML ns）保持小写输入 + tagName 大写（HTML 语义）。
        // R130：HTML-ness 按 contentType（spec dom-document-createelement「If document
        // is an HTML document」= contentType 'text/html'——XHTML createDocument 产物是
        // XML 语义文档，createElement 不小写；WPT createDocument metadata for XHTML
        // `createElement('DIV').localName === 'DIV'`。旧按 _docNS 判把 XHTML 当 HTML）。
        var _isHtmlDoc = !(typeof doc.contentType === 'string' && doc.contentType.indexOf('html') < 0)
          && doc.contentType !== 'application/xhtml+xml';
        var _tagIn = String(t);
        var e = _zwMEl({ tag: _isHtmlDoc ? _tagIn.toLowerCase() : _tagIn, preserveCase: !_isHtmlDoc }, null);
        e.ownerDocument = doc;
        e.namespaceURI = (doc._docNS !== undefined) ? doc._docNS : 'http://www.w3.org/1999/xhtml';
        e.prefix = null;
        e.nodeValue = null;
        return e;
      },
      createTextNode: function (t) { var n = _zwMText(String(t), null); n.ownerDocument = doc; return n; },
      // js-dom M4 R81：detached doc 的 createElementNS（WPT createElementNS_tests 经
      // document.implementation.createDocument 后的 NS 创建；XML 语义——不大写、带校验）。
      createElementNS: function (ns, q) {
        var _nsStr = (ns == null) ? '' : String(ns);
        var _qn = String(q);
        var _XML_NS = 'http://www.w3.org/XML/1998/namespace';
        var _XMLNS_NS = 'http://www.w3.org/2000/xmlns/';
        var _c1 = _qn.indexOf(':');
        var _pre = _c1 >= 0 ? _qn.slice(0, _c1) : null;
        var _loc = _c1 >= 0 ? _qn.slice(_c1 + 1) : _qn;
        // R130（js-dom M4）：校验对齐主文档 createElementNS（part06 R81 的 WPT 期望表——
        // '}'/'<' 非 NameStart 字符在非首位置合法；'0:a' prefix 段从宽；'f:o:o'/'f::oo'
        // 有 ns 合法（local 含冒号非 malformed）；XMLNS ns 仅 xmlns 元素）。旧
        // `_zwIsValidQualifiedName` + 冒号禁令把 'f}oo'/'f:o:o' 误判 Invalid。
        // R135：显式 invalid 字符集（NUL + ASCII 空白五字符 + '/' + '>'——JS /\s/ 含
        // \x0B 等非 XML 空白误拒；NUL 漏校验使 'null\0' local 不抛，WPT name-validation）。
        if (/[\u0000\u0009\u000A\u000C\u000D\u0020/>]/.test(_qn) || _qn === '' || _c1 === 0 || _c1 === _qn.length - 1) {
          throw new (globalThis.DOMException || Error)('The string contains invalid characters.', 'InvalidCharacterError');
        }
        if (_pre === null) {
          if (!_zwIsNameStartChar(Array.from(_qn)[0])) {
            throw new (globalThis.DOMException || Error)('The string contains invalid characters.', 'InvalidCharacterError');
          }
        } else {
          var _locChars = Array.from(_loc);
          if (!_locChars.length || !_zwIsNameStartChar(_locChars[0])) {
            throw new (globalThis.DOMException || Error)('The string contains invalid characters.', 'InvalidCharacterError');
          }
        }
        // R135：段校验走 spec regex（_r135IsValidName——首字符 ASCII 字母→后续任意合法集 /
        // ':'/'_'/>=0x80 → 后续 NameChar 集；镜像 part06 主文档 createElementNS。':soh\x01'
        // local 首字符 ':' 合法但 '\x01' 违 NameChar → 抛，WPT name-validation）。
        // **regex 语义放大**：prefix 段 ≥0x80 首（emoji 等）→ 后续限 NameChar，而 WPT
        // name-validation 的 validNamespacePrefixes 全码点（含 \x01 等）× valid local 组合
        // 都须不抛——对含 prefix 的名，prefix 段**从宽**（无字符集校验，仅禁空/NUL/ASCII
        // 空白/'/'/'>'——上方整名字符集已覆盖），只校验 local 段（spec regex）。
        if (typeof _r135IsValidName === 'function') {
          if (_pre === null ? !_r135IsValidName(_qn) : !_r135IsValidName(_loc)) {
            throw new (globalThis.DOMException || Error)('The string contains invalid characters.', 'InvalidCharacterError');
          }
        }
        if (_nsStr === _XMLNS_NS) {
          var _xmlnsOk = (_loc === 'xmlns' && _pre === null) || (_pre === 'xmlns');
          if (!_xmlnsOk) {
            throw new (globalThis.DOMException || Error)('The xmlns namespace is not allowed for elements.', 'NamespaceError');
          }
        }
        if (_pre !== null) {
          if (_nsStr === '') throw new (globalThis.DOMException || Error)('Namespace prefix provided but no namespace.', 'NamespaceError');
          if (_pre === 'xml' && _nsStr !== _XML_NS) throw new (globalThis.DOMException || Error)("Prefix 'xml' must be bound to the XML namespace.", 'NamespaceError');
          if (_pre === 'xmlns' && _nsStr !== _XMLNS_NS) throw new (globalThis.DOMException || Error)("Prefix 'xmlns' requires the XMLNS namespace.", 'NamespaceError');
        } else if (_loc === 'xmlns' && _nsStr !== _XMLNS_NS) {
          throw new (globalThis.DOMException || Error)("Local name 'xmlns' requires the XMLNS namespace.", 'NamespaceError');
        }
        var e2 = _zwMEl({ tag: _loc, preserveCase: true }, null);
        e2.ownerDocument = doc;
        e2.namespaceURI = _nsStr || null;
        e2.prefix = _pre;
        e2.tagName = _qn;
        e2.nodeName = _qn;
        e2.localName = _loc;
        e2.nodeValue = null;
        return e2;
      },
      // R51：spec `dom-document` CDATASection 工厂（XML 文档专有；WPT dom/common.js
      // setupRangeTests 经 `new Document().createCDATASection(...)` 建 testNodes——缺它整个
      // setup 中途崩 → testNodes undefined → dom/* mega-case 全体退化）。轻量节点对象
      //（nodeType=4 / nodeName='#cdata-section' / data/nodeValue；不可 append 到 HTML 树）。
      createCDATASection: function (d) {
        var v = String(d == null ? '' : d);
        var n4 = {
          nodeType: 4,
          nodeName: '#cdata-section',
          data: v,
          nodeValue: v,
          textContent: v,
          childNodes: [],
          hasChildNodes: function () { return false; },
          contains: function (other) { return _zwNodeContains(n4, other); },
          compareDocumentPosition: function (other) { return _zwCompareDocumentPosition(n4, other); },
          // R81：CDATA 导航面（WPT Node-properties cdata 族——旧 undefined）。
          get length() { return n4.data.length; },
          get firstChild() { return null; },
          get lastChild() { return null; },
          get parentElement() { var p = n4.parentNode; return p && p.nodeType === 1 ? p : null; },
          parentNode: null,
          ownerDocument: doc,
        };
        try { Object.setPrototypeOf(n4, globalThis.Node ? globalThis.Node.prototype : Object.prototype); } catch (_eR117x) {}
          return n4;
      },
      // R51：detached doc 的 ProcessingInstruction/Comment 工厂（common.js setupRangeTests
      // xmlDoc.createProcessingInstruction + createComment——同 createCDATASection 补齐，
      // 轻 + spec 命名校验对齐主文档 R9/R3 语义）。
      createProcessingInstruction: function (target, data) {
        var t = String(target == null ? '' : target);
        var v = String(data == null ? '' : data);
        if (t === '' || /[ \t\n\r\f]/.test(t)) {
          throw new (globalThis.DOMException || Error)('Invalid ProcessingInstruction target', 'InvalidCharacterError');
        }
        if (v.indexOf('?>') !== -1) {
          throw new (globalThis.DOMException || Error)('Invalid ProcessingInstruction data', 'InvalidCharacterError');
        }
        var n7 = {
          nodeType: 7,
          nodeName: t,
          target: t,
          data: v,
          nodeValue: v,
          textContent: v,
          childNodes: [],
          hasChildNodes: function () { return false; },
          contains: function (other) { return _zwNodeContains(n7, other); },
          compareDocumentPosition: function (other) { return _zwCompareDocumentPosition(n7, other); },
          // R81：PI 导航面（WPT Node-properties processingInstruction.parentElement/length/
          // firstChild/lastChild——旧 undefined）。length = data 长度（spec CharacterData）。
          get length() { return n7.data.length; },
          get firstChild() { return null; },
          get lastChild() { return null; },
          get parentElement() { var p = n7.parentNode; return p && p.nodeType === 1 ? p : null; },
          parentNode: null,
          ownerDocument: doc,
        };
        _zwMDefineSiblings(n7);
        try { Object.setPrototypeOf(n7, globalThis.Node ? globalThis.Node.prototype : Object.prototype); } catch (_eR117x) {}
          return n7;
      },
      createComment: function (d) {
        var v = String(d == null ? '' : d);
        var n8 = {
          nodeType: 8,
          nodeName: '#comment',
          data: v,
          nodeValue: v,
          textContent: v,
          childNodes: [],
          hasChildNodes: function () { return false; },
          contains: function (other) { return _zwNodeContains(n8, other); },
          compareDocumentPosition: function (other) { return _zwCompareDocumentPosition(n8, other); },
          // R81：Comment 导航面（WPT Node-properties foreignComment.length/parentElement——旧 undefined）。
          get length() { return n8.data.length; },
          get firstChild() { return null; },
          get lastChild() { return null; },
          get parentElement() { var p = n8.parentNode; return p && p.nodeType === 1 ? p : null; },
          parentNode: null,
          ownerDocument: doc,
        };
        try { Object.setPrototypeOf(n8, globalThis.Node ? globalThis.Node.prototype : Object.prototype); } catch (_eR117x) {}
          return n8;
      },
      // R51：detached doc 的 DocumentFragment 工厂（common.js setupRangeTests
      // foreignDoc.createDocumentFragment——轻量可变容器：appendChild/childNodes/
      // firstChild/lastChild 本地维护，够 testNodes 组装与遍历）。
      createDocumentFragment: function () {
        var frag = {
          nodeType: 11,
          nodeName: '#document-fragment',
          childNodes: [],
          children: [],
          parentNode: null,
          ownerDocument: doc,
          // js-dom M4 R79：Node.contains / compareDocumentPosition（WPT testNodes 的
          // docfrag/foreignDocfrag/xmlDocfrag 族——旧缺方法）。
          contains: function (other) { return _zwNodeContains(frag, other); },
          compareDocumentPosition: function (other) { return _zwCompareDocumentPosition(frag, other); },
          get firstChild() { return this.childNodes.length ? this.childNodes[0] : null; },
          get lastChild() { return this.childNodes.length ? this.childNodes[this.childNodes.length - 1] : null; },
          hasChildNodes: function () { return this.childNodes.length > 0; },
          // js-dom M4 R81：fragment 的 nodeValue/textContent（spec dom-node-nodevalue/textcontent
          //——DocumentFragment nodeValue 恒 null、textContent = 子树 Text 拼接；WPT Node-properties
          // xmlDocfrag/foreignDocfrag 族旧全 undefined）。
          get nodeValue() { return null; },
          get textContent() {
            var t = '';
            for (var i = 0; i < this.childNodes.length; i++) {
              var c = this.childNodes[i];
              if (c.nodeType === 3 || c.nodeType === 4) t += String(c.nodeValue != null ? c.nodeValue : '');
              else if (c.nodeType === 1 && typeof c.textContent === 'string') t += c.textContent;
            }
            return t;
          },
          // R81：fragment 导航面（WPT Node-properties xmlDocfrag/foreignDocfrag.nextSibling/
          // previousSibling/parentElement 恒 null——旧 undefined；detached fragment 无兄弟）。
          get nextSibling() { return null; },
          get previousSibling() { return null; },
          get parentElement() { var p = this.parentNode; return p && p.nodeType === 1 ? p : null; },
          appendChild: function (c) {
            if (c && c.nodeType === 11 && c !== this) {
              for (var i = 0; i < c.childNodes.length; i++) this.appendChild(c.childNodes[i]);
              c.childNodes = [];
              return c;
            }
            if (c && c.parentNode && c.parentNode.removeChild) { try { c.parentNode.removeChild(c); } catch (_e) {} }
            if (c) { c.parentNode = this; this.childNodes.push(c); if (c.nodeType === 1) this.children.push(c); }
            return c;
          },
          removeChild: function (c) {
            for (var i = 0; i < this.childNodes.length; i++) {
              if (this.childNodes[i] === c) {
                // R86：迭代器 retarget 通知（先于树状态变化）。
                if (globalThis._zwNotifyIteratorsRemove) {
                  try { globalThis._zwNotifyIteratorsRemove(c); } catch (_e86b) {}
                }
                this.childNodes.splice(i, 1);
                var ci = this.children.indexOf(c);
                if (ci >= 0) this.children.splice(ci, 1);
                c.parentNode = null;
                return c;
              }
            }
            return c;
          },
        };
        return frag;
      },
      // R15：detached doc 的 implementation（用例 doTest(doc,...) 经 doc.implementation.createDocumentType）。
      // ownerDocument 指向此 detached doc（spec：doctype.ownerDocument === 创建它的 document）。
      implementation: {
        hasFeature: function () { return true; },
        // R130（js-dom M4）：createHTMLDocument/createDocument 委托主文档 implementation
        //（WPT createHTMLDocument-with-saved-implementation——无 src iframe 的
        // contentDocument.implementation.createHTMLDocument() 旧 'not a function' 崩。
        // 产物归主文档（spec：implementation 方法与 browsing context 无关）。
        createHTMLDocument: function (t) {
          return globalThis.document.implementation.createHTMLDocument(t);
        },
        createDocument: function (ns, qn, dt) {
          return globalThis.document.implementation.createDocument(ns, qn, dt);
        },
        createDocumentType: function (qualifiedName, publicId, systemId) {
          var dt = {
            nodeType: 10,
            name: String(qualifiedName == null ? '' : qualifiedName),
            nodeName: String(qualifiedName == null ? '' : qualifiedName),
            publicId: String(publicId == null ? '' : publicId),
            systemId: String(systemId == null ? '' : systemId),
            ownerDocument: doc,
            nodeValue: null,
            // R117：cloneNode（WPT pre-insertion-validation-hierarchy 的 doctype 复制）+ remove
            //（doc.childNodes[0].remove()——doctype 的 ChildNode.remove）。
            cloneNode: function () {
              return doc.implementation.createDocumentType(dt.name, dt.publicId, dt.systemId);
            },
            remove: function () {
              if (dt.parentNode && dt.parentNode.removeChild) {
                try { dt.parentNode.removeChild(dt); } catch (_eR117dt) {}
              }
            },
            textContent: null,
            childNodes: [],
            hasChildNodes: function () { return false; },
            contains: function (other) { return _zwNodeContains(dt, other); },
            compareDocumentPosition: function (other) { return _zwCompareDocumentPosition(dt, other); },
            // R81：DocumentType 导航面（WPT Node-properties doctype/foreignDoctype.firstChild/
            // lastChild/parentElement/previousSibling/nextSibling——旧 undefined ≠ null）。
            get firstChild() { return null; },
            get lastChild() { return null; },
            get parentElement() { return null; },
            get previousSibling() {
              var p = this.parentNode;
              if (!p) return null;
              var i = p.childNodes.indexOf(this);
              return i > 0 ? p.childNodes[i - 1] : null;
            },
            get nextSibling() {
              var p = this.parentNode;
              if (!p) return null;
              var i = p.childNodes.indexOf(this);
              return i >= 0 && i < p.childNodes.length - 1 ? p.childNodes[i + 1] : null;
            },
            parentNode: null,
          };
          // R128：原型接线 DocumentType.prototype（WPT Node-cloneNode check_copy 断言
          // instanceof DocumentType）。dt 字面量构建后挂（对象内 IIFE 因 tdz 失败）。
          try {
            if (globalThis.DocumentType && globalThis.DocumentType.prototype) {
              Object.setPrototypeOf(dt, globalThis.DocumentType.prototype);
            }
          } catch (_e128dt) {}
          return dt;
        },
      },
    };
    // js-dom M4 R79 尾簇：detached 工厂节点缺 `_zwMDefineSiblings`（WPT Node-compareDocumentPosition
    // oracle 的 previousNode 经 previousSibling 后向树序遍历——PI/doctype 的 previousSibling 恒 null
    // 使期望值与文档序矛盾）。上述 createDocumentType/createCDATASection/createComment/
    // createProcessingInstruction 均为普通对象，在此统一补齐。
    (function () {
      var wire = function (factory) {
        return function () {
          var n = factory.apply(this, arguments);
          if (n && typeof n === 'object' && n.nodeType !== 10) _zwMDefineSiblings(n);
          return n;
        };
      };
      doc.createComment = wire(doc.createComment);
      doc.createProcessingInstruction = wire(doc.createProcessingInstruction);
    })();
    body.ownerDocument = doc;
    headEl.ownerDocument = doc;
    docEl.ownerDocument = doc;
    // js-dom M4 R79：html/head/body 树链接（spec Document 结构：documentElement 含 head+body，
    // 其父为 doc）。WPT common.js `foreignDoc.body.appendChild(foreignPara1)` 后
    // `foreignDoc.contains(foreignPara1)` 沿 parentNode 链上行须命中 foreignDoc。body.parentNode
    // 原 null（R3017 detached root 注释——DOMPurify walk 用 parentNode 向上只多一级，不影响）。
    headEl.parentNode = docEl;
    // R81：body.parentNode 已由上方 getter 承载（docEl），此处不再赋值（赋值会覆盖 getter 报错或失效）。
    docEl.childNodes = [headEl, body];
    docEl.children = [headEl, body];
    // js-dom M4 R84：headEl/docEl 兄弟导航 getter（R3018 _zwMDefineSiblings 同款）——
    // WPT dom/traversal oracle nextNode() 树序遍历经 nextNodeDescendants climb 依赖
    // head.nextSibling=body / docEl.nextSibling（doc 子列表定位）；旧缺失（undefined falsy
    // 恰似 null 但 climb 在 head 停住 → foreignDoc 遍历只到 HEAD，NodeIterator foreignDoc
    // 整簇 expected-null-but-got-object）。
    Object.defineProperty(headEl, 'nextSibling', { get: function () { return body; }, configurable: true });
    Object.defineProperty(headEl, 'previousSibling', { get: function () { return null; }, configurable: true });
    Object.defineProperty(docEl, 'nextSibling', { get: function () {
      var kids = doc.childNodes || [];
      var i = kids.indexOf(docEl);
      return i >= 0 && i < kids.length - 1 ? kids[i + 1] : null;
    }, configurable: true });
    Object.defineProperty(docEl, 'previousSibling', { get: function () {
      var kids = doc.childNodes || [];
      var i = kids.indexOf(docEl);
      return i > 0 ? kids[i - 1] : null;
    }, configurable: true });
    docEl.hasChildNodes = function () { return true; };
    docEl.contains = function (other) { return _zwNodeContains(docEl, other); };
    docEl.compareDocumentPosition = function (other) { return _zwCompareDocumentPosition(docEl, other); };
    docEl.parentNode = doc;
    // js-dom M4 R112：docEl/body 事件面（view 形态 _zwEvLs + tag 注册表——与 _zwParseEl
    // path 派发共享 fire 语义；WPT Event-dispatch-bubbles targets = [doc, docEl, body, ...]
    // 逐一 addEventListener）。tag 键经全局 _zwEvTagRegistry（part02 定义）注册，使解析
    // 元素派发（path 键 sig:HTML|.../sig:BODY|... miss 时按 tag 兜底）可达 docEl/body 的
    // listener。doc 入口仍走 doc._zwLocalListeners（_zwDispatchLocalDoc 链承载）。
    var _zwWireLocalEvents = function (node, tag) {
      node.addEventListener = function (type, fn, opts) {
        if (!node._zwEvLs) node._zwEvLs = {};
        var t = String(type);
        if (!node._zwEvLs[t]) node._zwEvLs[t] = [];
        var cap = opts != null && typeof opts === 'object' ? !!opts.capture : !!opts;
        var once = opts != null && typeof opts === 'object' ? !!opts.once : false;
        // R143：spec「add an event listener」步骤 4——重复 listener 静默丢弃。
        var _r143n = node._zwEvLs[t];
        for (var _r143j = 0; _r143j < _r143n.length; _r143j++) {
          if (_r143n[_r143j].fn === fn && _r143n[_r143j].capture === cap) return;
        }
        node._zwEvLs[t].push({ fn: fn, capture: cap, once: once });
        try { globalThis._zwEvTagRegistry['tag:' + tag] = node; } catch (_eR) {}
      };
      node.removeEventListener = function (type, fn, opts) {
        if (!node._zwEvLs) return;
        var t = String(type);
        var cap = opts != null && typeof opts === 'object' ? !!opts.capture : !!opts;
        var ls = node._zwEvLs[t];
        if (!ls) return;
        node._zwEvLs[t] = ls.filter(function (l) { return !(l.fn === fn && l.capture === cap); });
      };
      node.dispatchEvent = function (event) {
        globalThis._zwDispatchGuard(event);
        event.target = node;
        return _zwDispatchLocalDoc(doc, event);
      };
    };
    _zwWireLocalEvents(docEl, 'HTML');
    _zwWireLocalEvents(body, 'BODY');
    // R112：doc 链注册（part02 解析视图派发的 path 末端可达 doc 站）——最新 detached doc
    // 挂 globalThis（_zwParseEl.dispatchEvent 按链顶端 html/body tag 命中后派 doc 的
    // _zwLocalListeners）。后建覆盖先建（每用例一 doc；跨用例 listener 不复用）。
    try {
      globalThis._zwEvDocChain = { doc: doc, docEl: docEl, body: body };
    } catch (_eDC) {}
    // js-dom M4 R117：ParentNode/ChildNode 变异族（prepend/append/replaceChildren/before/after/
    // replaceWith）+ 层级校验（spec pre-insert 步骤 2/4/5——WPT pre-insertion-validation-hierarchy：
    // ① node 是 parent 的含 host 包容祖先 → HierarchyRequestError ② Document 节点插进非 doc /
    // Text 插进 doc / 不允许的节点类型 → HierarchyRequestError）。附到 doc 与 body（detached
    // 文档的 body 是普通对象，无元素 proxy 的 get trap）。实际插入 best-effort 走 appendChild
    //（doc.childNodes 追加）。
    var _r117Hre = function (msg) {
      var e = new (globalThis.DOMException || Error)(msg, 'HierarchyRequestError');
      return e;
    };
    var _r117Validate = function (parentNode, node) {
      if (!node || typeof node !== 'object') return;
      // 含 host 包容 inclusive ancestor：node === parent 或 node 是 parent 的祖先（沿 parentNode 上行）。
      var anc = parentNode;
      var hops = 0;
      while (anc && hops++ < 64) {
        if (anc === node) throw _r117Hre('The new node is an ancestor of the parent node.');
        anc = anc.parentNode;
      }
      // 节点类型约束：Document 只收 DocumentFragment/DocumentType/Element（Text/Comment/PI → HRE）；
      // 非 Document 不收 DocumentType/Document。
      var parentIsDoc = parentNode === doc || parentNode.nodeType === 9;
      var nt = node.nodeType | 0;
      if (parentIsDoc && (nt === 3 || nt === 4 || nt === 9)) {
        throw _r117Hre('Nodes of type ' + nt + ' cannot be inserted into a Document.');
      }
      if (!parentIsDoc && (nt === 9 || nt === 10)) {
        throw _r117Hre('Only a Document can contain nodes of type ' + nt + '.');
      }
      // Document 的额外规则（spec pre-insert 步骤 5）：frag 多元素 / doc 已有元素再加元素(frag
      // 含元素) → HRE。
      if (parentIsDoc) {
        var hasEl = function (n) { var k = n.childNodes || []; for (var q = 0; q < k.length; q++) if (k[q].nodeType === 1) return true; return false; };
        if (nt === 1 && hasEl(parentNode)) {
          throw _r117Hre('A Document cannot contain more than one Element.');
        }
        if (nt === 11) {
          var fragEls = 0;
          var fk = node.childNodes || [];
          for (var f = 0; f < fk.length; f++) if (fk[f].nodeType === 1) fragEls++;
          if (fragEls > 1) throw _r117Hre('A Document cannot contain more than one Element.');
          if (fragEls === 1 && hasEl(parentNode)) throw _r117Hre('A Document cannot contain more than one Element.');
        }
      }
    };
    var _r117Install = function (target, isChildNode) {
      if (!target) return;
      var _mk = function (mode) {
        return function () {
          for (var a = 0; a < arguments.length; a++) _r117Validate(target, arguments[a]);
          if (mode === 'prepend') {
            // best-effort：逆序 insertBefore 首子（无 ref → appendChild 前置近似）。
            for (var b = arguments.length - 1; b >= 0; b--) {
              var n = arguments[b];
              if (n && typeof n === 'object') { try { target.insertBefore ? target.insertBefore(n, target.firstChild || null) : target.appendChild(n); } catch (_e) {} }
            }
            return;
          }
          for (var c = 0; c < arguments.length; c++) {
            var nn = arguments[c];
            if (nn && typeof nn === 'object') { try { target.appendChild(nn); } catch (_e2) {} }
          }
        };
      };
      if (!target.prepend) target.prepend = _mk('prepend');
      if (!target.append) target.append = _mk('append');
      if (!target.replaceChildren) target.replaceChildren = function () {
        // js-dom M4 R119（WPT ParentNode-replaceChildren Document 域三缺口）：
        // ① 清空用 firstChild while 循环（旧快照 for 循环在 removeChild 内部状态分裂
        //（children/childNodes 不同步）时漏删——replaceChildren() 后残留 1 子）。
        // ② spec whatwg/dom#1045：replace-all 先移除现有子**再**做 pre-insert 校验——
        //「with an element, replacing an existing doctype and element」期望成功（校验时
        // doc 已空，单元素不撞「more than one Element」）。旧先校验后清空致误抛。
        // ③ 字符串参数在 Document 目标上抛 HierarchyRequestError（Text 节点不可进
        // Document——_r117Validate 只查 object 参数，doc.replaceChildren('text') 须抛）。
        var isDocT = false;
        try { isDocT = target.nodeType === 9; } catch (_eT) {}
        if (isDocT) {
          for (var s0 = 0; s0 < arguments.length; s0++) {
            if (arguments[s0] == null || typeof arguments[s0] !== 'object') {
              throw new (globalThis.DOMException || Error)(
                'Nodes of type 3 cannot be inserted into a Document.', 'HierarchyRequestError');
            }
          }
        }
        var guard = 0;
        while (target.firstChild && guard++ < 1024) {
          try { target.removeChild(target.firstChild); } catch (_e3) { break; }
        }
        for (var a2 = 0; a2 < arguments.length; a2++) {
          _r117Validate(target, arguments[a2]);
          if (arguments[a2] && typeof arguments[a2] === 'object') { try { target.appendChild(arguments[a2]); } catch (_e4) {} }
        }
      };
      if (isChildNode) {
        if (!target.before) target.before = function () { for (var a3 = 0; a3 < arguments.length; a3++) _r117Validate(target.parentNode || {}, arguments[a3]); };
        if (!target.after) target.after = function () { for (var a4 = 0; a4 < arguments.length; a4++) _r117Validate(target.parentNode || {}, arguments[a4]); };
        if (!target.replaceWith) target.replaceWith = function () {
          var p = target.parentNode;
          if (!p) return;
          for (var a5 = 0; a5 < arguments.length; a5++) _r117Validate(p, arguments[a5]);
          try { p.removeChild(target); } catch (_e5) {}
        };
      }
    };
    _r117Install(doc, false);
    _r117Install(body, true);
    _r117Install(docEl, true);
    // R117：ChildNode.remove（WPT pre-insertion-validation-hierarchy 的 setup 用
    // doc.documentElement.remove()）。detached docEl/body 无元素 proxy trap——直接补方法。
    if (docEl && !docEl.remove) {
      docEl.remove = function () {
        var kids = doc.childNodes || [];
        for (var i = 0; i < kids.length; i++) {
          if (kids[i] === docEl) { try { doc.removeChild(docEl); } catch (_e) {} return; }
        }
      };
    }
    if (body && !body.remove) {
      body.remove = function () {
        try { docEl.removeChild(body); } catch (_e2) {}
      };
    }
    // R128：原型接线 Document.prototype（WPT Node-cloneNode "implementation.createDocument/
    // createHTMLDocument" 的 `copy instanceof Document`——detached doc 旧为 plain object 恒
    // false）。挂原型经 Document.prototype → Node.prototype 链（instanceof Document/Node 均真）。
    try {
      if (globalThis.Document && globalThis.Document.prototype) {
        Object.setPrototypeOf(doc, globalThis.Document.prototype);
      }
    } catch (_e128doc) {}
  // R128：constructor 读回（WPT Node-cloneNode-XMLDocument `doc.constructor ===
    // XMLDocument`——own constructor 缺省回落 Object；按 contentType 惰性判定：XML 文档
    //（非 'text/html'）→ XMLDocument，HTML → Document。getter 形态因 createHTMLDocument
    // 在 _makeDetachedDocument 返回**后**才设 contentType（build 时值未定）。
    try {
      Object.defineProperty(doc, 'constructor', {
        get: function () {
          var _isHtml = typeof doc.contentType === 'string' && doc.contentType.indexOf('html') >= 0;
          return _isHtml ? globalThis.Document : globalThis.XMLDocument;
        },
        configurable: true, enumerable: false,
      });
    } catch (_e128cx) {}
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
    // R122：value/nodeValue 走原型 accessor（_r122V 存储 + setter 写回 ownerElement），
    // 不再建 own 数据属性（own writable 会拦截赋值使传播失效）。
    a._r122V = v;
    a.localName = n;
    a.prefix = null;
    a.namespaceURI = null;
    a.specified = true;
    // R116：textContent/data（WPT attributes.js attr_is 读 attr.textContent——Attr 的 textContent
    // 即 value，spec dom-attr `get value` 同源）。
    a.textContent = v;
    a.data = v;
    a.ownerElement = ownerEl || null;
    // R130（js-dom M4）：Attr 的 baseURI（WPT Node-baseURI "attributes ..." 三形态——
    // 与元素同源读 document URL；spec dom-node-baseuri：非文档节点回落 node document）。
    Object.defineProperty(a, 'baseURI', {
      get: function () {
        try { return globalThis.location ? String(globalThis.location.href) : 'about:blank'; }
        catch (_e130au) { return 'about:blank'; }
      },
      configurable: true, enumerable: true,
    });
    return a;
  }
  // `el.attributes`（NamedNodeMap）：length / item(i) / getNamedItem(name) / 数值索引 /
  // Symbol.iterator，每项 Attr-like {name,value,localName,...}。经 `__zw_attr_names`+`__zw_get_attr`。
  // R3198：handle 经 `__zw_attr_names_handle`（属性名仅来自 mutations，无快照基底）——旧 handle 元素 NamedNodeMap
  // 恒空（length 0 / item·getNamedItem 返 null / iterator 空）。setNamedItem/removeNamedItem 真 mutation（R3022，
  // 委托元素 setAttribute/removeAttribute host 路径，返旧/移除 Attr），非只读 no-op。
  // js-dom M4 R122：属性名统一读（NamedNodeMap readNames 与 getAttributeNames 共源）。
  // 顺序规则（WPT attributes.html attributes_are / getAttributeNames / own property）：
  // ① 实例层的 local 集合**涵盖** host 名单的 local 集合 → 实例序为权威（文档插入序——
  //   host 扁平存储无法表达同 local 多 ns 的位置，实例按创建序展开；同 qname 第 k 个
  //   加合成后缀 '\x00#k' 供 attrObj 定位）。
  // ② 未涵盖（parser 快照属性未被 JS 写过——实例只含部分 local）→ host 名单为权威，
  //   实例只在其 local 的 host 位次展开（值以实例为准），host 未含的实例（NS 二实例）
  //   追加尾部。
  function _zwAttrReadNames(sel, handle) {
    var base = [];
    try {
      var n = handle
        ? (typeof __zw_attr_names_handle === 'function' ? __zw_attr_names_handle(handle) : '')
        : (typeof __zw_attr_names === 'function' ? __zw_attr_names(sel) : '');
      base = n ? n.split('|').filter(Boolean) : [];
    } catch (_e) {}
    try {
      var elKey = _elKey(sel, handle);
      var inst = _zwAttrInstances.get(elKey);
      if (inst && inst.length) {
        var instLocal = {};
        for (var ii = 0; ii < inst.length; ii++) instLocal[inst[ii].local] = true;
        var baseLocal = {};
        for (var bi0 = 0; bi0 < base.length; bi0++) {
          var bn0 = base[bi0];
          var bl0 = bn0.indexOf(':') >= 0 ? bn0.slice(bn0.indexOf(':') + 1) : bn0;
          baseLocal[bl0] = true;
        }
        var out = [];
        var qc = {};
        var pushInst = function (it) {
          qc[it.qname] = (qc[it.qname] || 0) + 1;
          out.push(qc[it.qname] > 1 ? it.qname + '\x00#' + qc[it.qname] : it.qname);
        };
        // 实例覆盖 host 全部 local（setAttributeNS 双实例把 host 属性全覆盖等场景）——
        // **base 位次优先 + 每位消费一个实例**：host 名单是文档序权威（parser 快照位次 +
        // host 只写每 local 首实例），实例按创建序逐位对齐（同 local 多实例不在首位聚集——
        // WPT getAttributeNames 期望 foo,FOO,foo,dummy:foo 交错序），host 未含的实例
        // （NS 二/三实例）按实例序尾追。匹配规则双形态：instance.local === 冒号后 local
        // **或** === 整名（非 NS setAttribute 的 local 是整个限定名——WPT "Attribute with
        // prefix in local name" 的 'pre:fix'）。
        // 首版「实例全量前插」破坏文档序（classList.write upsert class 实例后 id/class/data-x
        // 错序——r44 own-enumeration 抓回）；二版「同 local 全部实例聚集首位」破坏交错序
        // （WPT getAttributeNames tests 抓回）。
        var usedInst = {};
        for (var b1 = 0; b1 < base.length; b1++) {
          var bn = base[b1];
          var bLocal = bn.indexOf(':') >= 0 ? bn.slice(bn.indexOf(':') + 1) : bn;
          var emitted = false;
          for (var i3 = 0; i3 < inst.length && !emitted; i3++) {
            if (usedInst[i3]) continue;
            if (inst[i3].local === bLocal || inst[i3].local === bn) {
              usedInst[i3] = true; pushInst(inst[i3]); emitted = true;
            }
          }
          if (!emitted && out.indexOf(bn) < 0) out.push(bn);
        }
        for (var i4 = 0; i4 < inst.length; i4++) {
          if (!usedInst[i4]) { usedInst[i4] = true; pushInst(inst[i4]); }
        }
        return out;
      }
    } catch (_e2) {}
    return base;
  }

  // js-dom M4 R122：按 readNames 名（含 '\x00#k' 合成后缀）取**稳定 Attr 对象**——
  // getAttributeNode 族与 NamedNodeMap 索引读共用（identity 统一入口）。缺省 null。
  function _zwAttrObjFor(sel, handle, name) {
    try {
      var nm = String(name);
      var names = _zwAttrReadNames(sel, handle);
      var idx = names.indexOf(nm);
      if (idx < 0) {
        // 合成名 miss（调用方可能传裸 qname）→ 退化为首个限定名匹配。
        for (var i = 0; i < names.length; i++) {
          if (_zwAttrStripSyn(names[i]) === nm) { idx = i; break; }
        }
      }
      if (idx < 0) return null;
      return _attributesProxy(sel, handle).item(idx);
    } catch (_e) { return null; }
  }

  function _attributesProxy(sel, handle) {
    // R3198：handle 经 `__zw_attr_names_handle`，sel 经 `__zw_attr_names`（latest-wins）。各方法
    //（length/item/getNamedItem/iterator）均经此，故 handle NamedNodeMap 旧全空。
    // R122：读统一走 _zwAttrReadNames（与 getAttributeNames 同源融合）。
    // R122：per-element 缓存命中**先于一切闭包/原型赋值**（否则每次访问 el.attributes 刷新
    // NamedNodeMap.prototype.item 为新材料闭包，缓存实例方法与原型 identity 分叉）。
    var _nnmCacheKey = _elKey(sel, handle);
    if (_zwNNMCache[_nnmCacheKey]) return _zwNNMCache[_nnmCacheKey];
    var readNames = function() {
      return _zwAttrReadNames(sel, handle);
    };
    // js-dom M3 R96：supported property names 的 HTML 文档规则（WPT attributes.html "only include
    // all-lowercase qualified names"）——**HTML 文档 + HTML 命名空间元素**的 named keys（ownKeys 的
    // 名字段 / named getter / gOPD）仅含全小写 qualified name；**索引语义不变**（length/item/索引读
    // 仍覆盖全部属性——期望数组 ["0".."5","g:h","j"] 索引 0-5 全可达）。非 HTML 文档（detached XML
    // doc）或非 HTML ns 元素（createElementNS(""/其它 ns)，即使主文档）保留全部原名（"include all
    // qualified names" 两变体）。HTML-ns 判定：仅 createElementNS 元素带 `_nsHandles` 条目——无条目
    // = 主文档 createElement（隐式 HTML ns）；文档 HTML-ness 经 ownerDocument.contentType。
    var supportedNames = function() {
      // R122：named keys 不含合成后缀（'\x00#k' 是内部多实例索引键，非 spec qualified name——
      // WPT own property correctness 期望数组无 "a\0#2"）。
      var names = readNames().map(_zwAttrStripSyn);
      var _apDoc = null;
      try { _apDoc = _makeProxy(sel, handle).ownerDocument; } catch (_eD) {}
      var _apDocHtml = !(_apDoc && typeof _apDoc.contentType === 'string'
        && _apDoc.contentType.indexOf('html') < 0);
      var _apNs = handle && _nsHandles[handle] ? _nsHandles[handle].namespace
        : 'http://www.w3.org/1999/xhtml'; // 无 NS 条目 = createElement（HTML ns）
      if (_apDocHtml && _apNs === 'http://www.w3.org/1999/xhtml') {
        names = names.filter(function (nm) {
          for (var ci = 0; ci < nm.length; ci++) {
            var cc = nm.charAt(ci);
            if (cc >= 'A' && cc <= 'Z') return false;
          }
          return true;
        });
      }
      return names;
    };
    var attrObj = function(name) {
      // js-dom M4 R122：**稳定 Attr identity**——同一 (元素, qname) 的 Attr 对象往返恒同
      //（WPT attributes.html：`el.attributes[0] === el.getAttributeNode('foo')`、
      // `attr === el2.getAttributeNode('foo')` setAttributeNode 绑定后）。经绑定表
      // （_zwAttrBindings，elKey → Map(限定名 → Attr)）取/建/登记；实例层（_zwAttrInstances）
      // 供多实例 NS 字段（prefix/localName/namespaceURI 以实例为准）。合成名（qname +
      // '\x00#k'）定位同 qname 第 k 实例。
      var _r122Name = String(name);
      var _r122Qname = _zwAttrStripSyn(_r122Name);
      var _r122ElKey = _elKey(sel, handle);
      var _r122Bind = _zwAttrBindMap(_r122ElKey);
      var _r122List = _zwAttrInstances.get(_r122ElKey);
      // 定位目标实例（合成名第 k 个 / 限定名首个）。
      var _r122Inst = null;
      if (_r122List) {
        var _r122M = /\x00#(\d+)$/.exec(_r122Name);
        if (_r122M) {
          var _r122K = parseInt(_r122M[1], 10);
          var _r122Cnt = 0;
          for (var _r122j = 0; _r122j < _r122List.length; _r122j++) {
            if (_r122List[_r122j].qname === _r122Qname) {
              _r122Cnt++;
              if (_r122Cnt === _r122K) { _r122Inst = _r122List[_r122j]; break; }
            }
          }
        } else {
          for (var _r122z = 0; _r122z < _r122List.length; _r122z++) {
            if (_r122List[_r122z].qname === _r122Qname) { _r122Inst = _r122List[_r122z]; break; }
          }
        }
      }
      // 绑定命中（含游离回归：ownerElement 由绑定维护）→ 复用。键用**完整名**（含 '\x00#k'
      // 合成后缀——同 qname 多实例各有 identity）。
      var _r122Cached = _r122Bind.get(_r122Name);
      if (_r122Cached) {
        // 同步实例值（setattr 路径直改实例时 Attr._r122V 同步——identity 不变值最新）。
        if (_r122Inst && _r122Cached._r122V !== _r122Inst.value) {
          _r122Cached._r122V = _r122Inst.value;
          _r122Cached.textContent = _r122Inst.value;
          _r122Cached.data = _r122Inst.value;
        }
        return _r122Cached;
      }
      // 新建（R3003 latest-wins 读值 + R116 NS meta / R122 实例层 NS 字段）。
      var v;
      if (_r122Inst) v = _r122Inst.value;
      else if (handle) v = __zw_get_attr_handle(handle, _r122Qname);
      else if (typeof __zw_get_attr_lw === 'function') v = __zw_get_attr_lw(sel, _r122Qname);
      else v = __zw_get_attr(sel, _r122Qname);
      var attr = _zwMakeAttr(_r122Qname, v != null ? v : '', _makeProxy(sel, handle));
      if (_r122Inst) {
        attr.prefix = _r122Inst.prefix;
        attr.localName = _r122Inst.local;
        attr.namespaceURI = _r122Inst.ns;
      } else {
        // R116：NS 属性的 Attr 字段（prefix/localName/namespaceURI）从 setAttributeNS 登记的
        // 元数据取（host 扁平名无 ns 语义——WPT case.js setAttributeNS 断言 attr.prefix）。
        var _r116Meta = _attrNSMeta[_r122ElKey] && _attrNSMeta[_r122ElKey][_r122Qname];
        if (_r116Meta) {
          attr.prefix = _r116Meta.prefix;
          attr.localName = _r116Meta.local;
          attr.namespaceURI = _r116Meta.ns;
        }
      }
      _r122Bind.set(_r122Name, attr);
      return attr;
    };
    // R122：方法 identity 统一——把本元素的实现闭包同步装上 NamedNodeMap.prototype
    //（`map.item === NamedNodeMap.prototype.item`，WPT namednodemap method-names 断言。
    // 多元素的 attributes 各自刷新原型方法——named property 冲突场景（attr 名 'item'）
    // 由 get trap 分支序保证方法优先，原型值仅供 strict-equality identity 对照）。
    var _nnmItemFn = function(i) {
      var names = readNames();
      var idx = i | 0;
      return idx >= 0 && idx < names.length ? attrObj(names[idx]) : null;
    };
    var _nnmGetFn = function(name) {
      var names = readNames();
      var n = String(name);
      if (names.indexOf(n) >= 0) return attrObj(n);
      for (var gi = 0; gi < names.length; gi++) {
        if (_zwAttrStripSyn(names[gi]) === n) return attrObj(names[gi]);
      }
      return null;
    };
    try {
      globalThis.NamedNodeMap.prototype.item = _nnmItemFn;
      globalThis.NamedNodeMap.prototype.getNamedItem = _nnmGetFn;
    } catch (_eNNM) {}
    var _nnmProxy = new Proxy({}, {
      get: function(_t, p) {
        if (p === 'length') return readNames().length;
        if (p === 'item') return _nnmItemFn;
        if (p === 'getNamedItem') return _nnmGetFn;
        if (p === 'setNamedItem' || p === 'setNamedItemNS') {
          // R3022：真 mutation——setNamedItem(attr) 等价 setAttributeNode（经元素 host 路径）。
          // R122：**identity 语义**——绑定传入 Attr 原对象（`map.setNamedItem(attr1)` 后
          // `map.attr1 === attr1`、removeNamedItem 返同一对象；WPT attributes-namednodemap
          // setNamedItem/removeNamedItem 簇）。经 _zwSetAttributeNodeCore（绑定表核心）。
          return function (attr) {
            return _zwSetAttributeNodeCore(sel, handle, _elKey(sel, handle), attr, p === 'setNamedItemNS');
          };
        }
        if (p === 'removeNamedItem' || p === 'removeNamedItemNS') {
          // R3022：真 mutation——removeNamedItem(name) 等价 removeAttributeNode（返移除的
          // 绑定 Attr；缺失抛 NotFoundError，spec `dom-namednodemap-removenameditem`）。
          return function (name) {
            var el = _makeProxy(sel, handle);
            var attr = el.getAttributeNode(String(name));
            if (!attr) {
              throw new (globalThis.DOMException || Error)(
                "Failed to execute 'removeNamedItem' on 'NamedNodeMap': The specified attribute was not found.",
                'NotFoundError');
            }
            return _zwRemoveAttributeNodeCore(sel, handle, _elKey(sel, handle), attr);
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
        // attrs.id.value 访问模式）。R96：named 集合用 supportedNames()（HTML 文档只含全小写
        // qualified name）。
        if (typeof p === 'string' && supportedNames().indexOf(p) >= 0) {
          return attrObj(p);
        }
        // js-dom M3 R96：Object.prototype 方法回落（hasOwnProperty/valueOf/isPrototypeOf 等）。
        // real NamedNodeMap 经原型链继承 Object.prototype——`obj.hasOwnProperty(prop)`（WPT
        // attributes.html getEnumerableOwnProps1 的 for-in own 过滤）可用。Proxy target {}
        // 的真实原型即 Object.prototype，miss 名直接沿 target 原型链查（不经元素 getPrototypeOf
        // trap——NamedNodeMap 无 per-node 链语义）。
        if (typeof p === 'string' && p !== 'constructor') {
          var _anDesc = Object.getOwnPropertyDescriptor(Object.prototype, p);
          if (_anDesc) return _anDesc.value;
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
        // R122：ownKeys 的 named 段用 supportedNames（已剥 '\x00#k' 合成后缀），named getter /
        // gOPD / for-in 均经 supportedNames 一致（WPT getEnumerableOwnProps 期望数组无合成键）。
        var names = readNames();
        var supported = supportedNames();
        var keys = [];
        var seen = {};
        var pushKey = function (k) {
          if (!seen[k]) { seen[k] = true; keys.push(k); }
        };
        for (var i = 0; i < names.length; i++) pushKey(String(i));
        for (var j = 0; j < supported.length; j++) pushKey(supported[j]);
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
        if (supportedNames().indexOf(String(p)) >= 0) {
          // js-dom M3 R96：named 属性 descriptor 改 enumerable:false（spec named properties 的
          // 平台对象枚举语义——WPT attributes.html getEnumerableOwnProps1 的 for-in own 过滤期望
          // 只见数值索引 ["0","1"]，named（"a","b"）不进 for-in 枚举）。getOwnPropertyNames 顺序
          // （R44 用例面）不依赖 enumerability，保持 3/3 pass。
          return { value: attrObj(String(p)), writable: false, enumerable: false, configurable: true };
        }
        return undefined;
      }
    });
    _zwNNMCache[_nnmCacheKey] = _nnmProxy;
    return _nnmProxy;
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
    // R57（FV M1）：约束校验 API 属性存在性——validator.js 的 pre_check 用
    // `"validity" in ctl`。has trap 返回 true（V8 直接 execute 生效）+ **target
    // 预置属性**（webview 页面脚本路径的 `in` 不调 has trap——实测——走 target
    // 默认；get 仍走 trap 返回实时计算值）。
    var _fvTarget = {};
    var _fvProps = ['validity', 'validationMessage', 'willValidate', 'checkValidity',
                    'reportValidity', 'setCustomValidity'];
    for (var _vi = 0; _vi < _fvProps.length; _vi++) {
      try { Object.defineProperty(_fvTarget, _fvProps[_vi], { configurable: true, value: undefined }); } catch (_e) {}
    }
    var proxy = new Proxy(_fvTarget, {
      has: function(_t, prop) {
        // R129 注：本键在 handler 字面量中**后被 part05 的 has 键覆盖**（JS 对象字面量
        // 重复键后者胜——拼接后同一 Proxy handler）——生效实现见 part05（expando +
        // R129 方法白名单）。此分支仅 FV 属性命中（validity 等），实际由 target own
        // key（_fvTarget 预置）覆盖，本函数体不参与运行。
        if (prop === 'validity' || prop === 'validationMessage' || prop === 'willValidate' ||
            prop === 'checkValidity' || prop === 'reportValidity' || prop === 'setCustomValidity') {
          return true;
        }
        return false;
      },
      get: function(_t, prop) {
        // js-dom M3 R95：`constructor` 顶部短路（原型链 own 命中，限 8 层）。真实 DOM 中
        // `el.constructor` 沿原型链命中（custom element 返用户类）——lit ReactiveElement 的
        // 实例方法 `_$E_` 读 `this.constructor.elementProperties`（e2e 实证：旧 get trap 对
        // 'constructor' 落到中间分支返 undefined → `undefined.elementProperties` TypeError，
        // ctor 链中断、用户 ctor 体不执行）。放顶部（先于一切属性分支）——中间分支会先吞掉
        // 该名（R93 通用回落太靠后，赶不上）。
        if (prop === 'constructor') {
          var _cChain = Object.getPrototypeOf(_makeProxy(sel, handle));
          var _cGuard = 0;
          while (_cChain && _cGuard < 8) {
            var _cDesc = Object.getOwnPropertyDescriptor(_cChain, 'constructor');
            if (_cDesc) return _cDesc.value;
            _cChain = Object.getPrototypeOf(_cChain);
            _cGuard++;
          }
        }
        // js-dom M3 R98：CE 用户类**首层原型** accessor getter 优先（先于 shim 反射属性分支）。
        // 真实 DOM 原型链序：用户类 prototype（lit createProperty 装的 get/set——响应式属性）
        // → … → HTMLElement.prototype（反射 getter）。shim 的反射 getter 是 get trap 中间分支
        // 而非原型 accessor——若分支先于用户 accessor，`el.name` 读反射属性空值（e2e 实证首
        // 渲染插值空：lit getter this[s] 拿不到 R98 set 分支存的 symbol expando 值——被 'name'
        // 反射分支先吞）。限定 **CE 元素**（tag 命中 customElements registry，getPrototypeOf
        // trap 首层即用户 ctor.prototype）的**首层** own accessor——lit/stencil 的响应式属性
        // 全装首层；非 CE 元素零路径变化（WPT A/B：Element-getElementsByTagNameNS div 元素
        // 'getElementsByTagNameNS' 读被过宽 accessor 派发破坏的回归教训）。symbol key 不入
        //（R98 set 的 symbol expando 走 R3042 读）。getter 异常吞返 undefined（loose，与 R93
        // 链回落行为一致）。
        if (typeof prop === 'string' && prop.length > 0) {
          var _r98Proto = Object.getPrototypeOf(_makeProxy(sel, handle));
          if (_r98Proto) {
            var _r98IsCE = false;
            try {
              var _r98Ctor = _r98Proto.constructor;
              if (globalThis.customElements && typeof globalThis.customElements.getName === 'function'
                  && _r98Ctor && globalThis.customElements.getName(_r98Ctor)) _r98IsCE = true;
            } catch (_e98ce) {}
            if (_r98IsCE) {
              var _r98GDesc;
              try { _r98GDesc = Object.getOwnPropertyDescriptor(_r98Proto, prop); } catch (_e98g) { _r98GDesc = undefined; }
              if (_r98GDesc && typeof _r98GDesc.get === 'function') {
                try { return _r98GDesc.get.call(_makeProxy(sel, handle)); } catch (_e98gc) { return undefined; }
              }
            }
          }
        }
        // QuickJS Proxy ToPrimitive 差异（2026-08-08）：V8 对 get(Symbol.toPrimitive)
        // 返回 undefined 时回退默认 valueOf/toString；QuickJS 直接抛 TypeError: toPrimitive
        //（createElement handle proxy 被隐式字符串化——appendChild/observer id 等——
        // 在 QuickJS 下中断脚本）。显式返回字符串化函数（有 sel 用 sel，否则 handle），
        // 保证 v8/quickjs 接口行为一致。
        // R134（js-dom M4）：`Symbol.unscopables`（spec WebIDL §[Unscopable]——ChildNode
        // 四方法 + prepend/append 在 with(element) 词法域不可见；WPT remove-unscopable：
        // inline handler `with(this){ remove }` 期望裸 remove 解析到 window 变量非元素
        // 方法）。with 语义消费本 symbol 的属性表对 has 判定做排除——proxy 的 has trap
        // 不识别该表故白名单/方法面照常命中；返回 Element.prototype 上的表（真实
        // 浏览器挂在 Element.prototype，六方法全 true）。
        if (prop === Symbol.unscopables) {
          try {
            if (globalThis.Element && globalThis.Element.prototype
                && globalThis.Element.prototype[Symbol.unscopables]) {
              return globalThis.Element.prototype[Symbol.unscopables];
            }
          } catch (_e134u) {}
          return undefined;
        }
        if (prop === Symbol.toPrimitive) {
          return function() { return sel ? sel : String(handle); };
        }
        // R34xx：显式 toString（_valToString 等直接调 .toString()——canvas WPT 的
        // assert 消息构建；与 Symbol.toPrimitive 同串化）。
        if (prop === 'toString') {
          return function() { return sel ? sel : String(handle); };
        }
        if (prop === '__zwHandle') return handle;
        if (prop === '__zwSelector') return sel;
        // R34xx：Symbol.toStringTag 按元素 tag 返接口名（2d.canvas.host.type.name 的
        // Object.prototype.toString.call(canvas) === '[object HTMLCanvasElement]'；
        // 此前无 toStringTag → '[object Object]'。generic：全部元素受益）。
        if (prop === Symbol.toStringTag) {
          var _tagIf = globalThis.__zwHtmlTagIface && globalThis.__zwHtmlTagIface[_realTag(sel, handle).toLowerCase()];
          return _tagIf || 'HTMLElement';
        }
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
          // R57（FV M3）：缓存读——setter 写入（_inputValuesSet 标记——JS-set 值/typed 值，
          // SetFormValue 在 applied view no-op，只能从缓存读回）始终可用；lazy-init 条目仅
          // 稳定身份键（#id/@handle）可用——位置选择器跨批指向不同元素，lazy-init 缓存碰撞
          //（见 _controlValue 注），位置键 lazy-init 跳过、每次直读 host。
          if (_inputValues[key] != null && (_inputValuesSet[key] === true || _stableValueKey(key))) {
            return _inputValues[key];
          }
          if (_inputValues[key] == null && _stableValueKey(key)) {
            if (!handle && sel && _isTag(sel, 'TEXTAREA')) {
              _inputValues[key] = __zw_get_text(sel) || '';
            } else {
              var va = handle ? __zw_get_attr_handle(handle, 'value') : __zw_get_attr(sel, 'value');
              _inputValues[key] = (va == null) ? '' : va;
            }
            return _inputValues[key];
          }
          var vaF = handle ? __zw_get_attr_handle(handle, 'value') : (sel ? __zw_get_attr(sel, 'value') : null);
          return (vaF == null) ? '' : vaF;
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
