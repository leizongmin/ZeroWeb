      self.readyState = EventSource.OPEN;
      self._dispatch('open', null);
      self._process(text);
      // finite stream 结束：真浏览器 spec 在连接关闭后重连（retry）；headless 视为结束 → onerror + CLOSED。
      if (!self._closed) {
        self.readyState = EventSource.CLOSED;
        self._dispatch('error', null);
      }
    }).catch(function () {
      if (!self._closed) {
        self.readyState = EventSource.CLOSED;
        self._dispatch('error', null);
      }
    });
  }
  EventSource.CONNECTING = 0;
  EventSource.OPEN = 1;
  EventSource.CLOSED = 2;
  EventSource.prototype = {
    constructor: EventSource,
    close: function () { this._closed = true; this.readyState = EventSource.CLOSED; },
    addEventListener: function (type, cb) {
      (this._listeners[type] || (this._listeners[type] = [])).push(cb);
    },
    removeEventListener: function (type, cb) {
      var arr = this._listeners[type];
      if (!arr) return;
      var i = arr.indexOf(cb);
      if (i >= 0) arr.splice(i, 1);
    },
    _dispatch: function (type, event) {
      var ev = event || new Event(type);
      ev.type = type;
      ev.target = this;
      var handler = this['on' + type];
      if (typeof handler === 'function') { try { handler.call(this, ev); } catch (_e) {} }
      var arr = this._listeners[type];
      if (arr) for (var i = 0; i < arr.length; i++) { try { arr[i].call(this, ev); } catch (_e) {} }
    },
    // text/event-stream 解析（HTML spec §9.2.6）：去 BOM → CRLF/CR/LF 分行 → 空行派发 → 字段 data/event/id。
    _process: function (text) {
      var lines = String(text).replace(/^\uFEFF/, '').split(/\r\n|\r|\n/);
      var data = [], eventType = '', id = '';
      for (var i = 0; i < lines.length; i++) {
        var line = lines[i];
        if (line === '') {
          if (data.length > 0 || eventType !== '') {
            if (id !== '') this._lastEventId = id;
            var evData = data.join('\n');
            this._dispatch(eventType || 'message',
              new MessageEvent(eventType || 'message', { data: evData, lastEventId: this._lastEventId, origin: this.url }));
          }
          data = []; eventType = '';
          continue;
        }
        var colon = line.indexOf(':');
        var field, value;
        if (colon === 0) continue; // 注释行（: 开头）
        if (colon > 0) {
          field = line.slice(0, colon);
          value = line.slice(colon + 1);
          if (value.charAt(0) === ' ') value = value.slice(1); // 去一个前导空格（spec）
        } else { field = line; value = ''; }
        if (field === 'data') data.push(value);
        else if (field === 'event') eventType = value;
        else if (field === 'id') id = value;
        // retry / 未知字段忽略
      }
    }
  };
  globalThis.EventSource = globalThis.EventSource || EventSource;

  // CSS——CSS 命名空间（escape 选择器转义 + supports 特性检测）。escape 纯 JS（CSSOM escape 算法，
  // 本地 Chromium 150 oracle 锚定）；supports 委托 host `__zw_css_supports`（known-property gate +
  // apply，两参声明 / 单参条件 not/括号/声明/and/or/嵌套——R2951 经 css-parser parse_supports_condition
  // 完整求值）。supports 语义近似「ZW 能 apply」（偏乐观）；selector() 恒 true（permissive）。
  var _zwCssNamespace = {
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
  globalThis.CSS = globalThis.CSS || {};
  globalThis.CSS.escape = globalThis.CSS.escape || _zwCssNamespace.escape;
  globalThis.CSS.supports = globalThis.CSS.supports || _zwCssNamespace.supports;

  // document.cookie 的 in-JS 存储（name → value）。document.cookie setter 写入，getter 序列化。
  // 不接真 cookie jar（host-layer defer）；per-上下文（无 origin 隔离）。
  var _doc_cookies = {};

  // document.title 缓存。null = 未初始化（惰性读 <title> 文本）；string = 显式 set 或已读。
  // getter 首访读 document.querySelector('title').textContent（空白折叠）；setter 仅更新缓存。
  var _doc_title = null;

  // document.currentScript 索引（R3258）。-1 = 不在 classic 脚本执行期（getter 返 null）；
  // >=0 = 当前 classic 脚本在「全部 <script> 元素」（含非 JS 类型）中的序号，与
  // document.getElementsByTagName('script') 文档序一一对应（host extract_page_scripts_indexed
  // 在过滤前递增）。宿主经 __zw_set_current_script(idx) / __zw_clear_current_script() 在每个
  // classic 脚本执行前后设/清；module 脚本不设（spec：module 执行期 currentScript 恒 null）。
  var _zwCurrentScriptIdx = -1;

  // document.designMode（R3261）——文档级 contentEditable 开关，默认 'off'。getter 返 'on'/'off'，
  // setter 接受 'on'/'off'/'inherit'（case-insensitive，'on'→'on'，其余→'off'，spec 文档级无真 'inherit'）。
  // headless 不真启用编辑（无真文本编辑/光标/输入处理）→ setter 仅存储，documented 惰性；让富文本编辑器
  //（TinyMCE/CKEditor 全屏模式）/笔记/测试工具读 `document.designMode === 'on'` 或设值不抛 TypeError。
  var _zwDesignMode = 'off';

  // document.activeElement 焦点追踪。null = 无焦点（activeElement 回落 body）；非空 = 焦点元素 key
  //（_elKey(sel,handle)）。focus()/blur() 经 Proxy get trap 操作。纯状态追踪，无真输入焦点/无事件派发。
  var _activeElKey = null;

  // R2938 Fullscreen + R2939 Pointer Lock 状态追踪。headless 无真 OS 全屏 / 指针锁（host 无窗口全屏 / 鼠标
  // 捕获信号源），但 spec 语义需可观察：fullscreenElement / pointerLockElement = 当前锁定元素（_elKey(sel,handle)
  // 三元组或 null）；requestFullscreen/requestPointerLock/exit* 经此状态 + change/error 事件实现 spec-alike 语义。
  // fullscreenEnabled 经 host `__zw_fullscreen_enabled`；requestPointerLock 可用性经 host `__zw_pointer_lock_enabled`
  //（均 '0'=禁用，如 iframe sandbox / Feature-Policy / 窗口特性拒绝），无注册→允许。
  // https://fullscreen.spec.whatwg.org/#dom-element-requestfullscreen
  // https://w3c.github.io/pointerlock/#dom-element-requestpointerlock
  var _fsSel = null;
  var _fsHandle = null;
  var _fsKey = null; // _elKey(sel,handle) of 全屏元素；「相同元素重复请求 no-op」判定 + null 表示非全屏
  var _plSel = null;
  var _plHandle = null;
  var _plKey = null; // _elKey(sel,handle) of 指针锁元素；同 _fsKey 语义

  // R2938/R2939 文档级事件派发（fullscreenchange/fullscreenerror/pointerlockchange/pointerlockerror）。
  // spec：在 document 上派发，bubbles、非 cancelable。document/window listener 同存于 _elKey('html', null)
  //（document.addEventListener 转发 html proxy，window dispatchEvent/addEventListener 同 key），故一次
  // _dispatchToListeners 触达 document.addEventListener + window.addEventListener + document/window 对应
  // on* IDL handler。currentTarget = document（spec 事件 target）。
  function _fireDocEvent(type) {
    try {
      var ev = _makeEvent(type, { bubbles: true, cancelable: false });
      _dispatchToListeners(_elKey('html', null), ev, 'all', globalThis.document);
    } catch (_e) {}
  }

  // NodeFilter 常量（spec `dom-nodefilter`，R41 对齐上游 NodeFilter-constants.html 全表）——
  // createTreeWalker/createNodeIterator 的 whatToShow 掩码 + acceptNode 返回值。
  // 修正：SHOW_PROCESSING_INSTRUCTION 0x10→0x40（原写错），补 SHOW_ATTRIBUTE/ENTITY_REFERENCE/ENTITY/NOTATION
  //（历史常量，掩码位保留）。
  globalThis.NodeFilter = globalThis.NodeFilter || {
    SHOW_ALL: 0xFFFFFFFF,
    SHOW_ELEMENT: 0x1,
    SHOW_ATTRIBUTE: 0x2,
    SHOW_TEXT: 0x4,
    SHOW_CDATA_SECTION: 0x8,
    SHOW_ENTITY_REFERENCE: 0x10,
    SHOW_ENTITY: 0x20,
    SHOW_PROCESSING_INSTRUCTION: 0x40,
    SHOW_COMMENT: 0x80,
    SHOW_DOCUMENT: 0x100,
    SHOW_DOCUMENT_TYPE: 0x200,
    SHOW_DOCUMENT_FRAGMENT: 0x400,
    SHOW_NOTATION: 0x800,
    FILTER_ACCEPT: 1,
    FILTER_REJECT: 2,
    FILTER_SKIP: 3,
    acceptNode: function () { return 1; }
  };

  // 内部：构造 TreeWalker/NodeIterator 共用的节点遍历器（R2803）。**eager pre-order** 经 `childNodes`
  // 递归收集子树（element 子为 selector-based proxy 可递归；文本/注释为静态叶节点），按 whatToShow 掩码 +
  // acceptNode 过滤。nextNode/previousNode 在过滤后序列上游走。TreeWalker 与 NodeIterator 共用基础接口；
  // `isTreeWalker` 时附加 DOM §4.2.6 层级方法（parentNode/firstChild/lastChild/previousSibling/nextSibling，
  // R3257）——NodeIterator 无这些方法（spec §4.2.5）。
  // **已知限制**：① eager（非 lazy，spec TreeWalker 惰性——小树无碍，结果序一致）；② currentNode setter
  // 不重置游标（spec 应从 currentNode 续遍历）；③ 无 live/detach（NodeIterator 移除节点 detach defer）。
  function _makeNodeWalker(root, whatToShow, filter, isTreeWalker) {
    // R41：spec `document-createtreewalker` 步骤 1——root 缺省/无效（无 nodeType）抛 TypeError
    //（WPT TreeWalker-basic "Give an invalid root node"）。
    if (!root || typeof root.nodeType !== 'number') {
      throw new globalThis.TypeError("Argument 1 of Document.createTreeWalker is not an object.");
    }
    // R82：whatToShow 无符号语义（WebIDL unsigned long——ToUint32）。`| 0` 是 ToInt32
    //（0xFFFFFFFF → -1），WPT NodeIterator/TreeWalker `whatToShow=0xFFFFFFFF` 期望
    // 4294967295；且 -1 的位掩码 `(-1 & 0x1)===1` 恰好仍全通过（补码），故 mask 行为
    // 旧实现侥幸正确——可见缺陷在 getter 读回值。`>>> 0` 保 ToUint32。null 语义由
    // 调用方（createTreeWalker/Iterator）按 arguments.length 区分后传入。
    var wts = (whatToShow == null) ? 0xFFFFFFFF : (whatToShow >>> 0);
    // R51：spec `document-createtreewalker` 步骤 4 + WebIDL §callback NodeFilter——filter 以
    // **callback 对象**形态保存（不一次性解绑）：函数 → 直接调用（this=undefined）；对象 → 每次
    // 遍历经 `Get(filter, "acceptNode")` 取 callable（getter 每次执行、抛错原样重抛、this=filter
    // 对象）；acceptNode 缺失/非 callable → 调用时抛 TypeError（WPT TreeWalker-acceptNode-filter
    // "lacking/non-function acceptNode"、"rethrows errors when getting acceptNode"、
    // "performs Get on every traverse"、"this value and node argument"）。
    var filterObj = (filter === null || filter === undefined) ? null : filter;
    function maskFor(node) {
      var nt = node && node.nodeType;
      // R83：全 nodeType 位掩码（spec dom-nodefilter SHOW_* 表——nodeType N 对应位
      // 1 << (N-1)）。旧仅 element/text/comment 三类 → doctype(10)/fragment(11)/PI(7)/
      // document(9)/CDATA(4) 全被掩掉（WPT NodeIterator "expected DocumentType/Document
      // Fragment but got null" 族）。document(9) 是 walker root 时 accepted 必含（spec
      // iteration 含 root——SHOW_ALL 下 root 全型可见）。
      return nt >= 1 && nt <= 13 ? (1 << (nt - 1)) >>> 0 : 0;
    }
    function callFilter(node) {
      if (typeof filterObj === 'function') return filterObj(node) | 0;
      var fn = filterObj.acceptNode; // 每次 Get（getter 副作用/抛错按 spec 传播）
      if (typeof fn !== 'function') {
        throw new globalThis.TypeError("NodeFilter object has no acceptNode method");
      }
      return fn.call(filterObj, node) | 0;
    }
    function check(node) {
      if ((wts & maskFor(node)) === 0) return 3; // 不在 whatToShow → SKIP（不入列，但仍遍历子树)
      if (!filterObj) return 1; // 无 filter → ACCEPT
      // R88：filter 执行期登记 in-flight 候选 + 方向指针态（filter 内移除节点时
      // _zwNotifyIteratorsRemove 消费——WPT removal-during-filtering）。方向由调用方
      // 经 checkFlight 约定（nextNode → before=false / previousNode → before=true）。
      var _r88PrevFlight = (typeof inFlightVal !== 'undefined') ? inFlightVal : null;
      var _r88PrevFlightB = (typeof inFlightBeforeVal !== 'undefined') ? inFlightBeforeVal : false;
      if (typeof inFlightVal !== 'undefined') { inFlightVal = node; }
      try {
        return _checkFiltered(node);
      } finally {
        if (typeof inFlightVal !== 'undefined') { inFlightVal = _r88PrevFlight; inFlightBeforeVal = _r88PrevFlightB; }
        // R88 注：flightRetargeted 不在此清——遍历方法 wrapper 须先读到它才能跳过 ref
        // 回写（check finally 先于 wrapper 运行；清理在 wrapper 返回路径完成）。
      }
    }
    function _checkFiltered(node) {
      // R51：spec `callbackdef-nodefilter`——filter 抛错**原样重抛**（不吞、不默认 ACCEPT；
      // WPT "filter function/object that throws" assert_throws_exactly + currentNode 不动）。
      // R85：返回**原始值**（含 false→0）——0 的遍历语义**按方法分叉**（WPT TreeWalker.html
      // oracle 实证）：firstChild/lastChild 的 children 循环对 0 不 dig 子（类 REJECT），
      // nextSibling/previousSibling 的 sibling 循环对 0 经 firstChild dig（类 SKIP），
      // NodeIterator 恒不剪。消费方按各自算法解释（R84 的统一归一是过度概化）。
      return callFilter(node);
    }
    var accepted = [];
    var parentAcceptedIdx = []; // R3257：每 accepted 节点的「最近 ACCEPTED 祖先」在 accepted 中的 idx（无=-1）
    var walked = false;
    // 深度优先 pre-order：ACCEPT/SKIP 入子树，REJECT 剪子树。ancestorIdx = 当前子树的最近 ACCEPTED 祖先 idx。
    // R84：**root 永不 filter**（spec：iteration collection 含 root，filter 只作用于遍历步进的
    // 候选节点；WPT paras[0]+#filter 的 firstChild 期望 #text——旧对 root 也 check 使 REJECT
    // 剪掉整棵子树全 null）。walk 入口对 root 直接递归子树。
    // R85：0（filter 返 false）在 walk 物化（firstChild/lastChild 消费的 accepted 表）按
    // REJECT 剪——oracle children 循环对 0 只横移不 dig。
    function walk(node, ancestorIdx) {
      if (!node) return;
      var r = check(node);
      var nextAncestor = ancestorIdx;
      if (r === 1) {
        var myIdx = accepted.length;
        accepted.push(node);
        parentAcceptedIdx.push(ancestorIdx);
        nextAncestor = myIdx; // ACCEPT → 我成为子树的最近 ACCEPTED 祖先
      }
      // SKIP(3) → 不入列，子树沿用 ancestorIdx；REJECT(2) 与 0 → 不入列且剪子树（不递归）。
      if ((r === 1 || r === 3) && node.childNodes) {
        var kids = node.childNodes;
        for (var i = 0; i < kids.length; i++) walk(kids[i], nextAncestor);
      }
    }
    function walkRoot() {
      var kids0 = nodeChildren(root);
      for (var i0 = 0; i0 < kids0.length; i0++) walk(kids0[i0], -1);
    }
    // R51：spec TreeWalker filter 是 **lazy** 的——eager walk 延迟到首个遍历方法调用（物化）。
    // WPT TreeWalker-acceptNode-filter：`createTreeWalker(root, wts, {})` 构造**不抛**，
    // `walker.firstChild()` 才抛 TypeError（acceptNode 缺失）/ filter 抛错原样重抛——eager 构造
    // 期执行会把异常提前到构造点，违反 spec 时序。物化失败（filter 抛错）清 accepted + walked，
    // 使下一次 traverse 调用重新物化（WPT "filter that throws"：firstChild 抛后 nextNode 也抛，
    // currentNode 保持 root）。
    function materialize() {
      if (walked) return;
      walked = true;
      try {
        walkRoot(); // R84：root 不 filter（iteration collection 含 root）
      } catch (_e) {
        accepted = [];
        parentAcceptedIdx = [];
        walked = false;
        throw _e;
      }
    }
    var idx = -1; // -1 = 尚未定位（fresh，currentNode=root）；nextNode 首调落到 root（若 accepted）→ R2803 语义
    // R84：currentNode 是否被显式重定位过（setter 设置）——fresh 与「重定位到被滤节点」须区分：
    // fresh（currentNode=root）时 effPos 按 root 是否 accepted 取 0/-1；重定位后被滤节点（如
    // doctype 对 SHOW_ELEMENT）恒虚拟 -1（WPT TreeWalker createTreeWalker(document,
    // SHOW_ELEMENT|SHOW_DOCUMENT) currentNode=doctype → nextSibling 期望 html——旧 fresh
    // 分支误返 0 走 accepted 同父扫描找不到）。
    var relocated = false;
    // R3257：层级方法的「有效位置」。idx>=0 时为 idx；fresh（idx=-1，currentNode=root）时——若 root accepted
    //（accepted[0]===root）逻辑位置=0，否则虚拟 -1（root 不在 accepted，其 filtered-子以 -1 为最近祖先）。
    function effPos() {
      if (idx >= 0) return idx;
      if (relocated) return -1;
      return accepted.length > 0 && accepted[0] === root ? 0 : -1;
    }
    function moveTo(i) { idx = i; currentNodeVal = accepted[i]; syncOrderPosTo(currentNodeVal); return accepted[i]; }
    var walker = {};
    // R41：spec `treewalker` 接口——root/whatToShow/filter 为 readonly attribute（WPT TreeWalker-basic
    // assert_readonly）。经 defineProperty getter-only 实现（无 setter → writable:false 语义）。
    // whatToShow：**显式传 null** 按 0 处理（spec ToUint32(null)=0，区别缺省 undefined → SHOW_ALL；
    // WPT "root, null, null" 断言 whatToShow===0）。
    // R82：whatToShow 显式 null → ToUint32(null)=0（spec WebIDL 缺省才 0xFFFFFFFF——本
    // 工厂把 null 视作「缺省 SHOW_ALL」是 R41 既有语义，保持；读回值与 wts 同源无符号）。
    var wtsStored = wts;
    Object.defineProperty(walker, 'root', { get: function () { return root; }, configurable: true });
    Object.defineProperty(walker, 'whatToShow', { get: function () { return wtsStored; }, configurable: true });
    Object.defineProperty(walker, 'filter', { get: function () { return filter || null; }, configurable: true });
    // R41：currentNode setter——spec：赋非 Node 值抛 TypeError（WPT TreeWalker-currentNode
    // "setting currentNode to non-Node values throws"）。Node 判定 = 有 nodeType 数字。赋合法 Node：
    // 更新 currentNode + 重置游标到「该节点在 accepted 中的位置」（命中则定位；root 外节点 -1 =
    // 从头语义的近似——lazy 续走属 M1 L2 lazy 重构，eager 模型下尽力而为）。
    var currentNodeVal = root;
    Object.defineProperty(walker, 'currentNode', {
      get: function () { return currentNodeVal; },
      set: function (v) {
        if (!v || typeof v.nodeType !== 'number') {
          throw new globalThis.TypeError("currentNode is not a Node");
        }
        // R89：spec TreeWalker.currentNode setter 是**纯赋值**——不跑 filter（WPT
        // "Recursive filters need to throw"：filter 在 setter 期间重入不抛、首个遍历
        // 方法才因 active flag 抛）。旧 setter eager materialize 让 filter 提前执行，
        // 既消耗 filter 副作用窗口又把重入异常从 setter 泄漏。改惰性：只记录节点，
        // accepted 定位（idx）延迟到下次遍历方法物化后按需重算（materialize 后
        // effPos 走 relocated 分支自然正确）。
        currentNodeVal = v;
        relocated = true; // R84：重定位标记（effPos 区分 fresh vs 被滤节点）
        idx = -1; // 物化后由 effPos/accepted 按需定位（setter 不再 eager 查表）
        syncOrderPosTo(v); // R51：lazy 步进游标随 currentNode 重定位
      },
      configurable: true
    });
    // R41：spec 接口 branding——WPT TreeWalker-basic 断言 `String(walker) === "[object TreeWalker]"`
    //（NodeIterator 同理 "[object NodeIterator]"）。普通 data 属性即可（String() 走 toString 优先）。
    walker.toString = function () { return isTreeWalker ? '[object TreeWalker]' : '[object NodeIterator]'; };
    // R51：spec `nodeiterator` 专有属性——referenceNode（最近 accepted 节点）+
    // pointerBeforeReferenceNode（游标在 referenceNode 前/后，nextNode 置前 true、previousNode
    // 置后 false；WPT NodeIterator.html 全程断言）。TreeWalker 无此二属性。
    if (!isTreeWalker) {
      var refNodeVal = root;
      var beforeRefVal = true;
      // R88：in-flight 遍历状态——nextNode/previousNode 的 filter 执行期间登记候选节点
      // 与方向语义的指针态（nextNode 候选 → before=false；previousNode 候选 → before=true），
      // filter 返回后清空（非 filter 期间的移除不受影响）。
      var inFlightVal = null;
      var inFlightBeforeVal = false;
      Object.defineProperty(walker, 'referenceNode', { get: function () { return refNodeVal; }, configurable: true });
      Object.defineProperty(walker, 'pointerBeforeReferenceNode', { get: function () { return beforeRefVal; }, configurable: true });
      // R82：同步钩子后移——wrapper 原定义在此处，被下方 R51 lazy nextNode/previousNode
      // 重赋值覆盖（reference/before 恒不更新——WPT pointerBeforeReferenceNode 40F 根因）。
      // 现移至工厂尾部（lazy 定义之后）统一包装。
    }
    // R51：nextNode/previousNode 改 **lazy 步进**（spec TreeWalker/NodeIterator 惰性遍历）。
    // 模型：物化**结构序**（pre-order 全节点数组，只读 childNodes 无 filter 调用——构造零异常），
    // 步进时才对候选节点调 filter（WPT "performs Get on every traverse"：两次 nextNode = 恰两次
    // acceptNode Get；"this value and node argument"：首调 node=A1）。REJECT 剪子树 = 跳过该节点
    // 的整个子树区间（orderEnd 预计算）。currentNode 经 setter/层级方法移动后，下次 nextNode 从
    // currentNode 的结构序后继续。filter 抛错原样传播，currentNode 不动（仅成功步进更新）。
    var order = null; // pre-order 全节点数组（含 root）
    var orderEnd = null; // 每节点的子树 exclusive-end 索引（REJECT 剪枝用）
    var orderPos = -1; // 结构序游标（-1 = 未定位，下次步进从头找 currentNode）
    function nodeChildren(n) {
      try { return (n && n.childNodes) ? Array.prototype.slice.call(n.childNodes) : []; } catch (_e) { return []; }
    }
    function orderInit() {
      if (order) return;
      order = [];
      orderEnd = [];
      collect2(root);
    }
    function collect2(n) {
      var i = order.length;
      order.push(n);
      var kids = nodeChildren(n);
      var end = i + 1;
      for (var k = 0; k < kids.length; k++) { var e2 = collect2(kids[k]); if (e2 > end) end = e2; }
      orderEnd[i] = end;
      return end;
    }
    function syncOrderPosTo(node) {
      orderInit(); // 层级方法路径可能先于 nextNode 触发（order 未建）
      if (node === root) { orderPos = 0; return 0; }
      for (var i = 0; i < order.length; i++) { if (order[i] === node) { orderPos = i; return i; } }
      orderPos = -1;
      return -1;
    }
    // R84：遍历中标志（filter 重入检测）。NodeIterator 专属 detach() no-op（spec：历史方法
    // 恒 no-op，WPT "detach() should be a no-op"——iter.detach 可调用且无副作用）。
    var active = false;
    if (!isTreeWalker) walker.detach = function () { return undefined; };
    // js-dom M3 R97：跨树重定位步进——currentNode 被赋值为 root 快照外的节点（detached
    // fragment / template content / 另一棵树）时，spec `treewalker` 语义是后续遍历从
    // currentNode 的位置继续（currentNode 不受 root 限制；lit-html 用单个全局 TreeWalker
    // 经 `P.currentNode = fragment` 重定位遍历 template parts——这正是其 Template/
    // TemplateInstance 的核心机制）。order 快照不含 root 外节点 → orderPos=-1 旧逻辑
    // 从 root 头遍历（探针实证 walk:HTML>HEAD>BODY...）。此函数以 currentNode 为起点
    // 沿真实 getter 导航（firstChild/nextSibling/parentNode）找结构序后继，对每候选
    // check（TreeWalker REJECT/0 剪子树 = 不入其 firstChild；NodeIterator 不剪）。
    // 终点：候选链上行到 currentNode 的根（root 外树的顶）即 null → 遍历耗尽返 null
    //（spec：TreeWalker 越过根后 nextNode 返 null 且 currentNode 不动）。
    function nextNodeOffOrder() {
      var start = currentNodeVal;
      if (!start) return null;
      // 状态机：descend（尝试 node.firstChild）vs advance（横移 node.nextSibling）。
      // 上行到祖先后**只横移不重入子**（祖先的 firstChild 半边已在前序中过——重入会
      // 死循环：EM→text(SKIP)→上行 EM→又 firstChild=text）。
      var descend = true;
      var node = start;
      var guard = 0;
      while (node && guard++ < 100000) {
        if (descend) {
          var kid = node.firstChild;
          if (kid) {
            var r = check(kid);
            if (r === 1) { currentNodeVal = kid; return kid; }
            if (isTreeWalker && (r === 2 || r === 0)) {
              // REJECT/0 剪子树：不 descend 进 kid，横移 kid 的 nextSibling。
              var ks = kid.nextSibling;
              if (ks) {
                var rk = check(ks);
                if (rk === 1) { currentNodeVal = ks; return ks; }
                if (isTreeWalker && (rk === 2 || rk === 0)) { node = ks; descend = false; continue; }
                node = ks; descend = true; continue;
              }
            } else {
              node = kid; descend = true; continue; // SKIP / NodeIterator-REJECT：入其子树
            }
          }
        }
        // 横移本层 nextSibling。
        var sib = node.nextSibling;
        if (sib) {
          var r2 = check(sib);
          if (r2 === 1) { currentNodeVal = sib; return sib; }
          if (isTreeWalker && (r2 === 2 || r2 === 0)) { node = sib; descend = false; continue; }
          node = sib; descend = true; continue;
        }
        // 兄弟尽 → 上行（只横移：descend=false）。
        node = node.parentNode;
        descend = false;
        // 上行到 start 所在树的顶（parentNode null）即耗尽（spec：越根返 null）。
      }
      return null;
    }
    walker.nextNode = function () {
      orderInit();
      // R84：spec NodeIterator/TreeWalker「recursive filters need to throw」——遍历方法
      // 重入（filter 内再调 iter.nextNode/previousNode）抛 InvalidStateError（WPT
      // NodeIterator "Recursive filters need to throw"）。active flag 在 finally 复位，
      // 外层 nextNode 继续正常步进（WPT：外层两次 nextNode 正常，第三次才因内层状态断言）。
      if (active) {
        throw new (globalThis.DOMException || Error)('Recursive filters are not allowed', 'InvalidStateError');
      }
      active = true;
      try {
        // R88：nextNode 的 in-flight 方向指针态——候选在指针后侧（before=false）。
        if (typeof inFlightBeforeVal !== 'undefined') inFlightBeforeVal = false;
        // R97：currentNode 被重定位到 root 快照外（orderPos<0 且非 fresh——relocated）→
        // order 快照不含该子树，改导航式步进（见 nextNodeOffOrder 注释）。
        if (orderPos < 0 && relocated) {
          var off = nextNodeOffOrder();
          if (off) { idx = accepted.indexOf(off); if (idx < 0) idx = -1; }
          return off;
        }
        // R83：fresh 起点按 walker 类型区分——NodeIterator 的迭代集合**含 root**（指针初始
        // 在 root 前 → 首个 nextNode 返 root；WPT NodeIterator-removal-during-filtering /
        // R2803 单测 DIV 首位）；TreeWalker 的 currentNode=root 表示**已位于 root**（visited）
        // → nextNode 越过 root 从其后继开始（WPT TreeWalker-acceptNode-filter "this value
        // and node argument" 期望首个 nextNode 的 filter 收 A1 非 root）。
        var i = orderPos < 0 ? (isTreeWalker ? 1 : 0) : orderPos + 1;
        while (i < order.length) {
          var node = order[i];
          // R86：live 迭代集合——已移除节点（含子树）退出集合（spec `nodeiterator`：
          // 移除的节点从 iterator list 剔除；WPT NodeIterator-removal 全簇）。order 是
          // 构造期快照，读时按移除标记跳过（子树节点经 _zwIsRemovedNode 沿父链上行
          // 命中被移除祖先）。
          if ((typeof _zwIsRemovedNode === 'function') && _zwIsRemovedNode(node)) {
            i = orderEnd[i] > i ? orderEnd[i] : i + 1; // 整子树跳过（子随父移除）
            continue;
          }
          var r = check(node);
          if (r === 1) { orderPos = i; idx = accepted.indexOf(node); currentNodeVal = node; return node; }
          if (isTreeWalker) {
            // R85：TreeWalker nextNode——显式 REJECT(2) 与 0（false）都剪子树（WPT TreeWalker
            // nextNode 期望树与 oracle 一致：document 序下一步跳过被拒节点整个子树）。
            if (r === 2 || r === 0) { i = orderEnd[i]; continue; }
          } else if (r === 2) { i = orderEnd[i]; continue; } // NodeIterator：仅显式 REJECT 剪（0 不剪）
          i++; // SKIP/0 → 下一节点（子树仍遍历）
        }
        orderPos = order.length;
        return null;
      } finally {
        active = false;
      }
    };
    walker.previousNode = function () {
      orderInit();
      // R84：重入守卫（同 nextNode——WPT "Recursive filters need to throw" 双向断言）。
      if (active) {
        throw new (globalThis.DOMException || Error)('Recursive filters are not allowed', 'InvalidStateError');
      }
      active = true;
      try {
        // R88：previousNode 的 in-flight 方向指针态——候选在指针前侧（before=true）。
        if (typeof inFlightBeforeVal !== 'undefined') inFlightBeforeVal = true;
        if (isTreeWalker) {
          // R85：TreeWalker previousNode 改**导航式逆向树序步进**（nextNode 的精确镜像，
          // WPT NodeIterator.html previousNode oracle 算法）：候选 = currentNode 的前驱
          //（有子 → lastChild 最深；无 → previousSibling；兄弟尽 → 父的前兄弟）。
          // REJECT 剪子树 = 越过该候选时直接步进到其结构前驱（不进入其子树——逆向
          // 天然不进入）。旧 order-scan 逆向对「被拒子树内节点先于被拒祖先命中」无法
          // 排除（WPT traversal-reject previousNode：B3→B2 期望 A1，B1 REJECT 后 C1
          // 仍在候选序内错返）。
          // R85：TreeWalker previousNode —— DOM 规范算法（旧版 DOM Traversal previousNode
          // 的镜像 nextNode）：sibling = node.previousSibling；REJECT → sibling = 其
          // previousSibling（跳子树）；SKIP/0 → sibling = lastChild || previousSibling
          //（先入子树尾）；兄弟尽 → node = parentNode（root 止 null）→ filter 续循环。
          var node = currentNodeVal;
          var guard = 0;
          while (node && guard++ < 100000) {
            var sibling = node.previousSibling;
            var inner = 0;
            while (sibling && inner++ < 100000) {
              node = sibling;
              var r = check(node);
              if (r === 1) {
                // R89：ACCEPT 且有子 → 先入子树尾继续找「filtered 序前驱」（WebKit/
                // Blink 的 previousNode = 前一个可见节点——WPT previousNodeLastChild
                // Reject：cur=B2、sibling=B1 ACCEPT 但 B1 有子 → 期望 C1 非 B1；
                // childless 才返）。traversal-reject 的 B2（childless）两模型同果。
                if (node.lastChild) { sibling = node.lastChild; continue; }
                currentNodeVal = node; idx = accepted.indexOf(node);
                // R316（js-dom M4）：着陆点不在 order 快照内（orderPos=-1）→ 快照已
                // stale（regraft 等树重构后新位置不在快照）。置 relocated 使后续
                // nextNode 走 live 导航（nextNodeOffOrder 沿真实 getter 步进——WPT
                // TreeWalker-walking-outside-a-tree 第 4-5 断言：title→p→body 回溯链
                // 在 stale 快照外，order-scan 越界恒 null）。
                if (syncOrderPosTo(node) < 0) relocated = true;
                return node;
              }
              if (r === 2) {
                sibling = node.previousSibling; // REJECT → 跳过子树
              } else {
                sibling = node.lastChild || node.previousSibling; // SKIP/0 → 先入子树尾
              }
            }
            node = node.parentNode;
            // R314（js-dom M4）：**root 止步的 spec 语义**——上行到的父是 root 自身
            // （或脱离 root 树）→ 前驱必在 root 外 → null（spec previousNode 的
            // 「node == root → return null」分支对双向 parent-up 生效——旧只挡
            // `node === root` 但**仍会对 root 自身跑 check 并可能返回**（探针实证
            // prevNode=DIV：currentNode=regraft 的 p、父链上到 DIV(root 外) 被误返）。
            // WPT TreeWalker-walking-outside-a-tree 的 root 边界即此形态）。
            if (!node || node === root) return null;
            try {
              var _r314InRoot = false;
              var _r314Anc = node;
              var _r314g = 0;
              while (_r314Anc && _r314g++ < 1000) {
                if (_r314Anc === root) { _r314InRoot = true; break; }
                _r314Anc = _r314Anc.parentNode;
              }
              if (!_r314InRoot) return null;
            } catch (_e314r) {}
            var rp = check(node);
            // R316：同上——climb 着陆快照外节点时置 relocated（快照 stale）。
            if (rp === 1) {
              currentNodeVal = node; idx = accepted.indexOf(node);
              if (syncOrderPosTo(node) < 0) relocated = true;
              return node;
            }
            // rp 非 ACCEPT → 回 sibling 循环（node 的 previousSibling 起）
          }
          return null;
        }
        // NodeIterator previousNode：结构序逆向 scan（迭代集合结构性——不剪枝，
        // REJECT/SKIP/0 都只排除自身）。R86：移除节点（含子树）退出集合。
        // R87：spec「pointer-before=false → 仅翻 before=true、节点不动、返当前 ref」
        //（nextNode 对 before=true 的对称半边——WPT NodeIterator-removal
        // "backed-up reference" 断言：advance 到 (b1,false) 后 previousNode 期望
        // 返 b1 自身而非树序前驱）。before 的读/改经 R82 wrapper 的 refNodeVal/
        // beforeRefVal（本函数在 wrapper 内层）。
        if (typeof beforeRefVal !== 'undefined' && beforeRefVal === false) {
          // 翻指针由外层 R82 wrapper 完成（previousNode 成功 → beforeRefVal=true）；
          // 此处返当前 ref 前仍须过 filter（spec previousNode 步骤 4-5：referenceNode
          // 非 ACCEPT 则继续前驱——WPT "Recursive filters need to throw" 对
          // previousNode 的断言在此触发）。R88：先取 ref 快照——filter 内移除会把
          // refNodeVal retarget 到存活节点，返回值须仍是被 filter 的 ref（WPT
          // removal-during-filtering「returns the filtered node」）。
          var _r88Ref = refNodeVal;
          if (check(_r88Ref) === 1) {
            return _r88Ref;
          }
        }
        syncOrderPosTo(currentNodeVal);
        var i = orderPos;
        if (i < 0) return null;
        i -= 1;
        while (i >= 0) {
          var nd = order[i];
          if ((typeof _zwIsRemovedNode === 'function') && _zwIsRemovedNode(nd)) { i--; continue; }
          var r2 = check(nd);
          if (r2 === 1) { orderPos = i; idx = accepted.indexOf(nd); currentNodeVal = nd; return nd; }
          i--;
        }
        return null;
      } finally {
        active = false;
      }
    };
    if (isTreeWalker) {
      // DOM §4.2.6 TreeWalker 层级方法（R3257）。基于 accepted[]（pre-order）+ parentAcceptedIdx：
      // - parentNode(): 最近 ACCEPTED 祖先 = accepted[parentAcceptedIdx[effPos()]]（无则 null）。
      // - firstChild()/lastChild(): 首个/末个 parentAcceptedIdx[i]===effPos() 的 i（直接 filtered-子）。
      // - nextSibling()/previousSibling(): 同 parentAcceptedIdx（= parentAcceptedIdx[effPos()]）的下一/上一 i。
      // ACCEPTED 节点的祖先必非 REJECT（REJECT 剪子树），故 parentAcceptedIdx 给出 spec 定义的「最近 accepted 祖先」。
      // R84：五个层级方法统一经 _guarded 包裹（filter 重入抛 InvalidStateError——WPT
      // TreeWalker "Recursive filters need to throw" 对 parentNode/firstChild/lastChild/
      // previousSibling/nextSibling 全系断言）。
      function _guarded(fn) {
        return function () {
          if (active) {
            throw new (globalThis.DOMException || Error)('Recursive filters are not allowed', 'InvalidStateError');
          }
          active = true;
          try { return fn.apply(this, arguments); }
          finally { active = false; }
        };
      }
      // R85：四个层级方法 + parentNode 改 **导航式 oracle 循环**（WPT TreeWalker.html
      // testTraverseChildren/testTraverseSiblings 的精确算法镜像——经真实 firstChild/
      // lastChild/nextSibling/previousSibling/parentNode getter 步进，R84 兄弟链修复后
      // 导航可靠）。旧 accepted/parentAcceptedIdx 扫描模型无法表达「0（filter 返 false）
      // 按方法分叉」语义：children 循环对 0 只横移不 dig；sibling 循环对 0 经 firstChild
      // dig（类 SKIP）；父站真值即止（含 3/2/1，仅 0 续走）。check() 保持原始返回值。
      function navKids(type) { // type: 'first' | 'last'
        var cur = currentNodeVal;
        var node = type === 'first' ? cur.firstChild : cur.lastChild;
        var guard = 0;
        while (node && guard++ < 100000) {
          var result = check(node);
          if (result === 1) { currentNodeVal = node; syncOrderPosTo(node); idx = accepted.indexOf(node); return node; }
          if (result === 3) { // SKIP → dig child
            var child = type === 'first' ? node.firstChild : node.lastChild;
            if (child) { node = child; continue; }
          }
          // 横移循环（result 0/2/或 SKIP 无 child）：沿 next/prevSibling 找有 child 的下站。
          var stepped = false;
          while (node) {
            var sib = type === 'first' ? node.nextSibling : node.previousSibling;
            if (sib) { node = sib; stepped = true; break; }
            var parent = node.parentNode;
            if (!parent || parent === root || parent === cur) return null;
            if (filterNodeTruthy(parent)) return null;
            node = parent;
          }
          if (!stepped && !node) return null;
        }
        return null;
      }
      function filterNodeTruthy(node) {
        // R85：爬升父站仅 **ACCEPT(1) 止**（DOM spec traverse-siblings 步骤「Filter node and
        // if the return value is FILTER_ACCEPT then return null」——被接受的父是 nextNode
        // 的下一目标，与 sibling 竞争；SKIP/REJECT/0 续爬）。WPT traversal-skip-most 实证：
        // B2(Skip) 父站不止单步，B1→B3 期望（truthy 止会 null）。
        return check(node) === 1;
      }
      function navSiblings(type) { // type: 'next' | 'previous'
        var node = currentNodeVal;
        if (node === root) return null;
        var guard = 0;
        do {
          var sibling = type === 'next' ? node.nextSibling : node.previousSibling;
          var inner = 0;
          while (sibling && inner++ < 100000) {
            node = sibling;
            var result = check(node);
            if (result === 1) { currentNodeVal = node; syncOrderPosTo(node); idx = accepted.indexOf(node); return node; }
            sibling = type === 'next' ? node.firstChild : node.lastChild;
            if (result === 2 || !sibling) {
              sibling = type === 'next' ? node.nextSibling : node.previousSibling;
            }
          }
          node = node.parentNode;
          if (!node || node === root) return null;
          if (filterNodeTruthy(node)) return null;
        } while (guard++ < 100000);
        return null;
      }
      walker.parentNode = _guarded(function () {
        materialize();
        var node = currentNodeVal;
        var guard = 0;
        while (node && node !== root && guard++ < 100000) {
          node = node.parentNode;
          if (node && node !== root && check(node) === 1) {
            currentNodeVal = node; syncOrderPosTo(node); idx = accepted.indexOf(node); return node;
          }
        }
        return null;
      });
      walker.firstChild = _guarded(function () { materialize(); return navKids('first'); });
      walker.lastChild = _guarded(function () { materialize(); return navKids('last'); });
      walker.nextSibling = _guarded(function () { materialize(); return navSiblings('next'); });
      walker.previousSibling = _guarded(function () { materialize(); return navSiblings('previous'); });
    }
    // R82：NodeIterator 的 referenceNode/pointerBeforeReferenceNode 同步钩子——**必须在
    // lazy nextNode/previousNode 定义之后**包装（R51 重赋值覆盖了 R2981 前的旧 wrapper，
    // reference/before 恒不更新——WPT NodeIterator pointerBeforeReferenceNode 40F 根因）。
    // 语义：nextNode 成功命中 → reference=node + before=false；到尾返 null → 不动（WPT
    // 实证：立即耗尽的迭代器 before 保持 true——「after nextNode() 1 time(s)」期望 true）。
    if (!isTreeWalker) {
      var _rawNext2 = walker.nextNode, _rawPrev2 = walker.previousNode;
      // R88：filter 内 retarget 后，遍历方法的成功返回不再回写 ref（retarget 已把
      // reference 落到存活节点——WPT removal-during-filtering：filter(b) 内 b.remove()
      // → retarget(a,false)，nextNode 返 b 但 reference 须留 a）。flightRetargeted 由
      // retarget() 在 in-flight 窗口内置位、check() finally 清零。
      var flightRetargeted = false;
      walker.nextNode = function () {
        var r = _rawNext2.apply(this, arguments);
        // R88：filter 内 retarget 已生效 → 不回写 ref（返回值仍是被 filter 节点）。
        if (r && !flightRetargeted) { refNodeVal = r; beforeRefVal = false; }
        flightRetargeted = false;
        return r;
      };
      walker.previousNode = function () {
        var r = _rawPrev2.apply(this, arguments);
        if (r && !flightRetargeted) { refNodeVal = r; beforeRefVal = true; }
        flightRetargeted = false;
        return r;
      };
      // R86：NodeIterator 移除 retarget（spec `nodeiterator-remove`——迭代集合 live：
      // 节点移除时，reference 在被移除节点/子树内 → 指针后置取 removed 的树序前驱、
      // 指针前置取后继）。注册到全局表，remove 路径（_zwNotifyIteratorsRemove）遍历。
      globalThis._zwIterRegistry.push({
        root: root,
        getRef: function () { return refNodeVal; },
        getBefore: function () { return beforeRefVal; },
        // R88：in-flight 候选（filter 正在执行的节点）+ 遍历方向指针态——filter 内
        // 移除时 pre-remove 步骤对 in-flight 位置生效（WPT removal-during-filtering）。
        getInFlight: function () { return inFlightVal || null; },
        getInFlightBefore: function () { return inFlightBeforeVal; },
        retarget: function (newRef, newBefore) {
          refNodeVal = newRef; beforeRefVal = newBefore;
        },
        // R88：in-flight 路径的 retarget（仅当 in-flight 候选自身在 removed 子树内时
        // 由 notify 调用）——置 flightRetargeted 使遍历方法返回时不回写 ref；常规
        // retarget（referenceNode 命中但 in-flight 无关，如 filter 内移除已访问节点）
        // 不抑制——spec 期望 filter 返回后 ref 落新 accepted（WPT "already-visited"）。
        retargetInFlight: function (newRef, newBefore) {
          refNodeVal = newRef; beforeRefVal = newBefore;
          if (inFlightVal) flightRetargeted = true;
        },
        dead: false,
      });
    }
    return walker;
  }
  // R86：全局迭代器注册表 + 移除通知（512 软上限防泄漏——迭代器随 GC 语义上失效，
  // 这里无 finalizer，靠容量压实的近似；retarget 对已耗尽/无关迭代器幂等 no-op）。
  globalThis._zwIterRegistry = globalThis._zwIterRegistry || [];
  globalThis._zwNotifyIteratorsRemove = function (removedNode) {
    var reg = globalThis._zwIterRegistry;
    if (!reg || !reg.length || !removedNode) return;
    // R87：容量压实改「保尾」——旧版 `reg.length = 0` 全清会在大用例中途（~500
    // iterator/子测试 × 多子测试）把**在档**迭代器一并清掉 → retarget 静默丢失
    //（WPT NodeIterator-removal doctype 子测试根因）。65536 上限 + 保最近 1024。
    if (reg.length > 65536) reg.splice(0, reg.length - 1024);
    // removed 的树序前驱/后继（限 removed 所属文档序；root 边界由 retarget 后集合
    // 自然约束——前驱超 root 时回落到后继，spec 步骤 3 的「first node preceding」边界）。
    var pred = null, succ = null;
    // 树序前驱：有子不算（removed 的子在其后）——previousSibling 的最深最右 / 父。
    try {
      if (removedNode.previousSibling) {
        var p = removedNode.previousSibling;
        while (p && p.lastChild) p = p.lastChild;
        pred = p;
      } else {
        pred = removedNode.parentNode || null;
      }
    } catch (_e2) { pred = null; }
    try {
      if (removedNode.parentNode) {
        var s = removedNode.nextSibling;
        if (s) { succ = s; }
        else {
          var c = removedNode.parentNode;
          while (c && !c.nextSibling) c = c.parentNode;
          succ = c ? c.nextSibling : null;
        }
      }
    } catch (_e3) { succ = null; }
    function inSubtree(node) {
      // node 是否在 removedNode 子树内（含 removed 自身）——沿 parentNode 上行。
      try {
        var cur = node, guard = 0;
        while (cur && guard++ < 128) {
          if (cur === removedNode) return true;
          cur = cur.parentNode;
        }
      } catch (_e) {}
      return false;
    }
    function isAnc(node) {
      // removedNode 是否是 node（root）的 inclusive ancestor（spec「toBeRemoved is an
      // inclusive ancestor of root → no-op」）——沿 root 的父链上行找 removed。
      try {
        var cur = node, guard = 0;
        while (cur && guard++ < 128) {
          if (cur === removedNode) return true;
          cur = cur.parentNode;
        }
      } catch (_e) {}
      return false;
    }
    function inRootOf(node, rootNode) {
      // node 是否在 rootNode 的 inclusive 子树内——沿 node 父链上行找 rootNode。
      try {
        var cur = node, guard = 0;
        while (cur && guard++ < 128) {
          if (cur === rootNode) return true;
          cur = cur.parentNode;
        }
      } catch (_e) {}
      return false;
    }
    for (var i = 0; i < reg.length; i++) {
      var it = reg[i];
      if (!it || it.dead) continue;
      var ref = it.getRef();
      if (isAnc(it.root)) continue; // removed 是 root 的祖先 → no-op
      // R88：filter 执行中的 in-flight 遍历——pre-remove 步骤对「正在被 filter 的候选
      // 位置」生效（spec concept-nodeiterator-traverse：filter 内移除时 in-flight 指针
      // 同步 retarget，返回值仍是被 filter 节点）。in-flight candidate 在 removed 子树
      // 内（filter 的就是 removed 或其后代）→ 以 candidate 代替 referenceNode 参与判定
      //（WPT removal-during-filtering：filter(b) 内 b.remove() → in-flight=b 在 b 子树
      // 内 → retarget 到 a；filter(b1) 内 remove(b) → in-flight=b1 在 b 子树内 →
      // 「pointer before + root 内无后继」分支翻 false 落 a1）。
      var inFlight = it.getInFlight && it.getInFlight();
      var subj = (inFlight && inSubtree(inFlight)) ? inFlight : null;
      if (!subj && !inSubtree(ref)) continue; // reference/in-flight 均不在 removed 子树内 → no-op
      // R87：succ 须在 root 子树内（spec「first node following... within root」——
      // 跨出 root 边界的后继不算；用 inRoot 判定）。
      var succInRoot = succ && inRootOf(succ, it.root);
      // R88：in-flight 的指针态取遍历方向（nextNode → before=false / previousNode →
      // before=true——candidate 尚未被接受，指针在语义上位于 candidate 的下一/上一侧）。
      var effBefore = subj ? (it.getInFlightBefore ? it.getInFlightBefore() : it.getBefore()) : it.getBefore();
      // R88：retarget 目标计算对 in-flight 与常规一致（pred/succ 分支同 spec），但
      // in-flight 路径经 retargetInFlight（置 flightRetargeted 抑制遍历返回时的 ref
      // 回写——返回值仍是被 filter 节点，reference 已落存活节点）。
      var fire = subj ? it.retargetInFlight : it.retarget;
      if (!fire) fire = it.retarget;
      if (effBefore) {
        // 指针前置 → reference = removed 后继（保持 before=true）；root 内无后继 →
        // spec 步骤 3：pointer 翻 false + reference = removed 前驱（其前兄弟的
        // last inclusive descendant——即 pred）。
        if (succInRoot) {
          fire(succ, true);
        } else {
          fire(pred || it.root, pred ? false : true);
        }
      } else {
        // 指针后置 → reference = removed 前驱；前驱为 null（root 前）→ 后继。
        fire(pred || succ, pred ? false : true);
      }
    }
  };

  // ── XPath（document.evaluate，R2981）─────────────────────────────────────
  // 实用 XPath 1.0 子集求值器。headless 测试/抓取/遗留代码经 `document.evaluate('//div[@id]',
  // document, null, XPathResult.ANY_TYPE, null).iterateNext()` 查询——此前全缺（document.evaluate /
  // XPathResult 零定义）→ ReferenceError 中断脚本。本子集覆盖真实页面最常见的查询模式：
  //   路径：`//tag`（descendant）、`/abs/path`（绝对 child 链）、`rel/rel`（相对 child）、`//a//b`、`.//x`、`../x`
  //   节点测试：tag、`*`、`text()`、`node()`、`comment()`
  //   谓词：`[n]`、`[last()]`、`[last()-n]`、`[@a]`、`[@a='v']`、`[@a!="v"]`、`[text()='v']`、`[text()!='v']`、
  //          `[contains(@a,'s')]`、`[contains(text(),'s')]`、`[contains(.,'s')]`、`[.='v']`、`[.!='v']`、
  //          `[position() op n]`（op = == != < > <= >=）
  //   属性轴结果：`//a/@href` → 伪 Attr 节点（nodeType 2，nodeValue/.value = 属性值）
  // **已知限制 / 近似**：① child 轴谓词 position 严格 per-parent（spec 一致）；descendant 轴（`//`）谓词
  //   position 取「整候选集文档序」位置（= `(//tag)[n]` 语义，多数人预期，非严格 XPath per-ancestor 分组）；
  // ② namespace resolver 忽略；③ 不支持的构造（命名轴 `axis::`、sum/floor 等函数、变量引用）→ 抛 SyntaxError
  //   （honest failure，spec INVALID_EXPRESSION_ERR 语义；优于静默错结果）；④ live 更新无（快照语义）。
  function _xpathAllDesc(node, out) {
    var kids = (node && node.childNodes) || [];
    for (var i = 0; i < kids.length; i++) {
      var k = kids[i];
      out.push(k);
      if (k && k.nodeType === 1) _xpathAllDesc(k, out);
    }
  }
  function _xpathParent(node) {
    if (!node) return null;
    try { return node.parentNode || node.parentElement || null; } catch (_e) { return null; }
  }
  // 节点稳定身份键（dedup）：元素用 __zwSelector；文本/注释用 parent 选择器 + 索引 + 值；属性用 owner + 名。
  function _xpathKey(n) {
    if (!n) return '';
    var nt = n.nodeType;
    if (nt === 1) return 'e:' + (n.__zwSelector || '');
    if (nt === 2) return 'a:' + (n.ownerElement && n.ownerElement.__zwSelector || '') + ':' + (n.name || '');
    if (nt === 3 || nt === 8) {
      var p = _xpathParent(n), pi = -1;
      if (p && p.childNodes) { for (var i = 0; i < p.childNodes.length; i++) { if (p.childNodes[i] === n) { pi = i; break; } } }
      return 't:' + (p && p.__zwSelector || '?') + ':' + pi + ':' + String(n.nodeValue == null ? '' : n.nodeValue);
    }
    return 'x:' + nt;
  }
  function _xpathTest(node, test) {
    if (!node) return false;
    var nt = node.nodeType;
    if (test === 'node') return true;
    if (test === 'text') return nt === 3;
    if (test === 'comment') return nt === 8;
    if (test === 'attr') return nt === 2;
    if (nt !== 1) return false;
    if (test === '*' || test === 'element') return true;
    try { return String(node.tagName).toUpperCase() === String(test).toUpperCase(); } catch (_e) { return false; }
  }
  function _xpathAxisCandidates(ctx, axis) {
    var out = [];
    if (axis === 'self') { out.push(ctx); }
    else if (axis === 'parent') { var p = _xpathParent(ctx); if (p) out.push(p); }
    else if (axis === 'child') { var kids = (ctx && ctx.childNodes) || []; for (var i = 0; i < kids.length; i++) out.push(kids[i]); }
    else if (axis === 'descendant') { _xpathAllDesc(ctx, out); }
    // attribute 轴在 _xpathApplyStep 内特判（产出伪 Attr 节点）。
    return out;
  }
  // 节点 string-value：元素 textContent；文本/注释 nodeValue；属性 value。
  function _xpathNodeStr(n) {
    if (!n) return '';
    if (n.nodeType === 2) return String(n.value == null ? '' : n.value);
    if (n.nodeType === 3 || n.nodeType === 8) return String(n.nodeValue == null ? '' : n.nodeValue);
    try { return String(n.textContent == null ? '' : n.textContent); } catch (_e) { return ''; }
  }
  function _xpathLit(raw) {
    var s = String(raw == null ? '' : raw).trim();
    if ((s[0] === '"' && s[s.length - 1] === '"') || (s[0] === "'" && s[s.length - 1] === "'")) return s.slice(1, -1);
    return s;
  }
  function _xpathAttrVal(node, name) {
    if (!node || node.nodeType !== 1) return null;
    try { var h = node.hasAttribute(name); return h ? node.getAttribute(name) : null; } catch (_e) { return null; }
  }
  // 谓词求值（pos/last = 当前候选集 1-based 位置/大小）。
  function _xpathPred(node, p, pos, last) {
    p = String(p == null ? '' : p).trim();
    if (!p) return true;
    // last() / last()-N → 数值位置比较。
    var mL = p.match(/^last\(\)\s*(?:([+-])\s*(\d+))?$/);
    if (mL) { var target = last; if (mL[1]) target = last + (parseInt(mL[2], 10) * (mL[1] === '-' ? -1 : 1)); return pos === target; }
    // 裸整数 → position == n。
    if (/^\d+$/.test(p)) return pos === parseInt(p, 10);
    // position() op N。
    var mP = p.match(/^position\(\)\s*(==|!=|<=|>=|<|>)\s*(\d+)$/);
    if (mP) return _xpathNumCmp(pos, mP[1], parseInt(mP[2], 10));
    if (p === 'position()') return true;
    // not(...)。
    var mNot = p.match(/^not\(\s*(.*)\)\s*$/);
    if (mNot) return !_xpathPred(node, mNot[1], pos, last);
    // contains(A, B)。
    var mC = p.match(/^contains\(\s*(.*?),\s*(.*)\)$/);
    if (mC) {
      var a = _xpathPredOperand(node, mC[1].trim());
      var b = _xpathLit(mC[2]);
      return String(a).indexOf(String(b)) >= 0;
    }
    // @name [op val]。
    var mA = p.match(/^@([\w:.-]+)\s*(?:(!=|==|=)\s*(.*))?$/);
    if (mA) {
      var av = _xpathAttrVal(node, mA[1]);
      if (!mA[2]) return av != null; // [attr] → 存在性
      // https://www.w3.org/TR/xpath-10/#section-BooleanExpressions：@name 是节点集，属性缺失时节点集为空，
      // `=`/`!=` 的存在量词对空集恒为 false（不存在满足的节点）。旧实现把缺失值归一为 '' 再比较，
      // 致 `@a!='v'` 错误命中无该属性的节点（R2990 修复）。
      if (av == null) return false;
      return _xpathStrCmp(av, mA[2], _xpathLit(mA[3]));
    }
    // text() [op val]。
    var mT = p.match(/^text\(\)\s*(?:(!=|==|=)\s*(.*))?$/);
    if (mT) {
      var tv = node.nodeType === 3 ? String(node.nodeValue == null ? '' : node.nodeValue) : _xpathNodeStr(node);
      if (!mT[1]) return node.nodeType === 3;
      return _xpathStrCmp(tv, mT[1], _xpathLit(mT[2]));
    }
    // . op val（节点 string-value 比较）。
    var mD = p.match(/^\.\s*(!=|==|=)\s*(.*)$/);
    if (mD) return _xpathStrCmp(_xpathNodeStr(node), mD[1], _xpathLit(mD[2]));
    return false; // 未知谓词 → 过滤掉（保守）。
  }
  function _xpathPredOperand(node, expr) {
    expr = String(expr || '').trim();
    if (expr === '.') return _xpathNodeStr(node);
    if (expr === 'text()') return node.nodeType === 3 ? String(node.nodeValue == null ? '' : node.nodeValue) : _xpathNodeStr(node);
    var mA = expr.match(/^@([\w:.-]+)$/);
    if (mA) { var v = _xpathAttrVal(node, mA[1]); return v == null ? '' : v; }
    return _xpathLit(expr);
  }
  function _xpathStrCmp(left, op, right) {
    if (op === '=' || op === '==') return String(left) === String(right);
    if (op === '!=') return String(left) !== String(right);
    return false;
  }
  function _xpathNumCmp(left, op, right) {
    if (op === '==' || op === '=') return left === right;
    if (op === '!=') return left !== right;
    if (op === '<') return left < right;
    if (op === '>') return left > right;
    if (op === '<=') return left <= right;
    if (op === '>=') return left >= right;
    return false;
  }
  // 单步：解析 head + 谓词组。
  function _xpathParseStep(tok, axis) {
    if (!tok) return null;
    if (tok === '.') return { axis: 'self', test: 'node', preds: [] };
    if (tok === '..') return { axis: 'parent', test: 'node', preds: [] };
    if (tok.charCodeAt(0) === 64) { // '@' 开头 → 属性轴
      var an = tok.slice(1);
      // 记录是否经 `//`（descendant）前置到达——属性轴默认仅查 ctx 自身属性，`//@name` 需展开到
      // ctx + 全部后代元素（R2990 修复：旧实现丢弃 'descendant' 轴致 `//@name` 恒返空）。
      return { axis: 'attribute', test: 'attr', arg: an, preds: [], fromDesc: axis === 'descendant' };
    }
    var m = tok.match(/^([^\[]+)([\s\S]*)$/);
    if (!m) return null;
    var head = m[1].trim();
    var rest = m[2];
    var preds = [];
    while (rest.length && rest[0] === '[') {
      var d = 0, q = null, j;
      for (j = 0; j < rest.length; j++) {
        var ch = rest[j];
        if (q) { if (ch === q) q = null; }
        else if (ch === '"' || ch === "'") q = ch;
        else if (ch === '[') d++;
        else if (ch === ']') { d--; if (d === 0) break; }
      }
      if (d !== 0) return null; // 括号不闭合
      preds.push(rest.slice(1, j));
      rest = rest.slice(j + 1);
    }
    if (rest.trim()) return null;
    var test;
    var mh = head.match(/^(text|node|comment)\(\)$/);
    if (mh) test = mh[1];
    else if (head === '*') test = '*';
    else if (/^[A-Za-z_][\w:.-]*$/.test(head)) test = head;
    else return null;
    return { axis: axis, test: test, preds: preds };
  }
  // 路径解析 → {absolute, list:[step]}。sep '/' → child 轴，'//' → descendant 轴。
  function _xpathParsePath(expr) {
    var s = String(expr == null ? '' : expr).trim();
    if (!s) return null;
    var absolute = false, i = 0, len = s.length, nextAxis = 'child';
    if (s.charCodeAt(0) === 47) { absolute = true; i = 1; if (s.charCodeAt(1) === 47) { nextAxis = 'descendant'; i = 2; } }
    var steps = [];
    while (i < len) {
      var startTok = i, depth = 0, q = null;
      while (i < len) {
        var ch = s[i];
        if (q) { if (ch === q) q = null; }
        else if (ch === '"' || ch === "'") q = ch;
        else if (ch === '[') depth++;
        else if (ch === ']') depth--;
        else if (ch === '/' && depth === 0) break;
        i++;
      }
      var tok = s.slice(startTok, i).trim();
      var sep = '';
      if (i < len && s[i] === '/') sep = (s[i + 1] === '/') ? '//' : '/';
      var step = _xpathParseStep(tok, nextAxis);
      if (!step) return null;
      steps.push(step);
      if (sep === '//') { i += 2; nextAxis = 'descendant'; }
      else if (sep === '/') { i += 1; nextAxis = 'child'; }
      else break;
    }
    if (!steps.length) return null;
    return { absolute: absolute, list: steps };
  }
  function _xpathApplyStep(contextNodes, step) {
    var survivors = [];
    var seen = {};
    for (var ci = 0; ci < contextNodes.length; ci++) {
      var ctx = contextNodes[ci];
      var matched = [];
      if (step.axis === 'attribute') {
        // 属性轴：对每个元素 ctx 产出伪 Attr 节点（nodeType 2）。
        // fromDesc（`//@name` 经 `//` 前置）：扩展到 ctx + 全部后代元素（descendant-or-self 语义，R2990）。
        var owners = [ctx];
        if (step.fromDesc) _xpathAllDesc(ctx, owners);
        for (var oi = 0; oi < owners.length; oi++) {
          var owner = owners[oi];
          if (owner && owner.nodeType === 1) {
            var names = [];
            if (step.arg === '*') {
              try { names = (typeof owner.getAttributeNames === 'function') ? owner.getAttributeNames() : []; } catch (_e) { names = []; }
            } else names = [step.arg];
            for (var ai = 0; ai < names.length; ai++) {
              var v = _xpathAttrVal(owner, names[ai]);
              if (v != null) matched.push({ nodeType: 2, name: names[ai], nodeName: names[ai], value: v, nodeValue: v, ownerElement: owner });
            }
          }
        }
      } else {
        var cands = _xpathAxisCandidates(ctx, step.axis);
        for (var k = 0; k < cands.length; k++) if (_xpathTest(cands[k], step.test)) matched.push(cands[k]);
      }
      var size = matched.length;
      for (var pi = 0; pi < step.preds.length; pi++) {
        var narrowed = [];
        for (var n = 0; n < matched.length; n++) if (_xpathPred(matched[n], step.preds[pi], n + 1, size)) narrowed.push(matched[n]);
        matched = narrowed;
        size = matched.length;
      }
      for (var s = 0; s < matched.length; s++) {
        var key = _xpathKey(matched[s]);
        if (!Object.prototype.hasOwnProperty.call(seen, key)) { seen[key] = 1; survivors.push(matched[s]); }
      }
    }
    return survivors;
  }
  // 文档序排序（多上下文 descendant 后保险；compareDocumentPosition 不可用时保持原序）。
  function _xpathSortDocOrder(nodes) {
    try {
      nodes.sort(function (a, b) {
        if (a === b) return 0;
        if (a && b && typeof a.compareDocumentPosition === 'function') {
          var rel = a.compareDocumentPosition(b);
          if (rel & 0x04) return -1; // a 在 b 前
          if (rel & 0x02) return 1;  // a 在 b 后
          return 0;
        }
        return 0;
      });
    } catch (_e) {}
    return nodes;
  }
  function _xpathRun(expr, contextNode) {
    var parsed = _xpathParsePath(expr);
    if (!parsed) throw new TypeError("Failed to execute 'evaluate' on 'Document': The string '" + expr + "' is not a valid XPath expression.");
    var ctx = parsed.absolute ? [globalThis.document.documentElement] : [contextNode];
    for (var i = 0; i < parsed.list.length; i++) ctx = _xpathApplyStep(ctx, parsed.list[i]);
    _xpathSortDocOrder(ctx);
    return ctx;
  }
  function _xpathMakeResult(nodes, type) {
    var snap = nodes.slice();
    var idx = 0;
    var rt = type;
    if (rt === 0) rt = 6; // ANY_TYPE → 按节点集（无序快照语义）报告。
    return {
      resultType: rt,
      get snapshotLength() { return snap.length; },
      snapshotItem: function (i) { i = i | 0; return (i >= 0 && i < snap.length) ? snap[i] : null; },
      get singleNodeValue() { return snap.length ? snap[0] : null; },
      iterateNext: function () { if (idx < snap.length) return snap[idx++]; return null; },
      get numberValue() { var s = snap.length ? _xpathNodeStr(snap[0]) : ''; var n = parseFloat(s); return isNaN(n) ? NaN : n; },
      get stringValue() { return snap.length ? _xpathNodeStr(snap[0]) : ''; },
      get booleanValue() { return snap.length > 0; },
      invalidIteratorState: false
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
    // js-dom M4 R113：handle-based owner（createElement('style') 后 append——CSS-in-JS /
    // WPT prefixed-animation 形态）。初始规则经 `__zw_style_rules_handle`（host 从 mutation
    // 历史解析该 handle 的 style 文本），写回经 `__zw_set_text_handle`（SetTextOnHandle）。
    var handle = !sel && owner ? owner.__zwHandle : null;
    var rulesCache = null;
    function getRules() {
      if (rulesCache) return rulesCache;
      rulesCache = [];
      var wire = '';
      if (sel && typeof __zw_style_rules === 'function') {
        try { wire = String(__zw_style_rules(sel)); } catch (_e0) { wire = ''; }
      } else if (handle && typeof __zw_style_rules_handle === 'function') {
        try { wire = String(__zw_style_rules_handle(handle)); } catch (_e0h) { wire = ''; }
      }
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
      return rulesCache;
    }
    // 从 cache 重建 `<style>` 文本（join cssText）+ 写回 owner 元素（下次 render 重解析 cascade）。
    function flushToOwner() {
      var text = getRules().map(function (r) { return r.cssText; }).join('\n');
      if (sel && typeof __zw_set_text === 'function') {
        try { __zw_set_text(sel, text); } catch (_e) {}
      } else if (handle && typeof __zw_set_text_handle === 'function') {
        try { __zw_set_text_handle(handle, text); } catch (_eh) {}
      }
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
      // IE legacy 别名（CSSOM 早期 IE 扩展，Chrome/Firefox 仍保留兼容）。spec 行为：
      //   addRule(selector, styleBlock, index?) — selector 默认 ''，styleBlock（声明文本，不含大括号）
      //     默认 ''，组合 `selector + '{' + styleBlock + '}'` 调 insertRule(combined, index)。
      //     **返回值恒 -1**（IE 成功 marker，Chrome 同——非失败，非真实 index；遗留兼容固定值）。
      //   removeRule(index) — 等价 deleteRule(index)。
      // CSS-in-JS 罕用，但旧库（早期 jQuery .css、legacy stylesheet 注入）feature-detect + 走此路径，
      // stub 时样式静默丢失 → R3276 落实真实组合 + 委托。
      addRule: function (selector, styleBlock, index) {
        var s = String(selector == null ? '' : selector);
        var b = String(styleBlock == null ? '' : styleBlock);
        // index 缺省 → insertRule 内 clamp 到末尾。
        var combined = s + '{' + b + '}';
        try { this.insertRule(combined, index); } catch (_e) { /* insertRule 失败 best-effort */ }
        return -1;
      },
      removeRule: function (index) {
        try { this.deleteRule(index); } catch (_e) { /* deleteRule 失败 best-effort */ }
      }
    };
    return ss;
  }

  // XPathResult 常量（spec，R2981）——document.evaluate 的 resultType 取值。
  globalThis.XPathResult = globalThis.XPathResult || {
    ANY_TYPE: 0,
    NUMBER_TYPE: 1,
    STRING_TYPE: 2,
    BOOLEAN_TYPE: 3,
    UNORDERED_NODE_ITERATOR_TYPE: 4,
    ORDERED_NODE_ITERATOR_TYPE: 5,
    UNORDERED_NODE_SNAPSHOT_TYPE: 6,
    ORDERED_NODE_SNAPSHOT_TYPE: 7,
    ANY_UNORDERED_NODE_TYPE: 8,
    FIRST_ORDERED_NODE_TYPE: 9
  };

  // R152（js-dom M4）：Document 侧 [Unscopable] 表（与 Element 侧 R134 表同源语义）。
  var DocumentUnscopables = { prepend: true, append: true, replaceChildren: true };

  globalThis.document = {
    // js-dom M3 R100：shim document 标记——generate_dom_api_polyfill（execute_script_with_dom
    // 每次前置的最小虚拟 DOM stub）据此跳过覆写（幂等安装，保 execute 路径上的真 document 桥）。
    __zwShimInstalled: true,
    // https://html.spec.whatwg.org/multipage/dom.html#dom-document-location
    // The final Document object is installed in this module; expose the live
    // Window Location here rather than on the bootstrap placeholder.
    get location() { return globalThis.location; },
    // R34xx：caretPositionFromPoint(x, y)——命中注册文本元素字形（index-from-offset
    // 的 DOM 对照侧，0 基几何；未命中 → null）。
    caretPositionFromPoint: function (x, y) {
      if (typeof _zwCaretFromPoint !== 'function') return null;
      try {
        var hit = _zwCaretFromPoint(+x || 0, +y || 0);
        return hit ? { offsetNode: hit.offsetNode, offset: hit.offset } : null;
      } catch (_e) { return null; }
    },
    querySelector: function(sel) {
      // R158：非法选择器守卫（spec SyntaxError——WPT runInvalidSelectorTest 的
      // document.querySelector 入口）。置于 pending 回落之前（非法即抛，不查）。
      if (globalThis._zwQueryGuard) globalThis._zwQueryGuard(sel, arguments.length);
      // M3 扩批 XV：track 查询触发面（querySelector('track') 静态形态）。
      if (String(sel).toLowerCase() === 'track'
          && typeof globalThis._zwScheduleAllTrackLoads === 'function') {
        try { globalThis._zwScheduleAllTrackLoads(); } catch (_eDqsT) {}
      }
      var hit = __zw_query_match(sel);
      if (hit) return _zwQueryWrapIdentity(hit);
      // js-dom M4 R51c：host 快照未命中 → 回落 pending added 扫描（同步 turn 内 append/insert 的
      // 节点对查询不可见是 testharness mega-case 的系统性破损源：WPT dom/common.js
      // setupRangeTests 每次开头 `querySelector('#test')` 取旧树 removeChild 重建——pending 旧树
      // 查不到 → 跳过 remove → 旧 proxy 泄漏进 pending 表 → O(n²)（Range-mutations dataChange
      // 超时根因）。保守语义：仅 host miss 时回落；`#id` 粯形式（getElementById 同源）。
      var m = /^#[A-Za-z_][\w-]*$/.exec(String(sel || ''));
      if (m) {
        var want = String(sel).slice(1);
        var arr = _zwPendingAddedById.get(want);
        if (arr && arr.length) return arr[arr.length - 1];
      }
      // R145（js-dom M4）：纯 tag 形式的 pending 回落——`document.querySelector('p')` 在
      // 同 turn append 后（host 快照未含）返回真节点而非 null。spec 查询同步反映 DOM 变更；
      // WPT pointer-event-document-move 的 `test_driver.click(document.querySelector('p'))`
      // 在 await 前求值（append 同 turn），null → "no stable selector"。
      var tagM = /^[A-Za-z][\w-]*$/.exec(String(sel || ''));
      if (tagM && typeof _zwPendingAdded !== 'undefined' && _zwPendingAdded.length) {
        var wantTag = String(sel).toUpperCase();
        for (var _r145p = 0; _r145p < _zwPendingAdded.length; _r145p++) {
          var _r145n = _zwPendingAdded[_r145p];
          if (!_r145n || _r145n.nodeType !== 1) continue;
          // R346：in-doc 门——createElement-only（从未 append）的孤儿不在文档中，spec 查询
          // 不可见（WPT Event-dispatch-on-disabled-elements：sync tests 留下 16 个孤儿
          // button/input，孤儿排在 _zwPendingAdded 首部 → 派发脚本 querySelector 命中孤儿
          // → 事件派到无 listener 的元素 → 链卡死）。R54/R120 的 `_zwMutationInDoc` 门同源。
          try {
            if (typeof _zwMutationInDoc === 'function'
                && !_zwMutationInDoc(_r145n.__zwSelector, _r145n.__zwHandle)) continue;
          } catch (_e346g) {}
          try {
            if (String(_r145n.tagName) === wantTag) return _r145n;
          } catch (_e145p) {}
        }
      }
      return null;
    },
    // R34xx：id 含特殊字符（点号等——canvas WPT 的 id="green.png"）时 '#'+id 选择器
    // 解析错误（点号被当类）→ 改用属性选择器（[id="..."] 精确匹配）。
    // js-dom M4 R117：主文档的 ParentNode 变异族（prepend/append/replaceChildren——WPT
    // pre-insertion-validation-hierarchy 经 insert(doc, node) 调用，缺方法直接 TypeError）。
    // 校验：Document 只收 DocumentFragment/DocumentType/Element（Text/Comment/PI → HRE）；
    // Document 节点本身不可插入（node.nodeType 9 → HRE，parent 非 doc）。插入 best-effort 经
    // appendChild（校验通过后）。
    prepend: function () {
      for (var _p117 = 0; _p117 < arguments.length; _p117++) {
        var _pn = arguments[_p117];
        if (_pn && typeof _pn === 'object') {
          var _nt117 = _pn.nodeType | 0;
          if (_nt117 === 3 || _nt117 === 4 || _nt117 === 9) {
            throw new (globalThis.DOMException || Error)(
              'Nodes of type ' + _nt117 + ' cannot be inserted into a Document.', 'HierarchyRequestError');
          }
          if (_nt117 === 11) {
            var _fe117 = 0;
            var _fk117 = _pn.childNodes || [];
            for (var _fq = 0; _fq < _fk117.length; _fq++) if (_fk117[_fq].nodeType === 1) _fe117++;
            var _hasEl117 = false;
            var _dk117 = globalThis.document.childNodes || [];
            for (var _dq = 0; _dq < _dk117.length; _dq++) if (_dk117[_dq].nodeType === 1) { _hasEl117 = true; break; }
            if (_fe117 > 1 || (_fe117 === 1 && _hasEl117)) {
              throw new (globalThis.DOMException || Error)(
                'A Document cannot contain more than one Element.', 'HierarchyRequestError');
            }
          }
          // js-dom M4 R119：doctype 参数 + doc 已有另一 doctype → HierarchyRequestError
          //（spec pre-insert 步骤 6 II；WPT pre-insertion-validation-hierarchy
          //「node is a doctype and parent is a document with another doctype」）。
          if (_nt117 === 10) {
            var _dt117 = globalThis.document.doctype;
            var _hasDt117 = false;
            if (_dt117) _hasDt117 = true;
            if (!_hasDt117) {
              var _dk117b = globalThis.document.childNodes || [];
              for (var _dq2 = 0; _dq2 < _dk117b.length; _dq2++) if (_dk117b[_dq2].nodeType === 10) { _hasDt117 = true; break; }
            }
            if (_hasDt117) {
              throw new (globalThis.DOMException || Error)(
                'A document cannot have more than one DocumentType node.', 'HierarchyRequestError');
            }
          }
        }
      }
      for (var _p117b = arguments.length - 1; _p117b >= 0; _p117b--) {
        var _pn2 = arguments[_p117b];
        if (_pn2 && typeof _pn2 === 'object') { try { globalThis.document.insertBefore(_pn2, globalThis.document.firstChild || null); } catch (_e117) {} }
      }
    },
    append: function () {
      for (var _a117 = 0; _a117 < arguments.length; _a117++) {
        var _an = arguments[_a117];
        if (_an && typeof _an === 'object') {
          var _nt117b = _an.nodeType | 0;
          if (_nt117b === 3 || _nt117b === 4 || _nt117b === 9) {
            throw new (globalThis.DOMException || Error)(
              'Nodes of type ' + _nt117b + ' cannot be inserted into a Document.', 'HierarchyRequestError');
          }
          if (_nt117b === 11) {
            var _fe117b = 0;
            var _fk117b = _an.childNodes || [];
            for (var _fqb = 0; _fqb < _fk117b.length; _fqb++) if (_fk117b[_fqb].nodeType === 1) _fe117b++;
            var _hasEl117b = false;
            var _dk117b = globalThis.document.childNodes || [];
            for (var _dqb = 0; _dqb < _dk117b.length; _dqb++) if (_dk117b[_dqb].nodeType === 1) { _hasEl117b = true; break; }
            if (_fe117b > 1 || (_fe117b === 1 && _hasEl117b)) {
              throw new (globalThis.DOMException || Error)(
                'A Document cannot contain more than one Element.', 'HierarchyRequestError');
            }
          }
        }
      }
      for (var _a117b = 0; _a117b < arguments.length; _a117b++) {
        var _an2 = arguments[_a117b];
        if (_an2 && typeof _an2 === 'object') { try { globalThis.document.appendChild(_an2); } catch (_e117b) {} }
      }
    },
    replaceChildren: function () {
      var _rc117 = globalThis.document.childNodes || [];
      for (var _r117 = 0; _r117 < _rc117.length; _r117++) { try { globalThis.document.removeChild(_rc117[_r117]); } catch (_e117c) {} }
      globalThis.document.append.apply(globalThis.document, arguments);
    },
    // R152（js-dom M4）：Document 的 ParentNode mixin 方法挂 [Unscopable] 表（spec
    // WebIDL §[Unscopable] 与 Element 侧 R134 同源——Document 同样实现 ParentNode）。
    // inline handler 编译为 with(document){with(this){…}}，bare `prepend`/`append`/
    // `replaceChildren` 经本表豁免后解析到 window 全局（WPT remove-unscopable 的
    // prepend/append 两断言：元素层豁免后外层 document 层命中 document.prepend 会
    // 再次吞掉裸名——真实浏览器两层都有表）。
    // https://dom.spec.whatwg.org/#interface-parentnode
    get [Symbol.unscopables]() {
      return DocumentUnscopables;
    },
    // R136（js-dom M4）：`document.getRootNode()`（spec `dom-node-getrootnode`——Document
    // 自身是 root，返回自身；WPT rootNode "document node" 断言 getRootNode() === document）。
    getRootNode: function () { return globalThis.document; },
    getElementById: function(id) {
      var idText = String(id);
      // R125：空串 id → null（浏览器 id 缓存只索引非空 id——静态 `<div id="">` 不入索引，
      // WPT "Calling document.getElementById with an empty string argument" 期望 null）。
      if (idText === '') return null;
      // R125：同批 id 变更覆盖表（querySelector('[id=…]') 读 host 快照，set
      // Attribute/removeAttribute/Attr.value= 改 id 后命中 stale）。
      // ① 快照命中先验覆盖表——命中元素被改为其它 id → null（WPT "shouldn't get the
      //    element by the old id"）；② 快照 miss 再正向查覆盖表——命中新 id 的元素拉回。
      function _r125Overridden(hitProxy) {
        if (!hitProxy || !globalThis._zwIdOverrideGet) return false;
        try {
          var k = _elKey(hitProxy.__zwSelector || null, hitProxy.__zwHandle || null);
          var ov = globalThis._zwIdOverrideGet(k);
          if (ov === undefined) return false;
          return ov !== idText;
        } catch (_e) { return false; }
      }
      var hit = globalThis.document.querySelector('[id="' + idText.replace(/"/g, '\\"') + '"]');
      // R125：快照命中但元素已 remove（pending-removed 表）→ 继续找下一个（spec tree
      // order 的下一候选）——removeChild 的 Remove mutation 不入查询视图（R3029 removed
      // proxy 属性读回落快照的约束），getElementById 侧定点消费 pending-removed。
      if (hit && !_r125Overridden(hit) && !_zwPRSet().has(hit) && !_r125AncestorRemoved(hit)) return hit;
      if (globalThis._zwIdOverridesEntries && idText !== '') {
        var ents = globalThis._zwIdOverridesEntries();
        for (var ei = 0; ei < ents.length; ei++) {
          if (ents[ei][1] === idText) {
            var pr = _proxyCache[ents[ei][0]];
            if (pr) return pr;
          }
        }
      }
      // https://dom.spec.whatwg.org/#dom-nonelementparentnode-getelementbyid
      // The host snapshot may lag appendChild/innerHTML within a running script.
      // Reuse the pending-ID index so synchronous document lookups see inserted
      // nodes before the queued mutation reaches the renderer DOM.
      // R125：JS 侧「祖先已移除」判定——_zwMutationInDoc 的 sel 分支查 host 快照
      // contains（stale：静态标记子树 removeChild 后快照未换代仍含）。沿 proxy 父链
      //（sel 父经 __zw_parent 一跳、handle 父经 _zwNodeParent）上行，任一祖先命中
      // pending-removed → 整棵子树 out-of-document（WPT "must not return nodes not
      // present in document"：outer.removeChild(middle) 后 inner.appendChild(h1) 的
      // h1 不可见）。
      function _r125AncestorRemoved(nd) {
        var cur = nd, guard = 0;
        while (cur && guard++ < 32) {
          if (_zwPRSet().has(cur)) return true;
          var next = null;
          try {
            var ph = cur.__zwHandle;
            if (ph && typeof _zwNodeParent !== 'undefined' && _zwNodeParent[ph]) {
              var link = _zwNodeParent[ph];
              if (link.parentSel) next = _wrapSelector(link.parentSel);
              else if (link.parentHandle) next = _wrapHandle(link.parentHandle);
            } else if (cur.__zwSelector && typeof __zw_parent === 'function') {
              var psel = __zw_parent(cur.__zwSelector);
              if (psel) next = _wrapSelector(psel);
            }
          } catch (_eP) { next = null; }
          cur = next;
        }
        return false;
      }
      var pending = _zwPendingAddedById.get(idText);
      if (pending && pending.length) {
        // R125：in-document 门（spec：getElementById 只返**树中**节点）——pending 条目
        // 挂 detached 容器（createElement('div').appendChild(...) 未入主文档）时不可见。
        // 与 live collection 的 R54 门同源（_zwMutationInDoc 沿 _zwNodeParent 反链上行）。
        // 同 id 多条目按**树序优先**（spec tree order 首个——pending 表是插入序，同父
        // 连续 append 时序=树序，跨父时以快照命中先行、pending 兜底的合成次序近似）。
        for (var pi = 0; pi < pending.length; pi++) {
          var pn = pending[pi];
          if (!pn || _zwPRSet().has(pn) || _r125AncestorRemoved(pn)) continue;
          var pSel = pn && pn.__zwSelector ? pn.__zwSelector : null;
          var pH = pn && pn.__zwHandle ? pn.__zwHandle : null;
          if (pH && _zwMutationInDoc(null, pH)) return pn;
          if (pSel && _zwMutationInDoc(pSel, null)) return pn;
        }
      }
      // A newly attached handle can contain a parsed innerHTML subtree whose
      // descendants have no handle of their own. Scan that small pending tree
      // until the renderer publishes its next DOM snapshot.
      function findPendingId(node) {
        if (!node) return null;
        try { if (node.id === idText) return node; } catch (_e) {}
        var children = null;
        try { children = node.childNodes; } catch (_e2) { children = null; }
        if (!children) return null;
        for (var i = 0; i < children.length; i++) {
          var found = findPendingId(children[i]);
          if (found) return found;
        }
        return null;
      }
      for (var i = _zwPendingAdded.length - 1; i >= 0; i--) {
        var cand = _zwPendingAdded[i];
        // R125：in-document 门同 pending-ID 索引路径（detached 容器子树不可见）。
        var cSel = cand && cand.__zwSelector ? cand.__zwSelector : null;
        var cH = cand && cand.__zwHandle ? cand.__zwHandle : null;
        var inDoc = cH ? _zwMutationInDoc(null, cH) : (cSel ? _zwMutationInDoc(cSel, null) : false);
        if (!inDoc || _r125AncestorRemoved(cand)) continue;
        var found = findPendingId(cand);
        if (found) {
          // 命中节点可能深在 pending 子树内——对其自身再验 in-doc（子树根 in-doc 但
          // 命中在 detached 分支的情形：R54 门在挂载点判定的对称面，此处单节点判定）。
          var fSel = found && found.__zwSelector ? found.__zwSelector : null;
          var fH = found && found.__zwHandle ? found.__zwHandle : null;
          if (!_r125AncestorRemoved(found)
              && (fH ? _zwMutationInDoc(null, fH) : (fSel ? _zwMutationInDoc(fSel, null) : true))) return found;
        }
      }
      return null;
    },
    // js-dom M4 R112：`document.cloneNode(deep)`（WPT Event-dispatch-bubbles "In
    // window.document.cloneNode(true)"——主文档缺此方法直接 TypeError）。返回**可查询的
    // detached Document**（body 子树 = 主文档 body innerHTML 快照，经 __zw_get_inner_html）
    // ——getElementById/getElementsByTagName/事件面（R112 doc 级 + 视图 path 派发）全可用。
    cloneNode: function(deep) {
      var d = _makeDetachedDocument(globalThis.document.title || '');
      if (deep) {
        try {
          var ih = typeof __zw_get_inner_html === 'function' ? __zw_get_inner_html('body') : '';
          if (ih) d.body.innerHTML = ih;
        } catch (_e112c) {}
      }
      return d;
    },
    // R3067：`document.getAnimations()`（Web Animations API）——返文档内全部动画（所有元素，cancelled/idle 排除；
    // finished 含）。_elementAnimations per-element 注册表 flat + filter。headless 瞬间完成 → finished 动画可查询/commitStyles。
    getAnimations: function() {
      var out = [];
      for (var k in _elementAnimations) {
        var arr = _elementAnimations[k];
        if (!arr) continue;
        for (var i = 0; i < arr.length; i++) {
          var a = arr[i];
          if (a && a.playState !== 'idle') out.push(a);
        }
      }
      return out;
    },
    // R2924 elementFromPoint：`document.elementFromPoint(x, y)` → 视口 CSS 像素 (x,y) 命中的最深元素。
    // 经 host `__zw_elementFromPoint(x, y)`（renderer/browser render 后 swap 进 HitTestCache）求命中选择器
    // → _wrapSelector。未注册（engine/reftest/polyfill 无渲染）/ 无命中 → null（spec）。
    elementFromPoint: function(x, y) {
      if (typeof __zw_elementFromPoint !== 'function') return null;
      var sel = __zw_elementFromPoint(String(x), String(y));
      return sel ? _wrapSelector(sel) : null;
    },
    // R2925 elementsFromPoint：`document.elementsFromPoint(x, y)` → 视口 (x,y) 处全部元素（绘制序，
    // 最前在前）。经 host `__zw_elementsFromPoint(x, y)` 返 `|` 分隔选择器 → split + _wrapSelector。
    // 未注册 / 空命中 → 空数组（spec）。
    elementsFromPoint: function(x, y) {
      if (typeof __zw_elementsFromPoint !== 'function') return [];
      var wire = __zw_elementsFromPoint(String(x), String(y));
      if (!wire) return [];
      return wire.split('|').filter(Boolean).map(_wrapSelector);
    },
    querySelectorAll: function(sel) {
      if (globalThis._zwQueryGuard) globalThis._zwQueryGuard(sel, arguments.length);
      var q = String(sel);
      // M3 扩批 XV：track 查询触发面（querySelectorAll('track') 静态形态）。
      if (q.toLowerCase() === 'track'
          && typeof globalThis._zwScheduleAllTrackLoads === 'function') {
        try { globalThis._zwScheduleAllTrackLoads(); } catch (_eDqsaT) {}
      }
      if (q === ':invalid' || q === ':valid') {
        // R57（FV M1）：:invalid/:valid 伪类查询（约束校验联动——host CSS 引擎
        // 未实现——infinite_backtracking 的 querySelectorAll(":invalid")）
        var wantInvalid = q === ':invalid';
        var out = [];
        try {
          // 逗号选择器顶层不支持——分开查询 concat
          var collected = [];
          for (var qi = 0; qi < ['input', 'select', 'textarea', 'button'].length; qi++) {
            var base = __zw_query_all(['input', 'select', 'textarea', 'button'][qi]);
            if (base) collected = collected.concat(base.split('|').filter(Boolean));
          }
          var seen = {};
          for (var ii = 0; ii < collected.length; ii++) {
            var it = _wrapSelector(collected[ii]);
            var sk = it.__zwSelector || ('h' + ii);
            if (seen[sk]) continue;
            seen[sk] = 1;
            try {
              var v = it.validity && it.validity.valid;
              if ((wantInvalid && !v) || (!wantInvalid && v)) out.push(it);
            } catch (_e) {}
          }
        } catch (_e) {}
        return _zwMakeCollection(out, false);
      }
      var all = __zw_query_all(q);
      // R161：tag 形态的 pending 回落（R145 querySelector 单点版的 QSA 镜像——
      // WPT `querySelectorAll(null)` 对 setup 同 turn append 的 `<null>` 元素
      // expect 1；host 快照 miss 时扫 pending added）。仅纯 tag。
      // js-dom M4 R331：query 返回点 identity 反查（R100 `_zwQueryWrapIdentity` 的 QSA 面
      //——命中 handle 建立的节点时返回原 handle proxy 而非新 sel proxy。document QSA 的
      // 单数入口 querySelector（:1430）已有反查，复数入口漏配——Vue v-for `li` 挂载后
      // `document.querySelectorAll('li.item')` 旧返 sel wrapper（`nth-child` 在 sel 键快照
      // 已含 li 时由 host 判定命中）+ pending 归并（R322 链）消费 handle proxy，同一 li 两
      // identity 双计（lis:A,B,A,B，vue_reconciliation 首渲染回归 R322 轮未发现——A/B 列表
      // 未含 vue e2e）。反查命中 → 原复用 = 双源合流；未命中 → 原 sel wrapper 零变化。
      var out161 = all ? all.split('|').filter(Boolean).map(_zwQueryWrapIdentity) : [];
      var tagM161 = /^[A-Za-z][\w-]*$/.exec(q);
      if (tagM161 && typeof _zwPendingAdded !== 'undefined' && _zwPendingAdded.length) {
        var seen161 = {};
        for (var si = 0; si < out161.length; si++) {
          try { seen161[_elKeyOf(out161[si]) || ('s' + si)] = 1; } catch (_e161s) {}
        }
        var wantTag161 = q.toUpperCase();
        for (var pi = 0; pi < _zwPendingAdded.length; pi++) {
          var pn161 = _zwPendingAdded[pi];
          if (!pn161 || pn161.nodeType !== 1) continue;
          try {
            if (String(pn161.tagName) !== wantTag161) continue;
            var k161 = null;
            try { k161 = pn161.__zwHandle ? ('@' + pn161.__zwHandle) : (pn161.id ? ('id:' + pn161.id) : null); } catch (_e161k) {}
            if (k161 && seen161[k161]) continue;
            out161.push(pn161);
          } catch (_e161p) {}
        }
      }
      var r159 = _zwMakeCollection(out161, false);
      try { r159.__zwQSA = true; } catch (_e159q) {} // R159：instanceof NodeList 标记
      return r159;
    },
    getElementsByClassName: function(cls) {
      // R3019：honor `this` for cross-document use（DOMPurify 等库 getElementsByClassName.call(parsedDoc, cls)
      // 须查 parsedDoc 而非页面 document）。this === 页面 document 时走页面 DOM；否则委托 this.querySelectorAll。
      // R3033：返 HTMLCollection（item + namedItem），包 _zwMakeCollection(arr, true)。
      // R50：liveSpec——同步脚本内 append/remove 后集合 lazy 重查（matches 按 class 判定归属）。
      // R185（js-dom M4）：class 参数 ASCII 空白分词（spec `dom-document-getelementsbyclassname`
      // 步骤 2——token 全含匹配；空/全空白 → 空 collection **不抛**——旧 `'.' + cls` 直接构
      // `'.'`/`'. '` 非法选择器触发 SyntaxError，WPT getElementsByClassName-empty-set 3F）。
      var _r185Parts = (typeof _zwSplitClassList === 'function') ? _zwSplitClassList(cls) : [String(cls)];
      if (_r185Parts.length === 0) return _zwMakeCollection([], true);
      var _r185Q = '.' + _r185Parts.join('.');
      if (this && this !== globalThis.document && typeof this.querySelectorAll === 'function') {
        return _zwMakeCollection(this.querySelectorAll(_r185Q), true);
      }
      var clsStr = _r185Parts.join(' ');
      return _zwMakeCollection(globalThis.document.querySelectorAll(_r185Q), true, {
        matches: function (el) {
          try {
            if (!el) return false;
            var cs = String(el.className || '').split(/\s+/).filter(Boolean);
            for (var _r185p = 0; _r185p < _r185Parts.length; _r185p++) {
              if (cs.indexOf(_r185Parts[_r185p]) < 0) return false;
            }
            return true;
          } catch (_e) { return false; }
        },
      });
    },
    getElementsByTagName: function(tag) {
      // R3019：honor `this` for cross-document use（DOMPurify _initDocument 经 getElementsByTagName.call(doc,'body')[0]
      // 取 parsed doc 的 body——旧实现恒查页面 document 致 DOMPurify 清洗空页面 body 返 ""）。
      // R3033：返 HTMLCollection（item + namedItem），包 _zwMakeCollection(arr, true)。
      // R50：liveSpec——同步脚本内 append/remove 后集合 lazy 重查（matches 按 tagName 判定归属）。
      if (this && this !== globalThis.document && typeof this.querySelectorAll === 'function') {
        return _zwMakeCollection(this.querySelectorAll(String(tag)), true);
      }
      // spec `dom-document-getelementsbytagname` 匹配模型（WPT case.js 期望模型实证）：
      // HTML 文档中查询参数 **ascii-lowercase**（'Abc'→'abc'，'ä' 不变——ascii-lower 不动
      // non-ASCII）；元素按 qualified name（prefix:local）**精确**比较。HTML ns 元素
      // localName 已小写（createElement；createElementNS 保留大小写 → 'Abc' ≠ 'abc' 不命中，
      // WPT case.html HTML 分支 get_qualified_name === expected_case 无元素侧 lowercase）；
      // 非 HTML ns 同样精确（'a:abc' ≠ 'abc'，WPT "non-HTML namespace, prefix"）。
      // js-dom M4 R120：统一走 _zwFilterByTagNameNS（与 Element 级 / NS 级同匹配算法）
      // + _zwDocAllElements 枚举源（快照 '*' ∪ 动态 handle 子——querySelectorAll(tag) 快照
      // 对 appendChild 动态子恒 miss，WPT「live collection」length 1≠2）。
      var _r120Tag = String(tag);
      // R330：htmlCtx 查询时捕获（主文档 HTML——document 级调用者的 context 是主文档；
      // cross-document 委托已在函数头分流）。
      var _r330Html = _zwCtxIsHtmlDoc(null, null);
      return _zwMakeCollection(_zwFilterByTagNameNS(_zwDocAllElements(), _r120Tag, undefined, _r330Html), true,
        { matches: _zwLiveMatchesFor(_r120Tag, undefined, _r330Html) });
    },
    // `document.getElementsByTagNameNS(ns, localName)`（spec `dom-document-getelementsbytagnamens`，R12）——
    // 命名空间作用域的标签集合查询。polyfill 无 ns 概念（HTML 单 ns），忽略 ns 按 localName 查
    //（同 getElementsByTagName）。case.html 用例 + 命名空间库高频。返 HTMLCollection（item + namedItem）。
    // js-dom M4 R120：NS 感知匹配（spec concept-getelementsbytagnamens）+ 动态子融合
    //（WPT Document-getElementsByTagNameNS：element.appendChild(createElementNS(...)) 的
    // handle 子不在快照——快照 '*' 查询 ∪ _zwPendingAdded 子树经 _zwDocElementsFor 函数）。
    getElementsByTagNameNS: function(ns, localName) {
      // R3019：honor `this`（cross-document 委托）。
      if (this && this !== globalThis.document && typeof this.getElementsByTagNameNS === 'function') {
        return this.getElementsByTagNameNS(ns, localName);
      }
      var ln = String(localName == null ? '' : localName);
      var els = _zwDocAllElements();
      var out = _zwFilterByTagNameNS(els, ln, ns);
      return _zwMakeCollection(out, true, { matches: _zwLiveMatchesFor(ln, ns) });
    },
    // `document.getElementsByName(name)`（R2980）——按 name 属性查全文档（表单字段 / a[name] 锚点 /
    // meta[name] 高频，如 `document.getElementsByName('csrf-token')`）。此前全缺 → ReferenceError
    // 中断含此调用的脚本。spec 返 live NodeList；headless 近似为静态数组（同 getElementsByTagName）。
    // 委托 querySelectorAll 经 `[name="…"]` 属性选择器——name 值含 `"` / `\` 时转义保证选择器合法。
    // R3033：返 NodeList（item），包 _zwMakeCollection(arr, false)。
    getElementsByName: function(name) {
      var v = String(name).replace(/\\/g, '\\\\').replace(/"/g, '\\"');
      return _zwMakeCollection(globalThis.document.querySelectorAll('[name="' + v + '"]'), false);
    },
    // `document.evaluate(expr, ctx, resolver, type, result)`（R2981）——XPath 1.0 实用子集求值。
    // 见 _xpathParsePath / _xpathRun 子集说明。返 XPathResult（snapshot/iterator/singleNode/scalar）。
    // type=null/0 → ANY_TYPE（按节点集报告）；不支持的构造抛 TypeError（spec INVALID_EXPRESSION_ERR）。
    evaluate: function(expr, contextNode, _resolver, type, _result) {
      var ctx = contextNode || globalThis.document.documentElement;
      var nodes = _xpathRun(expr, ctx);
      return _xpathMakeResult(nodes, (type == null) ? 0 : (type | 0));
    },
    createElement: function(tag) {
      tag = String(tag);
      // spec `dom-document-createelement` validate：非法标签名（空/首字符非 name-start）→
      // 抛 InvalidCharacterError DOMException。R81 spec 纠正：HTML createElement 用 Name
      // production（`_zwIsValidHtmlElementName`——非首字符宽容，`'f}oo'`/`'f<oo'` 合法；
      // WPT Document-createElement valid 列表），区别 createElementNS 的 QName 校验。
      // createElement(undefined)→"undefined" 合法通过。
      if (!_zwIsValidHtmlElementName(tag)) {
        // 用 globalThis.DOMException（native_dom=true 叠加路径下 = 原生 DOMException；纯 polyfill 下 =
        // part01b 的）——保证 e.constructor === self.DOMException（WPT assert_throws_dom "wrong global"
        // 要求，R6 定位）。裸 new DOMException 走词法作用域，叠加路径下 wrong global。
        throw new (globalThis.DOMException)('The tag name provided is not a valid name.', 'InvalidCharacterError');
      }
      if (tag.toLowerCase() === 'canvas') return _zwMakeCanvas();
      var handle = __zw_create_element(tag);
      var el = _wrapHandle(handle);
      // js-dom M3 R90→R94：createElement 命中已注册 custom element → 立即升级（spec
      // `custom-elements-upgrades`：创建即 upgrade = 原型挂接 + **用户 ctor 体执行**）。
      // R94 `_ceRunCtor`（part03）：class ctor 经 super() 返回值注入 this（HTMLElement
      // hook 消费 `_zwCeExisting`），function ctor 经 .call(el)——闭合 R90「ctor 体不可
      // 重放」限制（不是重放，是 this 注入）。getPrototypeOf trap（part05 R90）对
      // registry tag 动态返 ctor.prototype，_ceRunCtor 内 setPrototypeOf 使
      // Object.getPrototypeOf 同源。
      if (globalThis.customElements && typeof globalThis.customElements.get === 'function') {
        var _r90Ctor = globalThis.customElements.get(String(tag).toLowerCase());
        if (typeof _r90Ctor === 'function' && _r90Ctor.prototype) {
          _ceRunCtor(_r90Ctor, el);
        }
      }
      return el;
    },
    // `createElementNS(ns, qualifiedName)`（js-dom M4 / spec `dom-document-createelementns`）：
    // 大小写敏感创建（spec createElementNS **不**小写 localName，`"Abc"` → localName `"Abc"`，区别
    // `createElement` 的 HTML 无条件小写）。带 prefix 的 qualified name（`"p:l"`）解析为 prefix p / local l。
    // 经 `__zw_create_element_ns` → host `doc.create_element_ns`（保留大小写 + prefix + namespace），并把句柄
    // 记入 `_nsHandles`（存原 qualifiedName + ns），使 tagName/prefix/localName/namespaceURI getter 返正确值。
    // SVG 命名空间元素（filter/cursor 等）的专用渲染在本目标范围外，按通用元素创建（不渲染为 SVG 但避免
    // ReferenceError 中断脚本，crashtest 尤其依赖不抛）。
    createElementNS: function(ns, qualifiedName) {
      var _nsStr = (ns == null) ? '' : String(ns);
      var _q = String(qualifiedName);
      // js-dom M4 R80：spec validate-and-extract（dom-document-createelementns 步骤 2-3）——
      // ① qualifiedName 须匹配 QName 语法：空前缀段（':foo'）/空 localName 段（'foo:'）/
      //    非 Name 字符（'^^'/'fo o'/'-foo'/'.foo'）→ InvalidCharacterError
      // ② 命名空间绑定规则：ns 空/null 时带 prefix（'f:oo'）、prefix 或 localName 含第二个
      //    冒号（'f:o:o'）、prefix 'xml' 且 ns ≠ XML ns、prefix 'xmlns'、localName 'xmlns' 且
      //    ns ≠ XMLNS ns → NamespaceError（WPT Document-createElementNS 110F throw 簇）
      // https://dom.spec.whatwg.org/#validate-and-extract
      var _XML_NS = 'http://www.w3.org/XML/1998/namespace';
      var _XMLNS_NS = 'http://www.w3.org/2000/xmlns/';
      // R81 spec 对齐（WPT Document-createElementNS 全期望表）：**整个 qualifiedName** 须是
      // XML Name（首字符 NameStartChar；'}'/'<'/'\uffff' 等非首字符合法——XML 1.0 第五版
      // NameChar 宽集合）；带冒号时 prefix = 首冒号前、localName = 其余（localName 内可再含
      // 冒号——'f:o:o' 有 ns 时合法、无 ns 时 NamespaceError；'a:0' 因 '0' 破坏整名 Name 首字符
      // 规则外——注意 'a:0' 期望 INVALID：localName '0' 非法；'0:a' 期望合法——真浏览器
      // 仅校验整体 Name？实测表为准：整体通过 Name 校验 + localName 段也须 Name，prefix 段
      // 从宽（'0:a' 的 prefix '0' 不校验）。以 WPT 期望表逐条对齐：
      //   非法（InvalidCharacterError）：空名 / ':foo'（空前缀）/ 'foo:'（空 localName）/
      //     首/任意段首字符非 NameStartChar（'}foo'/'1foo'/'.foo'/'-foo'/'fo o'/'a:0'）
      //   非法（NamespaceError）：prefix 存在但 ns 空 / prefix 或 localName 为 'xmlns' 相关
      //     保留绑定违规 / localName 'xmlns' 且 ns 非 XMLNS ns
      //   合法：'f:o:o'（ns 非空）/ 'f::oo' / '0:a'（prefix 从宽）
      var _colon1 = _q.indexOf(':');
      var _pre = _colon1 >= 0 ? _q.slice(0, _colon1) : null;
      var _loc = _colon1 >= 0 ? _q.slice(_colon1 + 1) : _q;
      var _throwDom = function (name, msg) {
        throw new (globalThis.DOMException || Error)(msg, name);
      };
      // 整名 Name 校验（首字符 NameStartChar——'_'/':'/字母/≥0x80；'0:a' 首字符 '0' 按表应
      // 合法，故整名校验仅当**无 prefix** 时对首字符严格；有 prefix 时放宽为首段（prefix 段
      // 从宽，localName 段首字符严格）。
      if (_q === '' || _colon1 === 0 || _colon1 === _q.length - 1) {
        _throwDom('InvalidCharacterError', 'The string contains invalid characters.');
      }
      // R81 对 WPT 期望表逐条对齐（XML 1.0 5th ed NameChar 宽集合——'}'/'<'/'\uffff' 等
      // 非 NameStart 字符在**非首位置**合法；禁止的是空白与 '>'）：
      // ① 无 prefix：首字符 NameStartChar + 全名无空白/'>'（'foo>'/'fo o' Invalid；'f}oo' Valid）
      // ② 有 prefix：localName 段首字符 NameStartChar + 段内无空白/'>'（'a:0' Invalid；
      //    'namespaceURI:a ' 尾空白 Invalid）；prefix 段不校验（'0:a' Valid）
      // ③ ns = XMLNS ns：localName 恰为 'xmlns' 或 prefix 'xmlns' → 合法；其余 → NamespaceError
      //    （spec：XMLNS ns 仅允许 xmlns 元素）
      // R135（js-dom M4）：按 spec regex 逐段校验（WPT name-validation NS 名单）——
      // localName 段 = valid element local name 名单（首字符限制 + NUL/ASCII 空白五字符/
      // '/'/'>' 禁止——**JS /\s/ 含 \x0B/\x85 等非 XML 空白字符误拒**，用显式字符集）；
      // prefix 段 = valid namespace prefix 名单（同集合 + '=' 允许 + 禁 ':'）。旧整名
      // /[\s>]/ 对 'null\0' local 不抛（\0 非 \s）→ NUL 漏校验根因。
      var _r135NsInvalid = /[\u0000\u0009\u000A\u000C\u000D\u0020/>]/;
      if (_r135NsInvalid.test(_q) || _q.indexOf('\u0000') >= 0) {
        _throwDom('InvalidCharacterError', 'The string contains invalid characters.');
      }
      // R135：段校验走 spec regex（_r135IsValidName——首字符 ASCII 字母→后续任意合法集 /
      // ':'/'_'/>=0x80 → 后续 NameChar 集。'\x01' 在 local 中非法[非 NameChar]，':soh\x01'
      // local 首字符 ':' 合法但 '\x01' 违段集 → 抛，WPT name-validation）。
      // **prefix 段从宽**（不校验——WPT name-validation 的 validNamespacePrefixes 含 \x01
      // 等全码点 × valid local 组合都须不抛；regex 对 ≥0x80 首 prefix 的 NameChar 限制
      // 与浏览器实证宽松冲突，实证优先），只校验 local 段（spec regex）。
      if (_pre === null) {
        if (!_r135IsValidName(_q)) {
          _throwDom('InvalidCharacterError', 'The string contains invalid characters.');
        }
      } else {
        if (!_r135IsValidName(_loc)) {
          _throwDom('InvalidCharacterError', 'The string contains invalid characters.');
        }
      }
      if (_nsStr === _XMLNS_NS) {
        var _xmlnsOk = (_loc === 'xmlns' && _pre === null) || (_pre === 'xmlns');
        if (!_xmlnsOk) {
          _throwDom('NamespaceError', 'The xmlns namespace is not allowed for elements.');
        }
      }
      if (_pre !== null) {
        // prefix 存在：ns 须非空（'f:o:o' 无 ns → NamespaceError）；xml/xmlns prefix 保留绑定
        //（'xmlns' prefix 仅在 ns = XMLNS ns 时合法——上面 XMLNS 分支放行，其余 ns → Error）。
        if (_nsStr === '') {
          _throwDom('NamespaceError', 'Namespace prefix provided but no namespace.');
        }
        if (_pre === 'xml' && _nsStr !== _XML_NS) {
          _throwDom('NamespaceError', "Prefix 'xml' must be bound to the XML namespace.");
        }
        if (_pre === 'xmlns' && _nsStr !== _XMLNS_NS) {
          _throwDom('NamespaceError', "Prefix 'xmlns' requires the XMLNS namespace.");
        }
      } else if (_loc === 'xmlns' && _nsStr !== _XMLNS_NS) {
        _throwDom('NamespaceError', "Local name 'xmlns' requires the XMLNS namespace.");
      }
      var handle = (typeof __zw_create_element_ns === 'function')
        ? __zw_create_element_ns(_nsStr, _q)
        : __zw_create_element(_q);
      // js-dom M4 R80：HTML 文档 + HTML 命名空间 → tagName/nodeName 为 qualifiedName 的 ASCII 大写
      //（spec dom-document-createelementns 步骤「If document is an HTML document and namespace is
      // the HTML namespace, set qualifiedName to qualifiedName in ASCII uppercase」——大小写转换只
      // 作用于 qualified name（tagName），prefix/localName 仍从原值解析：`createElementNS(HTMLNS,
      // 'html:span')` → prefix 'html' / localName 'span' / tagName 'HTML:SPAN'）。非 HTML 命名空间
      // 或 detached doc（XML 语义）不转换。
      var _htmlUpper = _nsStr === 'http://www.w3.org/1999/xhtml';
      // R174 修正：WebIDL 空串 ns → namespaceURI null（WPT Document-
      // createElementNS "empty string namespace" 断言；EmptyNs 匹配经序列化
      // 标记协议覆盖）。保持 `|| null` 语义。
      if (handle) _nsHandles[handle] = { qualifiedName: _q, namespace: (_nsStr || null), htmlUpper: _htmlUpper };
      return _wrapHandle(handle);
    },
    // R3023：`document.createAttribute(name)`——建 Attr 节点（nodeType 2，value=''）。供 setAttributeNode /
    // element.attributes.setNamedItem(attr) 用法（属性库 / 序列化库高频）。真 Attr 实例（经 _zwMakeAttr，
    // 含 localName/namespaceURI=null/prefix=null/specified/ownerElement=null 全字段，非 plain {name,value}）。
    // R116：① 空名抛 InvalidCharacterError（spec validate-and-extract——WPT invalid_names ['']）；
    // ② HTML 文档 ASCII 小写、XML 文档保持原样（attr.name/localName——与 createElement 的
    // 文档类型语义一致）。HTML-ness 经本 document 的 contentType（缺省按 HTML——主文档）。
    createAttribute: function(name) {
      var t = String(name);
      if (t === '') {
        throw new (globalThis.DOMException || Error)(
          "Failed to execute 'createAttribute' on 'Document': The name provided is empty.",
          'InvalidCharacterError');
      }
      // R135（js-dom M4）：attribute 名单语义校验（无首字符限制；invalid = NUL/
      // ASCII 空白五字符/'/'/'>'/'='）。
      if (typeof _r135IsValidAttrName === 'function' && !_r135IsValidAttrName(t)) {
        throw new (globalThis.DOMException || Error)(
          "Failed to execute 'createAttribute' on 'Document': The name provided is not a valid name.",
          'InvalidCharacterError');
      }
      var isHtmlDoc = !(typeof this.contentType === 'string' && this.contentType.indexOf('html') < 0);
      var n = isHtmlDoc ? t.replace(/[A-Z]/g, function (c) { return String.fromCharCode(c.charCodeAt(0) + 32); }) : t;
      return _zwMakeAttr(n, '', null);
    },
    // R3024：`document.createAttributeNS(ns, qualifiedName)`——建命名空间 Attr（SVG/MathML/xlink）。
    // 解析 qualifiedName 的 `prefix:local`，设 namespaceURI/prefix/localName（区别 createAttribute 的 null ns）。
    // 值 ''，ownerElement=null（游离）。返 Attr instanceof Attr（经 _zwMakeAttr 的 Object.create(Attr.prototype)）。
    createAttributeNS: function(ns, qualifiedName) {
      var q = String(qualifiedName);
      // R135：NS attribute 名单语义校验（prefix 段禁 ':'、local 段禁 '='——两段都无
      // 首字符限制，比 element QName 宽：'\x01:attr' 合法；'null\0' local 抛）。
      if (typeof _r135IsValidAttrQNameSpec === 'function' && !_r135IsValidAttrQNameSpec(q)) {
        throw new (globalThis.DOMException || Error)(
          "Failed to execute 'createAttributeNS' on 'Document': The name provided is not a valid qualified name.",
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
    // `document.createCDATASection(data)`（R325，spec `dom-document-createcdatasection`）——
    // **主文档恒 HTML 文档**：CDATASection 只在 XML 文档可建（HTML parser 无 CDATA 节点
    // 语义），HTML 文档上调用抛 NotSupportedError。旧缺方法 → TypeError（非 DOMException，
    // WPT Document-createCDATASection 的 `assert_throws_dom("NotSupportedError", ...)` 失败：
    // "threw TypeError ... that is not a DOMException NotSupportedError"）。
    // https://dom.spec.whatwg.org/#dom-document-createcdatasection
    createCDATASection: function(data) {
      throw new (globalThis.DOMException || Error)(
        "Cannot create CDATASection nodes in HTML documents.", 'NotSupportedError');
    },
    // `document.createProcessingInstruction(target, data)`（js-dom M4，spec `dom-document-createprocessinginstruction`）——
    // PI 节点（nodeType 7，target/data/nodeName=target）。spec 校验在调用点同步抛 DOMException（与 native
    // dom_bindings factories.rs 对齐：① target 须合法 Name production ② data 不得含 `?>`，违则
    // InvalidCharacterError）。合法经 host `__zw_create_processing_instruction`（apply 时 doc.create_processing_instruction）。
    createProcessingInstruction: function(target, data) {
      var t = String(target == null ? '' : target);
      var d = String(data == null ? '' : data);
      // spec 步骤 2：target 须合法 Name。
      // R193（js-dom M4）：**XML Name 严格产生式**（spec XML §2.3 NameStartChar/NameChar
      // 全集——PI target 是 XML 构造，比 HTML createElement 的宽容 Name 窄）：首字符
      // 排除 \u00D7(×)/\u00F7(÷) 等非法 StartChar；非首字符排除 ×÷（NameChar 不含）。
      // WPT Document-createProcessingInstruction invalid 列表：·A/×A/A× → InvalidCharacterError，
      // valid 列表 A·A（· 中位合法）。R135 的 HTML 宽容版对 createElement 保持（HTML parser
      // 宽容性）。
      var _r193NameStart = /^[A-Za-z_:\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u02FF\u0370-\u037D\u037F-\u1FFF\u200C-\u200D\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD\u{10000}-\u{EFFFF}]$/u;
      var _r193NameChar = /^[A-Za-z0-9.\-:\u00B7\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u02FF\u0370-\u037D\u037F-\u1FFF\u200C-\u200D\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD\u{10000}-\u{EFFFF}]$/u;
      var _r193ValidName = function (nm) {
        if (!nm) return false;
        var first = nm.charAt(0);
        if (!_r193NameStart.test(first)) return false;
        for (var ci = 1; ci < nm.length; ci++) {
          if (!_r193NameChar.test(nm.charAt(ci))) return false;
        }
        return true;
      };
      if (!_r193ValidName(t)) {
        // globalThis.DOMException（R6 identity：叠加路径下 = 原生 DOMException，避免 wrong global）。
        throw new (globalThis.DOMException)('The target provided is not a valid name.', 'InvalidCharacterError');
      }
      // spec 步骤 3：data 不得含 `?>`。
      if (d.indexOf('?>') !== -1) {
        throw new (globalThis.DOMException)("The data provided contains '?>'.", 'InvalidCharacterError');
      }
      var handle = (typeof __zw_create_processing_instruction === 'function')
        ? __zw_create_processing_instruction(t, d)
        : __zw_create_text(t + ' ' + d);
      if (handle) _piHandles[handle] = { target: t, data: d };
      return _wrapHandle(handle);
    },
    // `document.createEvent(type)`——legacy 合成事件工厂（jQuery<3 / 旧库 / 分析脚本高频）。返空 type 事件，
    // 经 initEvent/initCustomEvent 填充后 dispatchEvent。type 大小写不敏感 + spec 别名（custom↔CustomEvent）；
    // 已知 Event 子类 type→对应构造器（R2779 / R2811 / R2812）；未知回落 Event（lenient，spec 抛
    // NotSupportedError——本沙箱不抛，避免中断脚本）。
    createEvent: function(type) {
      var t = String(type == null ? '' : type).toLowerCase();
      // spec `dom-document-createevent`：type（大小写不敏感）映射到 legacy event interface 构造器。
      // 别名表覆盖 WPT Document-createEvent.https.html aliases（含复数 Events/HTMLEvents/SVGEvents→Event、
      // MouseEvents→MouseEvent、UIEvents→UIEvent、custom→CustomEvent）。未知 type → 抛 NotSupportedError
      //（spec `dom-document-createevent` 步骤，WPT assert_throws_dom NOT_SUPPORTED_ERR；R14 由 lenient 回落
      // Event 改为 spec 合规抛）。已知别名返**空 type** 事件（initEvent/initCustomEvent 设 type）。
      var map = {
        event: globalThis.Event, events: globalThis.Event, htmlevents: globalThis.Event, svgevents: globalThis.Event,
        customevent: globalThis.CustomEvent, custom: globalThis.CustomEvent,
        keyboardevent: globalThis.KeyboardEvent,
        mouseevent: globalThis.MouseEvent, mouseevents: globalThis.MouseEvent,
        uievent: globalThis.UIEvent, uievents: globalThis.UIEvent,
        focusevent: globalThis.FocusEvent,
        inputevent: globalThis.InputEvent,
        compositionevent: globalThis.CompositionEvent,
        hashchangeevent: globalThis.HashChangeEvent,
        storageevent: globalThis.StorageEvent,
        dragevent: globalThis.DragEvent,
        messageevent: globalThis.MessageEvent,
        beforeunloadevent: globalThis.BeforeUnloadEvent,
        devicemotionevent: globalThis.DeviceMotionEvent,
        deviceorientationevent: globalThis.DeviceOrientationEvent,
        textevent: globalThis.TextEvent,
        touchevent: globalThis.TouchEvent,
        // R17：以下 modern event interface 为 non-createable（spec createEvent 仅支持 legacy event interface；
        // WPT someNonCreateableEvents 列表）——**不**入 map，createEvent 对其抛 NotSupportedError：
        // WheelEvent/PointerEvent/PopStateEvent/ProgressEvent/TransitionEvent/AnimationEvent/
        // PageTransitionEvent/ClipboardEvent/ErrorEvent（modern 路径走 `new XxxEvent()` 构造器）。
      };
      var Ctor = map[t];
      if (!Ctor || typeof Ctor !== 'function') {
        // spec：不支持的 event type 抛 NotSupportedError（DOMException code 9）。globalThis.DOMException
        // 保 identity（R6 教训：叠加路径用全局构造器，避免 wrong global）。
        throw new (globalThis.DOMException)('The provided event type is not supported.', 'NotSupportedError');
      }
      // 构造器接收 (type, options)；createEvent 返**空 type** 事件。
      // js-dom M4 R106：spec initialized flag——createEvent 返回的事件未初始化，
      // dispatchEvent 前须 initEvent（否则 InvalidStateError，WPT EventTarget-dispatchEvent）。
      var ev106 = new Ctor('');
      ev106._zwUninitialized = true;
      return ev106;
    },
    // execCommand / queryCommand*（R2826/R2936）——legacy 编辑/剪贴板命令表面（旧 copy 按钮
    // `el.select(); document.execCommand('copy')` / clipboard.js feature-detect `queryCommandSupported('copy')`
    // / contentEditable 编辑器 format 命令）。headless 无真剪贴板/格式化 → permissive stub：
    // R2936 copy/cut/paste 派发 ClipboardEvent 到 document.activeElement（焦点元素或 body，bubbles+cancelable），
    // 使 copy/cut/paste listener + oncopy/oncut/onpaste handler（R2932/R2933 on* 路由）触发；不真写剪贴板
    //（modern 路径走 navigator.clipboard）。format 命令不真应用。返 true（spec copy/cut 返 true=成功）。
    execCommand: function (commandId /*, showUI, value*/) {
      var cmd = String(commandId == null ? '' : commandId).toLowerCase();
      if (cmd === 'copy' || cmd === 'cut' || cmd === 'paste') {
        try {
          var ev = new ClipboardEvent(cmd, { bubbles: true, cancelable: true });
          var target = globalThis.document.activeElement || globalThis.document.body;
          if (target && typeof target.dispatchEvent === 'function') {
            target.dispatchEvent(ev);
          }
        } catch (_e) {}
      } else if (cmd === 'inserthtml' || cmd === 'inserttext') {
        // R57（FV M1）：execCommand InsertHTML/InsertText——向 activeElement
        //（text control）插入文本 + maxlength 截断（UTF-16 单元、代理对安全——
        // input-maxlength-emoji 的 ZWJ 序列 11 单元截 10 → 回退代理对 → 9）。
        var ins = String(arguments[2] == null ? '' : arguments[2]);
        try {
          var tgt = globalThis.document.activeElement;
          if (tgt && (tgt.tagName === 'INPUT' || tgt.tagName === 'TEXTAREA')) {
            var cur = String(tgt.value || '');
            var combined = cur + ins;
            var ml = null;
            try { ml = tgt.maxLength; } catch (_e) {}
            if (ml != null && !isNaN(+ml) && combined.length > +ml) {
              combined = combined.slice(0, +ml);
              var cc = combined.charCodeAt(combined.length - 1);
              if (cc >= 0xd800 && cc <= 0xdbff) combined = combined.slice(0, -1);
            }
            tgt.value = combined;
          }
        } catch (_e) {}
      }
      return true;
    },
    queryCommandSupported: function (_commandId) { return true; },
    queryCommandEnabled: function (_commandId) { return true; },
    queryCommandValue: function (_commandId) { return ''; },
    // `document.designMode`（R3261，HTML §3.2.5）——文档级编辑模式（'on' 使整文档可编辑）。
    // getter 返存储值（默认 'off'）；setter 'on'→'on'，'off'/'inherit'/其它→'off'（spec case-insensitive）。
    // headless 不真启用编辑 → 惰性 setter（仅存储），documented。覆盖 execCommand 富文本编辑器 feature-detect。
    get designMode() { return _zwDesignMode; },
    set designMode(v) {
      var s = String(v == null ? '' : v).toLowerCase();
      _zwDesignMode = (s === 'on') ? 'on' : 'off';
    },
    // `document.createTreeWalker(root, whatToShow, filter)` / `createNodeIterator(...)`——DOM 子树遍历器
    //（库 / sanitizer / a11y tree walker 高频）。whatToShow 掩码 + acceptNode FILTER_ACCEPT/REJECT/SKIP。
    // 经 `_makeNodeWalker`（eager pre-order via childNodes 递归）。两者共用工厂（接口同：nextNode/previousNode）。
    // R82：spec WebIDL optional unsigned long whatToShow——**省略**才缺省 SHOW_ALL
    // (0xFFFFFFFF)；**显式 null/undefined** 走 ToUint32(null)=0（WPT TreeWalker-basic
    // dom-document-createtreewalker 与 R41 断言「显式 null → 0」）。经 arguments.length
    // 区分省略与显式传值。
    createTreeWalker: function (root, whatToShow, filter) {
      // R83：WebIDL §optional-arg——**省略或 undefined** 都取缺省 SHOW_ALL（0xFFFFFFFF）；
      // 仅显式 null 走 ToUint32(null)=0（WPT "with undefined as arguments" 期望 4294967295、"with null" 期望 0）。
      if (whatToShow === undefined) return _makeNodeWalker(root, 0xFFFFFFFF, filter, true);
      return _makeNodeWalker(root, (whatToShow === null ? 0 : whatToShow), filter, true);
    },
    createNodeIterator: function (root, whatToShow, filter) {
      if (whatToShow === undefined) return _makeNodeWalker(root, 0xFFFFFFFF, filter, false);
      return _makeNodeWalker(root, (whatToShow === null ? 0 : whatToShow), filter, false);
    },
    // R184（js-dom M4）：`document.normalize()`（spec Node.normalize——Document 是 Node，
    // 对 documentElement 子树跑合并语义）。委托 html/body 代理的 normalize（part04 get trap
    // 的 R184 实作——handle/registry 子合并 + 空 Text 移除；WPT Node-normalize #1）。
    normalize: function () {
      try {
        var _r184Dn = globalThis.document.documentElement;
        if (_r184Dn && typeof _r184Dn.normalize === 'function') _r184Dn.normalize();
        var _r184Db = globalThis.document.body;
        if (_r184Db && typeof _r184Db.normalize === 'function') _r184Db.normalize();
      } catch (_e184dn) {}
      return undefined;
    },
    // `document.createRange()`——新建 Range（R2804，Selection/Range）。详见 `_makeRange`。
    createRange: function () {
      // R179：同 new Range() 接 Range.prototype（prototype 方法通道）。
      var _r179cr = _makeRange();
      // R183：初始边界 (document, 0)（spec `dom-document-createrange`——new range 的
      // start/end 都在 document 上 offset 0；WPT CAC-2 "Detached Range" 断言
      // commonAncestorContainer === document）。
      try {
        _r179cr.startContainer = globalThis.document;
        _r179cr.endContainer = globalThis.document;
        _r179cr._startOffsetBase = 0;
        _r179cr._endOffsetBase = 0;
      } catch (_e183d) {}
      try { Object.setPrototypeOf(_r179cr, globalThis.Range.prototype); } catch (_e179c) {}
      return _r179cr;
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
    // R192（js-dom M4）：spec `dom-document-adoptnode`——① node 是 Document →
    // NotSupportedError（WPT "Adopting a Document should throw"）；② 从原父摘除
    //（spec concept-node-adopt 前置——WPT "Explicitly adopting a DocumentType" 断言
    // adopt 后 parentNode null）；③ 子树 ownerDocument 重指本文档（handle 注册表 +
    // plain defineProperty，与 R191 appendChild adopt 同款）；④ 返回节点自身。
    adoptNode: function(node) {
      if (!node || typeof node !== 'object') return node;
      if ((node.nodeType | 0) === 9) {
        throw new (globalThis.DOMException || Error)(
          'Cannot adopt a Document node.', 'NotSupportedError');
      }
      try {
        if (node.parentNode && typeof node.parentNode.removeChild === 'function') {
          node.parentNode.removeChild(node);
        }
      } catch (_e192ap) {}
      try {
        (function _r192adopt(n2) {
          if (!n2 || typeof n2 !== 'object') return;
          if (n2.__zwHandle) {
            if (!globalThis.__zwAdoptDocByHandle) globalThis.__zwAdoptDocByHandle = {};
            globalThis.__zwAdoptDocByHandle[String(n2.__zwHandle)] = globalThis.document;
          } else if (n2.__zwSelector) {
            // R192：sel-based 子树（解析产物）——'s:'+sel 键落表（ownerDocument trap 查）。
            if (!globalThis.__zwAdoptDocBySel) globalThis.__zwAdoptDocBySel = {};
            globalThis.__zwAdoptDocBySel[String(n2.__zwSelector)] = globalThis.document;
          } else if (n2.nodeType === 1 || n2.nodeType === 3 || n2.nodeType === 8 || n2.nodeType === 10) {
            try { n2.__zwAdoptDoc191 = globalThis.document; } catch (_e192a1) {}
            try {
              Object.defineProperty(n2, 'ownerDocument', {
                get: function () { return n2.__zwAdoptDoc191 || undefined; },
                configurable: true,
              });
            } catch (_e192a2) {}
          }
          var k2 = n2.childNodes;
          if (k2 && typeof k2.length === 'number') {
            for (var i2 = 0; i2 < k2.length; i2++) _r192adopt(k2[i2]);
          }
        })(node);
      } catch (_e192as) {}
      return node;
    },
    // `document.importNode(node, deep?)`（R2818；R132 spec 语义收口）——spec
    // `dom-document-importnode` = clone node + **adopt 到本文档**（副本的 ownerDocument
    // 是 import 的目标文档——WPT Document-importNode 四变体断言 `newDiv.ownerDocument
    // === document`；旧委托 cloneNode 后 plain-object 副本无 ownerDocument 恒 undefined）。
    // deep 默认 true（spec IDL optional boolean deep = true——「No 'deep' argument」与
    // undefined 变体期望浅克隆**是历史行为**，WHATWG 现行 spec deep 缺省 true；但 WPT
    // 该用例仍按旧断言 firstChild null）→ 按用例：undefined/缺省 = 浅（历史语义），
    // true = 深。Attr 走 _zwMakeAttr 全字段（prefix/ns/localName 复制——"Import an Attr
    // node with namespace/prefix correctly"）。
    importNode: function(node, deep) {
      if (!node || typeof node !== 'object') return node;
      // Attr：全字段复制（spec cloning steps: namespace/prefix/local name/value）
      if ((node.nodeType | 0) === 2) {
        var c = String(node.name || node.nodeName || '');
        var ci = c.indexOf(':');
        var a = _zwMakeAttr(c, node.value != null ? node.value : '', null);
        a.prefix = node.prefix !== undefined ? node.prefix : (ci >= 0 ? c.slice(0, ci) : null);
        a.namespaceURI = node.namespaceURI !== undefined ? node.namespaceURI : null;
        a.localName = node.localName !== undefined ? node.localName : (ci >= 0 ? c.slice(ci + 1) : c);
        return a;
      }
      var isDeep = deep === true;
      var copy;
      if (typeof node.cloneNode === 'function') copy = node.cloneNode(isDeep);
      else return node;
      // R132：浅克隆剥子（Element.prototype.cloneNode 的 deepClone 无 shallow 语义——
      // 恒深复制 childNodes；importNode 浅变体 spec 期望 childNodes 空 + firstChild
      // null。深变体不受影响）。
      if (!isDeep && copy && copy.nodeType === 1 && copy.childNodes && copy.childNodes.length) {
        copy.childNodes = [];
        copy.children = [];
      }
      // adopt：副本子树 ownerDocument 全指本文档（spec concept-node-adopt 递归）。
      // R132：plain-object 副本（Element.prototype.deepClone 产物）补叶子导航面
      //（firstChild/lastChild getter + hasChildNodes——WPT "No/Undefined 'deep' argument"
      // 断言 newDiv.firstChild === null，旧副本缺 getter 读到 undefined）。
      try {
        var adoptAll = function (n) {
          if (!n || typeof n !== 'object') return;
          try {
            Object.defineProperty(n, 'ownerDocument', {
              get: function () { return globalThis.document; },
              configurable: true,
            });
          } catch (_eOd) { n.ownerDocument = globalThis.document; }
          // R186（js-dom M4）：HTML 文档 adopt 的**HTML ns 元素** tagName/nodeName ASCII
          // 大写（spec `dom-element-tagname`——HTML-uppercased local name 随文档 HTML-ness
          // 重算；WPT Element-tagName "tagName should be updated when changing
          // ownerDocument" 三变体：XML 小写 div → import 后 DIV / foo:div → FOO:DIV）。
          // 非 HTML ns（SVG/MathML/自定义）保持原样大小写敏感。
          if (n.nodeType === 1 && n.tagName
              && (n.namespaceURI == null || n.namespaceURI === 'http://www.w3.org/1999/xhtml')) {
            var _r186Up = '';
            var _r186Qn = String(n.tagName);
            for (var _r186i = 0; _r186i < _r186Qn.length; _r186i++) {
              var _r186c = _r186Qn.charAt(_r186i);
              _r186Up += (_r186c >= 'a' && _r186c <= 'z') ? String.fromCharCode(_r186c.charCodeAt(0) - 32) : _r186c;
            }
            try {
              n.tagName = _r186Up;
              n.nodeName = _r186Up;
            } catch (_e186u) {}
          }
          if (n.nodeType === 1 && !(Object.getOwnPropertyDescriptor(n, 'firstChild') || {}).get) {
            try {
              Object.defineProperty(n, 'firstChild', {
                get: function () { return (this.childNodes || [])[0] || null; },
                configurable: true,
              });
              Object.defineProperty(n, 'lastChild', {
                get: function () { var k = this.childNodes || []; return k.length ? k[k.length - 1] : null; },
                configurable: true,
              });
              if (typeof n.hasChildNodes !== 'function') {
                n.hasChildNodes = function () { return (this.childNodes || []).length > 0; };
              }
            } catch (_eLf) {}
          }
          var kids = n.childNodes;
          if (kids && typeof kids.length === 'number') {
            for (var i = 0; i < kids.length; i++) adoptAll(kids[i]);
          }
        };
        adoptAll(copy);
      } catch (_e132a) {}
      return copy;
    },
    // `document.implementation`（DOMImplementation，R2815）——feature-detection（jQuery support 等查 hasFeature）
    // + createDocument/createHTMLDocument（R3013：返 queryable detached Document——body.innerHTML setter +
    // querySelector 族经 __zw_parse_html_query 查解析树，jQuery/DOMPurify feature-detect / 模板引擎可用）。
    implementation: {
      hasFeature: function() { return true; }, // spec：deprecated，恒返 true
      // js-dom M4 R79：createDocument/createHTMLDocument 的 doctype 参数/预置——WPT common.js
      // `foreignDoc = createHTMLDocument("")`（真浏览器恒含 <!DOCTYPE html> → foreignDoctype 非
      // null）+ `xmlDoc = createDocument(null, null, xmlDoctype)`（第三参 doctype 须 append 进
      // xmlDoc 树）。spec：createDocument(namespace, qualifiedName, doctype) 步骤 8 附 doctype。
      createDocument: function(_ns, _qn, doctype) {
        // R130（js-dom M4）：WebIDL 必参缺省 TypeError（namespace/qualifiedName 必选，
        // doctype 第三可选 nullable——WPT "with missing arguments"：`createDocument()`/
        // `createDocument('')` throw，`createDocument(ns, qn)` 2 参合法）。
        if (arguments.length < 2) {
          throw new globalThis.TypeError(
            "Failed to execute 'createDocument' on 'DOMImplementation': 2 arguments required, but only " + arguments.length + " present.");
        }
        // R130：doctype 参数非 null/undefined 且非 DocumentType（nodeType 10）→ TypeError
        //（WebIDL nullable DocumentType 校验——WPT null,null,false 期望 TypeError）。
        if (doctype != null && !(typeof doctype === 'object' && (doctype.nodeType | 0) === 10)) {
          throw new globalThis.TypeError(
            "Failed to execute 'createDocument' on 'DOMImplementation': parameter 3 is not of type 'DocumentType'.");
        }
        var d = _makeDetachedDocument('');
        // js-dom M4 R81：spec DOMImplementation.createDocument —— 返回 XML Document（contentType
        // 'application/xml'，createElement 的 ns 恒 null——除非 XHTML/SVG ns 调用）。ns 参数为
        // 文档的默认 ns，但 spec createElement 的元素 ns 由 **document 类型**派生（XML → null，
        // XHTML ns 调用 → HTML ns）；WPT Document-createElement-namespace 期望 'application/xhtml+xml'
        //（XHTML ns 调用）→ HTML ns，SVG/MathML/XML → null。
        var nsStr = (_ns == null) ? '' : String(_ns);
        d.contentType = nsStr === 'http://www.w3.org/1999/xhtml'
          ? 'application/xhtml+xml'
          : (nsStr === 'http://www.w3.org/2000/svg' ? 'image/svg+xml' : 'application/xml');
        d._docNS = nsStr === 'http://www.w3.org/1999/xhtml' ? 'http://www.w3.org/1999/xhtml' : null;
        // R130（js-dom M4）：XML 文档原型接线 XMLDocument.prototype（WPT
        // DOMImplementation-createDocument `Object.getPrototypeOf(doc) ===
        // XMLDocument.prototype` 全非 throw 用例——spec：createDocument 返 XMLDocument）。
        try {
          if (globalThis.XMLDocument && globalThis.XMLDocument.prototype) {
            Object.setPrototypeOf(d, globalThis.XMLDocument.prototype);
            // R130：Node 常量挂 XMLDocument.prototype（native 路径下 XMLDocument.prototype
            // 是 native ObjectTemplate 产物，无 polyfill Node 常量链——WPT createDocument
            // `doc.nodeType === doc.DOCUMENT_NODE` 断言 native 叠加路径 111F 的主因之一；
            // polyfill 路径 prototype 经 Object.create(Document.prototype) 已含，幂等）。
            // 注：常量值用字面量（native 路径 globalThis.Node 上常量同样缺失——R130 diag7
            // 实证 native Node.DOCUMENT_NODE undefined，不能从 Node 读值）。
            if (d.DOCUMENT_NODE === undefined) {
              var _r130XdcConsts = {
                ELEMENT_NODE: 1, ATTRIBUTE_NODE: 2, TEXT_NODE: 3, CDATA_SECTION_NODE: 4,
                COMMENT_NODE: 8, DOCUMENT_NODE: 9, DOCUMENT_TYPE_NODE: 10,
                DOCUMENT_FRAGMENT_NODE: 11, NOTATION_NODE: 12,
                DOCUMENT_POSITION_DISCONNECTED: 1, DOCUMENT_POSITION_PRECEDING: 2,
                DOCUMENT_POSITION_FOLLOWING: 4, DOCUMENT_POSITION_CONTAINS: 8,
                DOCUMENT_POSITION_CONTAINED_BY: 16, DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC: 32,
              };
              for (var _r130Xn in _r130XdcConsts) {
                if (Object.prototype.hasOwnProperty.call(_r130XdcConsts, _r130Xn)) {
                  try {
                    Object.defineProperty(globalThis.XMLDocument.prototype, _r130Xn,
                      { value: _r130XdcConsts[_r130Xn], enumerable: false });
                  } catch (_e130xc) {}
                }
              }
            }
          }
        } catch (_e130x) {}
        if (doctype && doctype.nodeType === 10) {
          // R81：doctype 归属重指（spec：append 时 adopt——createDocumentType 的 ownerDocument
          // 在主文档创建，append 进 xmlDoc 后属 xmlDoc；WPT Node-properties
          // xmlDoctype.ownerDocument 期望 xmlDoc（4 children））。
          try { delete doctype.ownerDocument; } catch (_eDt) {}
          Object.defineProperty(doctype, 'ownerDocument', { get: function () { return d; }, configurable: true });
          d.appendChild(doctype);
        }
        // R192（js-dom M4）：detached doc 的 adoptNode（spec dom-document-adoptnode——
        // WPT Document-adoptNode "Adopting an Element called 'x<'/'​:good:times:'" 的
        // `doc.adoptNode(y)` 形态：摘除 + 子树 ownerDocument 重指本 doc + 返回节点）。
        // Document 参数抛 NotSupportedError（同主文档）。
        d.adoptNode = function (node) {
          if (!node || typeof node !== 'object') return node;
          if ((node.nodeType | 0) === 9) {
            throw new (globalThis.DOMException || Error)(
              'Cannot adopt a Document node.', 'NotSupportedError');
          }
          try {
            if (node.parentNode && typeof node.parentNode.removeChild === 'function') {
              node.parentNode.removeChild(node);
            }
          } catch (_e192dp) {}
          try {
            (function _r192dAdopt(n2) {
              if (!n2 || typeof n2 !== 'object') return;
              if (n2.__zwHandle) {
                if (!globalThis.__zwAdoptDocByHandle) globalThis.__zwAdoptDocByHandle = {};
                globalThis.__zwAdoptDocByHandle[String(n2.__zwHandle)] = d;
              } else if (n2.__zwSelector) {
                if (!globalThis.__zwAdoptDocBySel) globalThis.__zwAdoptDocBySel = {};
                globalThis.__zwAdoptDocBySel[String(n2.__zwSelector)] = d;
              } else if (n2.nodeType === 1 || n2.nodeType === 3 || n2.nodeType === 8 || n2.nodeType === 10) {
                try { n2.__zwAdoptDoc191 = d; } catch (_e192d1) {}
                try {
                  Object.defineProperty(n2, 'ownerDocument', {
                    get: function () { return n2.__zwAdoptDoc191 || undefined; },
                    configurable: true,
                  });
                } catch (_e192d2) {}
              }
              var k2 = n2.childNodes;
              if (k2 && typeof k2.length === 'number') {
                for (var i2 = 0; i2 < k2.length; i2++) _r192dAdopt(k2[i2]);
              }
            })(node);
          } catch (_e192ds) {}
          return node;
        };
        // R130（js-dom M4）：qualifiedName 非空 → 创建 documentElement（spec
        // `dom-domimplementation-createdocument` 步骤 5「If qualifiedName is not empty:
        // append its element」——WPT createDocument 全非 throw 用例的
        // doc.documentElement/prefix/localName 断言族；旧不创建 → documentElement null）。
        // 元素经本 doc createElementNS（ns/大小写/prefix 语义同源）；append 进 childNodes。
        var _r130Qn = (_qn === null) ? '' : String(_qn);
        if (_r130Qn !== '') {
          try {
            var _r130Root = d.createElementNS(
              nsStr === '' ? null : nsStr, _r130Qn);
            // createElementNS 的 QName 校验对非法名抛 NamespaceError/InvalidCharacterError
            //（spec 步骤同——WPT throw 用例主路径，此处自然传播）。
            d.appendChild(_r130Root);
          } catch (_e130r) { throw _e130r; }
        }
        return d;
      },
      createHTMLDocument: function(title) {
        var d = _makeDetachedDocument(title);
        d.appendChild(d.implementation.createDocumentType('html', '', ''));
        // R81：spec —— HTML Document：contentType 'text/html'，createElement ns = HTML ns。
        // R130：contentType **先于** documentElement append 设置（documentElement getter 的
        // HTML 回落按 contentType 判——未设时 getter 返 null 使 appendChild(null) no-op，
        // doc.childNodes 缺 html 子）。
        d.contentType = 'text/html';
        d._docNS = 'http://www.w3.org/1999/xhtml';
        // R81：spec —— createHTMLDocument 的树 = [doctype, html(含 head+body)]。documentElement
        // 须入 doc.childNodes（WPT Node-properties foreignDoc.childNodes.length 期望 3 =
        // [doctype, html, foreignComment]；旧只有 doctype + 后续 append 的节点）。
        d.appendChild(d.documentElement);
        // R130：HTML 文档原型保持 Document.prototype（XML 默认接线在此改回——spec
        // createHTMLDocument 返 HTML Document 非 XMLDocument）。
        try {
          if (globalThis.Document && globalThis.Document.prototype) {
            Object.setPrototypeOf(d, globalThis.Document.prototype);
          }
        } catch (_e130h) {}
        return d;
      },
      // `createDocumentType(qualifiedName, publicId, systemId)`（spec `dom-domimplementation-createdocumenttype`，
      // R15）——建 DocumentType 节点（nodeType 10）。spec：不校验（publicId/systemId 任意串；qualifiedName 校验
      // 在 createDocument 而非此处）。返 DocumentType：name=nodeName=qualifiedName、publicId、systemId、
      // nodeType 10、ownerDocument。ownerDocument 经 `this` 上下文取所属 document（主 document vs detached doc）。
      createDocumentType: function(qualifiedName, publicId, systemId) {
        // R130（js-dom M4）：qualifiedName 校验（WPT DOMImplementation-createDocumentType
        // 期望表实证宽松——''/'1foo'/'edi:`'/'edi:<'/{/} 全 pass；仅含 ASCII 空白或 '>' throw
        // INVALID_CHARACTER_ERR——与元素 Name 产线不同，doctype 名仅禁此二类）。
        // R135：/\s/ → 显式 ASCII 空白五字符（\x0B 非 XML 空白，WPT name-validation
        // doctype 名单 '\x0B' 合法）+ NUL（名单 invalid）。
        var _r130Dq = String(qualifiedName == null ? '' : qualifiedName);
        if (/[\u0000\u0009\u000A\u000C\u000D\u0020>]/.test(_r130Dq)) {
          throw new (globalThis.DOMException || Error)('The string contains invalid characters.', 'InvalidCharacterError');
        }
        var owner = globalThis.document;
        var dt = {
          nodeType: 10,
          name: String(qualifiedName == null ? '' : qualifiedName),
          nodeName: String(qualifiedName == null ? '' : qualifiedName),
          publicId: String(publicId == null ? '' : publicId),
          systemId: String(systemId == null ? '' : systemId),
          ownerDocument: owner,
          get nodeValue() { return null; },
          set nodeValue(_v) {},
          // R81：DocumentType textContent 恒 null + setter no-op（spec；WPT "created by script"）。
          get textContent() { return null; },
          set textContent(_v) {},
          // js-dom M4 R79：Node.contains / compareDocumentPosition（testNodes 的 doctype 族）。
          childNodes: [],
          hasChildNodes: function () { return false; },
          // R117：cloneNode（WPT pre-insertion-validation-hierarchy 用 doc.childNodes[0].cloneNode()
          // 复制 doctype）。浅拷贝（doctype 无子）。
          cloneNode: function () {
            return globalThis.document.implementation.createDocumentType(dt.name, dt.publicId, dt.systemId);
          },
          // R185（js-dom M4）：isSameNode（spec `dom-node-issamenode`——引用比较；
          // WPT Node-isSameNode "doctypes should be compared on reference"）。
          isSameNode: function (other) { return other === dt; },
          isEqualNode: function (other) {
            if (!other || other.nodeType !== 10) return false;
            return String(other.name) === String(dt.name)
              && String(other.publicId) === String(dt.publicId)
              && String(other.systemId) === String(dt.systemId);
          },
          contains: function (other) { return _zwNodeContains(dt, other); },
          compareDocumentPosition: function (other) { return _zwCompareDocumentPosition(dt, other); },
          // js-dom M4 R81：导航面补齐（WPT Node-properties doctype.previousSibling/nextSibling/
          // parentElement/firstChild/lastChild——旧全 undefined ≠ null）。
          get firstChild() { return null; },
          get lastChild() { return null; },
          get parentElement() { var p = this.parentNode; return p && p.nodeType === 1 ? p : null; },
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
          // R192（js-dom M4）：ChildNode.remove()（spec dom-childnode-remove——doctype 是
          // ChildNode mixin 成员；WPT DocumentType-remove 四变体：remove() 从父的
          // childNodes 摘除 + parentNode 置 null；无父 no-op）。
          remove: function () {
            var p = dt.parentNode;
            if (!p) return;
            try {
              if (typeof p.removeChild === 'function') { p.removeChild(dt); return; }
            } catch (_e192r) {}
            try {
              var k = p.childNodes || [];
              var ix = k.indexOf(dt);
              if (ix >= 0) k.splice(ix, 1);
              dt.parentNode = null;
            } catch (_e192r2) {}
          },
        };
          try { Object.setPrototypeOf(dt, globalThis.Node ? globalThis.Node.prototype : Object.prototype); } catch (_eR117dt2) {}
          // R128：原型接线 DocumentType.prototype（WPT Node-cloneNode check_copy 断言
          // `copy instanceof DocumentType`——旧 plain object 恒 false）。dt 字面量构建后
          // 挂（对象内 IIFE 因 tdz 拿不到 dt 本体，首版失败根因）。
          try {
            if (globalThis.DocumentType && globalThis.DocumentType.prototype) {
              Object.setPrototypeOf(dt, globalThis.DocumentType.prototype);
            }
          } catch (_e128dt2) {}
        return dt;
      },
    },
    documentElement: _wrapSelector('html'),
    // `document.scrollingElement`（HTML §3.1.1）——返回文档视口滚动元素。standards 模式
    //（compatMode==='CSS1Compat'）→ documentElement；quirks 模式（'BackCompat'）→ body；无则 null。
    // headless 恒 CSS1Compat（无 quirks 跟踪）→ documentElement。scroll 库/框架读视口滚动容器的高频 API
    //（locomotive-scroll / smoothscroll / lazy-load / 视口滚动监听）——此前缺 → 返 undefined 致
    // `document.scrollingElement.scrollTop` 抛 TypeError。
    get scrollingElement() { return globalThis.document.documentElement || null; },
    body: _wrapSelector('body'),
    head: _wrapSelector('head'),
    // js-dom M4 R79：`document.doctype`（spec Document.doctype：首个 DocumentType 子或 null）。
    // WPT dom/common.js `doctype = document.doctype` + testNodes 遍历（缺 → undefined →
    // Node-contains/compareDocumentPosition 的 doctype 行 eval 崩）。host 无 doctype 跟踪——
    // testharness 用例恒 `<!doctype html>`：静态 DocumentType（name 'html'，publicId/systemId 空）。
    // ownerDocument/parentNode 经 getter 惰性绑（对象字面量求值期 globalThis.document 尚未赋值）。
    doctype: (function () {
      // R317（js-dom M4）：doctype 元数据从 host 解析树读真实值（WPT DocumentType-literal
      // 的 `<!DOCTYPE html PUBLIC "STAFF" "staffNS.dtd">`——静态硬编码空串使 publicId/
      // systemId 断言失败；spec DocumentType.name/publicId/systemId 反映 DOCTYPE 声明）。
      // **惰性求值**：IIFE 在 shim 加载时执行、此时 __zw_* 回调尚未注册（register_dom_callbacks
      // 在 execute(shim) 之后）——快照式读取恒得默认值。改 getter + 一次性缓存（首次属性
      // 访问时回调已就绪）。
      var _r317Meta = null;
      function _r317MetaOf() {
        if (_r317Meta === null) {
          _r317Meta = { name: 'html', publicId: '', systemId: '' };
          try {
            if (typeof __zw_doc_doctype_json === 'function') {
              var _r317dt = JSON.parse(__zw_doc_doctype_json() || 'null');
              if (_r317dt) {
                _r317Meta.name = String(_r317dt.name || 'html');
                _r317Meta.publicId = String(_r317dt.publicId || '');
                _r317Meta.systemId = String(_r317dt.systemId || '');
              }
            }
          } catch (_e317dtm) {}
        }
        return _r317Meta;
      }
      var dt = {
        nodeType: 10,
        get name() { return _r317MetaOf().name; },
        get nodeName() { return _r317MetaOf().name; },
        get publicId() { return _r317MetaOf().publicId; },
        get systemId() { return _r317MetaOf().systemId; },
        get ownerDocument() { return globalThis.document; },
        get parentNode() { return globalThis.document; },
        get nodeValue() { return null; },
        set nodeValue(_v) {},
        // R81：DocumentType 的 textContent 恒 null + setter no-op（spec；WPT "For DocumentType
        // created by parser/script, setting textContent should do nothing"——旧普通属性被赋值覆盖）。
        get textContent() { return null; },
        set textContent(_v) {},
        childNodes: [],
        hasChildNodes: function () { return false; },
        // R177（js-dom M4）：DocumentType 是叶子节点——mutation 族抛
        // HierarchyRequestError（spec `dom-node-pre-insert`「parent 不是
        // Element/Document/DocumentFragment」；WPT Node-appendChild "Appending to
        // a doctype" 期望 throw——旧缺方法直接 TypeError 非 DOMException）。
        appendChild: function (child) {
          if (child === null || child === undefined || typeof child.nodeType !== 'number') {
            throw new globalThis.TypeError(
              "Failed to execute 'appendChild' on 'Node': parameter 1 is not of type 'Node'.");
          }
          throw new (globalThis.DOMException || Error)(
            'Nodes of type 10 cannot have children.', 'HierarchyRequestError');
        },
        insertBefore: function (child) {
          if (child === null || child === undefined || typeof child.nodeType !== 'number') {
            throw new globalThis.TypeError(
              "Failed to execute 'insertBefore' on 'Node': parameter 1 is not of type 'Node'.");
          }
          throw new (globalThis.DOMException || Error)(
            'Nodes of type 10 cannot have children.', 'HierarchyRequestError');
        },
        contains: function (other) { return _zwNodeContains(dt, other); },
        compareDocumentPosition: function (other) { return _zwCompareDocumentPosition(dt, other); },
        // R81：主文档 doctype 导航面（WPT Node-properties doctype.nextSibling 期望 html——
        // document.childNodes = [doctype, html]；firstChild/lastChild/parentElement 恒 null）。
        get firstChild() { return null; },
        get lastChild() { return null; },
        get parentElement() { return null; },
        get previousSibling() { return null; },
        get nextSibling() { return (globalThis.document && globalThis.document.documentElement) || null; },
        // R152（js-dom M4）：DocumentType 的 namespace 查找恒 null / default ns 恒空
        //（spec `dom-node-lookupnamespaceuri`——doctype 无 prefix 映射、其父 Document
        // 的 default ns 由 documentElement 决定但 doctype 查找返回的是「doctype 自身
        // 上下文」……WPT 期望：lookupNamespaceURI 任何 prefix 均 null、isDefaultNamespace
        // 仅 null/'' true）。非元素分支无 xml/xmlns 预绑定。
        lookupNamespaceURI: function (_prefix) { return null; },
        isDefaultNamespace: function (ns) { return ns == null || ns === ''; },
      };
      // R317（js-dom M4）：原型接线 DocumentType.prototype（spec DOM 速查表 DocumentType :
      // Node 接口——WPT Document-doctype "Doctype should be a DocumentType" 断言
      // `document.doctype instanceof DocumentType`。主文档 doctype 是本 IIFE 字面量，
      // R128 的接线只落在了 implementation.createDocumentType 产物上（同文件下方），
      // 本字面量漏配 → instanceof 恒 false）。
      try {
        if (globalThis.DocumentType && globalThis.DocumentType.prototype) {
          Object.setPrototypeOf(dt, globalThis.DocumentType.prototype);
        }
      } catch (_e317dt) {}
      return dt;
    })(),
    // node-level 身份与连入态（Document 节点恒 connected + 恒有 documentElement 子）。`document.nodeType`
    // =9 / nodeName='#document'（Node 接口常查 `node.nodeType === 9` / `=== Node.DOCUMENT_NODE`）。
    nodeType: 9,
    nodeName: '#document',
    isConnected: true,
    hasChildNodes: function () { return true; },
    // js-dom M4 R81：Document 的 textContent/nodeValue 恒 null（spec dom-node-textcontent：
    // Document/DocumentType 的 textContent 为 null——无 Text 子拼接语义）。旧缺 → undefined。
    // setter no-op（spec：Document 的 textContent 设置不产生效果——WPT "setting textContent
    // should do nothing"；getter-only accessor 使赋值静默失败/忽略）。
    get nodeValue() { return null; },
    set nodeValue(_v) { /* no-op */ },
    get textContent() { return null; },
    set textContent(_v) { /* spec：Document textContent setter 不生效 */ },
    // R81：Document 导航面（WPT Node-properties document.parentNode/parentElement/
    // nextSibling/previousSibling/ownerDocument 恒 null——spec：Document 是树根）。
    get parentNode() { return null; },
    get parentElement() { return null; },
    get nextSibling() { return null; },
    get previousSibling() { return null; },
    get ownerDocument() { return null; },
    // R152（js-dom M4）：document 的 namespace 查找——spec「locate a namespace」的
    // Document 分支：**default（无 prefix）查找返 document 自身 namespace**（HTML
    // document 恒 HTML ns——WPT "Document should have xhtml namespace"：即使
    // documentElement 声明 xmlns="bazURI"，document.lookupNamespaceURI(null) 仍是
    // xhtml；区别于元素作用域的 default 声明继承）；**有 prefix 查找**经
    // documentElement 的 xmlns:p 声明（"Document has bar namespace" 期望 barURI）
    // + xml/xmlns 预绑定。
    lookupNamespaceURI: function (prefix) {
      var p = (prefix == null) ? null : String(prefix);
      if (p === 'xml') return _ZW_XML_NS_R152;
      if (p === 'xmlns') return _ZW_XMLNS_NS_R152;
      if (p == null || p === '') return 'http://www.w3.org/1999/xhtml';
      var de = this.documentElement;
      if (!de || typeof de.lookupNamespaceURI !== 'function') return null;
      return de.lookupNamespaceURI(p);
    },
    isDefaultNamespace: function (ns) {
      var mine = this.lookupNamespaceURI(null);
      return (ns == null || ns === '') ? (mine == null || mine === '') : mine === String(ns);
    },
    // js-dom M4 R79：Node.contains / compareDocumentPosition 在 document 上（WPT testNodes 含
    // "document"——`paras[0].compareDocumentPosition(document)` 等）。spec：document.contains(x)
    // = x 在 document 子树（一切 connected 节点）；document.compareDocumentPosition 是 LCA 判定
    // 的链端（html.parentNode === document 由 _parentNodeFor 对 html 的 null 返回……不对——
    // document 不在 __zw_parent 快照链上，html 的 parentNode 返 null）。
    // **关键**：document 必须进 parentNode 链才能作 root——`_zwDocParentOverride`（下方
    // defineProperty）把 html.parentNode 指向 document。childNodes：doctype + documentElement
    //（树序）；document 不可再有父。
    contains: function (other) { return _zwNodeContains(globalThis.document, other); },
    compareDocumentPosition: function (other) { return _zwCompareDocumentPosition(globalThis.document, other); },
    // R185（js-dom M4）：isSameNode（spec `dom-node-issamenode` 引用比较；WPT
    // Node-isSameNode "documents should be compared on reference"——document1/2 互为
    // implementation.createDocument 产物，引用不同即 false）。
    isSameNode: function (other) { return other === globalThis.document; },
    get childNodes() {
      // R79 注记：曾「不含 doctype」与 WPT oracle previousNode 遍历世界对齐（html.previousSibling
      // 快照恒 null）。R81 spec 纠正：真浏览器 document.childNodes = [doctype, html]（WPT
      // Node-properties document.childNodes.length 期望 2、childNodes[0] 为 DocumentType）。
      // html.previousSibling 仍走 __zw_sibling_nodes 快照（R80 后 R79 的遍历一致性问题已由
      // JS 侧 _zwCompareDocumentPosition 链式判定取代，不依赖该子序）。
      // R87：_docDtorRemoved（removeChild(doctype) 本地标记）时剔除 doctype（恢复段
      // insertBefore 还原）。
      // R317（js-dom M4）：前导注释/PI 入 childNodes（spec 解析树——doctype 与文档元素
      // 之前的 comment/PI 是 document 子节点；WPT Document-doctype 断言
      // `document.childNodes[1] === document.doctype` 依赖 `<!-- comment -->` 占位）。
      // host `__zw_doc_comments` 读解析树的根级 Comment；JS 视图合成 [comments…, dt, html]
      //（Doctype/Element 仍由本侧合成——R81 形态保持，探测缓存防每读一往返）。
      var _r317cm = this._r317CommentCache;
      if (_r317cm === undefined && typeof __zw_doc_comments === 'function') {
        try {
          _r317cm = JSON.parse(__zw_doc_comments() || '[]').map(function (c) {
            return { nodeType: 8, nodeName: '#comment', data: String(c.v || ''), nodeValue: String(c.v || ''),
              get ownerDocument() { return globalThis.document; },
              get parentNode() { return globalThis.document; },
              childNodes: [], hasChildNodes: function () { return false; },
              get textContent() { return String(c.v || ''); },
              get firstChild() { return null; }, get lastChild() { return null; },
              cloneNode: function () { return globalThis.document.childNodes && null; },
            };
          });
        } catch (_e317c) { _r317cm = []; }
        this._r317CommentCache = _r317cm;
      }
      if (!_r317cm || !_r317cm.length) _r317cm = [];
      if (this._docDtorRemoved) return _r317cm.concat([_wrapSelector('html')]);
      return _r317cm.concat([this.doctype, _wrapSelector('html')]);
    },
    // R87 修复回归：firstChild/lastChild getter 曾被 R87 注释块误删（oracle
    // nextNode(document) 助手无法下行 → NodeIterator.html document root 变体 8F）。
    get firstChild() { return this._docDtorRemoved ? _wrapSelector('html') : this.doctype; },
    get lastChild() { return _wrapSelector('html'); },
    // js-dom M4 R87：主文档的 removeChild/insertBefore（WPT NodeIterator-removal 的
    // doctype 子测试经 oldParent= document remove/恢复——旧缺方法 TypeError 崩用例）。
    // host DOM 无 doctype 移除能力——JS 侧本地标记（_zwDocDtorRemoved）：移除后
    // childNodes 视图剔除 doctype、恢复后还原。html 的移除仍不支持（返自身，罕见路径）。
    removeChild: function (c) {
      if (c === this.doctype) {
        if (globalThis._zwNotifyIteratorsRemove) {
          try { globalThis._zwNotifyIteratorsRemove(c); } catch (_e87d) {}
        }
        this._docDtorRemoved = true;
        // R192（js-dom M4）：spec remove 语义——被移除子的 parentNode 置 null（WPT
        // Document-adoptNode "Explicitly adopting a DocumentType" 断言 adopt 后
        // doctype.parentNode === null——adopt 的摘除经此路径）。
        try {
          var _r192dk = this.childNodes || [];
          var _r192di = _r192dk.indexOf(c);
          if (_r192di >= 0) _r192dk.splice(_r192di, 1);
        } catch (_e192ds) {}
        try {
          c.parentNode = null;
          // getter-only accessor 时赋值静默 no-op——defineProperty 兜底（R191 同款教训）。
          if (c.parentNode !== null && c.parentNode !== undefined) {
            Object.defineProperty(c, 'parentNode', { value: null, writable: true, configurable: true });
          }
        } catch (_e192dp) {}
        return c;
      }
      if (c && c.__zwSelector && typeof __zw_remove === 'function') {
        if (globalThis._zwNotifyIteratorsRemove) {
          try { globalThis._zwNotifyIteratorsRemove(c); } catch (_e87e) {}
        }
        _zwRemoveIframeWindowClientForNode(c);
        try { __zw_remove(c.__zwSelector); } catch (_e2) {}
        if (typeof _zwMarkRemoved === 'function') _zwMarkRemoved(c.__zwSelector);
      }
      return c;
    },
    // R152（js-dom M4）：`document.appendChild(node)`（spec `dom-node-appendchild` =
    // pre-insert + child ops）。旧 document 无 appendChild（R117 的 append/prepend/
    // replaceChildren 内部调 document.appendChild 是 no-op 吞异常路径）。校验沿用 R117
    // Document 收点规则（Text/Comment→可、Document 节点→HRE；单 Element 约束对
    // Element 收点）——WPT Node-parentElement "(comment)" 用 document.appendChild(
    // createComment) 挂注释读 parentElement null。
    appendChild: function (c) {
      if (c && typeof c === 'object') {
        var _nt152 = c.nodeType | 0;
        if (_nt152 === 9) {
          throw new (globalThis.DOMException || Error)(
            'A Document node cannot be inserted into a Document.', 'HierarchyRequestError');
        }
        if (_nt152 === 3 || _nt152 === 4) {
          throw new (globalThis.DOMException || Error)(
            'Nodes of type ' + _nt152 + ' cannot be inserted into a Document.', 'HierarchyRequestError');
        }
        // Comment（8）/DocumentType（10）/ProcessingInstruction（7）→ 本记账插入
        //（host 快照无 sel-based comment API，走 _zwNodeParent 反链 + doc 子视图近似）。
        if (_nt152 === 8 || _nt152 === 7 || _nt152 === 10) {
          try {
            var _dc152 = this.childNodes || (this.childNodes = []);
            if (_dc152.indexOf(c) < 0) _dc152.push(c);
            if (typeof c.__zwHandle === 'string' && typeof _zwNodeParent !== 'undefined' && _zwNodeParent) {
              _zwNodeParent[c.__zwHandle] = { parentHandle: null, parentSel: null, nextSibling: null };
            }
          } catch (_e152a) {}
          return c;
        }
        // Element：委托 append 的收点校验 + childNodes 记账（best-effort）。
        if (_nt152 === 1) {
          var _hasEl152 = false;
          var _dk152 = this.childNodes || [];
          for (var _dq152 = 0; _dq152 < _dk152.length; _dq152++) if (_dk152[_dq152].nodeType === 1) { _hasEl152 = true; break; }
          if (_hasEl152) {
            throw new (globalThis.DOMException || Error)(
              'A Document cannot contain more than one Element.', 'HierarchyRequestError');
          }
          try {
            if (!(this.childNodes)) this.childNodes = [];
            this.childNodes.push(c);
          } catch (_e152b) {}
        }
      }
      return c;
    },
    insertBefore: function (c, ref) {
      if (c === this.doctype) { this._docDtorRemoved = false; return c; }
      return c;
    },
    compatMode: 'CSS1Compat',
    characterSet: 'UTF-8',
    charset: 'UTF-8',
    // R81：inputEncoding（WPT Node-properties document.inputEncoding 期望 "UTF-8"；spec
    // DOM Document.inputEncoding = characterSet 别名，readonly）。
    get inputEncoding() { return 'UTF-8'; },
    contentType: 'text/html',
    readyState: 'complete',
    // fullscreen（R2817 stub → R2938 spec-alike 状态追踪 + 事件）。headless 无真 OS 全屏，但 fullscreenElement
    // 反映 requestFullscreen/exitFullscreen 设置的元素（_makeProxy(_fsSel,_fsHandle) 或 null）；fullscreenEnabled
    // 经 host `__zw_fullscreen_enabled`（'0'=禁用），无注册→true；exitFullscreen 清状态 + 派 fullscreenchange。
    // https://fullscreen.spec.whatwg.org/#dom-document-fullscreenelement
    get fullscreenElement() {
      if (!_fsKey) return null;
      return _makeProxy(_fsSel, _fsHandle);
    },
    get fullscreenEnabled() {
      if (typeof __zw_fullscreen_enabled === 'function') {
        try { return __zw_fullscreen_enabled() === '1'; } catch (_e) {}
      }
      return true;
    },
    exitFullscreen: function () {
      return new Promise(function (resolve) {
        if (!_fsKey) { resolve(undefined); return; } // 非全屏 → resolve，不派事件（spec）
        _fsKey = null; _fsSel = null; _fsHandle = null;
        _fireDocEvent('fullscreenchange');
        resolve(undefined);
      });
    },
    // pointerLock（R2939，镜像 R2938 Fullscreen）。headless 无真 OS 指针锁，但 pointerLockElement 反映
    // requestPointerLock/exitPointerLock 设置的元素（_makeProxy(_plSel,_plHandle) 或 null）；exitPointerLock
    // 清状态 + 派 pointerlockchange。**注**：exitPointerLock 返 void（undefined，spec 与 exitFullscreen
    // 返 Promise 不同——Pointer Lock spec 的 exitPointerLock 无返回值）。
    // https://w3c.github.io/pointerlock/#dom-document-pointerlockelement
    get pointerLockElement() {
      if (!_plKey) return null;
      return _makeProxy(_plSel, _plHandle);
    },
    exitPointerLock: function () {
      if (!_plKey) return; // 非锁定 → no-op，不派事件（spec）
      _plKey = null; _plSel = null; _plHandle = null;
      _fireDocEvent('pointerlockchange');
    },
    // document.title——getter 返首 <title> 文本（空白折叠，spec 一致）；首访惰性读 querySelector('title')
    // 并缓存；setter 更新缓存。R3035：setter 经 `__zw_set_text('title', v)` 写回 host `<title>` 元素（SetText
    // mutation），使 render 反映新 title + fresh snapshot 读回保持（闭合 R2815 限制①）。**已知限制**：
    // 不创建 `<head><title>`（spec 无 title 时应建——本沙箱无渲染 title 需求，仅已有 `<title>` 时写回，无则 no-op）。
    get title() {
      if (_doc_title !== null) return _doc_title;
      var t = null;
      try { t = globalThis.document.querySelector('title'); } catch (e) { t = null; }
      _doc_title = t && t.textContent ? String(t.textContent).replace(/\s+/g, ' ').trim() : '';
      return _doc_title;
    },
    set title(v) {
      _doc_title = v == null ? '' : String(v);
      // R3035：写回 host <title>（已有则 SetText，无则 find_by_selector 不命中 → no-op）。
      if (typeof __zw_set_text === 'function') {
        try { __zw_set_text('title', _doc_title); } catch (_e) {}
      }
    },
    // document.URL / documentURI = 页面 URL（= location.href）；referrer = ''（无 referrer 追踪，
    // net-layer defer；standalone 渲染/reftest 无来源页，spec 空串可接受）。
    get URL() { return globalThis.location ? globalThis.location.href : ''; },
    get documentURI() { return globalThis.location ? globalThis.location.href : ''; },
    get referrer() { return ''; },
    // document.activeElement——当前焦点元素（focus()/blur() 操作 _activeElKey）；无焦点回落 body（spec）。
    // R148：解析节点焦点（_zwMElFocused——R114 focus() 设置）优先于 proxy 态（所有权互斥，
    // _zwMEl focus 已清 _activeElKey，双态并存时解析节点为准是防御性回落）。
    get activeElement() {
      if (globalThis._zwMElFocused) return globalThis._zwMElFocused;
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
    // document.currentScript（R3258，HTML §4.11.3.1）——classic 脚本执行期间指向当前 <script> 元素；
    // 非脚本执行期 / module 脚本执行期返 null（spec：module currentScript 恒 null）。宿主在每个 classic
    // 脚本执行前经 __zw_set_current_script(idx) 设索引（idx = 该脚本在全部 <script> 元素中的文档序，
    // 含非 JS 类型，与 getElementsByTagName('script') 对齐），执行后 __zw_clear_current_script() 清。
    // 主用例：分析 SDK / 脚本加载器读 currentScript.src 定位自身来源（GA / requirejs / 广告 SDK）。
    // **已知限制**：idx 对齐依赖执行期 DOM 与解析期 HTML 一致——若先前脚本动态插 <script>（document.write /
    // appendChild）使 getElementsByTagName('script') 序偏移，则后续脚本 currentScript 可能指向错误元素
    //（real browser 按解析器记录的「脚本元素」身份而非序号，无此问题；headless 序号近似，documented）。
    get currentScript() {
      if (_zwCurrentScriptIdx < 0) return null;
      var scripts = globalThis.document.getElementsByTagName('script');
      var el = scripts[_zwCurrentScriptIdx];
      return el || null;
    },
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
      // R40：document 注册打 tgt='doc' 标（document/window/html 三合一 _elKey('html') key 内槽位区分，
      // 派发期 document 虚站只触发本槽位注册，currentTarget=document 本体）。不再经 _makeProxy('html')
      //（那是 html 元素槽位，无标记）。
      var key = _elKey('html', null);
      var t = String(type);
      if (!_listenerStore[key]) _listenerStore[key] = {};
      if (!_listenerStore[key][t]) _listenerStore[key][t] = [];
      // R143（js-dom M4）：spec「add an event listener」步骤 4——重复 listener（同 type/callback/
      // capture/槽位）静默丢弃（WPT handler-count document 变体）。
      var _r143Cap = _optCapture(opts);
      var _r143List = _listenerStore[key][t];
      for (var _r143d = 0; _r143d < _r143List.length; _r143d++) {
        if (_r143List[_r143d].fn === fn && _r143List[_r143d].capture === _r143Cap
            && _r143List[_r143d].tgt === 'doc') return;
      }
      // R105：document target 的 touch/wheel 族默认 passive（spec default-passive-value）。
      _listenerStore[key][t].push({ fn: fn, capture: _r143Cap, once: _optOnce(opts), tgt: 'doc',
        passive: _listenerPassiveDefault(t, opts, true) });
      if (t === 'pageshow') _maybeFirePageShow(); // R2931：首次 pageshow listener → _defer 派发一次
    },
    removeEventListener: function(type, fn, opts) {
      var key = _elKey('html', null);
      var t = String(type);
      if (!_listenerStore[key] || !_listenerStore[key][t]) return;
      var cap = _optCapture(opts);
      _listenerStore[key][t] = _listenerStore[key][t].filter(function(l) {
        return !(l.fn === fn && l.capture === cap && l.tgt === 'doc');
      });
    },
    // R3082 `document.dispatchEvent`——document 为 EventTarget（spec 有 dispatchEvent）。
    // R40 改经 `_dispatchWithBubble('html', …)`：document 为 target（AT_TARGET 只触发 tgt='doc' 構位），
    // bubble 上行 window 虚站（tgt='win'），currentTarget 身份正确（WPT Event-dispatch-multiple-
    // stopPropagation 第 3 断言：document.dispatchEvent → [document, window]）。
    // 返 `!defaultPrevented`（spec）。
    dispatchEvent: function (event) {
      // R106：spec 入口守卫（TypeError / InvalidStateError）——非 Event 抛 TypeError、
      // 未初始化抛 InvalidStateError（旧版静默 return true）。
      globalThis._zwDispatchGuard(event);
      return _dispatchWithBubble(_elKey('html', null), 'html', null, event, 'doc');
    },
    attachEvent: function(type, fn) {
      _attachEventForKey(_elKey('html', null), type, fn);
    },
    detachEvent: function(type, fn) {
      _detachEventForKey(_elKey('html', null), type, fn);
    }
  };
  // R179（js-dom M4）：document.implementation 接 DOMImplementation.prototype
  //（spec 接口方法在 prototype；WPT node-creation-realm 的 `inner.DOMImplementation
  // .prototype.createDocumentType.call(document.implementation, ...)`——part03 建
  // 构造器 + 转发，此处把主 document 的 implementation 对象接入原型链）。
  try {
    if (globalThis.DOMImplementation && globalThis.DOMImplementation.prototype
        && globalThis.document.implementation) {
      Object.setPrototypeOf(globalThis.document.implementation, globalThis.DOMImplementation.prototype);
    }
  } catch (_e179w) {}
  globalThis.window = globalThis;
  globalThis.addEventListener = _globalAddEventListener;
  globalThis.removeEventListener = _globalRemoveEventListener;
  // R2932 `window.dispatchEvent`——window 为 EventTarget（spec 有 dispatchEvent）。R40 改经
  // `_dispatchWithBubble(…, 'win')`：window 为 target（AT_TARGET 只触发 tgt='win' 槽位注册，含
  // window.addEventListener + on* handler 注册），path = [window]，返 `!defaultPrevented`（spec）。
  globalThis.dispatchEvent = function(event) {
    // R106：spec 入口守卫（同 document.dispatchEvent）。
    globalThis._zwDispatchGuard(event);
    // R139（js-dom M4）：window 'load' 派发前物化全部 named iframe 的
    // contentWindow 全局注册（HTML「window named access」——`<iframe name="x">` 使
    // 全局 `x` 解析到其 contentWindow）。lazy 注册（R139 part04 首读 contentWindow
    // 时注册）对「load listener 内直接读全局名」形态来不及（listener 先于任何
    // contentWindow 读触发）——WPT EventListener-handleEvent-cross-realm 的
    // `eventListenerGlobalObject.Object` 引用是 5F 直接根因。load 前一次性物化，
    // 后续读走已注册值（幂等，重复派发 no-op）。
    if (event && event.type === 'load' && typeof globalThis.__zwRegisterNamedIframes === 'function') {
      try { globalThis.__zwRegisterNamedIframes(); } catch (_e139r) {}
    }
    return _dispatchWithBubble(_elKey('html', null), 'html', null, event, 'win');
  };
  // R2983 `window.postMessage(message, targetOrigin [, transfer])`——canonical 跨窗口消息 API。
  // 此前缺（MessagePort/MessageChannel/BroadcastChannel 既有，但 window.postMessage 本身零定义）→
  // `window.postMessage({x:1}, '*')` + `addEventListener('message')` 同窗口异步消息模式（routing /
  // polyfill / iframe-bridge mock / 测试）抛 TypeError。headless 单窗口：经 structuredClone 深拷贝 payload +
  // queueMicrotask **异步**派发 MessageEvent 到自身（spec 异步语义），触发 window 'message' listener + onmessage。
  // targetOrigin 安全校验（spec：不匹配 → 同步 throw SecurityError）：'*' / '/'（同源简写）/ 当前 origin 放行；
  // 其余 origin 同步抛。缺省（undefined/null）按 '*' 放行（lenient，兼容 `postMessage(msg)` 简写）。
  globalThis.postMessage = function (message, targetOrigin, _transfer) {
    var self = globalThis;
    var origin = (self.location && self.location.origin) || '';
    var t = (targetOrigin == null) ? '*' : String(targetOrigin);
    if (t !== '*' && t !== '/' && t !== origin) {
      throw new DOMException(
        "Failed to execute 'postMessage' on 'Window': The target origin provided ('" + t + "') does not match the recipient window's origin ('" + origin + "').",
        'SecurityError'
      );
    }
    var payload = (typeof structuredClone === 'function') ? structuredClone(message) : message;
    queueMicrotask(function () {
      try {
        self.dispatchEvent(new MessageEvent('message', { data: payload, origin: origin, source: self }));
      } catch (_e) {}
    });
  };
  // R376（js-dom M4/DC-3）：**window 级 attachEvent/detachEvent 移除**——spec 已从
  // Window 删除（IE 专有遗留；WPT dom/historical "Window member must be removed:
  // attachEvent/detachEvent" 期望 window 上不存在）。元素 proxy 的 attachEvent
  // （part04 get trap）保留——legacy 页面在元素上的 IE 兼容调用面不受影响。
  // https://dom.spec.whatwg.org/#interface-window（无此成员）
  // R2932 window IDL on-event handler 属性（onload/onerror/onmessage/onpopstate/onhashchange/onpageshow/...）。
  // window===globalThis 为 V8 内置全局对象（非 Proxy，无法拦截 on* 赋值）→ 经 Object.defineProperty 为每个
  // 标准 window 事件类型定义 getter/setter：setter 把 fn 经 _globalAddEventListener 注册为 listener（先移除旧），
  // getter 返存储的 fn。等价 addEventListener（spec IDL handler 语义）。pageshow setter 经 _globalAddEventListener
  // 触发首次注册 _defer 派发（同 R2931）。**已知限制**：onerror/onload 等的「真事件」需 host 集成（错误拦截/
  // load 派发——headless 无 host load 钩子），此处仅注册 listener + 属性可读写 + 合成 dispatchEvent / 既有派发
  // 路径（pageshow/popstate/hashchange）可触。element 级 on* handler（onclick 等）未含（element Proxy 另处理）。
  //（本地并行实现的 onload-only accessor 已被本通用实现取代——rebase 069c9035 时采用 R2932 侧。）
  var _winOnHandlers = {};
  function _defineWinOnHandler(type) {
    Object.defineProperty(globalThis, 'on' + type, {
      configurable: true,
      get: function () { return _winOnHandlers[type] || null; },
      set: function (fn) {
        var prev = _winOnHandlers[type];
        if (typeof prev === 'function') _globalRemoveEventListener(type, prev);
        if (typeof fn === 'function') {
          _winOnHandlers[type] = fn;
          _globalAddEventListener(type, fn);
        } else {
          _winOnHandlers[type] = null;
        }
      },
    });
  }
  // WindowEventHandlers + GlobalEventHandlers（window 级常用）：页面生命周期 / 路由 / 消息 / 输入 / 可见性。
  // R143（js-dom M4）：补 GlobalEventHandlers 的鼠标/键盘/输入/拖拽/剪贴板/触Pointer 全族
  //（spec HTML GlobalEventHandlers——`window.onclick = fn` 是合法 IDL handler 属性，旧缺
  // 定义时赋值落 plain 属性、派发不触发；WPT handler-count onclick 计数族）。
  [
    'afterprint', 'beforeprint', 'beforeunload', 'hashchange', 'languagechange', 'message', 'messageerror',
    'offline', 'online', 'pagehide', 'pageshow', 'popstate', 'rejectionhandled', 'storage', 'unhandledrejection',
    'unload', 'load', 'error', 'resize', 'scroll', 'focus', 'blur',
    'click', 'dblclick', 'auxclick', 'contextmenu', 'mousedown', 'mouseup', 'mousemove', 'mouseover', 'mouseout',
    'mouseenter', 'mouseleave', 'pointerdown', 'pointerup', 'pointermove', 'pointerover', 'pointerout',
    'pointerenter', 'pointerleave', 'pointercancel', 'gotpointercapture', 'lostpointercapture',
    'keydown', 'keyup', 'input', 'beforeinput', 'change', 'submit', 'reset', 'invalid', 'select',
    'wheel', 'drag', 'dragstart', 'dragend', 'dragenter', 'dragleave', 'dragover', 'drop',
    'copy', 'cut', 'paste', 'abort', 'canplay', 'canplaythrough', 'durationchange', 'emptied', 'ended',
    'loadeddata', 'loadedmetadata', 'loadstart', 'pause', 'play', 'playing', 'progress', 'ratechange',
    'seeked', 'seeking', 'stalled', 'suspend', 'timeupdate', 'volumechange', 'waiting', 'toggle',
    'animationstart', 'animationend', 'animationiteration', 'animationcancel',
    'transitionstart', 'transitionend', 'transitionrun', 'transitioncancel',
    'afterscriptexecute', 'beforescriptexecute', 'securitypolicyviolation', 'slotchange',
  ].forEach(_defineWinOnHandler);
  Object.defineProperty(globalThis.document, 'defaultView', {
    get: function() { return globalThis.window; }
  });
  // R2938/R2939 document IDL on-event handler（spec Document handler 属性：onfullscreenchange/onfullscreenerror /
  // onpointerlockchange/onpointerlockerror）。document 为普通对象（非 Proxy，无法经 on* get/set trap）→
  // defineProperty getter/setter：setter 注册/移除 listener（document.addEventListener 转发 html key，与
  // _fireDocEvent 派发点一致 → 可触）；getter 返存储 fn。element 侧 on* 经 R2933 通用 on* 路由已支持，无需此处定义。
  var _docOnHandlers = {};
  function _defineDocOnHandler(type) {
    Object.defineProperty(globalThis.document, 'on' + type, {
      configurable: true,
      get: function () { return _docOnHandlers[type] || null; },
      set: function (fn) {
        var prev = _docOnHandlers[type];
        if (typeof prev === 'function') globalThis.document.removeEventListener(type, prev);
        if (typeof fn === 'function') {
          _docOnHandlers[type] = fn;
          globalThis.document.addEventListener(type, fn);
        } else {
          _docOnHandlers[type] = null;
        }
      },
    });
  }
  // R329（js-dom M4）：document IDL on* 补 GlobalEventHandlers 全族（spec DOM §interface
  // Document : Node + GlobalEventHandlers——`document.onclick = fn` 合法 IDL handler，旧只列
  // 4 个 Document 专有事件 + DOMContentLoaded，click 赋值落 plain 属性、冒泡派发不触发；
  // WPT handler-count ?document 变体的 onclick 计数族）。列表与 window 级 R143 同源（spec
  // GlobalEventHandlers + DocumentAndElementEventHandlers），document 派发虚站（tgt='doc'）
  // 消费 addEventListener 注册——setter 经 document.addEventListener 同链路可触。
  [
    'click', 'dblclick', 'auxclick', 'contextmenu', 'mousedown', 'mouseup', 'mousemove', 'mouseover', 'mouseout',
    'mouseenter', 'mouseleave', 'pointerdown', 'pointerup', 'pointermove', 'pointerover', 'pointerout',
    'pointerenter', 'pointerleave', 'pointercancel', 'gotpointercapture', 'lostpointercapture',
    'keydown', 'keyup', 'input', 'beforeinput', 'change', 'submit', 'reset', 'invalid', 'select',
    'wheel', 'drag', 'dragstart', 'dragend', 'dragenter', 'dragleave', 'dragover', 'drop',
    'copy', 'cut', 'paste', 'abort', 'canplay', 'canplaythrough', 'durationchange', 'emptied', 'ended',
    'loadeddata', 'loadedmetadata', 'loadstart', 'pause', 'play', 'playing', 'progress', 'ratechange',
    'seeked', 'seeking', 'stalled', 'suspend', 'timeupdate', 'volumechange', 'waiting', 'toggle',
    'animationstart', 'animationend', 'animationiteration', 'animationcancel',
    'transitionstart', 'transitionend', 'transitionrun', 'transitioncancel',
    'securitypolicyviolation', 'slotchange', 'scroll',
  ].forEach(_defineDocOnHandler);
  ['fullscreenchange', 'fullscreenerror', 'pointerlockchange', 'pointerlockerror', 'DOMContentLoaded'].forEach(_defineDocOnHandler);

  // R2947 CSS Font Loading API：`document.fonts` FontFaceSet——@font-face 字体加载事件的 JS 入口。
  // 常见用法：`document.fonts.ready.then(重排/重测)`（字体加载库 / icon font / FOUT 处理）、
  // `document.fonts.addEventListener('loadingdone'/'loadingerror', cb)`、`document.fonts.status`。
  // 宿主在 load 完成时（finish_page_load）经 `__zw_font_settle(hadLoaded, hadError)` 触发：
  // 有成功加载 → 派 'loadingdone'；有失败 → 派 'loadingerror'；并 resolve `.ready` Promise（settle 语义，
  // 不论成败 ready 都 resolve——spec：ready 在字体集「不再 loading」时 resolve）。无 @font-face 页面 ready
  // 仍 resolve（font set 从不进入 loading）。minimal EventTarget（listener store + IDL handler + dispatchEvent）。
  // https://drafts.csswg.org/css-font-loading/#FontFaceSet-interface
  var _fontsListeners = {};
  // R34xx：宏任务 fallback settle（宿主反射未在同步期 resolve 时兜底）。
  // js-dom R99：注册条件化——顶层无条件 setTimeout(0) 在每次 shim 注入（含 renderer
  // `reset_document_state` 重注入）时经 part01 setTimeout polyfill 在 `__zw_pending`
  // 注册瞬时 `_t_` 键，与 host TimerBridge resolve 线程竞态（CI 负载下 resolve 晚于
  // reset 断言 → `renderer_js_worker_document_reset_...` flake，8/16 起多轮 CI 守护
  // 归因记录）。改为**惰性**：`.ready` 被消费时（then 首次调用）才注册一次 fallback
  // ——无 font 需求的页面（绝大多数）零注册；有消费的页面注册语义不变（settle 兜底
  // 仍在一个宏任务内到达）。幂等守卫防多路径重复注册。
  var _fontsFallbackArmed = false;
  function _armFontsFallback() {
    if (_fontsFallbackArmed) return;
    _fontsFallbackArmed = true;
    setTimeout(function () { if (globalThis.document && globalThis.document.fonts && globalThis.document.fonts.__zwSettle) globalThis.document.fonts.__zwSettle(); }, 0);
  }
  var _fontsReadyResolve = null;
  var _fontFaceSetFaces = []; // FontFace 对象列表（add/delete 管理；values/forEach/size/迭代反映）
  var _fontFaceSet = {
    status: 'loaded', // 'loading' | 'loaded'（headless 简化：初始即 loaded，settle 时不改）
    onloading: null, onloadingdone: null, onloadingerror: null,
    // R99：ready 惰性化——thenable 包装（then 首调时 arm fallback 定时器 + 委托底层
    // Promise）。无消费则零注册（闭合 reset 断言竞态 flake，见 _armFontsFallback 注释）。
    ready: (function () {
      var _p = new Promise(function (resolve) { _fontsReadyResolve = resolve; });
      return {
        then: function (onF, onR) { _armFontsFallback(); return _p.then(onF, onR); },
        'catch': function (onR) { _armFontsFallback(); return _p.catch(onR); },
        'finally': function (cb) { _armFontsFallback(); return _p.finally(cb); },
      };
    })(),
    // R34xx：无宿主字体反射（wpt-runner/testharness 环境无 @font-face 加载事件）时
    // ready 仍 resolve（spec：无加载任务时 ready settle）——await document.fonts.ready
    // 的 WPT 用例（2d.text.draw.align.*）不再挂起。宿主反射（R2950）先于宏任务完成时
    // resolve 已发生 → 本 fallback 无操作。
    __zwSettle: function () {
      if (_fontsReadyResolve) { var r = _fontsReadyResolve; _fontsReadyResolve = null; r(); }
    },
    addEventListener: function (type, fn) {
      if (typeof fn !== 'function') return;
      (_fontsListeners[type] = _fontsListeners[type] || []).push(fn);
    },
    removeEventListener: function (type, fn) {
      var l = _fontsListeners[type];
      if (l) _fontsListeners[type] = l.filter(function (f) { return f !== fn; });
    },
    dispatchEvent: function (ev) {
      var idl = this['on' + ev.type];
      if (typeof idl === 'function') { try { idl.call(this, ev); } catch (_e) {} }
      var l = _fontsListeners[ev.type];
      if (l) for (var i = 0; i < l.length; i++) { try { l[i].call(this, ev); } catch (_e) {} }
      return true;
    },
    // R2949：FontFace 对象集合管理（add/delete/values/forEach/size/迭代反映 _fontFaceSetFaces）。
    // check/load 仍 minimal（不真按 spec 解析字体描述符）。
    check: function () { return true; },
    load: function () { return this.ready; },
    values: function () { return _fontFaceSetFaces[Symbol.iterator](); },
    entries: function () {
      var arr = _fontFaceSetFaces.map(function (f) { return [f, f]; });
      return arr[Symbol.iterator]();
    },
    keys: function () { return _fontFaceSetFaces[Symbol.iterator](); },
    forEach: function (cb) {
      for (var i = 0; i < _fontFaceSetFaces.length; i++) {
        try { cb(_fontFaceSetFaces[i], i, this); } catch (_e) {}
      }
    },
    add: function (face) {
      if (face && _fontFaceSetFaces.indexOf(face) < 0) _fontFaceSetFaces.push(face);
      return this;
    },
    clear: function () { _fontFaceSetFaces = []; },
    delete: function (face) {
      var i = _fontFaceSetFaces.indexOf(face);
      if (i >= 0) { _fontFaceSetFaces.splice(i, 1); return true; }
      return false;
    },
  };
  Object.defineProperty(_fontFaceSet, 'size', {
    get: function () { return _fontFaceSetFaces.length; },
  });
  if (Symbol && Symbol.iterator) {
    _fontFaceSet[Symbol.iterator] = function () { return _fontFaceSetFaces[Symbol.iterator](); };
  }
  Object.defineProperty(globalThis.document, 'fonts', { configurable: true, value: _fontFaceSet });
  // R2947 宿主字体 settle 入口：hadLoaded/hadError 为 bool（本轮 drain 的 font_events 是否含 loaded/error）。
  // 派 loadingdone/loadingerror + resolve ready。best-effort（失败静默，不影响后续）。
  globalThis.__zw_font_settle = function (hadLoaded, hadError) {
    try {
      var fs = globalThis.document && globalThis.document.fonts;
      if (!fs) return;
      fs.status = 'loaded';
      if (hadLoaded) fs.dispatchEvent(new Event('loadingdone'));
      if (hadError) fs.dispatchEvent(new Event('loadingerror'));
      if (typeof _fontsReadyResolve === 'function') {
        var resolve = _fontsReadyResolve;
        _fontsReadyResolve = null; // 仅 resolve 一次（spec：ready 是单 Promise，settle 一次）
        try { resolve(fs); } catch (_e) {}
      }
    } catch (_e) {}
  };
  // R3258 document.currentScript 宿主入口：classic 脚本执行前 __zw_set_current_script(idx) 设索引，
  // 执行后 __zw_clear_current_script() 清。idx = 该脚本在「全部 <script> 元素」（含非 JS 类型）中的文档序，
  // 与 document.getElementsByTagName('script') 对齐（见 document.currentScript getter 注释）。
  // 调用方（renderer/browser/webview page-script 执行路径）仅在 classic 脚本（非 module）执行前后调用——
  // module 脚本 currentScript 恒 null（spec），故不设。
  globalThis.__zw_set_current_script = function (idx) {
    _zwCurrentScriptIdx = (typeof idx === 'number' && idx >= 0) ? (idx | 0) : -1;
  };
  globalThis.__zw_clear_current_script = function () { _zwCurrentScriptIdx = -1; };
  // `new FontFace(family, source, descriptors)`：family 字符串、source = URL 字串（binary source 非标准 headless 不支持）、
  // descriptors = {style, weight, stretch, unicodeRange, variant, featureSettings}（默认 normal/400）。.status =
  // 'unloaded'|'loading'|'loaded'|'error'。.load() 返 Promise<FontFace>——经 host `__zw_load_font(family, src, id,
  // weightNum, isItalic)` 异步加载（worker 投递 → runtime fetch_get 字节 + load_font/register/set_resolver +
  // async_resolver.resolve），成功 status='loaded' resolve(this)，失败 status='error' reject。.loaded getter 同 .load()
  //（spec：loaded 属性返 load Promise）。weightNum/isItalic 供 host register_family_alias 按 weight/style 构键
  //（R2417/R2493）。host 桥不可用时（engine/reftest/polyfill 无注入）fallback：status='loaded' resolve（不谎称失败）。
  var _fontFaceLoadId = 0;
  function _parseFontWeight(w) {
    if (w == null) return 400;
    var s = String(w).trim();
    if (s === 'normal') return 400;
    if (s === 'bold') return 700;
    var n = parseInt(s, 10);
    return isNaN(n) ? 400 : n;
  }
  function FontFace(family, source, descriptors) {
    this.family = String(family != null ? family : '');
    this._src = typeof source === 'string' ? source : '';
    var d = descriptors || {};
    this.style = d.style != null ? String(d.style) : 'normal';
    this.weight = d.weight != null ? String(d.weight) : 'normal';
    this.stretch = d.stretch != null ? String(d.stretch) : 'normal';
    this.unicodeRange = d.unicodeRange != null ? String(d.unicodeRange) : 'U+0-10FFFF';
    this.variant = d.variant != null ? String(d.variant) : 'normal';
    this.featureSettings = d.featureSettings != null ? String(d.featureSettings) : 'normal';
    this.status = 'unloaded';
    this._loadPromise = null;
  }
  FontFace.prototype.load = function () {
    var self = this;
    if (this._loadPromise) return this._loadPromise;
    this.status = 'loading';
    var weightNum = _parseFontWeight(this.weight);
    var isItalic = this.style === 'italic' || this.style === 'oblique';
    this._loadPromise = new Promise(function (resolve, reject) {
      if (typeof __zw_load_font !== 'function') {
        // host 桥未注入（engine/reftest/polyfill）→ fallback resolve（不谎称失败，测试/polyfill 可用）。
        self.status = 'loaded';
        resolve(self);
        return;
      }
      var id = '__ff_' + (++_fontFaceLoadId);
      globalThis.__zw_pending[id] = function (raw) {
        if (typeof raw === 'string' && raw.indexOf('ok') === 0) {
          self.status = 'loaded';
          resolve(self);
        } else {
          self.status = 'error';
          reject(new Error('Failed to load FontFace "' + self.family + '" from ' + self._src));
        }
      };
      try {
        // R34xx：同步返回契约（headless __zw_load_font 直返 'ok'/'err'——webview 同步加载）。
        var sync = __zw_load_font(self.family, self._src, id, weightNum, isItalic);
        if (typeof sync === 'string' && sync.indexOf('ok') === 0 && globalThis.__zw_pending[id]) {
          globalThis.__zw_pending[id]('ok');
        } else if (typeof sync === 'string' && sync.indexOf('err') === 0 && globalThis.__zw_pending[id]) {
          globalThis.__zw_pending[id]('err');
        }
      } catch (e) {
        delete globalThis.__zw_pending[id];
        self.status = 'error';
        reject(e);
      }
    });
    return this._loadPromise;
  };
  Object.defineProperty(FontFace.prototype, 'loaded', {
    get: function () { return this.load(); },
  });
  globalThis.FontFace = globalThis.FontFace || FontFace;
  // R2950 宿主→JS：把已加载的 @font-face 字体反映为 FontFace 对象加入 document.fonts（补全 FontFaceSet
  // 语义——set 应含文档 @font-face 字体，R2949 前仅程序化 add 的 FontFace 在内）。finish_page_load 经
  // `__zw_add_fontface(family, status)` 对每个 font_event 调用：构造 FontFace(family, '', {}) + 设 status
  //（'loaded'/'error'）+ add 进 set（按 family 去重）。descriptors 默认 normal/400（weight/style 反射 follow-up）。
  globalThis.__zw_add_fontface = function (family, status) {
    try {
      var fs = globalThis.document && globalThis.document.fonts;
      if (!fs || typeof FontFace !== 'function') return;
      var fam = String(family != null ? family : '');
      if (!fam) return;
      // 去重：已存在同 family 的 FontFace 则不重复加（避免 settle 重投累积）。
      var exists = false;
      for (var i = 0; i < _fontFaceSetFaces.length; i++) {
        if (_fontFaceSetFaces[i] && _fontFaceSetFaces[i].family === fam) { exists = true; break; }
      }
      if (exists) return;
      var face = new FontFace(fam, '', {});
      face.status = (status === 'error') ? 'error' : 'loaded';
      fs.add(face);
    } catch (_e) {}
  };

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

  // R203（js-dom M4）：**边界点序比较**（spec `concept-range-bp` 的 position-of——
  // (containerA, offsetA) 是否**严格在** (containerB, offsetB) 之后）。三段判定：
  // ① 同容器：offsetA > offsetB；② A 容器是 B 容器的祖先：比较 B 侧在 A offset 处
  // 的 child 位置（B 容器自身或其祖先链在 A 的 childNodes 中的索引 ≥ offsetA 即 after；
  // spec「boundary-point (parent, index) 比较」——child (A, offsetA) 及之后的位置在
  // B 子树之前）；③ 其余：A、B 容器在**共同父**下的 child 索引序（含 A===B 容器的
  // 兄弟形态）。跨文档/无共同根（异 doc 节点、detached 树）恒 after——spec 的
  // position-of 在异树间不可比，setStart/setEnd 的「或 in different document」分支
  // 语义 = 触发对侧重设（WPT Range-set "in different document" 正断言族）。
  // 实现：child 索引定位经 parentNode/childNodes 链（shim 各节点形态均有）；
  // 共同父 = 沿 A 容器祖先链收集命中 B 容器祖先链的首个共同节点。
  function _zwRangeBpAfter(cA, oA, cB, oB) {
    if (!cA || !cB) return false;
    if (cA === cB) return (oA | 0) > (oB | 0);
    // A 是 B 的祖先：A 侧边界点 (cA, oA) 指向 cA 的第 oA 个子——索引 < oA 的子树
    // 在边界点**之前**，>= oA 的在**之后**。B 的「cA 直接子」索引 childB：B 在
    // 边界点之后 iff childB >= oA；故 A 点在 B 之后 iff childB < oA。
    if (_zwIsAncestorOrSelf(cA, cB)) {
      var childB = _zwBpChildOf(cB, cA);
      return childB != null && (childB | 0) < (oA | 0);
    }
    if (_zwIsAncestorOrSelf(cB, cA)) {
      // A 是 B 的后代：B 侧边界点 (cB, oB) 指向 cB 的第 oB 个子。A 的「cB 直接子」
      // 索引 >= oB → A 在 B 边界点之后（该子及其后子树）；< oB → 之前。
      var childA = _zwBpChildOf(cA, cB);
      return childA != null && (childA | 0) >= (oB | 0);
    }
    // R203 修正：**深度感知双 climb**（首版逐父同步 walk 在深浅不一形态误序——
    // 单测 noncross 实证：point(t1,0) vs (p2,1)，t1 深 2 层、p2 深 1 层，走一步后
    // xA=p0 vs xB=div 触发 isAnc 短路误判 after）。先把深侧 climb 到与浅侧同深，
    // 再同步上行至共同父，双方**直接子**索引定序（offset 不跨共同父层）。
    var dA = _zwTreeDepth(cA), dB = _zwTreeDepth(cB);
    var xA = cA, xB = cB;
    while (dA > dB && xA.parentNode) { xA = xA.parentNode; dA--; }
    while (dB > dA && xB.parentNode) { xB = xB.parentNode; dB--; }
    var guard203 = 0;
    while (xA && xB && xA !== xB && guard203++ < 256) {
      var ppA = xA.parentNode, ppB = xB.parentNode;
      if (!ppA || !ppB) break;
      if (ppA === ppB) {
        var jA = _zwIndexOfChild(ppA, xA), jB = _zwIndexOfChild(ppB, xB);
        return jA >= 0 && jB >= 0 ? jA > jB : false;
      }
      xA = ppA; xB = ppB;
    }
    return true; // 无共同根（跨文档/detached）——恒 after（触发对侧重设）
  }
  // 节点深度（root 层级计数；无父 = 0）。
  function _zwTreeDepth(node) {
    var d = 0, cur = node;
    var guard = 0;
    while (cur && cur.parentNode && guard++ < 512) { d++; cur = cur.parentNode; }
    return d;
  }
  // node 是否为 self 或 other 的祖先（沿 parentNode 链）。
  function _zwIsAncestorOrSelf(anc, node) {
    var cur = node;
    var guard = 0;
    while (cur && guard++ < 256) {
      if (cur === anc) return true;
      cur = cur.parentNode;
    }
    return false;
  }
  // boundary-point 的「child 侧」索引：container 自身在 anc 的 childNodes 中的索引；
  // anc 为 null 时返回其在父下的索引（无父 null）。
  function _zwBpChildOf(node, anc) {
    if (anc !== null && node === anc) return null;
    var p = node.parentNode;
    if (!p || (anc !== null && !_zwIsAncestorOrSelf(anc, node))) return null;
    // 沿链上行到 anc 的直接子（anc 的 childNodes 索引即 boundary child 位置）。
    var cur = node;
    var guard = 0;
    while (cur && cur.parentNode && cur.parentNode !== anc && guard++ < 256) cur = cur.parentNode;
    if (!cur || !cur.parentNode || cur.parentNode !== anc) return null;
    return _zwIndexOfChild(anc, cur);
  }
  function _zwIndexOfChild(parent, child) {
    var kids = parent.childNodes;
    if (!kids) return -1;
    for (var i = 0; i < kids.length; i++) if (kids[i] === child) return i;
    return -1;
  }


  // selectNodeContents（整节点子树文本）+ 同文本节点 setStart/setEnd（slice 偏移）；其余 setStart/setEnd
  // 组合 best-effort 取 commonAncestor 子树文本（跨节点偏移不精确截取）；② deleteContents/extractContents/
  // insertNode/cloneContents/surroundContents（R2929/R2930）经既有 mutation-emitting proxy
  //（remove/insertBefore/appendChild/cloneNode）真实变更——精确覆盖 start==end 元素容器的 offset 区间
  //（selectNode/selectNodeContents 后），sel/handle 子均支持；surroundContents 精确落位仅在覆盖块延伸到容器
  // 末尾（selectNodeContents 包整元素内容），非尾部 best-effort 落末尾；跨容器/文本节点部分切片仍 best-effort；
  // ③ getBoundingClientRect/getClientRects 返空（无 layout 选择几何）；④ 无真 live。
  function _makeRange() {
    // R183（js-dom M4）：offset 活性 getter——spec「range 边界点随树变更更新」
    //（`concept-range` 的 boundary-point 引用 (node, offset) 对；`dom-node-remove` 末段
    // 「removed node 是 boundary node 的子 → offset 减 1」；被移除的是 start/end 自身 →
    // 移到 (parent, index)）。_mode.kind==='node'（selectNode 形态）时 startOffset/
    // endOffset 按被追踪节点在容器中的**当前**位置现算（WPT Range-adopt-test 四断言：
    // 移除唯一元素后 endOffset 期望 0——旧静态数据恒 1）。其他形态（setStart/setEnd
    // 自由边界）保持写入值（data 槽），无追踪锚点。
    var r183 = {
      startContainer: null, startOffset: 0, endContainer: null, endOffset: 0,
      commonAncestorContainer: null, collapsed: true, _mode: null,
      // js-dom M4 R42：spec `range-set-start/end` 校验——① node 无效（非 Node / DocumentType / Attr
      // 的 setStartBefore 族无 parent）抛 InvalidNodeTypeError；② offset 超节点 length 抛 IndexSizeError
      //（Attr length=0[子节点数]，Text/Comment=length[data]，Element=childNodes.length；WPT
      // Range-attribute-nodes "past its length throws IndexSizeError"）。length 计算按 spec
      // `concept-node-length`。
      _nodeLength: function (node) {
        if (!node) return 0;
        if (node.nodeType === 2 || node.nodeType === 10) return 0; // Attr / DocumentType：子节点数为 0
        if (node.nodeType === 3 || node.nodeType === 4 || node.nodeType === 7 || node.nodeType === 8) {
          return (node.data != null ? String(node.data).length : (node.nodeValue != null ? String(node.nodeValue).length : 0));
        }
        return node.childNodes ? node.childNodes.length : 0;
      },
      // R42 修正：spec `range-set-start/end` 仅拒 **DocumentType**（InvalidNodeTypeError）——Attr 允许作
      // 端点容器（length=0，offset 0 合法、>0 抛 IndexSizeError，WPT Range-attribute-nodes 正反两断言）。
      // 旧初版把 Attr 一并拒绝 → "at offset 0 is allowed" 误伤。R288 起元素容器的超长 offset
      // 恢复精确校验（旧版对 nodeType===1 放宽的历史动机——handle proxy childNodes 视图缺失——
      // 已被 R286 registry 事实源消解）。
      setStart: function (node, off) {
        if (!node || typeof node.nodeType !== 'number' || node.nodeType === 10) {
          throw new globalThis.DOMException('The given node is invalid.', 'InvalidNodeTypeError');
        }
        var o = off | 0;
        if (o < 0) throw new globalThis.DOMException('The given offset is out of bounds.', 'IndexSizeError');
        // R288（js-dom M4）：spec `range-set-start` 步骤 2——offset > node length 对
        // **元素容器同样抛 IndexSizeError**（旧版对 nodeType===1 整体放宽——WPT
        // Range-set "setStart()/setEnd() to a too-large offset must throw
        // INDEX_SIZE_ERR" point 30/39/40 [documentElement,7]/[paras[0],2]/
        // [paras[1],2] 240F 簇：html 2 子、p 1 子，offset 7/2 超 length 须抛）。
        // handle/registry 容器的 childNodes 融合视图自 R286 起以 registry 为
        // 事实源，长度可判定。
        if (o > this._nodeLength(node)) {
          throw new globalThis.DOMException('The given offset is out of bounds.', 'IndexSizeError');
        }
        // R203（js-dom M4）：spec `range-set-start` 步骤 3——新 start 在当前 end 之后
        //（边界点比较 FOLLOWING，含跨文档形态）时 end 一并设为 (node, offset)
        //（WPT Range-set 的 "setStart(node, offset) where node is after current end
        // or in different document must set the end node to node too"——旧版 end 残留
        // 旧容器）。比较经 `_zwRangeBpAfter`：跨文档（无共同根）恒 after（spec
        // position-of 边界点在异文档间不可比时 setStart 的 end 重设同样触发——
        // WPT "in different document" 正断言族）。
        var _r203End = this.endContainer;
        if (_r203End && _zwRangeBpAfter(node, o, _r203End, this.endOffset | 0)) {
          this.endContainer = node; this.endOffset = o;
        }
        this.startContainer = node; this.startOffset = o; this._recalc(); return this;
      },
      setEnd: function (node, off) {
        if (!node || typeof node.nodeType !== 'number' || node.nodeType === 10) {
          throw new globalThis.DOMException('The given node is invalid.', 'InvalidNodeTypeError');
        }
        var o = off | 0;
        if (o < 0) throw new globalThis.DOMException('The given offset is out of bounds.', 'IndexSizeError');
        // R288：同 setStart——元素容器的超长 offset 同样抛（spec `range-set-end` 步骤 2）。
        if (o > this._nodeLength(node)) {
          throw new globalThis.DOMException('The given offset is out of bounds.', 'IndexSizeError');
        }
        // R203（js-dom M4）：spec `range-set-end` 步骤 3 镜像——新 end 在当前 start
        // 之前（PRECEDING）时 start 一并设为 (node, offset)。
        var _r203Start = this.startContainer;
        if (_r203Start && _zwRangeBpAfter(_r203Start, this.startOffset | 0, node, o)) {
          this.startContainer = node; this.startOffset = o;
        }
        this.endContainer = node; this.endOffset = o; this._recalc(); return this;
      },
      // js-dom M4 R42：spec setStartBefore/After、setEndBefore/After、selectNode——ref 的父为 null
      //（Attr 无 parent / detached）抛 InvalidNodeTypeError（WPT "with an Attr node throws
      // InvalidNodeTypeError (null parent)"）。
      setStartBefore: function (node) { var p = node && node.parentNode; if (!p) throw new globalThis.DOMException('The given node has no parent.', 'InvalidNodeTypeError'); return this.setStart(p, this._indexOf(p, node)); },
      setStartAfter: function (node) { var p = node && node.parentNode; if (!p) throw new globalThis.DOMException('The given node has no parent.', 'InvalidNodeTypeError'); return this.setStart(p, this._indexOf(p, node) + 1); },
      setEndBefore: function (node) { var p = node && node.parentNode; if (!p) throw new globalThis.DOMException('The given node has no parent.', 'InvalidNodeTypeError'); return this.setEnd(p, this._indexOf(p, node)); },
      setEndAfter: function (node) { var p = node && node.parentNode; if (!p) throw new globalThis.DOMException('The given node has no parent.', 'InvalidNodeTypeError'); return this.setEnd(p, this._indexOf(p, node) + 1); },
      selectNode: function (node) {
        var sp = (node && node.parentNode) || null;
        if (!sp) throw new globalThis.DOMException('The given node has no parent.', 'InvalidNodeTypeError');
        var i = this._indexOf(sp, node);
        // R191（js-dom M4）：ownerDocument 快照——spec `concept-node-adopt` 对 live range
        // 的 retarget：被 adopt 子树内的 range 边界 collapse（WPT Range-adopt-test
        // "Parented range container moved..."——container 移入新 doc 后 endOffset 期望 0）。
        // offset getter 比对 tracked node 的当前 ownerDocument 与快照。
        this.startContainer = sp; this.startOffset = i;
        this.endContainer = sp; this.endOffset = i + 1;
        this.commonAncestorContainer = sp; this.collapsed = false;
        // R191：ownerDocument 快照（adopt collapse 判定——offset getter 比对）。
        this._mode = { node: node, kind: 'node', ownerDoc: node && node.ownerDocument ? node.ownerDocument : null };
        return this;
      },
      selectNodeContents: function (node) {
        // R289（js-dom M4）：node 是 DocumentType → 抛 InvalidNodeTypeError（spec
        // `dom-range-select-node-contents` 步骤 1「If node is a doctype, rethrow」
        // ——WPT "selectNodeContents() on a doctype must throw" 12F 簇
        // current doc[0]/xmlDoc[0] qorflesnorf × 4 range 域）。
        if (node && node.nodeType === 10) {
          throw new globalThis.DOMException(
            'The given node is invalid.', 'InvalidNodeTypeError');
        }
        // R289（js-dom M4）：endOffset = **node length**（spec
        // `dom-range-select-node-contents` 步骤 2「length of node」——
        // `concept-node-length`：CharacterData = data.length，其他 = 子节点数）。
        // 旧版恒读 childNodes.length 使 text/comment/PI 容器 endOffset 恒 0
        // （WPT Range-selectNode "endOffset must equal node length" 144F 簇：
        // #text 112 + #comment 24 + somepi 8）。
        var cnt;
        if (node && (node.nodeType === 3 || node.nodeType === 4 || node.nodeType === 7 || node.nodeType === 8)) {
          var d289 = node.data != null ? node.data : (node.__nv != null ? node.__nv : '');
          cnt = String(d289).length;
        } else {
          cnt = node && node.childNodes ? node.childNodes.length : 0;
        }
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
      // R2929：收集 range 覆盖的「顶层子节点」（start==end 元素容器 + offset 区间）。selectNode/
      // selectNodeContents 后的精确情况。跨容器 / 文本节点部分切片 → null（toString 已处理文本 slice）。
      _coveredChildren: function () {
        if (!this.startContainer || this.startContainer !== this.endContainer) return null;
        var sc = this.startContainer;
        // R194（js-dom M4）：fragment/shadow 容器（nodeType 11）纳入覆盖面——shadow root
        // 的 Range 操作（WPT Range-{delete,extract,clone}Contents-in-ShadowRoot 九用例：
        // setStart(shadowRoot, offset) 后 deleteContents 须按 offset 区间移 registry 子）。
        // R284（js-dom M4）：Document 容器（nodeType 9）纳入——同容器 doc 区间的
        // contained 子语义与元素一致（WPT Range-extractContents 51,x
        // `[document,1,document,2]`：html 子 move 入 frag + 塌缩 (doc,1)；旧版
        // 对 doc 返 null 使 extract/delete/clone 三侧同容器 doc 形态全空转）。
        if (sc.nodeType !== 1 && sc.nodeType !== 11 && sc.nodeType !== 9 && !sc.tagName) return null; // 非元素/fragment/doc 容器（文本切片）→ defer
        var kids = sc.childNodes;
        if (!kids || !kids.length) return [];
        var a = Math.max(0, Math.min(this.startOffset | 0, kids.length));
        var b = Math.max(a, Math.min(this.endOffset | 0, kids.length));
        var out = [];
        for (var i = a; i < b; i++) if (kids[i]) out.push(kids[i]);
        return out;
      },
      deleteContents: function () {
        // 删除范围内子节点（逆序 remove，保索引稳定）。复用 child.remove()（sel→__zw_remove / handle→__zw_remove_handle）。
        // R213（js-dom M4）：**CharData 区间删侧分支**（spec
        // `dom-range-delete-contents` 的删侧三段——start 容器尾段 deleteData +
        // contained 子移除 + end 容器头段 deleteData；同节点取中段；range 收缩到
        // (parent, si+1)——与 R213 的 extract 收缩偏移一致）。
        // https://dom.spec.whatwg.org/#dom-range-deletecontents
        var _r213sc = this.startContainer, _r213ec = this.endContainer;
        var _r213isCd = function (n) {
          return !!n && (n.nodeType === 3 || n.nodeType === 4
            || n.nodeType === 7 || n.nodeType === 8);
        };
        // R266（js-dom M4）：**同节点 CharData（detached 含 comment/PI）放宽**——
        // 与 extractContents 的 R228 同款：sc===ec 时无需父容器（spec
        // `dom-range-delete-contents` 的 replace-data 段不依赖 parent；WPT
        // Range-deleteContents 32-37,x `[detachedTextNode,0,…,8]` 等 12F 簇：
        // 旧 parentNode 门使 detached 同节点整体空转——期望 deleteData 削区间
        // 而 data 原样）。异节点仍需同父（中段 remove 循环依赖 kids）。
        // https://dom.spec.whatwg.org/#dom-range-deletecontents
        if (_r213isCd(_r213sc) && _r213isCd(_r213ec)
          && ((_r213sc === _r213ec)
            || (_r213sc.parentNode && _r213sc.parentNode === _r213ec.parentNode))) {
          var _r213p = _r213sc.parentNode;
          var _r213kids = _r213p ? (_r213p.childNodes || []) : [];
          var _r213si = _r213kids.indexOf(_r213sc);
          var _r213ei = _r213kids.indexOf(_r213ec);
          // R266：detached 同节点（_r213p null）——si/ei 双 -1 仍走同节点
          // deleteData + collapse（无中段兄弟需移除）；si>=0 门只对异节点形态。
          if (_r213sc === _r213ec || (_r213si >= 0 && _r213ei >= _r213si)) {
            if (_r213sc === _r213ec) {
              var _r213m = String(_r213sc.data != null ? _r213sc.data : '');
              var _r213a = Math.max(0, Math.min(this.startOffset | 0, this.endOffset | 0));
              var _r213b = Math.max(_r213a, Math.min(this.endOffset | 0, _r213m.length));
              if (_r213b > _r213a) {
                try { _r213sc.deleteData(_r213a, _r213b - _r213a); } catch (_eR213m) {}
              }
              this.collapse(true);
              return;
            }
            try { _r213sc.deleteData(this.startOffset | 0,
              String(_r213sc.data != null ? _r213sc.data : '').length - (this.startOffset | 0)); } catch (_eR213h) {}
            for (var _r213k = _r213ei - 1; _r213k > _r213si; _r213k--) {
              var _r213c = _r213kids[_r213k];
              if (!_r213c) continue;
              try { _r213p.removeChild(_r213c); } catch (_eR213c) {}
            }
            try { _r213ec.deleteData(0, this.endOffset | 0); } catch (_eR213t) {}
            this.setStart(_r213p, _r213si + 1);
            this.setEnd(_r213p, _r213si + 1);
            return;
          }
        }
        // R267（js-dom M4）：**祖先元素容器 + 直接 CharData 子**的 delete 侧
        //（R236 extract 的 ancestor 分支对称缺口）——sc 是 ec 的父元素且 ec 是
        // sc 的直接 text/comment/PI/CDATA 子（WPT Range-deleteContents 23,x
        // `[paras[0],0,paras[0].firstChild,7]`：期望削 text 头部 [0,eo) 保留
        // remainder "̈efgh\n" + contained 中段子移除 + 塌缩 (sc,so)；旧版落
        // _coveredChildren 融合视图空转——text 头部残留）。spec 序：
        // first-partially（ec 头部）deleteData + contained children 移除 +
        // collapse（delete 无 start 尾段——sc 自身是容器，so 之前的子不在区间）。
        // https://dom.spec.whatwg.org/#dom-range-deletecontents
        var _r267handled = false;
        (function _r267AncestorDel(self) {
          var sc = self.startContainer, ec = self.endContainer;
          var isCd = function (n) {
            return !!n && (n.nodeType === 3 || n.nodeType === 4
              || n.nodeType === 7 || n.nodeType === 8);
          };
          if (!sc || sc === ec || !isCd(ec) || ec.parentNode !== sc) return;
          if (!sc.childNodes || typeof ec.deleteData !== 'function') return;
          var so = self.startOffset | 0;
          var eo = self.endOffset | 0;
          // contained children：sc 的直接子中 [so, ecIdx) 区间全含者（逆序移除保索引）。
          var kidsNow = sc.childNodes;
          var ecIdx = -1;
          for (var i = 0; i < kidsNow.length; i++) if (kidsNow[i] === ec) { ecIdx = i; break; }
          if (ecIdx < 0) return;
          try { ec.deleteData(0, eo); } catch (_eR267d) {}
          for (var k = ecIdx - 1; k >= so; k--) {
            var c267 = kidsNow[k];
            if (!c267) continue;
            // R267：remove() 优先 + 结果校验兜底（泛型 Node.prototype.remove 对
            // 部分域形态静默失败——探针实证 post-parent 不变；removeChild 直调
            // 走 part04 域分发更可靠）。
            try {
              if (typeof c267.remove === 'function') c267.remove();
              if (c267.parentNode != null && typeof sc.removeChild === 'function') {
                sc.removeChild(c267);
              }
            } catch (_eR267r) {
              try { sc.removeChild(c267); } catch (_eR267r2) {}
            }
          }
          try { self.setStart(sc, so); self.setEnd(sc, so); } catch (_eR267c) {}
          _r267handled = true;
        })(this);
        if (_r267handled) return;
        // R268（js-dom M4）：**跨容器泛化**（对齐 common.js myDeleteContents 的
        // 树序算法）——sc/ec 不同容器（WPT Range-deleteContents 20-22/24/52/53,x：
        // `[paras[0].firstChild,0,paras[1].firstChild,8]` 跨段、`[testDiv,1,
        // paras[2].firstChild,5]` 深祖先、`[paras[3],1,comment,8]` 元素 sc + doc
        // comment ec）。四段：sc 尾段 deleteData + sc 侧爬升路径右侧兄弟移除 +
        // ec 头段 deleteData + ec 侧爬升路径左侧兄弟移除，塌缩 (reference 父,
        // idx+1)。contained 判定以「父也在区间内的剔除」（nodesToRemove 语义）。
        // https://dom.spec.whatwg.org/#dom-range-deletecontents
        var _r268handled = false;
        (function _r268CrossDel(self) {
          var sc = self.startContainer, ec = self.endContainer;
          if (!sc || !ec || sc === ec) return;
          var sn = sc.nodeType | 0, en = ec.nodeType | 0;
          // R268 首版教训：泛化 climb-tail 规则对 **element 端点**错（ec 是元素时
          // 其子树是 partially-contained——本体保留仅内部 [0,eo) 删，而 sc 元素
          // 尾部规则把 ec 子树一并删掉；ancestor 方向同理）。本切片先收敛到已验证
          // 正确的形态：**双侧 CharData 端点 + 子树不相交**（WPT 20/21,x 跨段 text
          // 形态）；element 端点/ancestor 方向走既有回落（R269 靶点——需按方向分支
          // 的 contained 递归算法）。
          var isCd2 = function (n) {
            var t = n ? (n.nodeType | 0) : 0;
            return t === 3 || t === 4 || t === 7 || t === 8;
          };
          if (!isCd2(sc) || !isCd2(ec)) return;
          // cac：sc 祖先链上首个含 ec 的容器。
          var chain = [];
          var cur = sc, hops = 0;
          while (cur && hops++ < 128) { chain.push(cur); cur = cur.parentNode; }
          var cac = null;
          for (var ci = 0; ci < chain.length; ci++) {
            var anc = chain[ci], probe = ec, h2 = 0;
            while (probe && h2++ < 128) {
              if (probe === anc) { cac = anc; break; }
              probe = probe.parentNode;
            }
            if (cac) break;
          }
          if (!cac) return;
          var isCd = function (n) {
            var t = n ? (n.nodeType | 0) : 0;
            return t === 3 || t === 4 || t === 7 || t === 8;
          };
          var rmNode = function (n, parent) {
            try {
              if (typeof n.remove === 'function') n.remove();
              if (n.parentNode != null && parent && typeof parent.removeChild === 'function') {
                parent.removeChild(n);
              }
            } catch (_e) {
              try { if (parent && typeof parent.removeChild === 'function') parent.removeChild(n); } catch (_e2) {}
            }
          };
          // sc 侧：CharData 尾段；元素 sc 的 [so, end) 右侧子；逐级爬升移除右侧兄弟。
          var so = self.startOffset | 0, eo = self.endOffset | 0;
          if (isCd(sc) && typeof sc.deleteData === 'function') {
            var sl = String(sc.data != null ? sc.data : '').length;
            if (so < sl) { try { sc.deleteData(so, sl - so); } catch (_eR268a) {} }
          } else if (sn === 1 || sn === 11) {
            var sk = sc.childNodes || [];
            for (var i2 = sk.length - 1; i2 >= so; i2--) rmNode(sk[i2], sc);
          }
          // sc 爬升：到 cac 前的每级，移除「本级路径子」的右侧兄弟（含本级尾部）。
          // R268：cac 级跳过（右侧全删会误删 ec 路径子——cac 级中段由下方
          // cac-middle 段统一处理 (sIdx, eIdx) 区间）。
          var lvl = sc, lvlPar = sc.parentNode;
          var hp = 0;
          while (lvlPar && lvl !== cac && hp++ < 128) {
            if (lvlPar === cac) break;
            var pk = lvlPar.childNodes || [];
            var pi = -1;
            for (var pj = 0; pj < pk.length; pj++) if (pk[pj] === lvl) { pi = pj; break; }
            if (pi < 0) break;
            for (var q2 = pk.length - 1; q2 > pi; q2--) rmNode(pk[q2], lvlPar);
            lvl = lvlPar;
            lvlPar = lvlPar.parentNode;
          }
          // ec 侧：爬升移除左侧兄弟（不含路径子本身），到 cac 停（cac 级的中段
          // 由 sc 侧的右侧移除覆盖——两路径在 cac 相遇）。
          var elvl = ec, epar = ec.parentNode;
          var hp2 = 0;
          while (epar && elvl !== cac && hp2++ < 128) {
            // R268：cac 级的中段（(sIdx, eIdx)）由下方 cac-middle 段统一处理——
            // 本循环跳过 cac 级（此处左侧全删会误删 sc 路径子及其左侧内容）。
            if (epar === cac) break;
            var ek = epar.childNodes || [];
            var ei2 = -1;
            for (var ej = 0; ej < ek.length; ej++) if (ek[ej] === elvl) { ei2 = ej; break; }
            if (ei2 < 0) break;
            for (var q3 = ei2 - 1; q3 >= 0; q3--) rmNode(ek[q3], epar);
            elvl = epar;
            epar = epar.parentNode;
          }
          // cac 级中段（sc 路径子 与 ec 路径子 之间）——两侧爬升循环都在 cac 级
          // break，中段统一在此移除（(sIdx, eIdx) 开区间）。
          var ck = cac.childNodes || [];
          var sIdx = -1, eIdx = -1;
          var sRef = sc;
          while (sRef && sRef.parentNode !== cac && sRef.parentNode) sRef = sRef.parentNode;
          var eRef = ec;
          while (eRef && eRef.parentNode !== cac && eRef.parentNode) eRef = eRef.parentNode;
          for (var ck2 = 0; ck2 < ck.length; ck2++) {
            if (sRef && ck[ck2] === sRef) sIdx = ck2;
            if (eRef && ck[ck2] === eRef) eIdx = ck2;
          }
          if (sIdx >= 0 && eIdx > sIdx) {
            for (var q4 = eIdx - 1; q4 > sIdx; q4--) rmNode(ck[q4], cac);
          }
          if (isCd(ec) && typeof ec.deleteData === 'function') {
            if (eo > 0) { try { ec.deleteData(0, eo); } catch (_eR268b) {} }
          } else if (en === 1 || en === 11) {
            var ek2 = ec.childNodes || [];
            for (var i3 = eo - 1; i3 >= 0; i3--) rmNode(ek2[i3], ec);
          }
          // 塌缩：reference node = sc 侧爬到 cac 的路径子（原位）；(其父= cac,
          // idx+1)。sc 是 cac 的 CharData 后代时 reference = sc 的 cac 直接祖先。
          var ref = sc;
          while (ref && ref.parentNode && ref.parentNode !== cac) ref = ref.parentNode;
          var rpk = cac.childNodes || [];
          var rIdx = -1;
          for (var rj = 0; rj < rpk.length; rj++) if (rpk[rj] === ref) { rIdx = rj; break; }
          try {
            if (rIdx >= 0) { self.setStart(cac, rIdx + 1); self.setEnd(cac, rIdx + 1); }
            else { self.setStart(cac, 0); self.setEnd(cac, 0); }
          } catch (_eR268c) {}
          _r268handled = true;
        })(this);
        if (_r268handled) return;
        // R279（js-dom M4）：**sc 元素端点的跨容器删除**——R278 分支的对称缺口
        //（sc=element + ec=element/CharData，两形态共用骨架）。spec 语义：
        // ① sc 元素 partially-contained——**本体保留，仅 [so, end) 直接子删除**
        //（不是删子树——R268 首版教训的另一半：element sc 的尾部规则）；
        // ② ec 元素同 R278（本体保留、[0, eo) 子删）；ec CharData 头段 deleteData；
        // ③ cac 级 (sIdx, eIdx) 开区间中段 + 两侧爬升兄弟移除（R268 同款）。
        // WPT Range-deleteContents 24,x `[testDiv,2,paras[4],1]`（ec=元素：
        // DIV 删 [2,4) 子 + P#e 空壳化 + 塌缩 (DIV,2)）；48,x `[testDiv,1,
        // paras[2].firstChild,5]`（ec=深后代 CharData：DIV 删 [1,2) 子 + 爬升
        // 右侧兄弟 + ec 头段削）；53,x `[paras[3],1,comment,8]`（sc=P#d 元素 +
        // ec=DIV comment——P#d 删 [1,2) 子（其 text）+ 爬升 + comment 头段削）。
        // 旧版 miss（R268/R278 都要求 sc 是 CharData）→ 回落空转。
        // https://dom.spec.whatwg.org/#dom-range-deletecontents
        var _r279handled = false;
        (function _r279ElDel(self) {
          var sc = self.startContainer, ec = self.endContainer;
          if (!sc || !ec || sc === ec) return;
          var sn = sc.nodeType | 0;
          if (sn !== 1 && sn !== 11) return;
          var isCd = function (n) {
            var t = n ? (n.nodeType | 0) : 0;
            return t === 3 || t === 4 || t === 7 || t === 8;
          };
          var en = ec.nodeType | 0;
          var ecIsEl = (en === 1 || en === 11);
          if (!ecIsEl && !isCd(ec)) return;
          if (!sc.parentNode || !ec.parentNode) return;
          // cac：sc 祖先链上首个含 ec 的容器（R268 同款）。
          var chain = [], cur = sc, hops = 0;
          while (cur && hops++ < 128) { chain.push(cur); cur = cur.parentNode; }
          var cac = null;
          for (var ci = 0; ci < chain.length && !cac; ci++) {
            var probe = ec, h2 = 0;
            while (probe && h2++ < 128) {
              if (probe === chain[ci]) { cac = chain[ci]; break; }
              probe = probe.parentNode;
            }
          }
          if (!cac) return;
          var so = self.startOffset | 0, eo = self.endOffset | 0;
          // **同树位守卫**（WPT 49/50,x `[docEl,1,body,0]` 形态）：sc 的 so 位子
          // === ec 且 eo===0 → (sc,so) 与 (ec,0) 是**同一树位**——删除区间为空，
          // 零删除。spec 塌缩序「sc 是 ec 的 ancestor container → (sc, so)」
          //（common.js myDeleteContents 同款）——首版教训：塌 (ec,0) 使
          // startContainer !== 期望的 (sc,so)。
          var sk0 = sc.childNodes || [];
          if (sk0[so] === ec && eo === 0) {
            try { self.setStart(sc, so); self.setEnd(sc, so); } catch (_eR279g) {}
            _r279handled = true;
            return;
          }
          var rmNode = function (n, parent) {
            try {
              if (typeof n.remove === 'function') n.remove();
              if (n.parentNode != null && parent && typeof parent.removeChild === 'function') {
                parent.removeChild(n);
              }
            } catch (_e) {
              try { if (parent && typeof parent.removeChild === 'function') parent.removeChild(n); } catch (_e2) {}
            }
          };
          // ① sc 元素尾部：[so, ecIdx) 直接子逆序移除（partially-contained——本体
          // 保留，尾部**止于 ec 的路径子**：ec 是端点本体不动、ec 后的兄弟不在
          // 区间。ec 非 sc 后代（隔层）时 ecIdx=-1 → 尾部止于爬升段处理，此处
          // 只删 [so, end) 中 sc 的全部子——与 R268 sc 侧爬升对称由下方统一）。
          var sk279 = sc.childNodes || [];
          var ecIdxIn279 = -1;
          for (var k279f = 0; k279f < sk279.length; k279f++) {
            if (sk279[k279f] === ec) { ecIdxIn279 = k279f; break; }
            // ec 的祖先在 sc 直接子中（ec 是深后代）→ 尾部止于该路径子。
            var anc279 = ec, ah279 = 0;
            while (anc279 && ah279++ < 128) {
              if (sk279[k279f] === anc279) { ecIdxIn279 = k279f; break; }
              anc279 = anc279.parentNode;
            }
            if (ecIdxIn279 >= 0) break;
          }
          var tailEnd279 = (ecIdxIn279 >= 0) ? ecIdxIn279 : sk279.length;
          for (var i279 = tailEnd279 - 1; i279 >= so; i279--) rmNode(sk279[i279], sc);
          // ② sc 侧爬升：到 cac 前逐级移除路径子右侧兄弟（cac 级跳过——mid 段处理）。
          var lvl279 = sc, lvlPar279 = sc.parentNode, hp279 = 0;
          while (lvlPar279 && lvl279 !== cac && hp279++ < 128) {
            if (lvlPar279 === cac) break;
            var pk279 = lvlPar279.childNodes || [];
            var pi279 = -1;
            for (var pj279 = 0; pj279 < pk279.length; pj279++) if (pk279[pj279] === lvl279) { pi279 = pj279; break; }
            if (pi279 < 0) break;
            for (var q279b = pk279.length - 1; q279b > pi279; q279b--) rmNode(pk279[q279b], lvlPar279);
            lvl279 = lvlPar279;
            lvlPar279 = lvlPar279.parentNode;
          }
          // ③ ec 侧：元素端点 [0, eo) 子删（R278 同款）；CharData 头段 deleteData
          //（先做头段——R268 序）；再爬升移除左侧兄弟（cac 级跳过）。
          if (ecIsEl) {
            var ek279 = ec.childNodes || [];
            for (var i279c = Math.min(eo, ek279.length) - 1; i279c >= 0; i279c--) rmNode(ek279[i279c], ec);
          } else if (typeof ec.deleteData === 'function') {
            if (eo > 0) { try { ec.deleteData(0, eo); } catch (_eR279d) {} }
          }
          var elvl279 = ec, epar279 = ec.parentNode, hp279b = 0;
          while (epar279 && elvl279 !== cac && hp279b++ < 128) {
            if (epar279 === cac) break;
            var ekp279 = epar279.childNodes || [];
            var ei279 = -1;
            for (var ej279 = 0; ej279 < ekp279.length; ej279++) if (ekp279[ej279] === elvl279) { ei279 = ej279; break; }
            if (ei279 < 0) break;
            for (var q279c = ei279 - 1; q279c >= 0; q279c--) rmNode(ekp279[q279c], epar279);
            elvl279 = epar279;
            epar279 = epar279.parentNode;
          }
          // ④ cac 级中段 (sIdx, eIdx) 开区间移除（R268 同款；ec CharData 时其
          // cac 直接子就是 eRef，头段已削）。
          var ck279 = cac.childNodes || [];
          var sRef279 = sc;
          while (sRef279 && sRef279.parentNode !== cac && sRef279.parentNode) sRef279 = sRef279.parentNode;
          var eRef279 = ec;
          while (eRef279 && eRef279.parentNode !== cac && eRef279.parentNode) eRef279 = eRef279.parentNode;
          var sIdx279 = -1, eIdx279 = -1;
          for (var ck279b = 0; ck279b < ck279.length; ck279b++) {
            if (sRef279 && ck279[ck279b] === sRef279) sIdx279 = ck279b;
            if (eRef279 && ck279[ck279b] === eRef279) eIdx279 = ck279b;
          }
          if (sIdx279 >= 0 && eIdx279 > sIdx279) {
            for (var q279d = eIdx279 - 1; q279d > sIdx279; q279d--) rmNode(ck279[q279d], cac);
          }
          // ⑤ 塌缩（spec 塌缩序 + common.js myDeleteContents 同款）：**sc 是 ec 的
          // ancestor container（cac === sc）→ (sc, so)**（24,x 期望 (DIV,2)；首版
          // R268 式 (cac, sIdx+1) 对 cac===sc 形态把 sRef 爬过头 → rIdx=-1 → 0）；
          // 否则 (cac, sIdx+1)（sc 路径子之后，R268 同款）。
          if (cac === sc) {
            try { self.setStart(sc, so); self.setEnd(sc, so); } catch (_eR279e) {}
            _r279handled = true;
            return;
          }
          var rpk279 = cac.childNodes || [];
          var rIdx279 = -1;
          for (var rj279 = 0; rj279 < rpk279.length; rj279++) if (rpk279[rj279] === sRef279) { rIdx279 = rj279; break; }
          try {
            if (rIdx279 >= 0) { self.setStart(cac, rIdx279 + 1); self.setEnd(cac, rIdx279 + 1); }
            else { self.setStart(cac, 0); self.setEnd(cac, 0); }
          } catch (_eR279c) {}
          _r279handled = true;
        })(this);
        if (_r279handled) return;
        // R278（js-dom M4）：**sc CharData + ec 元素端点的跨容器删除**——R268
        // 首版教训把 element 端点整体排除（「climb-tail 规则对元素端点错」），
        // 但 spec 的 ec 元素形态有独立正解：**partially-contained 语义——ec 本体
        // 保留，仅其 [0, eo) 直接子（fully contained，因 ec 是端点）删除**。
        // WPT Range-deleteContents 22,x `[paras[0].firstChild,3,paras[3],1]`：
        // sc=text 尾段删（保 "Äb"）+ 中段 contained 子移除（paras[1..2]）+
        // ec=P#d 保留本体、其首个子（text "Yzabcdef"）删 + 塌缩。旧版全分支
        // miss（R268 要求双 CharData）→ `_coveredChildren` 回落对 sc≠ec 恒
        // null → 整体 no-op（probe 实证：POST sc.nodeValue 仍全量、P#a kids=1）。
        // 与 R268 的差别仅 ec 侧两段：头段 deleteData 换成 [0,eo) 子移除，
        // ec 侧爬升从「ec 的父」起步（ec 本体不动）。sc 侧/mid/collapse 复用
        // R268 同款（已验证形态）。
        // https://dom.spec.whatwg.org/#dom-range-deletecontents
        var _r278handled = false;
        (function _r278CdElDel(self) {
          var sc = self.startContainer, ec = self.endContainer;
          if (!sc || !ec || sc === ec) return;
          var isCd = function (n) {
            var t = n ? (n.nodeType | 0) : 0;
            return t === 3 || t === 4 || t === 7 || t === 8;
          };
          if (!isCd(sc)) return;
          var en = ec.nodeType | 0;
          if (en !== 1 && en !== 11 && !ec.tagName) return;
          if (!sc.parentNode || !ec.parentNode) return;
          // cac：sc 祖先链上首个含 ec 的容器（R268 同款）。
          var chain = [], cur = sc, hops = 0;
          while (cur && hops++ < 128) { chain.push(cur); cur = cur.parentNode; }
          var cac = null;
          for (var ci = 0; ci < chain.length && !cac; ci++) {
            var probe = ec, h2 = 0;
            while (probe && h2++ < 128) {
              if (probe === chain[ci]) { cac = chain[ci]; break; }
              probe = probe.parentNode;
            }
          }
          if (!cac) return;
          var so = self.startOffset | 0, eo = self.endOffset | 0;
          var rmNode = function (n, parent) {
            try {
              if (typeof n.remove === 'function') n.remove();
              if (n.parentNode != null && parent && typeof parent.removeChild === 'function') {
                parent.removeChild(n);
              }
            } catch (_e) {
              try { if (parent && typeof parent.removeChild === 'function') parent.removeChild(n); } catch (_e2) {}
            }
          };
          // ① sc 尾段 deleteData（保 [0, so)）。
          if (typeof sc.deleteData === 'function') {
            var sl = String(sc.data != null ? sc.data : '').length;
            if (so < sl) { try { sc.deleteData(so, sl - so); } catch (_eR278a) {} }
          }
          // ② sc 侧爬升：到 cac 前逐级移除路径子右侧兄弟（cac 级跳过——mid 段处理）。
          var lvl = sc, lvlPar = sc.parentNode, hp = 0;
          while (lvlPar && lvl !== cac && hp++ < 128) {
            if (lvlPar === cac) break;
            var pk = lvlPar.childNodes || [];
            var pi = -1;
            for (var pj = 0; pj < pk.length; pj++) if (pk[pj] === lvl) { pi = pj; break; }
            if (pi < 0) break;
            for (var q2 = pk.length - 1; q2 > pi; q2--) rmNode(pk[q2], lvlPar);
            lvl = lvlPar;
            lvlPar = lvlPar.parentNode;
          }
          // ③ ec 侧（元素端点 partially-contained）：本体保留，[0, eo) 直接子
          // 逆序移除；再从 ec 的父爬升移除左侧兄弟（不含 ec 本身，cac 级跳过）。
          var ek = ec.childNodes || [];
          for (var i3 = Math.min(eo, ek.length) - 1; i3 >= 0; i3--) rmNode(ek[i3], ec);
          var elvl = ec, epar = ec.parentNode, hp2 = 0;
          while (epar && elvl !== cac && hp2++ < 128) {
            if (epar === cac) break;
            var ekp = epar.childNodes || [];
            var ei2 = -1;
            for (var ej = 0; ej < ekp.length; ej++) if (ekp[ej] === elvl) { ei2 = ej; break; }
            if (ei2 < 0) break;
            for (var q3 = ei2 - 1; q3 >= 0; q3--) rmNode(ekp[q3], epar);
            elvl = epar;
            epar = epar.parentNode;
          }
          // ④ cac 级中段 (sIdx, eIdx) 开区间移除（R268 同款）。
          var ck = cac.childNodes || [];
          var sRef = sc;
          while (sRef && sRef.parentNode !== cac && sRef.parentNode) sRef = sRef.parentNode;
          var eRef = ec;
          while (eRef && eRef.parentNode !== cac && eRef.parentNode) eRef = eRef.parentNode;
          var sIdx = -1, eIdx = -1;
          for (var ck2 = 0; ck2 < ck.length; ck2++) {
            if (sRef && ck[ck2] === sRef) sIdx = ck2;
            if (eRef && ck[ck2] === eRef) eIdx = ck2;
          }
          if (sIdx >= 0 && eIdx > sIdx) {
            for (var q4 = eIdx - 1; q4 > sIdx; q4--) rmNode(ck[q4], cac);
          }
          // ⑤ 塌缩 (cac, sIdx+1)（R268 同款 reference 定位）。
          var rpk = cac.childNodes || [];
          var rIdx = -1;
          for (var rj = 0; rj < rpk.length; rj++) if (rpk[rj] === sRef) { rIdx = rj; break; }
          try {
            if (rIdx >= 0) { self.setStart(cac, rIdx + 1); self.setEnd(cac, rIdx + 1); }
            else { self.setStart(cac, 0); self.setEnd(cac, 0); }
          } catch (_eR278c) {}
          _r278handled = true;
        })(this);
        if (_r278handled) return;
        // Range-deleteContents 25/26,x `[document,0,document,1/2]`：doc 的
        // contained 子（doctype/html）旧 `_coveredChildren` 融合视图对主文档
        // proxy 恒空 → 无移除（「expected 1/0 got 2」）。spec 同容器形态 =
        // contained children 直接删（deleteContents 的中段），塌缩 (容器, so)。
        // **限定 nodeType 9**：元素/fragment 容器已被 _coveredChildren 回落
        // 正确处理（防行为面漂移）。doctype 走主文档 removeChild 的本地标记
        // 路径；html 移除 host 不支持但 JS 视图按记录反映。
        // https://dom.spec.whatwg.org/#dom-range-deletecontents
        var _r269handled = false;
        (function _r269SameContainerDel(self) {
          var sc = self.startContainer, ec = self.endContainer;
          if (!sc || sc !== ec) return;
          var sn = sc.nodeType | 0;
          if (sn !== 9) return;
          if (!sc.childNodes || !sc.childNodes.length) return;
          var so = self.startOffset | 0, eo = self.endOffset | 0;
          if (so >= eo) return;
          var rm269 = function (n, parent) {
            try {
              if (typeof n.remove === 'function') n.remove();
              if (n.parentNode != null && parent && typeof parent.removeChild === 'function') {
                parent.removeChild(n);
              }
            } catch (_e) {
              try { if (parent && typeof parent.removeChild === 'function') parent.removeChild(n); } catch (_e2) {}
            }
          };
          for (var k269 = eo - 1; k269 >= so; k269--) {
            var c269 = sc.childNodes[k269];
            if (!c269) continue;
            rm269(c269, sc);
          }
          try { self.setStart(sc, so); self.setEnd(sc, so); } catch (_eR269c) {}
          _r269handled = true;
        })(this);
        if (_r269handled) return;
        var kids = this._coveredChildren();
        if (kids) {
          for (var i = kids.length - 1; i >= 0; i--) {
            try {
              if (typeof kids[i].remove === 'function') kids[i].remove();
              else this.startContainer.removeChild(kids[i]);
            } catch (_e) {}
          }
          this.collapse(true);
        }
        return;
      },
      extractContents: function () {
        // 提取范围内容到 DocumentFragment。sel 子无法直接 move（appendChild 须 child handle），故 clone 到
        // fragment + 移除原件（净效果等价：文档去内容、fragment 得内容；fragment 内为克隆非原件，documented）。
        // 克隆正序（保文档序进 fragment）；移除**逆序**——nth-child 结构选择器逆序稳定（移除末尾不前移兄长），
        // 正序移除会因 sibling 前移致选择器错位（删错节点）。同 deleteContents 的逆序移除。
        // R284（js-dom M4）：**frag 归 start 节点的 ownerDocument 域**（spec
        // `dom-range-extract-contents` 步骤 1——common.js myExtractContents 同款；
        // 旧版恒主 document fragment，跨域 append 对 iframe/detached 域子被
        // flat 成裸 text——WPT Range-extractContents 53,x frag 期望 <p id="e">
        // 包裹 got "Ghijklmn"）。
        // https://dom.spec.whatwg.org/#dom-range-extractcontents
        var _r284od = null;
        try { _r284od = this.startContainer.ownerDocument || (this.startContainer.nodeType === 9 ? this.startContainer : null); } catch (_e284od) {}
        var f = (_r284od && typeof _r284od.createDocumentFragment === 'function')
          ? _r284od.createDocumentFragment()
          : globalThis.document.createDocumentFragment();
        // R211（js-dom M4）：**CharacterData 区间提取分支**（spec
        // `dom-range-extract-contents` 的 first/last partially contained
        // CharacterData 子 + contained children 三段——common.js
        // myExtractContents 同款算法）。适用形态：start/end 容器是 Text/CDATA
        //（或其一在 CharData 内），cac 是其父元素。产出：frag =
        // [start 容器尾部切片克隆, contained 子本体, end 容器头部切片克隆]，
        // 原树 deleteData 掉切片 + 移除 contained 子。CDATA cloneNode 的 nt=4
        // 分支（R211 成对 land）供 sim 侧对齐。
        // https://dom.spec.whatwg.org/#dom-range-extractcontents
        var _r211sc = this.startContainer, _r211ec = this.endContainer;
        var _r211isCd = function (n) {
          return !!n && (n.nodeType === 3 || n.nodeType === 4
            || n.nodeType === 7 || n.nodeType === 8);
        };
        // R228（js-dom M4）：**同节点 CharData 区间（detached 含 comment/PI）**——
        // start===end 容器时无需父容器（spec `dom-range-extract-contents` 对
        // partially-contained CharacterData 的 clone-切片 + deleteData 不依赖父；
        // common.js myExtractContents 同款）。旧版 guard 要求 parentNode 非空使
        // detachedComment/detachedProcessingInstruction 的区间 surround 整族
        // 不变（WPT Range-surroundContents 35,x/36,x「Stuwxyz got Stuvwxyz」84F
        // 簇——extract 空转，树不变）。仅同节点形态放宽（异节点仍需同父定位）。
        // https://dom.spec.whatwg.org/#dom-range-extractcontents
        var _r228sameNode = (_r211sc === _r211ec && _r211isCd(_r211sc));
        if (_r211isCd(_r211sc) && _r211isCd(_r211ec)
          && (_r228sameNode
            || (_r211sc.parentNode && _r211sc.parentNode === _r211ec.parentNode))) {
          var _r211p = _r211sc.parentNode;
          // R228：detached 同节点（_r211p null）——kids 空数组（下文同节点分支
          // 不消费父定位；null.childNodes 直接 TypeError）。
          var _r211kids = (_r211p && _r211p.childNodes) || [];
          // R228：detached 同节点（_r211p null）——frag 归主 document。
          var _r211frag = (_r211p && _r211p.ownerDocument)
            ? _r211p.ownerDocument.createDocumentFragment()
            : globalThis.document.createDocumentFragment();
          var _r211si = _r211kids.indexOf(_r211sc);
          var _r211ei = _r211kids.indexOf(_r211ec);
          if (_r228sameNode && _r211si < 0) {
            // R228：detached 同节点——中段切片 + deleteData + collapse 到
            // (容器, startOffset)（无父定位/collapse-to-parent 步骤）。
            var _r228m = String(_r211sc.data != null ? _r211sc.data : '');
            var _r228a = Math.max(0, Math.min(this.startOffset | 0, this.endOffset | 0));
            var _r228b = Math.max(_r228a, Math.min(this.endOffset | 0, _r228m.length));
            if (_r228b > _r228a) {
              try {
                var _r228mid = _r211sc.cloneNode(false);
                _r228mid.data = _r228m.slice(_r228a, _r228b);
                _r211frag.appendChild(_r228mid);
              } catch (_eR228m) {}
              try { _r211sc.deleteData(_r228a, _r228b - _r228a); } catch (_eR228d) {}
            }
            // R229（js-dom M4）：detached 无父——range 边界**保持不变**（sim 的
            // myExtractContents 塌缩步走 parent_ 定位，无父时不重设边界；WPT
            // Range-surroundContents 32,x「endOffset expected 8 got 0」18F——
            // 旧版强 collapse 到 (容器, a) 使 position 断言分歧）。
            return _r211frag;
          }
          if (_r211si >= 0 && _r211ei >= _r211si) {
            // ① start 容器尾部切片（若 start==end 且同节点 → 单区间中段）
            if (_r211sc === _r211ec) {
              var _r211m = String(_r211sc.data != null ? _r211sc.data : '');
              var _r211a = Math.max(0, Math.min(this.startOffset | 0, this.endOffset | 0));
              var _r211b = Math.max(_r211a, Math.min(this.endOffset | 0, _r211m.length));
              if (_r211b > _r211a) {
                try {
                  var _r211mid = _r211sc.cloneNode(false);
                  _r211mid.data = _r211m.slice(_r211a, _r211b);
                  _r211frag.appendChild(_r211mid);
                } catch (_eR211m) {}
                try { _r211sc.deleteData(_r211a, _r211b - _r211a); } catch (_eR211md) {}
              }
            } else {
              try {
                var _r211head = _r211sc.cloneNode(false);
                _r211head.data = String(_r211sc.data != null ? _r211sc.data : '')
                  .slice(this.startOffset | 0);
                _r211frag.appendChild(_r211head);
              } catch (_eR211h) {}
              try { _r211sc.deleteData(this.startOffset | 0,
                String(_r211sc.data != null ? _r211sc.data : '').length - (this.startOffset | 0)); } catch (_eR211hd) {}
              // ② contained 子（两容器间，不含端点）本体移动——源父的 removed
              // record（含兄弟字段）由 appendChild move 路径的 R301 修复统一发出
              //（part04 appendChild trap 旧父 record 捕获 prev/next）。
              for (var _r211k = _r211si + 1; _r211k < _r211ei; _r211k++) {
                var _r211c = _r211kids[_r211k];
                if (!_r211c) continue;
                try { _r211frag.appendChild(_r211c); } catch (_eR211c) {}
              }
              // ③ end 容器头部切片
              try {
                var _r211tail = _r211ec.cloneNode(false);
                _r211tail.data = String(_r211ec.data != null ? _r211ec.data : '')
                  .slice(0, this.endOffset | 0);
                _r211frag.appendChild(_r211tail);
              } catch (_eR211t) {}
              try { _r211ec.deleteData(0, this.endOffset | 0); } catch (_eR211td) {}
            }
            // R213（js-dom M4）：collapse 偏移修正——spec「Set new offset to one
            // plus the index of reference node」（referenceNode = start 容器，
            // common.js myExtractContents 的 newOffset = 1 + indexOf 同款）。
            // 旧版 setStart(si) 使后续 insertNode 落在削弱的 start 容器**前**
            // （sim 落后一位——6,x positionTests 的 offsets A=0,1 E=1,2 根因）。
            // R229（js-dom M4）：**同节点区间**（sc===ec）的 sim 分支是
            // isAncestorContainer(start, end) 的 self 命中——collapse 到
            // (容器, startOffset) 而非 (父, si+1)（common.js myExtractContents
            // 的首分支；WPT Range-surroundContents 39,x「startOffset expected 0
            // got 3」17F——PI 同节点区间被 collapse 到 (xmlDoc, 3)）。异节点
            // 同父保持 (父, si+1)（else 分支）。
            // R231（js-dom M4）：同节点区间 extract **不塌缩边界**——沙箱内直接
            // 执行 common.js myExtractContents 源探针实证：[t,2,8] 的 data 削为
            // "Op" 但 range 保持 (t,2)-(t,8)（sim 的 CharacterData first/last
            // 子路径在中段 clone/deleteData 后早返回，尾部的 setStart/setEnd
            // 塌缩不执行；WPT Range-surroundContents 2,x/27,x 的
            // 「endOffset expected 8 got 2」~93F）。异节点同父保持 (父, si+1)。
            if (_r211sc !== _r211ec) {
              this.setStart(_r211p, _r211si + 1);
              this.setEnd(_r211p, _r211si + 1);
            }
            return _r211frag;
          }
        }
        // R280（js-dom M4）：**跨容器提取泛化**（sc/ec 异容器、非直接子形态——
        // R278 oracle 复活后重聚类的 extract 侧镜像缺口）。覆盖簇（WPT
        // Range-extractContents）：20/21,x `[paras[0].firstChild,0,paras[1].
        // firstChild,8]`（CD→CD 跨 P 段）、22,x `[paras[0].firstChild,3,
        // paras[3],1]`（sc CD + ec 元素）、48,x `[testDiv,1,paras[2].firstChild,5]`
        //（sc 元素 + ec 深后代 CD）、52,x `[paras[2].firstChild,4,comment,2]`
        //（sc CD + ec DIV comment）、29/31,x（doc 容器 + comment）——旧版全
        // miss（R211 同父 / R236/R242 直接子限定）→ `_coveredChildren` sc≠ec
        // null → 空转（fail 面三件：DOM 未剪 / fragment 空 / cursor 未塌）。
        // 对齐 common.js myExtractContents 的扁平化三段：sc 尾段（CD 切片克隆
        // 入 frag + 源 deleteData；sc 元素的尾段子属 contained 子走 move）+
        // 中段 contained 子**本体 move** 入 frag（wrapper 域 move 兜底同
        // R241：append 后残留则 removeChild 强制离场）+ ec 头段（CD 切片克隆
        // 入 frag + 源 deleteData；ec 元素 shallow clone + 其 [0,eo) 子 move
        // 入 clone 后 clone 入 frag）。源树修剪 = R278/R279 的 delete 侧同款
        //（sc 尾部止于 ec 路径子 + cac 中段 + ec 头部）。
        // https://dom.spec.whatwg.org/#dom-range-extractcontents
        // R282（js-dom M4）：**doctype contained 抛 HRE**（spec `dom-range-
        // extract-contents` 与 cloneContents 同款步骤「If a contained child is
        // a DocumentType, throw」——WPT Range-extractContents 25/26,x
        // `[document,0,document,1/2]`：expected HierarchyRequestError；旧版
        // 静默返回 frag。R281b 只接了 clone 侧——本切片对称移植）。
        // https://dom.spec.whatwg.org/#concept-range-extract
        (function _r282DocThrow(self) {
          var sc = self.startContainer, ec = self.endContainer;
          if (!sc) return;
          // 同容器（含 doc 同节点）：contained = [so, eo) 子区间。
          // 跨容器：cac 级 (sIdx, eIdx) 开区间 + sc/ec 侧路径区间——近似取
          // 主区间（spec 的 contained children 全集里 doctype 只会在 doc 直下）。
          var checkKids = function (parent, from, to) {
            if (!parent || !parent.childNodes) return false;
            var ks = parent.childNodes;
            var a = Math.max(0, from | 0), b = Math.min(to | 0, ks.length);
            for (var i = a; i < b; i++) {
              if (ks[i] && ks[i].nodeType === 10) return true;
            }
            return false;
          };
          if (sc === ec) {
            if (checkKids(sc, self.startOffset, self.endOffset)) {
              throw new (globalThis.DOMException || Error)(
                'The range includes a DocumentType node.', 'HierarchyRequestError');
            }
            return;
          }
          if (!ec || sc.nodeType !== 9) return;
          // doc sc 跨容器：doc 的 [so, ecPathIdx) 尾段区间。
          var ks9 = sc.childNodes || [];
          var ePath9 = -1;
          for (var k9 = 0; k9 < ks9.length; k9++) {
            var anc9 = ec, h9 = 0;
            while (anc9 && h9++ < 128) {
              if (ks9[k9] === anc9) { ePath9 = k9; break; }
              anc9 = anc9.parentNode;
            }
            if (ePath9 >= 0) break;
          }
          var end9 = (ePath9 >= 0) ? ePath9 : ks9.length;
          if (checkKids(sc, self.startOffset, end9)) {
            throw new (globalThis.DOMException || Error)(
              'The range includes a DocumentType node.', 'HierarchyRequestError');
          }
        })(this);
        (function _r280CrossExtract(self) {
          var sc = self.startContainer, ec = self.endContainer;
          if (!sc || !ec || sc === ec) return;
          var isCd = function (n) {
            var t = n ? (n.nodeType | 0) : 0;
            return t === 3 || t === 4 || t === 7 || t === 8;
          };
          var scCd = isCd(sc), ecCd = isCd(ec);
          // R280b：sc Document 容器（nodeType 9）纳入——29/31,x `[foreignDoc,1,
          // foreignComment,2]` 族（doc 的 [so, ecPathIdx) 子 move + ec 头段）。
          if (!scCd && !(sc.nodeType === 1 || sc.nodeType === 11 || sc.nodeType === 9)) return;
          if (!ecCd && !(ec.nodeType === 1 || ec.nodeType === 11)) return;
          // R282：**doc sc 的 parentNode 恒 null 是合法形态**（Document 无父——
          // 旧守卫在此拒绝 doc sc 使 29/31,x 的 ⓪ 尾段从未执行，probe 实证
          // dbg=unset 而 collapse 却发生）。仅要求 sc 非 doc 时有父。
          var scParOk = (sc.nodeType === 9) || !!sc.parentNode;
          if (!scParOk || !ec.parentNode) return;
          // cac：sc 祖先链上首个含 ec 的容器（R268 同款）。
          var chain = [], cur = sc, hops = 0;
          while (cur && hops++ < 128) { chain.push(cur); cur = cur.parentNode; }
          var cac = null;
          for (var ci = 0; ci < chain.length && !cac; ci++) {
            var probe = ec, h2 = 0;
            while (probe && h2++ < 128) {
              if (probe === chain[ci]) { cac = chain[ci]; break; }
              probe = probe.parentNode;
            }
          }
          if (!cac) return;
          // **形态限流（首版教训：泛化分支抢了 R242/R236 已正确处理的直接子
          // 形态——24/28/30,x 回归 +9）**：本分支接 sc 是 CharData 或 Document
          // 的跨容器形态 + **R283 element-sc 一层递归形态**（ec 的父链在 sc
          // 直接子一级内——48,x `[testDiv,1,pc.firstChild,5]`：sc 尾段子中
          // fully-contained 者本体 move、partially-contained 的 ec 路径子
          // clone 后把 ec 头段 move 进 clone）；更深的 element sc 递归与
          // R242/R236 直接子形态仍让位（防 R280 首版 +9 回归）。
          var r283elSc = false;
          if (!scCd && sc.nodeType !== 9) {
            r283elSc = (sc.nodeType === 1 || sc.nodeType === 11)
              && sc !== ec
              && ec.parentNode !== sc
              && (function () {
                // 形态 A（48,x）：ec 的父在 sc 的直接子里（一层递归——sc 尾段
                // fully-contained 子 move + ec 路径子由 ④' lastPartial 组树）。
                var ep = ec.parentNode;
                var ks = sc.childNodes || [];
                for (var q = 0; q < ks.length; q++) if (ks[q] === ep) return true;
                // 形态 B（53,x）：ec 的父 === sc 的父（sibling 方向——sc 尾段 +
                // cac 级 (sIdx,eIdx) 中段 + ec 头段，全部既有段可表达）。
                if (ep && ep === sc.parentNode) return true;
                return false;
              })();
            if (!r283elSc) return;
          }
          var so = self.startOffset | 0, eo = self.endOffset | 0;
          var sameTreePos = false;
          if (!scCd && (sc.childNodes || [])[so] === ec && eo === 0) sameTreePos = true;
          // R241 同款 wrapper 域 move 兜底：proxy fragment 的 appendChild 对 plain
          // 子只登记/改 parentNode 不摘原件（探针实证 pd 同时在 tree 与 frag 的
          // childNodes）——先记原父与原列表，append 后仍残留则强制 removeChild。
          var moveIn = function (kid, into) {
            if (!kid) return;
            var origPar280 = kid.parentNode;
            var origKids280 = origPar280 && origPar280.childNodes ? origPar280.childNodes : null;
            try { into.appendChild(kid); } catch (_e280a) {}
            try {
              if (origKids280 && origKids280.indexOf(kid) >= 0
                && origPar280 !== into
                && typeof origPar280.removeChild === 'function') {
                origPar280.removeChild(kid);
              }
            } catch (_e280r) {}
          };
          // ⓪ Document sc 尾段：doc 的 [so, ecPathIdx) 直接子 move 入 frag
          //（doc 是 cac 时 middle 段不覆盖此区间——sRef=doc 自身）。
          if (sc.nodeType === 9 && sc === cac) {
            try {
              var dk280 = sc.childNodes || [];
              var decPath280 = -1;
              for (var dk2 = 0; dk2 < dk280.length; dk2++) {
                var danc280 = ec, dah280 = 0;
                while (danc280 && dah280++ < 128) {
                  if (dk280[dk2] === danc280) { decPath280 = dk2; break; }
                  danc280 = danc280.parentNode;
                }
                if (decPath280 >= 0) break;
              }
              var dtailEnd280 = (decPath280 >= 0) ? decPath280 : dk280.length;
              var dsnap280 = dk280.slice(so, dtailEnd280);
              for (var dq280 = 0; dq280 < dsnap280.length; dq280++) {
                moveIn(dsnap280[dq280], f);
              }
            } catch (_e280dc) {}
          }
          // ①' sc 侧路径克隆组树（spec frag 结构 [firstPartial.clone(subtree),
          // contained..., lastPartial.clone(subtree)]——flat 尾段文本首版教训：
          // 20,x frag 首节点期望 <p id="a">全文本</p> 而非裸 #text）。
          // firstClone = sc 的 cac 直接子（P#a 域）shallow clone；sc 尾段文本
          // 与 sc 侧爬升的右侧兄弟按所属层级挂进 clone 链（每级 clone 一层，
          // 内容 = 该级路径子的 [so|0, end) 区间——近似递归 subfrag 的扁平实现）。
          if (scCd) {
            try {
              // sc 尾段切片 + 源削（数据面）。
              var sData = String(sc.data != null ? sc.data : '');
              var sl = sData.length;
              if (so < sl) { try { sc.deleteData(so, sl - so); } catch (_e280d) {} }
              // sc 的 cac 直接子（firstPartial）与逐级路径（P#a ← … ← cac）。
              var fPath = [];
              var walk = sc;
              while (walk && walk.parentNode !== cac && fPath.length < 128) {
                fPath.unshift(walk);
                walk = walk.parentNode;
              }
              var firstPartial = (walk && walk.parentNode === cac) ? walk : null;
              if (firstPartial && firstPartial.nodeType === 1) {
                var cloneStack = [];
                var topClone = firstPartial.cloneNode(false);
                f.appendChild(topClone);
                cloneStack.push(topClone);
                // fPath 末位是 sc 自身（文本端点）——不 clone 该层（tail 文本
                // 就是该层内容，首版教训：多 clone 一层空壳使 P#c=[#text(""),#text("3")]）。
                for (var fp = 0; fp < fPath.length - (sc === fPath[fPath.length - 1] ? 1 : 0); fp++) {
                  var lvlClone = fPath[fp].cloneNode(false);
                  cloneStack[cloneStack.length - 1].appendChild(lvlClone);
                  cloneStack.push(lvlClone);
                }
                // 最内层 clone 承载 sc 尾段文本。
                var tailTxt = sc.cloneNode(false);
                tailTxt.data = sData.slice(so);
                cloneStack[cloneStack.length - 1].appendChild(tailTxt);
                // sc 侧爬升：每级把路径子的右侧兄弟 move 进**同层 clone**。
                for (var lv = fPath.length - 1; lv >= 0; lv--) {
                  var lvlNode = fPath[lv];
                  var lp = lvlNode.parentNode;
                  if (!lp || !lp.childNodes) continue;
                  var pk = lp.childNodes;
                  var pi = -1;
                  for (var pj = 0; pj < pk.length; pj++) if (pk[pj] === lvlNode) { pi = pj; break; }
                  if (pi < 0) continue;
                  // 层内承载：lv 层的右侧兄弟挂进 lv 层的 clone——sc 层（末位，
                  // 无 clone）的兄弟挂最内层 clone（cloneStack 末位）。
                  var hostClone = cloneStack[Math.min(lv + 1, cloneStack.length - 1)];
                  var rsnap = pk.slice(pi + 1);
                  for (var q2 = 0; q2 < rsnap.length; q2++) {
                    moveIn(rsnap[q2], hostClone);
                  }
                }
              } else {
                // firstPartial 非 elements（doc 容器直挂 CD 等）——flat 兜底。
                var tailF = sc.cloneNode(false);
                tailF.data = sData.slice(so);
                f.appendChild(tailF);
              }
            } catch (_e280t) {}
          }
          // ③ sc 是 cac 直接子（element sc）时：sc 自身 partially-contained
          //（本体留树），其 [so, ecPathIdx) 尾段子 move 入 frag（R279 尾部
          // 规则同款——止于 ec 路径子）。
          // R283：**sc 即 cac 的 element-sc 形态**（`cac === sc`，48,x——
          // sc 自身是容器：fully-contained 尾段子（非 ec 路径子）move 入
          // frag；ec 路径子本体留树，其内容由 ④' 的 lastPartial 组树承载）。
          if (!scCd && cac !== sc && sc.parentNode === cac) {
            // R285：**firstPartial（sc 自身）的 clone 引导**——sc 是 cac 的直接子
            // 且 partially-contained（R283 形态 A/B 的共同前置）：frag 以 sc 的
            // shallow clone 开头，其内容 = sc 的 [so, scEcPath) 子区间提取
            //（53,x `[P#d,1,comment,8]`：P#d 的 clone（so=1 越过唯一 text 子 →
            // 空壳）引导 + P#e/P5 中段 + comment 头段——旧版缺引导使 A/E 首节点
            // 错位「expected Element got Text」）。**cac===sc 时无引导**（sc 是
            // 容器自身非 firstPartial——48,x 的 frag 以 contained/middle 开头；
            // 首版全形态引导 -3 回归的教训）。
            // https://dom.spec.whatwg.org/#dom-range-extractcontents
            try {
              var fb285 = sc.cloneNode(false);
              if (fb285) {
                var fbk285 = sc.childNodes || [];
                var fbPath285 = -1;
                for (var fk285 = 0; fk285 < fbk285.length; fk285++) {
                  var fa285 = ec, fh285 = 0;
                  while (fa285 && fh285++ < 128) {
                    if (fbk285[fk285] === fa285) { fbPath285 = fk285; break; }
                    fa285 = fa285.parentNode;
                  }
                  if (fbPath285 >= 0) break;
                }
                var fbEnd285 = (fbPath285 >= 0) ? fbPath285 : fbk285.length;
                var fbSnap285 = fbk285.slice(so, fbEnd285);
                for (var fq285 = 0; fq285 < fbSnap285.length; fq285++) {
                  moveIn(fbSnap285[fq285], fb285);
                }
                f.appendChild(fb285);
              }
            } catch (_e285fb) {}
          }
          if (!scCd && cac === sc) {
            var skR283 = sc.childNodes || [];
            var ecPathR283 = -1;
            for (var kR283 = 0; kR283 < skR283.length; kR283++) {
              var ancR283 = ec, ahR283 = 0;
              while (ancR283 && ahR283++ < 128) {
                if (skR283[kR283] === ancR283) { ecPathR283 = kR283; break; }
                ancR283 = ancR283.parentNode;
              }
              if (ecPathR283 >= 0) break;
            }
            var tailEndR283 = (ecPathR283 >= 0) ? ecPathR283 : skR283.length;
            var tsnapR283 = skR283.slice(so, tailEndR283);
            for (var tR283 = 0; tR283 < tsnapR283.length; tR283++) moveIn(tsnapR283[tR283], f);
          }
          if (!scCd && sc.parentNode === cac) {
            var sk280 = sc.childNodes || [];
            var ecPath280 = -1;
            for (var k280 = 0; k280 < sk280.length; k280++) {
              var anc280 = ec, ah280 = 0;
              while (anc280 && ah280++ < 128) {
                if (sk280[k280] === anc280) { ecPath280 = k280; break; }
                anc280 = anc280.parentNode;
              }
              if (ecPath280 >= 0) break;
            }
            var tailEnd280 = (ecPath280 >= 0) ? ecPath280 : sk280.length;
            var tsnap = sk280.slice(so, tailEnd280);
            for (var t280 = 0; t280 < tsnap.length; t280++) moveIn(tsnap[t280], f);
          }
          // ④ ec 侧 + 中段：ec 头段（CD 切片克隆入 frag + 源 deleteData 保
          // [eo,)；元素 shallow clone + [0,eo) 子 move 入 clone，clone 入 frag）
          // + ec 侧爬升把路径子左侧兄弟 move 入 frag + cac 级 (sIdx,eIdx)
          // 开区间中段 move。同树位（(sc,so)===(ec,0)）只塌缩零提取。
          if (sameTreePos) {
            try { self.setStart(sc, so); self.setEnd(sc, so); } catch (_e280g) {}
            return;
          }
          var elvl = ec, epar = ec.parentNode, hp2 = 0;
          while (epar && elvl !== cac && hp2++ < 128) {
            if (epar === cac) break;
            var ekp = epar.childNodes || [];
            var ei = -1;
            for (var ej = 0; ej < ekp.length; ej++) if (ekp[ej] === elvl) { ei = ej; break; }
            if (ei < 0) break;
            var lsnap = ekp.slice(0, ei);
            for (var q3 = lsnap.length - 1; q3 >= 0; q3--) moveIn(lsnap[q3], f);
            elvl = epar;
            epar = epar.parentNode;
          }
          var ck = cac.childNodes || [];
          var sRef = sc;
          while (sRef && sRef.parentNode !== cac && sRef.parentNode) sRef = sRef.parentNode;
          var eRef = ec;
          while (eRef && eRef.parentNode !== cac && eRef.parentNode) eRef = eRef.parentNode;
          var sIdx = -1, eIdx = -1;
          for (var ck2 = 0; ck2 < ck.length; ck2++) {
            if (sRef && ck[ck2] === sRef) sIdx = ck2;
            if (eRef && ck[ck2] === eRef) eIdx = ck2;
          }
          if (sIdx >= 0 && eIdx > sIdx) {
            var msnap = ck.slice(sIdx + 1, eIdx);
            for (var q4 = 0; q4 < msnap.length; q4++) moveIn(msnap[q4], f);
          }
          // ④' ec 侧路径克隆组树（sc 侧 ①' 的对称——lastPartial.clone 包 ec
          // 头段；20,x frag 尾节点期望 <p id="b"></p> 空壳而非裸 #text）。
          // 数据面：ec 头段切片 + 源削 [0,eo)。
          try {
            var eData = String(ec.data != null ? ec.data : '');
            if (ecCd && eo > 0) { try { ec.deleteData(0, eo); } catch (_e280hd) {} }
            var ePath = [];
            var ewalk = ec;
            while (ewalk && ewalk.parentNode !== cac && ePath.length < 128) {
              ePath.unshift(ewalk);
              ewalk = ewalk.parentNode;
            }
            var lastPartial = (ewalk && ewalk.parentNode === cac) ? ewalk : null;
            if (lastPartial && lastPartial.nodeType === 1) {
              var eStack = [];
              var eTop = lastPartial.cloneNode(false);
              f.appendChild(eTop);
              eStack.push(eTop);
              var eSkip = (ecCd && ec === ePath[ePath.length - 1]) ? 1 : 0;
              for (var ep = 0; ep < ePath.length - eSkip; ep++) {
                var eLvlClone = ePath[ep].cloneNode(false);
                eStack[eStack.length - 1].appendChild(eLvlClone);
                eStack.push(eLvlClone);
              }
              if (ecCd) {
                var headTxt = ec.cloneNode(false);
                headTxt.data = eData.slice(0, eo);
                eStack[eStack.length - 1].appendChild(headTxt);
              } else {
                // ec 元素形态：ec 的 [0, eo) 子 move 进最内层 clone。
                var ek280 = ec.childNodes || [];
                var hsnap = ek280.slice(0, eo);
                for (var h280 = 0; h280 < hsnap.length; h280++) {
                  moveIn(hsnap[h280], eStack[eStack.length - 1]);
                }
              }
              // ec 侧爬升：每级把路径子的左侧兄弟 move 进同层 clone（含本层）。
              for (var elv = ePath.length - 1; elv >= 0; elv--) {
                var eLvlNode = ePath[elv];
                var elp = eLvlNode.parentNode;
                if (!elp || !elp.childNodes) continue;
                var ekp = elp.childNodes;
                var eip = -1;
                for (var epj = 0; epj < ekp.length; epj++) if (ekp[epj] === eLvlNode) { eip = epj; break; }
                if (eip < 0) continue;
                var eHost = eStack[Math.min(elv + 1, eStack.length - 1)];
                var lsnap = ekp.slice(0, eip);
                for (var eq3 = 0; eq3 < lsnap.length; eq3++) {
                  moveIn(lsnap[eq3], eHost);
                }
              }
            } else if (ecCd) {
              var headF = ec.cloneNode(false);
              headF.data = eData.slice(0, eo);
              f.appendChild(headF);
            }
          } catch (_e280h) {}
          // ⑤ 塌缩：cac===sc → (sc,so)（R279 同款）；否则 (cac, sIdx+1)。
          if (cac === sc) {
            try { self.setStart(sc, so); self.setEnd(sc, so); } catch (_e280e) {}
            return;
          }
          try {
            if (sIdx >= 0) { self.setStart(cac, sIdx + 1); self.setEnd(cac, sIdx + 1); }
            else { self.setStart(cac, 0); self.setEnd(cac, 0); }
          } catch (_e280c) {}
        })(this);
        var kids = this._coveredChildren();
        if (kids) {
          for (var i = 0; i < kids.length; i++) {
            try { f.appendChild(kids[i].cloneNode(true)); } catch (_e) {}
          }
          for (var j = kids.length - 1; j >= 0; j--) {
            try { if (typeof kids[j].remove === 'function') kids[j].remove(); } catch (_e) {}
            // R234（js-dom M4）：plain 子摘除后的**无父登记**——spec
            // `dom-range-extract-contents` contained children 是 move 语义（原件
            // 入 fragment、parentNode=frag）；host 是 clone+remove（host 桥 proxy
            // 子不能直接 move），但 plain 子（iframe 子文档克隆树/_zwMEl 产物）
            // remove 后 parentNode 置 null 成为**无根游离树**，harness
            // （Range-extractContents 12–14,x「different number of pieces
            // expected 1 got 2」）把游离原件数作第二棵根树。把摘除原件的
            // parentNode 记到 fragment（与 move 语义的无根判定等价；proxy 子
            // remove 后自身维护 parentNode，不经过此登记）。
            // https://dom.spec.whatwg.org/#dom-range-extractcontents
            try {
              if (kids[j] && typeof kids[j].__zwHandle !== 'string'
                && kids[j].parentNode == null) {
                kids[j].parentNode = f;
              }
            } catch (_eR234p) {}
          }
          this.collapse(true);
        }
        // R234（js-dom M4）：**跨容器提取塌缩**（spec `dom-range-extract-contents`
        // 末步「collapse to start」在移除 contained 后边界同容器化——harness
        // 断言「startContainer and endContainer must always be the same after
        // extractContents()」；WPT Range-{extract,delete,clone}Contents 49,x
        // `[documentElement, 1, document.body, 0]`——动态 documentElement getter
        // （R234 part05）后该形态从空转 no-op 变为真实跨容器提取，旧版容器
        // 保持 (docEl, body) 异侧）。_coveredChildren null（跨容器/文本切片）
        // 时 best-effort 塌缩到 (startContainer, startOffset)。
        // https://dom.spec.whatwg.org/#dom-range-extractcontents
        else if (this.startContainer !== this.endContainer
          && this.startContainer.nodeType === 1
          && this.endContainer.nodeType === 1
          && this.endContainer.parentNode === this.startContainer
          && (this.endOffset | 0) <= (this.endContainer.childNodes
            ? this.endContainer.childNodes.length : 0)
          && (this.startOffset | 0) <= this.startContainer.childNodes.indexOf(this.endContainer)) {
          // R242（js-dom M4）：**sc 元素祖先 + ec 元素直接子 + 双侧 clean 边界**
          //（so 在 sc 的子边界、eo 在 ec 的子边界——`[testDiv,2,paras[4],1]` 形态，
          // common.js myExtractContents 的 ancestor 分支：contained = sc 的
          // [so, ecIdx) 子**本体移入 frag**；last partially contained = ec（元素）
          // → shallow clone 入 frag + 子区间 [ec,0,ec,eo] 递归提取（ec 的 [0,eo)
          // 子移入 clone）；range 塌缩 (sc, so)。旧版该形态 defer 空转（WPT
          // Range-surroundContents 24,x assert_unreached 32F 簇）。中段移动复用
          // R241 的 wrapper 域 move 兜底（append 后原件残留则强制摘除）。
          // https://dom.spec.whatwg.org/#dom-range-extractcontents
          var _r242sc = this.startContainer;
          var _r242ec = this.endContainer;
          var _r242kids = _r242sc.childNodes || [];
          var _r242ecIdx = _r242kids.indexOf(_r242ec);
          var _r242so = this.startOffset | 0;
          var _r242eo = this.endOffset | 0;
          var _r242moveIn = function (kid, into) {
            try { into.appendChild(kid); } catch (_eR242a) {}
            try {
              var still = _r242kids.indexOf(kid);
              if (still >= 0 && typeof _r242sc.removeChild === 'function') {
                _r242sc.removeChild(kid);
              }
            } catch (_eR242r) {}
          };
          if (_r242ecIdx > _r242so) {
            var _r242snap = _r242kids.slice(_r242so, _r242ecIdx);
            for (var _r242i = 0; _r242i < _r242snap.length; _r242i++) {
              if (_r242snap[_r242i]) _r242moveIn(_r242snap[_r242i], f);
            }
          }
          try {
            var _r242clone = _r242ec.cloneNode(false);
            f.appendChild(_r242clone);
            var _r242ekids = _r242ec.childNodes || [];
            if (_r242eo > 0) {
              var _r242esnap = _r242ekids.slice(0, _r242eo);
              for (var _r242j = 0; _r242j < _r242esnap.length; _r242j++) {
                var _r242k = _r242esnap[_r242j];
                if (!_r242k) continue;
                try { _r242clone.appendChild(_r242k); } catch (_eR242c) {}
                try {
                  var still2 = _r242ekids.indexOf(_r242k);
                  if (still2 >= 0 && typeof _r242ec.removeChild === 'function') {
                    _r242ec.removeChild(_r242k);
                  }
                } catch (_eR242r2) {}
              }
            }
          } catch (_eR242cl) {}
          this.setStart(_r242sc, _r242so);
          this.setEnd(_r242sc, _r242so);
        } else if (this.startContainer !== this.endContainer
          && this.startContainer.nodeType === 1
          && (this.endContainer.nodeType === 3 || this.endContainer.nodeType === 4
            || this.endContainer.nodeType === 7 || this.endContainer.nodeType === 8)
          && this.endContainer.parentNode === this.startContainer) {
          // R236（js-dom M4）：**sc 是 ec 的元素祖先容器且 ec 是其直接 CharData 子**
          //（spec `dom-range-extract-contents` 的 ancestor 分支：first partially
          // contained child = null，last partially contained child = ec——common.js
          // myExtractContents 全序对齐：clone ec 的 substringData(0, eo) 入 frag +
          // ec.deleteData(0, eo) 削头，range 塌缩到 (sc, so)）。旧版落 collapse-only
          // 空转使树保留区间原文（WPT Range-surroundContents 23,x
          // `[paras[0],0,paras[0].firstChild,7]` 32F differing 簇——expected 削头
          // "̈efgh\n" got 原文全串）。
          // https://dom.spec.whatwg.org/#dom-range-extractcontents
          var _r236ec = this.endContainer;
          // R240（js-dom M4）：**contained 中段子的移动**（spec
          // `dom-range-extract-contents` ancestor 分支的 containedChildren——
          // common.js myExtractContents 对 [sc,so,ec,eo]（ec 为 sc 直接 CD 子）
          // 除 ec 削头外还把 sc 的 [so, ecIdx) 子**本体移入 frag**。旧版只削头
          // 使中段子残留原树（WPT Range-surroundContents 28,x
          // `[testDiv,0,comment,5]` differing 簇——paras[0..4] 应入 frag）。
          // https://dom.spec.whatwg.org/#dom-range-extractcontents
          var _r236scNode = this.startContainer;
          var _r236kids = _r236scNode.childNodes || [];
          var _r236ecIdx = _r236kids.indexOf(_r236ec);
          var _r236so0 = this.startOffset | 0;
          if (_r236ecIdx > _r236so0) {
            // R240 修正：**快照后移动**——appendChild 把子移入 frag 时 sc的
            // childNodes 数组同步收缩，按下标迭代会滑位（首版 ecIdx 失效把 ec
            // 本体也移入 frag——探针 ex-frag=[P,"oup?","bet s"] 实证）。
            var _r240snap = _r236kids.slice(_r236so0, _r236ecIdx);
            for (var _r240i = 0; _r240i < _r240snap.length; _r240i++) {
              var _r240k = _r240snap[_r240i];
              if (!_r240k) continue;
              try { f.appendChild(_r240k); } catch (_eR240m) {}
              // R241（js-dom M4）：**本体未离场时强制摘除**——WPT iframe 的
              // wrapper 域子（setupRangeTests 经 querySelector("#test") 返
              // wrapper，append 落 wrapper 列表）对 frag 的 appendChild 是
              // clone 语义而非 move，原件残留 sc 使树出现**双份**（R241-probe
              // 实证 DIV=[newParent[拷贝…], 原件…]）。append 后若原件仍在
              // sc.childNodes 则 removeChild 强制离场（move 语义兜底）。
              // https://dom.spec.whatwg.org/#dom-range-extractcontents
              try {
                var _r241still = _r236kids.indexOf(_r240k);
                if (_r241still >= 0 && typeof _r236scNode.removeChild === 'function') {
                  _r236scNode.removeChild(_r240k);
                }
              } catch (_eR241r) {}
            }
          }
          var _r236eo = Math.max(0, Math.min(this.endOffset | 0,
            String(_r236ec.data != null ? _r236ec.data : '').length));
          if (_r236eo > 0) {
            try {
              var _r236head = _r236ec.cloneNode(false);
              _r236head.data = String(_r236ec.data).slice(0, _r236eo);
              f.appendChild(_r236head);
            } catch (_eR236c) {}
            try {
              if (typeof _r236ec.deleteData === 'function') {
                _r236ec.deleteData(0, _r236eo);
              } else {
                _r236ec.data = String(_r236ec.data).slice(_r236eo);
              }
            } catch (_eR236d) {}
          }
          var _r236so = this.startOffset | 0;
          this.setStart(this.startContainer, _r236so);
          this.setEnd(this.startContainer, _r236so);
        } else {
          this.collapse(true);
        }
        return f;
      },
      cloneContents: function () {
        // R2929：真实子树克隆（cloneNode deep）到 fragment。元素容器 + offset 区间精确；
        // 跨容器/文本节点容器回落文本（既有 best-effort）。
        // R284：frag 归 start 节点 ownerDocument 域（同 extractContents——spec
        // `dom-range-clone-contents` 同款步骤 1）。
        var _r284od2 = null;
        try { _r284od2 = this.startContainer.ownerDocument || (this.startContainer.nodeType === 9 ? this.startContainer : null); } catch (_e284od2) {}
        var f = (_r284od2 && typeof _r284od2.createDocumentFragment === 'function')
          ? _r284od2.createDocumentFragment()
          : globalThis.document.createDocumentFragment();
        // R281（js-dom M4）：**跨容器 clone 的路径克隆组树**（R280 extract 同款
        // 结构的纯 clone 版——无 move 无删源；WPT Range-cloneContents 29F 的
        // 主簇「Returned fragment」expected `<p id="a">full-text</p>` vs got 裸
        // #text / __n 句柄——旧 `_coveredChildren` sc≠ec 恒 null → toString
        // 文本回落）。frag = [firstPartial.clone(sc 侧子树切片), contained 中段
        // deep clone, lastPartial.clone(ec 侧子树切片)]，spec
        // `dom-range-clone-contents` 与 common.js myCloneContents 同构。
        // https://dom.spec.whatwg.org/#dom-range-clonecontents
        (function _r281CrossClone(self) {
          var sc = self.startContainer, ec = self.endContainer;
          if (!sc || !ec) return;
          var isCd281 = function (n) {
            var t = n ? (n.nodeType | 0) : 0;
            return t === 3 || t === 4 || t === 7 || t === 8;
          };
          // R281b：**同节点 CharData 的 clone 切片**（spec `dom-range-clone-contents`
          // 首分支——frag = clone + substringData [so,eo)；WPT 27/35/36/37/39,x 的
          // comment/PI 同节点簇：旧版回落 toString 文本 frag）。
          if (sc === ec && isCd281(sc)) {
            try {
              var sd281 = String(sc.data != null ? sc.data : '');
              var sa281 = Math.max(0, Math.min(self.startOffset | 0, sd281.length));
              var sb281 = Math.max(sa281, Math.min(self.endOffset | 0, sd281.length));
              // 空切片（collapsed / 零宽）→ 空 frag（首版教训：空 #text 克隆
              // 使 0/4/8/56-59,x 的 collapsed 文本族整簇翻红——spec 返回空 frag）。
              if (sb281 > sa281) {
                var sn281 = cl281(sc);
                if (sn281) {
                  try { sn281.data = sd281.slice(sa281, sb281); } catch (_e281s) {}
                  f.appendChild(sn281);
                }
              }
            } catch (_e281t) {}
            return;
          }
          // R281b：**doctype contained 抛 HRE**（spec 步骤「If a contained child is
          // a DocumentType, throw」——WPT 25/26,x `[document,0,document,1/2]` 族：
          // 期望 HierarchyRequestError；旧版静默返回 frag）。
          if (sc.nodeType === 9 || sc === ec) {
            try {
              var dk281b = (sc === ec) ? (sc.childNodes || []) : (sc.childNodes || []);
              var dso281 = self.startOffset | 0, deo281 = self.endOffset | 0;
              for (var di281 = dso281; sc === ec && di281 < Math.min(deo281, dk281b.length); di281++) {
                if (dk281b[di281] && dk281b[di281].nodeType === 10) {
                  throw new (globalThis.DOMException || Error)(
                    'The range includes a DocumentType node.', 'HierarchyRequestError');
                }
              }
            } catch (e281hre) {
              if (e281hre && (e281hre.name === 'HierarchyRequestError' || (globalThis.DOMException && e281hre instanceof globalThis.DOMException))) {
                throw e281hre;
              }
            }
          }
          function cl281(n) {
            try { return n.cloneNode(false); } catch (_e) { return null; }
          }
          var isCd = function (n) {
            var t = n ? (n.nodeType | 0) : 0;
            return t === 3 || t === 4 || t === 7 || t === 8;
          };
          var scCd = isCd(sc), ecCd = isCd(ec);
          var scOk = scCd || sc.nodeType === 1 || sc.nodeType === 11 || sc.nodeType === 9;
          var ecOk = ecCd || ec.nodeType === 1 || ec.nodeType === 11;
          if (!scOk || !ecOk) return;
          // R287：doc sc 的 parentNode 恒 null 是合法形态（R282 在 extract 侧的
          // 同款修正——clone 侧漏对称移植使 29/31,x 的 doc-sc 路径从未执行）。
          var scParOk287 = (sc.nodeType === 9) || !!sc.parentNode;
          if (!scParOk287 || !ec.parentNode) return;
          // 同容器形态留给 _coveredChildren 既有路径（含 doc 容器 [so,eo)）。
          if (sc === ec) {
            if (scCd) return;
            return;
          }
          // cac：sc 祖先链上首个含 ec 的容器（R268 同款）。
          var chain = [], cur = sc, hops = 0;
          while (cur && hops++ < 128) { chain.push(cur); cur = cur.parentNode; }
          var cac = null;
          for (var ci = 0; ci < chain.length && !cac; ci++) {
            var probe = ec, h2 = 0;
            while (probe && h2++ < 128) {
              if (probe === chain[ci]) { cac = chain[ci]; break; }
              probe = probe.parentNode;
            }
          }
          if (!cac) return;
          var so = self.startOffset | 0, eo = self.endOffset | 0;
          var cl = function (n) {
            try { return n.cloneNode(false); } catch (_e) { return null; }
          };
          // sc 侧：firstPartial.clone + 路径层 clone + sc 尾段文本切片（CD）或
          // sc 元素的 [so, ecPathIdx) 子区间（element sc 尾部规则——R279 同款）。
          var sPath = [];
          var swalk = sc;
          while (swalk && swalk.parentNode !== cac && sPath.length < 128) {
            sPath.unshift(swalk);
            swalk = swalk.parentNode;
          }
          var firstPartial = (swalk && swalk.parentNode === cac) ? swalk : null;
          if (scCd) {
            if (firstPartial && firstPartial.nodeType === 1) {
              var sStack = [];
              var sTop = cl(firstPartial);
              if (sTop) {
                f.appendChild(sTop);
                sStack.push(sTop);
                var sSkip = (sc === sPath[sPath.length - 1]) ? 1 : 0;
                for (var sp = 0; sp < sPath.length - sSkip; sp++) {
                  var sLvl = cl(sPath[sp]);
                  if (!sLvl) { sStack = null; break; }
                  sStack[sStack.length - 1].appendChild(sLvl);
                  sStack.push(sLvl);
                }
                if (sStack) {
                  var sTxt = cl(sc);
                  if (sTxt) {
                    try { sTxt.data = String(sc.data != null ? sc.data : '').slice(so); } catch (_e281a) {}
                    sStack[sStack.length - 1].appendChild(sTxt);
                  }
                  // sc 侧爬升：每级路径子的右侧兄弟 deep clone 进同层。
                  for (var slv = sPath.length - 1; slv >= 0; slv--) {
                    var sLvlNode = sPath[slv];
                    var slp = sLvlNode.parentNode;
                    if (!slp || !slp.childNodes) continue;
                    var spk = slp.childNodes;
                    var spi = -1;
                    for (var spj = 0; spj < spk.length; spj++) if (spk[spj] === sLvlNode) { spi = spj; break; }
                    if (spi < 0) continue;
                    var sHost = sStack[Math.min(slv + 1, sStack.length - 1)];
                    for (var sq = spi + 1; sq < spk.length; sq++) {
                      try { sHost.appendChild(spk[sq].cloneNode(true)); } catch (_e281b) {}
                    }
                  }
                }
              }
            } else {
              var sFlat = cl(sc);
              if (sFlat) {
                try { sFlat.data = String(sc.data != null ? sc.data : '').slice(so); } catch (_e281c) {}
                f.appendChild(sFlat);
              }
            }
          } else if (sc.nodeType === 1 || sc.nodeType === 11) {
            // R285：**sc 是 cac 直接子的 clone 引导**（extract 侧同款对称——
            // 53,x clone 的 [P#d-empty-clone, P#e, P5, comment-head] 首节点）：
            // sc 的 shallow clone 引导 frag，其内承载 [so, scEcPath) 子 deep clone。
            if (sc !== cac && sc.parentNode === cac) {
              try {
                var cb285 = cl(sc);
                if (cb285) {
                  var cbk285 = sc.childNodes || [];
                  var cbPath285 = -1;
                  for (var cf285 = 0; cf285 < cbk285.length; cf285++) {
                    var ca285 = ec, ch285 = 0;
                    while (ca285 && ch285++ < 128) {
                      if (cbk285[cf285] === ca285) { cbPath285 = cf285; break; }
                      ca285 = ca285.parentNode;
                    }
                    if (cbPath285 >= 0) break;
                  }
                  var cbEnd285 = (cbPath285 >= 0) ? cbPath285 : cbk285.length;
                  var cbSnap285 = cbk285.slice(so, cbEnd285);
                  for (var cq285 = 0; cq285 < cbSnap285.length; cq285++) {
                    try { cb285.appendChild(cbSnap285[cq285].cloneNode(true)); } catch (_e285cb) {}
                  }
                  f.appendChild(cb285);
                }
              } catch (_e285cbs) {}
            }
            // element sc：本体不动，[so, ecPathIdx) 子 deep clone 直接入 frag
            //（sc 是 cac 时；sc 深于 cac 时由中段/爬升覆盖——保守只接 sc===cac）。
            if (sc === cac) {
              var sk281 = sc.childNodes || [];
              var secPath281 = -1;
              for (var sk2 = 0; sk2 < sk281.length; sk2++) {
                var sanc281 = ec, sah281 = 0;
                while (sanc281 && sah281++ < 128) {
                  if (sk281[sk2] === sanc281) { secPath281 = sk2; break; }
                  sanc281 = sanc281.parentNode;
                }
                if (secPath281 >= 0) break;
              }
              var stailEnd281 = (secPath281 >= 0) ? secPath281 : sk281.length;
              var ssnap281 = sk281.slice(so, stailEnd281);
              for (var sq2 = 0; sq2 < ssnap281.length; sq2++) {
                try { f.appendChild(ssnap281[sq2].cloneNode(true)); } catch (_e281d) {}
              }
            }
          } else if (sc.nodeType === 9 && sc === cac) {
            // doc sc：[so, ecPathIdx) 子 deep clone。
            var dk281 = sc.childNodes || [];
            var decPath281 = -1;
            for (var dk2b = 0; dk2b < dk281.length; dk2b++) {
              var danc281b = ec, dah281b = 0;
              while (danc281b && dah281b++ < 128) {
                if (dk281[dk2b] === danc281b) { decPath281 = dk2b; break; }
                danc281b = danc281b.parentNode;
              }
              if (decPath281 >= 0) break;
            }
            var dtailEnd281 = (decPath281 >= 0) ? decPath281 : dk281.length;
            var dsnap281 = dk281.slice(so, dtailEnd281);
            for (var dq281 = 0; dq281 < dsnap281.length; dq281++) {
              try { f.appendChild(dsnap281[dq281].cloneNode(true)); } catch (_e281e) {}
            }
          }
          // 中段：cac 级 (sIdx, eIdx) 开区间 deep clone。
          var ck = cac.childNodes || [];
          var sRef = sc;
          while (sRef && sRef.parentNode !== cac && sRef.parentNode) sRef = sRef.parentNode;
          var eRef = ec;
          while (eRef && eRef.parentNode !== cac && eRef.parentNode) eRef = eRef.parentNode;
          var sIdx = -1, eIdx = -1;
          for (var ck2 = 0; ck2 < ck.length; ck2++) {
            if (sRef && ck[ck2] === sRef) sIdx = ck2;
            if (eRef && ck[ck2] === eRef) eIdx = ck2;
          }
          if (sIdx >= 0 && eIdx > sIdx) {
            var msnap281 = ck.slice(sIdx + 1, eIdx);
            for (var mq = 0; mq < msnap281.length; mq++) {
              try { f.appendChild(msnap281[mq].cloneNode(true)); } catch (_e281f) {}
            }
          }
          // ec 侧：lastPartial.clone + 路径层 + ec 头段切片（CD）/ [0,eo) 子 clone
          //（element ec）；ec 侧爬升左侧兄弟 deep clone 进同层。
          var ePath = [];
          var ewalk = ec;
          while (ewalk && ewalk.parentNode !== cac && ePath.length < 128) {
            ePath.unshift(ewalk);
            ewalk = ewalk.parentNode;
          }
          var lastPartial = (ewalk && ewalk.parentNode === cac) ? ewalk : null;
          if (lastPartial && lastPartial.nodeType === 1) {
            var eStack = [];
            var eTop = cl(lastPartial);
            if (eTop) {
              f.appendChild(eTop);
              eStack.push(eTop);
              var eSkip = (ecCd && ec === ePath[ePath.length - 1]) ? 1 : 0;
              for (var ep = 0; ep < ePath.length - eSkip; ep++) {
                var eLvl = cl(ePath[ep]);
                if (!eLvl) { eStack = null; break; }
                eStack[eStack.length - 1].appendChild(eLvl);
                eStack.push(eLvl);
              }
              if (eStack) {
                if (ecCd) {
                  var eTxt = cl(ec);
                  if (eTxt) {
                    try { eTxt.data = String(ec.data != null ? ec.data : '').slice(0, eo); } catch (_e281g) {}
                    eStack[eStack.length - 1].appendChild(eTxt);
                  }
                } else {
                  var ek281 = ec.childNodes || [];
                  var esnap281 = ek281.slice(0, eo);
                  for (var eq2 = 0; eq2 < esnap281.length; eq2++) {
                    try { eStack[eStack.length - 1].appendChild(esnap281[eq2].cloneNode(true)); } catch (_e281h) {}
                  }
                }
                for (var elv = ePath.length - 1; elv >= 0; elv--) {
                  var eLvlNode = ePath[elv];
                  var elp = eLvlNode.parentNode;
                  if (!elp || !elp.childNodes) continue;
                  var ekp = elp.childNodes;
                  var eip = -1;
                  for (var epj = 0; epj < ekp.length; epj++) if (ekp[epj] === eLvlNode) { eip = epj; break; }
                  if (eip < 0) continue;
                  var eHost = eStack[Math.min(elv + 1, eStack.length - 1)];
                  for (var eq3 = 0; eq3 < eip; eq3++) {
                    try { eHost.appendChild(ekp[eq3].cloneNode(true)); } catch (_e281i) {}
                  }
                }
              }
            }
          } else if (ecCd) {
            var eFlat = cl(ec);
            if (eFlat) {
              try { eFlat.data = String(ec.data != null ? ec.data : '').slice(0, eo); } catch (_e281j) {}
              f.appendChild(eFlat);
            }
          }
        })(this);
        var kids = this._coveredChildren();
        if (kids) {
          for (var i = 0; i < kids.length; i++) {
            try { f.appendChild(kids[i].cloneNode(true)); } catch (_e) {}
          }
        } else {
          var t = this.toString();
          if (t && !f.childNodes.length) f.appendChild(globalThis.document.createTextNode(t));
        }
        return f;
      },
      insertNode: function (node) {
        // 在 startContainer 的 startOffset 位置插入 node（created 节点）。off < 子数 → insertBefore(ref)，否则
        // appendChild。复用既有 insertBefore/appendChild（emit mutation）。返回 node（spec）。
        // R178（js-dom M4）：spec `dom-range-insertnode` 步骤 2-3——Attr 不是合法
        // 插入父（HierarchyRequestError）/ Attr 不能作被插内容（WPT Range-attribute-
        // nodes 的 insertNode 两形态——旧静默 no-op / 吞错不抛）。
        if (node === null || node === undefined || typeof node.nodeType !== 'number') {
          throw new globalThis.TypeError(
            "Failed to execute 'insertNode' on 'Range': parameter 1 is not of type 'Node'.");
        }
        if (node.nodeType === 2 || this.startContainer.nodeType === 2) {
          throw new (globalThis.DOMException || Error)(
            'Nodes of type 2 cannot be inserted or inserted into.', 'HierarchyRequestError');
        }
        // R209（js-dom M4）：spec `dom-range-insertnode` 步骤 1 的「startContainer
        // 是 node 自身」分支——HierarchyRequestError，树不变（common.js myInsertNode
        // 同款首查；WPT mega-case "node is startContainer" 族）。其余分支（PI/
        // Comment 容器、无父 Text）对 host proxy 形态误伤面大（parentNode getter
        // 形态差异），只收此分支。
        // https://dom.spec.whatwg.org/#dom-range-insertnode
        if (this.startContainer === node) {
          throw new (globalThis.DOMException || Error)(
            'The Range object is invalid.', 'HierarchyRequestError');
        }
        // R215（js-dom M4）：**ensure-pre-insertion validity 前置**（spec
        // `dom-node-pre-insert` 校验族，common.js ensurePreInsertionValidity 同款
        // ——insertNode 336F HRE 簇 + 8,9/9,9 的 P→DIV→P 循环根因：旧版 splitText
        // 路径无校验直接 insertBefore，把**插入目标自身的祖先**插进目标形成
        // parentNode 环（upwalk 探针 P→DIV→P→DIV… 101 hops 实证），后续 sim 的
        // isInclusiveAncestor 上行 walk 栈溢出）。校验四件：
        // ① parent 非 Element/Document/DocumentFragment → HRE
        // ② node 是 parent 的 host-including inclusive ancestor → HRE
        // ③ Text 入 Document / Doctype 入非 Document → HRE
        // https://dom.spec.whatwg.org/#concept-node-ensure-pre-insertion-validity
        (function _r215Validate(self, node215) {
          if (globalThis._r215NoValidate) return; // surround 叶子路径的先变更后抛序（R215）
          var sc215 = self.startContainer;
          // referenceNode：Text 容器 → 自身；否则 childNodes[startOffset]。
          var ref215 = null;
          if (sc215.nodeType === 3 || sc215.nodeType === 4) ref215 = sc215;
          else if (sc215.childNodes && (self.startOffset | 0) < sc215.childNodes.length) {
            ref215 = sc215.childNodes[self.startOffset | 0];
          }
          var parent215 = ref215 === null ? sc215 : (ref215.parentNode || sc215);
          if (parent215.nodeType !== 1 && parent215.nodeType !== 9
            && parent215.nodeType !== 11) {
            throw new (globalThis.DOMException || Error)(
              'Nodes of type ' + parent215.nodeType + ' cannot have children.',
              'HierarchyRequestError');
          }
          // R224（js-dom M4）：**node 自身类型合法性**（spec
          // `concept-node-ensure-pre-insertion-validity` 步骤「If node is not a
          // DocumentFragment, DocumentType, Element, or CharacterData node, throw
          // HierarchyRequestError」——common.js ensurePreInsertionValidity 同款第
          // 四查）。旧版缺此查使 Document（nt=9，xmlDoc/foreignDoc/document 作
          // node）插入静默成功——sim 返 HRE 而 host 不抛（WPT
          // Range-insertNode「A HIERARCHY_REQUEST_ERR must be thrown」71F 簇：
          // foreignDoc 27 / xmlDoc 31 / document 13）。Attr（2）同拒。
          // https://dom.spec.whatwg.org/#concept-node-ensure-pre-insertion-validity
          var _r224nt = node215.nodeType | 0;
          if (_r224nt !== 11 && _r224nt !== 10 && _r224nt !== 1
            && _r224nt !== 3 && _r224nt !== 4 && _r224nt !== 7
            && _r224nt !== 8) {
            throw new (globalThis.DOMException || Error)(
              'Nodes of type ' + _r224nt + ' cannot be inserted.',
              'HierarchyRequestError');
          }
          // ② node 自身或后代 === parent → 环（guard 128 防既有环失控）。
          var cur215 = parent215, hops215 = 0;
          while (cur215 && hops215++ < 128) {
            if (cur215 === node215) {
              throw new (globalThis.DOMException || Error)(
                'The new child element contains the parent.',
                'HierarchyRequestError');
            }
            cur215 = cur215.parentNode;
          }
          if ((node215.nodeType === 3 || node215.nodeType === 4)
            && parent215.nodeType === 9) {
            throw new (globalThis.DOMException || Error)(
              'Nodes of type ' + node215.nodeType + ' cannot be inserted into a Document.',
              'HierarchyRequestError');
          }
          if (node215.nodeType === 10 && parent215.nodeType !== 9) {
            throw new (globalThis.DOMException || Error)(
              'Nodes of type 10 cannot be inserted into a non-Document.',
              'HierarchyRequestError');
          }
          // R217（js-dom M4）：**Document 子位置规则**（spec
          // `dom-node-pre-insert` 的「If parent is a Document」四分支——WPT
          // Range-insertNode 25,x 族：element 入已有 element 子的 Document →
          // HRE；frag 多 element 子 / Text 子 → HRE；doctype 位序（element 前
          // 有 doctype 期望 / doctype 入已有 doctype）→ HRE。common.js
          // ensurePreInsertionValidity 的 switch 同款）。
          // https://dom.spec.whatwg.org/#concept-node-ensure-pre-insertion-validity
          if (parent215.nodeType === 9) {
            var isEl217 = function (x) { return !!x && x.nodeType === 1; };
            var isDt217 = function (x) { return !!x && x.nodeType === 10; };
            var k217 = parent215.childNodes || [];
            var elCount217 = 0, dtCount217 = 0;
            for (var c217 = 0; c217 < k217.length; c217++) {
              if (isEl217(k217[c217])) elCount217++;
              if (isDt217(k217[c217])) dtCount217++;
            }
            var hre217 = function (m) {
              throw new (globalThis.DOMException || Error)(m, 'HierarchyRequestError');
            };
            if (node215.nodeType === 11) {
              var fk217 = node215.childNodes || [];
              var fEl217 = 0, fTxt217 = 0;
              for (var f217 = 0; f217 < fk217.length; f217++) {
                if (isEl217(fk217[f217])) fEl217++;
                if (fk217[f217] && (fk217[f217].nodeType === 3 || fk217[f217].nodeType === 4)) fTxt217++;
              }
              if (fEl217 > 1) hre217('Fragment has more than one element child.');
              if (fTxt217 > 0) hre217('Fragment has a Text child.');
              if (fEl217 === 1) {
                if (elCount217 > 0) hre217('Document already has an element child.');
                if (ref215 && isDt217(ref215)) hre217('Insertion point is before the doctype.');
                // child 后有 doctype → HRE（ref215 之后存在 doctype）
                if (ref215) {
                  var ri217 = k217.indexOf(ref215);
                  for (var a217 = ri217 + 1; a217 < k217.length; a217++) {
                    if (isDt217(k217[a217])) hre217('Doctype follows the insertion point.');
                  }
                }
              }
            } else if (node215.nodeType === 1) {
              if (elCount217 > 0) hre217('Document already has an element child.');
              if (ref215 && isDt217(ref215)) hre217('Insertion point is before the doctype.');
              if (ref215) {
                var ri2217 = k217.indexOf(ref215);
                for (var b217 = ri2217 + 1; b217 < k217.length; b217++) {
                  if (isDt217(k217[b217])) hre217('Doctype follows the insertion point.');
                }
              }
            } else if (node215.nodeType === 10) {
              if (dtCount217 > 0) hre217('Document already has a doctype child.');
              if (ref215) {
                var ri2317 = k217.indexOf(ref215);
                for (var d217 = 0; d217 < ri2317; d217++) {
                  if (isEl217(k217[d217])) hre217('Element child precedes the insertion point.');
                }
              }
              if (!ref215 && elCount217 > 0) hre217('Document has an element child and no insertion point.');
            }
          }
        })(this, node);
        if (!node || !this.startContainer) return node;
        // R209（js-dom M4）：spec `dom-range-insertnode`——startContainer 是 Text/
        // CDATA 时先 splitText(startOffset)（原节点保前半、尾半为新节点在父内），
        // 再把 node 插到**尾节点之前**（WPT mega-case 的 common.js myInsertNode 模拟
        // 同款——surroundContents 折叠路径的树中间态与此对齐）。工厂形态文本的
        // splitText 由 R209 补齐（part05/parte03）；host proxy 文本经 part04 get trap。
        // https://dom.spec.whatwg.org/#dom-range-insertnode
        var sc209 = this.startContainer;
        if ((sc209.nodeType === 3 || sc209.nodeType === 4)
          && typeof sc209.splitText === 'function') {
          var tail209 = null;
          try { tail209 = sc209.splitText(this.startOffset | 0); } catch (_eR209sp) {}
          var self209 = this;
          // R209：末步 range.end 同步（spec 步骤「If range's start and end are the
          // same, set range's end to (parent, newOffset)」——common.js myInsertNode
          // 的 range.setEnd(parent_, newOffset) 同款；newOffset = node 插入后索引 + 1）。
          var syncEnd209 = function () {
            if (self209.startContainer !== self209.endContainer
              || self209.startOffset !== self209.endOffset) return;
            var p = tail209 && tail209.parentNode;
            if (!p || !p.childNodes) return;
            // R225（js-dom M4）：**handle-aware identity**——父 childNodes 视图元素是
            // `_wrapHandle` 包装 proxy，与插入时的 raw 节点对象不同 identity（旧版
            // `=== node` 对 handle 形态恒 miss → collapsed 插入后 end 不同步，WPT
            // Range-insertNode 0/4/8/10/15,20 的 endOffset 族）。按 `__zwHandle`
            // 相等判节点同一。
            var _r225Same = function (a, b) {
              if (a === b) return true;
              try {
                return !!(a && b && a.__zwHandle && a.__zwHandle === b.__zwHandle);
              } catch (_e225s) { return false; }
            };
            var idx = -1;
            for (var q = 0; q < p.childNodes.length; q++) {
              if (_r225Same(p.childNodes[q], node)) { idx = q; break; }
            }
            // R225（js-dom M4）：node 是 fragment 时已展平（本体不在父内）——按 sim
            // myInsertNode 的 newOffset 语义取 `indexOf(tail) + nodeLength(node)`：
            // 空 df → tail 索引（同族 endOffset expected 1 got 2/0——旧版 idx=-1
            // 不同步或按本体索引多 1）。
            if (idx < 0 && node && node.nodeType === 11) {
              var ti225 = -1;
              for (var q225 = 0; q225 < p.childNodes.length; q225++) {
                if (_r225Same(p.childNodes[q225], tail209)) { ti225 = q225; break; }
              }
              if (ti225 >= 0) idx = ti225 + (node.childNodes ? node.childNodes.length : 0) - 1;
            }
            if (idx >= 0) {
              try { self209.setEnd(p, idx + 1); } catch (_eR209se) {}
            }
          };
          if (tail209 && tail209.parentNode
            && typeof tail209.parentNode.insertBefore === 'function') {
            try { tail209.parentNode.insertBefore(node, tail209); } catch (_eR209ib) {}
            syncEnd209();
            return node;
          }
          if (tail209 && tail209.parentNode
            && typeof tail209.parentNode.appendChild === 'function') {
            try { tail209.parentNode.appendChild(node); } catch (_eR209ab) {}
            syncEnd209();
            return node;
          }
        }
        try {
          var kids = this.startContainer.childNodes;
          var off = this.startOffset | 0;
          if (kids && off < kids.length && kids[off]) {
            this.startContainer.insertBefore(node, kids[off]);
          } else {
            this.startContainer.appendChild(node);
          }
          // R219（js-dom M4）：spec `dom-range-insertnode` 末步——「If range's start
          // and end are the same, set range's end to (parent, newOffset)」（与上方
          // Text 分支的 syncEnd209 同款；common.js myInsertNode 的
          // `range.setEnd(parent_, newOffset)`）。newOffset = 插入后 node 在父内的
          // 索引 + 1（fragment 按其子长度计——此处插入已展平，取实际索引即可）。
          // WPT Range-insertNode 15,x「resulting range position」的 endOffset
          // expected 2 got 1 簇（element 容器 collapsed 插入后 end 未同步）。
          // https://dom.spec.whatwg.org/#dom-range-insertnode
          if (this.startContainer === this.endContainer
            && this.startOffset === this.endOffset) {
            var _r219p = node.parentNode || this.startContainer;
            var _r219kids = _r219p.childNodes;
            if (_r219kids && typeof _r219kids.indexOf === 'function') {
              var _r219ni = _r219kids.indexOf(node);
              if (_r219ni >= 0) {
                try { this.setEnd(_r219p, _r219ni + 1); } catch (_eR219se) {}
              }
            }
          }
        } catch (_e) {}
        return node;
      },
      surroundContents: function (newParent) {
        // R2930：spec 把范围内容「提取进 newParent」再「把 newParent 插到范围原位」——rich-text wrap 高频
        //（如把选区包进 <b>）。实现（避 stale-ref apply 失败 + nth-child 选择器 sibling 前移失效）：
        // ① 正序 clone 覆盖子进 newParent；② 逆序 remove 覆盖原件（nth-child 逆序稳定）；③ appendChild newParent。
        // **精确落位仅在覆盖块延伸到容器末尾时**（selectNodeContents 包整元素内容——headline 用法）：覆盖块为
        // 容器尾部，逆序移除后 newParent appendChild 即落原位。非尾部（覆盖块后有兄弟）→ newParent 落容器末尾
        //（位置近似）：非尾部精确插位须 id-stable ref 或 host 回调（nth-child 经移除前移失效），defer。
        // collapsed（0 覆盖子）→ insertNode(newParent)。跨容器/文本切片 → best-effort no-op（defer）。
        if (!newParent || !this.startContainer) return;
        // R239（js-dom M4）：**部分包含检查先于 newParent 类型检查**——common.js
        // mySurroundContents 的序（步骤 2 partial → INVALID_STATE 在步骤 1
        // nodeType → INVALID_NODE_TYPE 之前执行；WPT 20–22,x/29/31,x 的
        // document/foreignDoc/xmlDoc/docfrag/doctype 作 newParent 期望
        // INVALID_STATE_ERR——host 旧序先抛 InvalidNodeTypeError 30F 簇）。
        // R210 注释原文如下。
        // R210（js-dom M4）：spec `dom-range-surroundcontents` 步骤 2——「If a
        // non-Text node is partially contained in the context object, throw
        // InvalidStateError」。部分包含 = 是 start 或 end 边界容器的祖先但非双方
        // 共同祖先（common.js isPartiallyContained 同款）。cac 子树内非 Text 节点
        // 逐个检查（探针 20,x 族 115F：cac=DIV 正确但 host 不抛 → assert_throws_dom
        // "did not throw"）。
        // https://dom.spec.whatwg.org/#dom-range-surroundcontents
        (function _r210PartialCheck(self) {
          var ancIn = function (a, d) {
            while (d) { if (d === a) return true; d = d.parentNode; }
            return false;
          };
          var partial = function (n) {
            var c1 = ancIn(n, self.startContainer);
            var c2 = ancIn(n, self.endContainer);
            return (c1 && !c2) || (c2 && !c1);
          };
          var cac = self.commonAncestorContainer;
          if (!cac || !cac.childNodes) return;
          // R239（js-dom M4）：**nextNode 序遍历**（common.js mySurroundContents 的
          // `for (node = cac; node != stop; node = nextNode(node))` 同款）——旧 DFS
          // 全覆盖比 sim 的 nextNode 遍历**更完备**：sim 在 sibling getter 断链处
          // 提前终止（24,x 的 paras[4] 未被扫到 → WPT 期望 INVALID_NODE_TYPE 而
          // host DFS 命中 partial 抛 INVALID_STATE，12F 反向翻转）。遍历原语与
          // sim 一致（hasChildNodes→firstChild；否则沿 parentNode 爬到有
          // nextSibling 的祖先取 nextSibling），盲区与 sim 对齐。
          var stop239 = (function () {
            var n = cac, h = 0;
            while (n && !n.nextSibling && h++ < 128) n = n.parentNode;
            return n && n.nextSibling ? n.nextSibling : null;
          })();
          var node239 = cac, guard239 = 0;
          while (node239 && node239 !== stop239 && guard239++ < 2048) {
            if (node239.nodeType !== 3 && node239.nodeType !== 4 && partial(node239)) {
              throw new (globalThis.DOMException || Error)(
                'The Range has partially selected a non-Text node.', 'InvalidStateError');
            }
            var nx = null;
            if (typeof node239.hasChildNodes === 'function' && node239.hasChildNodes()) {
              nx = node239.firstChild;
            } else {
              var cn = node239, hh = 0;
              while (cn && !cn.nextSibling && hh++ < 128) cn = cn.parentNode;
              nx = cn && cn.nextSibling ? cn.nextSibling : null;
            }
            node239 = nx;
          }
        })(this);
        // R209（js-dom M4）：spec `dom-range-surroundcontents` 步骤 1——newParent 是
        // Document/DocumentType/DocumentFragment 抛 InvalidNodeTypeError（WPT
        // mega-case 的 INVALID_NODE_TYPE_ERR 簇）。
        // 注：Text/Comment/PI newParent 的 HRE 发生在**步骤 5**（appendChild(fragment)
        // 到叶子节点）——extract/insert 先行部分变更树（与 common.js mySurroundContents
        // 模拟的中间态一致），故此处不提前抛（提前抛使 positionTests 的树比较与
        // 模拟侧中间态分歧）。
        // https://dom.spec.whatwg.org/#dom-range-surroundcontents
        // R212（js-dom M4）：补 nodeType 11（DocumentFragment）——spec 步骤 1 的
        // 三类型之一，旧版漏检使 docfrag newParent 走到 CharData 路径实际变更树
        //（模拟侧 InvalidNodeTypeError 树不变——,20 族 positionTests 48F 根因）。
        if (newParent.nodeType === 9 || newParent.nodeType === 10
          || newParent.nodeType === 11) {
          throw new (globalThis.DOMException || Error)(
            'The Range has partially selected a non-Text node.', 'InvalidNodeTypeError');
        }
        // R178（js-dom M4）：surroundContents 内含 insertNode 语义——Attr-rooted
        // range / Attr 参数同抛 HierarchyRequestError（spec `dom-range-surroundcontents`
        // 步骤 2「部分选区」与本处 Attr 非法父；WPT "surroundContents() on an
        // Attr-rooted range throws"）。
        if (newParent.nodeType === 2 || this.startContainer.nodeType === 2) {
          throw new (globalThis.DOMException || Error)(
            'Nodes of type 2 cannot be inserted or inserted into.', 'HierarchyRequestError');
        }
        // R244（js-dom M4）：**contained children 含 DocumentType →
        // HierarchyRequestError（树不变）**——spec `dom-range-extract-contents`
        // 步骤 9「If any member of contained children is a DocumentType, throw」
        // （surroundContents 步骤 3 调 extractContents，HRE 原样上抛）。common.js
        // myExtractContents 的 containedChildren 循环同款；contained = cac 的子
        // 中「after (sc,so) 且 before (ec,eo)」全含者。WPT Range-surroundContents
        // 25/26,x 元素 newParent（paras[0]/foreignPara1/detachedPara1/detachedDiv/
        // foreignPara2/xmlElement——j=0,4,6,9,11,13）12F 簇：range 覆盖 doc 的
        // doctype 子（`[document,0,document,1/2]`——iframe doc 首子是 doctype），
        // sim 步骤 3 先抛 HRE 而 host 对元素 newParent 无任何拦截（NO_THROW——
        // 探针 25/26,x 24 行实证：j 非 6 元素族全部 NO_THROW、文本族 host 已抛）。
        // 阈值门：仅 cac 是 Document（nodeType 9）时做 contained 扫描——contained
        // children 只对 cac 直接子有意义，其余容器 cac 无 doctype 子（doctype
        // 只能挂在 Document 下），零扫描成本。
        // https://dom.spec.whatwg.org/#dom-range-extractcontents
        (function _r244DoctypeCheck(self) {
          var cac = self.commonAncestorContainer;
          if (!cac || cac.nodeType !== 9 || !cac.childNodes) return;
          var _r244isContained = function (n) {
            // contained = 严格在 (sc,so) 之后且严格在 (ec,eo) 之前（common.js
            // isContained 的 getPosition 语义；sc===ec===cac 时退化为区间算术：
            // so < idx+? —— 子 idx 边界点是 idx 与 idx+1，含 = idx >= so 且
            // idx+1 <= eo（eo 计到子数））。
            if (self.startContainer === self.endContainer
              && self.startContainer === cac) {
              var idx = cac.childNodes.indexOf(n);
              if (idx < 0) return false;
              return idx >= (self.startOffset | 0)
                && (idx + 1) <= (self.endOffset | 0);
            }
            // 跨容器形态（cac 是 Document 的 sc/ec 深容器祖先）：按 ancestor-of
            // 边界判定——n 含 start 边界容器 → 非全含；n 含 end 边界容器 →
            // 非全含；否则 n 在 cac 直接子序列且 start/end 边界容器都在 n 的
            // 兄弟序列外时近似 contained（保守：仅同容器精确，跨容器仅当
            // start/end 容器都是 cac 自身或 n 的严格后代时判 idx 区间）。
            var ancOf = function (a, d) {
              while (d) { if (d === a) return true; d = d.parentNode; }
              return false;
            };
            if (ancOf(n, self.startContainer) || ancOf(n, self.endContainer)) return false;
            var idx2 = cac.childNodes.indexOf(n);
            if (idx2 < 0) return false;
            // start 边界在 cac 直接子序列上的落点：边界容器是 cac 某子 k 的
            // 后代 → 落点该子；是 cac 自身 → 落点 so。
            var sideIdx = function (container, offset) {
              if (container === cac) return offset | 0;
              var cur = container;
              while (cur && cur.parentNode && cur.parentNode !== cac) cur = cur.parentNode;
              if (!cur || !cur.parentNode) return -1;
              return cac.childNodes.indexOf(cur) + 1; // 边界在子树末尾之后
            };
            var sSide = sideIdx(self.startContainer, self.startOffset);
            var eSide = sideIdx(self.endContainer, self.endOffset);
            if (sSide < 0 || eSide < 0) return false;
            return idx2 >= sSide && (idx2 + 1) <= eSide;
          };
          var ks244 = cac.childNodes;
          for (var c244 = 0; c244 < ks244.length; c244++) {
            if (ks244[c244] && ks244[c244].nodeType === 10
              && _r244isContained(ks244[c244])) {
              throw new (globalThis.DOMException || Error)(
                'A DocumentType node cannot be extracted.', 'HierarchyRequestError');
            }
          }
        })(this);
        var kids = this._coveredChildren();
        // R209（js-dom M4）：spec `dom-range-surroundcontents` 步骤 5——最终
        // `newParent.appendChild(fragment)`：newParent 是 Text/Comment/PI 等叶子类型
        // 时必抛 HierarchyRequestError（WPT mega-case 的 HIERARCHY_REQUEST_ERR 簇；
        // 旧版吞错/defer-no-op 不抛 → "did not throw"）。抛出**前**先走步骤 3-4 的
        // insertNode 树变更（折叠路径的 splitText/插入中间态与 common.js
        // mySurroundContents 模拟一致——先变更后抛，positionTests 的树比较才对齐）。
        // https://dom.spec.whatwg.org/#dom-range-surroundcontents
        if (newParent.nodeType === 3 || newParent.nodeType === 4
          || newParent.nodeType === 7 || newParent.nodeType === 8) {
          // R215（js-dom M4）：叶子 newParent 的「先变更后抛」序恢复——
          // sim（common.js mySurroundContents）在步骤 3 extract 变更树之后
          // 步骤 5 才抛。R215 的 insertNode pre-insertion 校验会拦在变更前
          // ——对**本 surround 路径**抑制校验（_r215NoValidate 帧标志），恢复
          // 旧 split/insert 行为后抛（R212 序，829 基线）。8,9 循环场景的
          // 校验在 insertNode 直接调用时仍然生效。
          globalThis._r215NoValidate = true;
          try {
            // R259（js-dom M4）：kids===0 也先 extract——sim（common.js
            // mySurroundContents）对步骤 3 无形态分支：myExtractContents 无条件
            // setStart/setEnd 折叠对（WPT 16,x `[document.body,4,body,5]` 的
            // E 侧探针：extract 后 (body,4→4)，myInsertNode 尾步 setEnd(body,2)
            // 经 shim 的 R203 crossing 重设把 start 一并拉到 2——终态 (2,2)）。
            // 旧版只 insertNode：range 保持 (4,5) 未折叠使 R219 的
            // start===end 守卫跳过、边界漂移（A 侧终态 (5,5)）。
            // https://dom.spec.whatwg.org/#dom-range-surroundcontents
            if (kids !== null && kids.length === 0) {
              try { this.extractContents(); } catch (_eR259x) {}
              this.insertNode(newParent);
            }
            else if (kids === null
              && (this.startContainer.nodeType === 3 || this.startContainer.nodeType === 4)
              && this.startContainer === this.endContainer) {
              // R230（js-dom M4）：Text/CDATA 同节点容器的 leaf-newParent 同样
              // **先 extract 再 insert**（sim 序：步骤 3 extract 变更源 data →
              // 步骤 4 insertNode(newParent) → 步骤 5 appendChild(frag) 抛 HRE。
              // 旧版只 insertNode 使源 text 保留区间原文（WPT
              // Range-surroundContents 9,x「got "qrstuv"」族——expected 源已削
              // 为前缀 + newParent 文本在位）。
              // https://dom.spec.whatwg.org/#dom-range-surroundcontents
              this.extractContents();
              this.insertNode(newParent);
            } else if (kids === null
              && this.startContainer !== this.endContainer
              && (this.startContainer.nodeType === 3 || this.startContainer.nodeType === 4
                || this.startContainer.nodeType === 7 || this.startContainer.nodeType === 8)
              && this.endContainer.nodeType !== undefined
              && this.startContainer.parentNode === this.endContainer.parentNode) {
              // R235（js-dom M4）：**异节点同父 CharData 区间**的 leaf-newParent
              // 同款「先 extract 再 insert 后抛」（sim 序——common.js
              // mySurroundContents 对步骤 3 无形态分支：extract 变更树（首尾
              // 切片 deleteData + contained 子移除）→ 步骤 4 myInsertNode 插
              // newParent → 步骤 5 appendChild(frag) 抛 HRE。旧版直接抛使树
              // 保留区间原文（WPT Range-surroundContents 6,x
              // `[paras[5].firstChild,2,paras[5].lastChild,4]` 46F——CDATA#1→
              // text 同父区间的「assert_unreached DOMs were not equal」簇）。
              // https://dom.spec.whatwg.org/#dom-range-surroundcontents
              this.extractContents();
              this.insertNode(newParent);
            } else if (kids === null
              && this.startContainer.nodeType === 1
              && (this.endContainer.nodeType === 3 || this.endContainer.nodeType === 4
                || this.endContainer.nodeType === 7 || this.endContainer.nodeType === 8)
              && this.endContainer.parentNode === this.startContainer) {
              // R236（js-dom M4）：**sc 是 ec 的元素祖先容器且 ec 为直接
              // CharData 子**的 leaf-newParent（WPT Range-surroundContents 23,x
              // `[paras[0],0,paras[0].firstChild,7]` + Text newParent——sim 序
              // 步骤 3 extract 削 ec 头部（deleteData(0,eo)，remainder 留树）→
              // 步骤 4 insertNode 插 newParent 到 (sc, so) → 步骤 5 抛 HRE。
              // 旧版直接抛使 text 保留区间原文）。extractContents 的 R236 分支
              // 承担树变更，此处按 sim 序先 extract 再 insert 后抛。
              // https://dom.spec.whatwg.org/#dom-range-surroundcontents
              this.extractContents();
              this.insertNode(newParent);
            } else if (kids === null
              && this.startContainer !== this.endContainer
              && this.startContainer.nodeType === 1
              && this.endContainer.nodeType === 1
              && this.endContainer.parentNode === this.startContainer
              && (this.endOffset | 0) <= (this.endContainer.childNodes
                ? this.endContainer.childNodes.length : 0)
              && (this.startOffset | 0) <= this.startContainer.childNodes.indexOf(this.endContainer)) {
              // R242（js-dom M4）：**sc 元素祖先 + ec 元素直接子（clean 边界）**
              // 的 leaf-newParent（WPT 24,x Text/Comment 型 newParent——sim 序
              // 步骤 3 extract（中段子移出 + ec shallow clone 承接）→ 步骤 4
              // insertNode → 步骤 5 抛 HRE；旧版直接抛使树保留原文）。
              // https://dom.spec.whatwg.org/#dom-range-surroundcontents
              this.extractContents();
              this.insertNode(newParent);
            } else if (kids !== null && kids.length > 0) {
              // R235（js-dom M4）：**元素容器含覆盖子**的 leaf-newParent——
              // sim 序同样是 extract 先行（covered 子移出容器）再 insertNode
              // 插 newParent（插到 (容器, startOffset)）再抛 HRE。旧版只对
              // kids.length===0 走 insertNode，kids>0 直接抛使容器保留原文
              // （WPT Range-surroundContents 18,x
              // `[paras[0],0,paras[0],1]` + Text newParent 的 differing 簇）。
              // https://dom.spec.whatwg.org/#dom-range-surroundcontents
              this.extractContents();
              this.insertNode(newParent);
            }
          } finally { globalThis._r215NoValidate = false; }
          // R229（js-dom M4）：comment/PI 同节点容器（7/8）的 leaf-newParent 序——
          // sim（common.js mySurroundContents）先 extractContents（**变更容器
          // data**：中段切片 deleteData）再在 myInsertNode 处抛 HRE。旧版直接抛
          // 使容器 data 不变（WPT Range-surroundContents 35,1–2 / 36,x 的
          // 「Stuwxyz got Stuvwxyz」残余族——newParent 为 Text/Comment 形态）。
          // https://dom.spec.whatwg.org/#dom-range-surroundcontents
          if (kids === null
            && (this.startContainer.nodeType === 7 || this.startContainer.nodeType === 8)
            && this.startContainer === this.endContainer) {
            try { this.extractContents(); } catch (_eR229x) {}
            throw new (globalThis.DOMException || Error)(
              'Nodes of type ' + newParent.nodeType + ' cannot have children.',
              'HierarchyRequestError');
          }
          throw new (globalThis.DOMException || Error)(
            'Nodes of type ' + newParent.nodeType + ' cannot have children.',
            'HierarchyRequestError');
        }
        if (kids === null) {
          // R212（js-dom M4）：**CharData 区间 range 的 surround 路径**——
          // extractContents 的 R211 分支（start/end 容器为 Text/CDATA 同父）+
          // insertNode 把 newParent 插到区间原位 + newParent.appendChild(frag)
          //（R212 工厂元素 appendChild 补 fragment 展平——首版 frag 末子丢失的
          // 根因）。与 common.js mySurroundContents 模拟树形态对齐（6,x 族）。
          var _r212sc2 = this.startContainer, _r212ec2 = this.endContainer;
          var _r212isCd2 = function (n) {
            return !!n && (n.nodeType === 3 || n.nodeType === 4
              || n.nodeType === 7 || n.nodeType === 8);
          };
          // R228（js-dom M4）：同节点 CharData（detached 含 comment/PI）放宽——
          // 与 extractContents 的 R228 同节点分支成对（WPT Range-surroundContents
          // 35,x/36,x「Stuwxyz got Stuvwxyz」族：detached Comment/PI 区间 surround
          // 旧版因 parentNode 守卫整体空转）。异节点仍需同父。
          if (_r212isCd2(_r212sc2) && _r212isCd2(_r212ec2)
            && (_r212sc2 === _r212ec2
              || (_r212sc2.parentNode && _r212sc2.parentNode === _r212ec2.parentNode))) {
            var _r212frag2 = this.extractContents();
            // R212：spec 步骤 3——「While newParent has children, remove its
            // first child」（common.js mySurroundContents 同款；旧版漏此步使
            // wrapped 元素残留 setup 期原文本）。
            var _r212guard = 0;
            while (newParent.childNodes && newParent.childNodes.length && _r212guard++ < 256) {
              try { newParent.removeChild(newParent.childNodes[0]); } catch (_eR212rm) { break; }
            }
            // R228：insertNode 的 HRE **不再吞**（sim 序：extract 变更树后
            // insertNode 对叶子容器抛 HRE——common.js mySurroundContents 把
            // myInsertNode 的返回串原样上抛；旧吞错使 WPT 35–38,x 的
            // detached Comment/PI/Text 区间「must be thrown」族不抛）。
            this.insertNode(newParent);
            try { newParent.appendChild(_r212frag2); } catch (_eR212a2) {}
            // selectNode(newParent) 语义：range 落到 newParent 的父 + 索引。
            try {
              var _r212np = newParent.parentNode;
              var _r212ni = _r212np && _r212np.childNodes
                ? _r212np.childNodes.indexOf(newParent) : -1;
              if (_r212ni < 0 && this.startContainer && this.startContainer.childNodes) {
                var _r258si1 = this.startContainer.childNodes.indexOf(newParent);
                if (_r258si1 >= 0) { _r212np = this.startContainer; _r212ni = _r258si1; }
              }
              if (_r212ni >= 0) { this.setStart(_r212np, _r212ni); this.setEnd(_r212np, _r212ni + 1); }
            } catch (_eR212s2) {}
          }
          // R236（js-dom M4）：**sc 是 ec 的元素祖先容器且 ec 为直接 CharData 子**
          // 的元素 newParent（WPT Range-surroundContents 23,4/6/9/11/13——sim 序：
          // 步骤 3 extract 削 ec 头部（remainder 留树）→ 步骤 4 insertNode(newParent)
          // 到 (sc, so)（newParent === sc 自身时 ensurePreInsertion 的 inclusive
          // ancestor 查抛 HRE——步骤 3 已变更树后再抛，与 mySurroundContents 对齐，
          // 即 23,0 形态）→ 步骤 5 appendChild(frag) → selectNode(newParent)）。
          // https://dom.spec.whatwg.org/#dom-range-surroundcontents
          if (this.startContainer !== this.endContainer
            && this.startContainer.nodeType === 1
            && (this.endContainer.nodeType === 3 || this.endContainer.nodeType === 4
              || this.endContainer.nodeType === 7 || this.endContainer.nodeType === 8)
            && this.endContainer.parentNode === this.startContainer) {
            var _r236frag2 = this.extractContents();
            // 步骤 2：清 newParent 既有子（sim「While newParent has children,
            // remove its first child」——R212 同款；旧漏此步使 newParent 残留
            // 原内容（expected 头切片 got "Efghijkl"）。
            var _r236guard = 0;
            while (newParent.childNodes && newParent.childNodes.length && _r236guard++ < 256) {
              try { newParent.removeChild(newParent.childNodes[0]); } catch (_eR236rm) { break; }
            }
            // R258（js-dom M4）：**selectNode 落位的 sc 回退**——newParent 自身是
            // covered 子时（30,x：foreignPara1 在 [fb,0→foreignTextNode,36] 覆盖
            // 子内），extract 把 newParent 本体移进 frag2 使其 parentNode 指向
            // fragment；insertNode 后工厂 body 的 _tree 视图已含 newParent 但
            // parentNode 修复链（_tree.appendChild 事后 if 判定）对 frag 旧链
            // miss——`np.childNodes.indexOf(newParent)` 返 -1 使 selectNode 落位
            // 整体跳过（WPT「endOffset expected 1 got 0」——DOM 断言已过，仅
            // 边界缺写）。回退：以**本分支插入前的 sc**（insertNode 未改 sc 的
            // 形态下二者一致）为父查 indexOf（树视图权威）。
            // https://dom.spec.whatwg.org/#dom-range-surroundcontents
            this.insertNode(newParent);
            try { newParent.appendChild(_r236frag2); } catch (_eR236a) {}
            try {
              var _r236np2 = newParent.parentNode;
              var _r236ni2 = _r236np2 && _r236np2.childNodes
                ? _r236np2.childNodes.indexOf(newParent) : -1;
              if (_r236ni2 < 0 && this.startContainer && this.startContainer.childNodes) {
                var _r258si2 = this.startContainer.childNodes.indexOf(newParent);
                if (_r258si2 >= 0) { _r236np2 = this.startContainer; _r236ni2 = _r258si2; }
              }
              if (_r236ni2 >= 0) {
                this.setStart(_r236np2, _r236ni2);
                this.setEnd(_r236np2, _r236ni2 + 1);
              }
            } catch (_eR236s) {}
            return;
          }
          // R242（js-dom M4）：**sc 元素祖先 + ec 元素直接子 + clean 边界**的
          // surround（24,x `[testDiv,2,paras[4],1]`——extractContents 的 R242
          // 分支承担树变更：中段子本体移入 frag + ec shallow clone 承接
          // [0,eo) 子；surround 侧同 sim 全序：extract → 清 newParent 子 →
          // insertNode 到 (sc, so) → appendChild(frag) → selectNode）。
          // https://dom.spec.whatwg.org/#dom-range-surroundcontents
          if (this.startContainer !== this.endContainer
            && this.startContainer.nodeType === 1
            && this.endContainer.nodeType === 1
            && this.endContainer.parentNode === this.startContainer
            && (this.endOffset | 0) <= (this.endContainer.childNodes
              ? this.endContainer.childNodes.length : 0)
            && (this.startOffset | 0) <= this.startContainer.childNodes.indexOf(this.endContainer)) {
            var _r242frag2 = this.extractContents();
            var _r242clr2 = 0;
            while (newParent.childNodes && newParent.childNodes.length && _r242clr2++ < 256) {
              try { newParent.removeChild(newParent.childNodes[0]); } catch (_eR242rm) { break; }
            }
            this.insertNode(newParent);
            try { newParent.appendChild(_r242frag2); } catch (_eR242a2) {}
            try {
              var _r242np2 = newParent.parentNode;
              var _r242ni2 = _r242np2 && _r242np2.childNodes
                ? _r242np2.childNodes.indexOf(newParent) : -1;
              if (_r242ni2 < 0 && this.startContainer && this.startContainer.childNodes) {
                var _r258si3 = this.startContainer.childNodes.indexOf(newParent);
                if (_r258si3 >= 0) { _r242np2 = this.startContainer; _r242ni2 = _r258si3; }
              }
              if (_r242ni2 >= 0) {
                this.setStart(_r242np2, _r242ni2);
                this.setEnd(_r242np2, _r242ni2 + 1);
              }
            } catch (_eR242s2) {}
            return;
          }
          return; // 其余跨容器/文本切片 defer
        }
        // R237（js-dom M4）：步骤 2 清 newParent 既有子（sim mySurroundContents
        // 「While newParent has children, remove its first child」——旧漏此步使
        // newParent 残留原内容（12–14,x 探针 P{2} 形态）。**先清再 clone**——
        // clone 循环把 covered 子克隆进 newParent，后置清理会把克隆一并误删）。
        // https://dom.spec.whatwg.org/#dom-range-surroundcontents
        var _r237clr = 0;
        while (newParent.childNodes && newParent.childNodes.length && _r237clr++ < 256) {
          try { newParent.removeChild(newParent.childNodes[0]); } catch (_eR237c) { break; }
        }
        // R257（js-dom M4）：**newParent 仍是 range 容器的 inclusive ancestor
        // → HRE（先 extract/清子后抛）**——spec `dom-range-surroundcontents` 步骤
        // 4 调 insertNode，其 pre-insertion validity（`concept-node-pre-insertion-
        // validity`「node 是 parent 的 inclusive ancestor → HRE」）对 self-
        // surround（WPT Range-surroundContents 18,0 `[paras[0],0,paras[0],1]` +
        // paras[0] / 19,6 `[detachedPara1,0,…]` + detachedPara1）必抛。**检查时点
        // 在清 newParent 子之后**（sim 序：步骤 3 extract 移出 covered 子 →
        // 步骤 2 清 newParent 子——若 sc 是 newParent 的子，此步已断 sc→newParent
        // 父链，步骤 4 的 inclusive ancestor 查经 parentNode 上行不再命中（19,9
        // 的 detachedDiv 族 sim 不抛而成功 wrap）；self-surround 的 newParent===
        // sc 父链不经 newParent 子列表，仍命中 → 抛。旧版元素主路径不经
        // insertNode（直接 clone 循环 + insertBefore），对 self-surround 无任何
        // 拦截：清 newParent 子（误删自身内容）+ newParent.remove()（把自己摘出
        // 旧父）后静默成功。
        // https://dom.spec.whatwg.org/#dom-range-surroundcontents
        var _r257npAnc = false;
        try {
          var _r257cur = this.startContainer, _r257h = 0;
          while (_r257cur && _r257h++ < 128) {
            if (_r257cur === newParent) { _r257npAnc = true; break; }
            _r257cur = _r257cur.parentNode;
          }
        } catch (_eR257a) {}
        if (_r257npAnc) {
          // sim 步骤 3 等价：covered 子移出（对 sc===ec 元素即 kids 逆序
          // remove + range 塌缩），再抛。
          for (var _r257k = kids.length - 1; _r257k >= 0; _r257k--) {
            try { if (typeof kids[_r257k].remove === 'function') kids[_r257k].remove(); }
            catch (_eR257rm) {}
          }
          try { this.collapse(true); } catch (_eR257c) {}
          throw new (globalThis.DOMException || Error)(
            'The new child element contains the parent.',
            'HierarchyRequestError');
        }
        // R254（js-dom M4）：**clone 循环前先摘除 newParent**——covered 子的深克隆
        // 源树里 newParent 仍挂在旧父（如 13,0 的 docEl[0,2] 覆盖 BODY>div#test>
        // paras[0]=newParent 自身）时，`kids[i].cloneNode(true)` 会把 newParent 的
        // **克隆中间态**（先克隆进 HEAD-clone、BODY 未克隆时的半完成形态）一并烘进
        // BODY-clone 内的 div#test（WPT Range-surroundContents 13/14,x 的「幽灵
        // P#a{HEAD-only}」——probe R254-v5/v6 实证：BODY-clone 的 div#test 首子 =
        // isNP=false 的第三对象，outerHTML=`<p id="a"><head>…`，sim 侧无此形态，
        // 因 common.js 步骤 3 extract 先移出原件再插 newParent）。spec 序
        // （surround 步骤 2「清 newParent 子」在步骤 3 extract 之后，extract 时
        // newParent 若在覆盖子树内已被移出）等效于「克隆前 newParent 不在覆盖
        // 子树内」。提前摘除与 R248 的 insert 前摘除幂等（已 detached 时 no-op）。
        // https://dom.spec.whatwg.org/#dom-range-surroundcontents
        try {
          if (typeof newParent.remove === 'function' && newParent.parentNode) newParent.remove();
        } catch (_eR254rm) {}
        for (var i = 0; i < kids.length; i++) {
          try { newParent.appendChild(kids[i].cloneNode(true)); } catch (_e) {}
        }
        // js-dom M4 R47：record 顺序与树操作顺序解耦——removed records 按**文档序**（WPT
        // surroundContents 期望 [removed first, removed last, added]），但树操作保持**逆序 remove**
        //（R2930：正序 remove 后 nth-child selector 前移失效，移错节点——renderer R2930 测试捕获）。
        // 实现：先按文档序捕获每个 child 的兄弟快照（remove 前），置 _zwSuppressRemoveRecord 抑制
        // remove() 的逐次 notify，逆序 remove（树正确），再按文档序统一补发 records。
        var _rmSnap = [];
        var _scSel = null, _scHandle = null;
        try {
          _scSel = this.startContainer.__zwSelector || null;
          _scHandle = this.startContainer.__zwHandle || null;
        } catch (_e) {}
        // R301（js-dom M4）：record 兄弟按**顺序移除后的树形态**取值——spec 每条
        // record 反映该次移除时刻的邻居；全范围移除时已移除的兄弟不可见（WPT
        // MutationObserver-childList "Range.surroundContents" 期望第二条 record 的
        // previousSibling === null——s1 已移除后 s2 无左邻）。旧快照一次取全部
        // 移除前兄弟使 record2.pv 停留 s1。计算：prev/next 沿 kids 边界外扩到
        // 真实父 childNodes 的首个**不在本移除集**的邻居（含集外兄弟）。
        var _r301inSet = [];
        try { for (var _r301i2 = 0; _r301i2 < kids.length; _r301i2++) _r301inSet.push(kids[_r301i2]); } catch (_e301is) {}
        for (var j = 0; j < kids.length; j++) {
          var _rprev = null, _rnext = null;
          try {
            // 左邻：kids 内更早者均已移除 → 集外首个左兄弟（沿真实 previousSibling
            // 链跳过集内节点）。
            var _r301w = kids[j].previousSibling || null;
            while (_r301w && _r301inSet.indexOf(_r301w) >= 0) _r301w = _r301w.previousSibling || null;
            _rprev = _r301w;
            // 右邻：集内更晚者尚未移除但将移除——spec 顺序语义下对 record j 而言
            // 后续移除的兄弟在**本 record 时刻仍在**……WPT 期望第二条（s2）的
            // nextSibling null（s2 已是末子）。next 按移除前原值（集外右邻）。
            var _r301v = kids[j].nextSibling || null;
            _rnext = _r301v;
          } catch (_e) {}
          _rmSnap.push({ node: kids[j], prev: _rprev, next: _rnext });
        }
        globalThis._zwSuppressRemoveRecord = true;
        for (var k = kids.length - 1; k >= 0; k--) {
          try { if (typeof kids[k].remove === 'function') kids[k].remove(); } catch (_e) {}
        }
        globalThis._zwSuppressRemoveRecord = false;
        for (var m = 0; m < _rmSnap.length; m++) {
          _mo_notify(_scSel, _scHandle, {
            type: 'childList', addedNodes: [], removedNodes: [_rmSnap[m].node],
            previousSibling: _rmSnap[m].prev, nextSibling: _rmSnap[m].next,
          });
        }
        // R237（js-dom M4）：路径 4 收尾对齐 sim（common.js mySurroundContents
        // 步骤 2/4/6 全序——WPT Range-surroundContents 12–14,x
        // `[documentElement,0,…]` 探针实证三分歧：host docEl=[BODY,P] 而 sim
        // [P{head},BODY]）：
        // ① 步骤 2 清 newParent 既有子（旧漏——paras[0] 原文本残留使 P{2}）；
        // ② 步骤 4 插到 (startContainer, startOffset) 位（旧 appendChild 恒末尾
        //    ——docEl 下 P 落到 body 后）；
        // ③ 步骤 6 selectNode(newParent) 边界（(父, idx)-(父, idx+1)）。
        // https://dom.spec.whatwg.org/#dom-range-surroundcontents
        try {
          var _r237sc = this.startContainer;
          // R248（js-dom M4）：**先经 ChildNode.remove 摘除 newParent**——旧父可能是
          // iframe-doc/主文档的 wrapper 域容器，factory docEl insertBefore 的
          // 「从旧父摘除」按 identity 在 wrapper 域 miss（旧父 childNodes 存的是
          // wrapper 而非本体）→ 摘除静默失败，newParent 原件留在旧父（WPT
          // Range-surroundContents 13/14,0 的 DIV 六 P vs sim 五 P——probe ROOT dump
          // 实证）。Node.prototype.remove（R238 泛型）按节点自身 parentNode 走
          // mutation-emitting 路径，wrapper/handle/plain 三域通吃。
          // https://dom.spec.whatwg.org/#dom-range-surroundcontents
          try {
            if (typeof newParent.remove === 'function' && newParent.parentNode) newParent.remove();
          } catch (_eR248rm) {}
          var _r237ref = _r237sc.childNodes && _r237sc.childNodes[this.startOffset | 0];
          if (_r237ref != null && typeof _r237sc.insertBefore === 'function') {
            _r237sc.insertBefore(newParent, _r237ref);
          } else if (typeof _r237sc.appendChild === 'function') {
            _r237sc.appendChild(newParent);
          }
        } catch (_eR237ins) {
          try { this.startContainer.appendChild(newParent); } catch (_e237b) {}
        }
        try {
          var _r237np = newParent.parentNode;
          if (_r237np && _r237np.childNodes) {
            var _r237ni = _r237np.childNodes.indexOf(newParent);
            if (_r237ni >= 0) {
              this.setStart(_r237np, _r237ni);
              this.setEnd(_r237np, _r237ni + 1);
            }
          }
        } catch (_eR237sel) {}
      },
      cloneRange: function () {
        // 复制 Range（独立边界，互不影响）。spec AbstractRange 边界 + _mode/commonAncestor。
        var r = _makeRange();
        r.startContainer = this.startContainer; r.startOffset = this.startOffset;
        r.endContainer = this.endContainer; r.endOffset = this.endOffset;
        r.commonAncestorContainer = this.commonAncestorContainer;
        r.collapsed = this.collapsed; r._mode = this._mode;
        return r;
      },
      // R178（js-dom M4）：root-of（沿 parentNode 上行到根——spec「concept-tree-root」；
      // Attr 的 parentNode null → root 是自身，与文档根永不相等——WPT Range-attribute-nodes
      // 的 WrongDocumentError 短路族判据）。
      _rootOf178: function (node) {
        var cur = node, guard = 0;
        while (cur && cur.parentNode && guard++ < 128) cur = cur.parentNode;
        return cur;
      },
      // R178（js-dom M4）：`comparePoint(node, offset)`（spec `dom-range-comparepoint`）——
      // ① node root ≠ range root → WrongDocumentError；② offset 超 node length →
      // IndexSizeError；③ 返回 node 起 boundary 相对 range 起点的文档序（-1/0/+1，
      // 折叠态按 offset 比较）。
      comparePoint: function (node, offset) {
        if (node === null || node === undefined || typeof node.nodeType !== 'number') {
          throw new globalThis.TypeError(
            "Failed to execute 'comparePoint' on 'Range': parameter 1 is not of type 'Node'.");
        }
        // R288（js-dom M4）：spec 步骤序——root 不同 → WrongDocumentError **先于**
        // DocumentType 检查（WPT "Must throw WrongDocumentError if node and range
        // have different roots" 88/89,x 124F 簇：cross-root doctype 旧序先抛
        // InvalidNodeTypeError）。isPointInRange 同款序但 root 不同返 false 不抛。
        var myRoot = this._rootOf178(this.startContainer);
        var nodeRoot = this._rootOf178(node);
        if (myRoot !== nodeRoot) {
          throw new (globalThis.DOMException || Error)(
            'The two ranges are in different documents.', 'WrongDocumentError');
        }
        // R205（js-dom M4）：spec 步骤 3——node 是 DocumentType 抛
        // InvalidNodeTypeError（WPT "Must throw InvalidNodeTypeError if node is a
        // doctype" 簇）。
        if (node.nodeType === 10) {
          throw new (globalThis.DOMException || Error)(
            'The given node is invalid.', 'InvalidNodeTypeError');
        }
        var len = this._nodeLength(node);
        if ((offset | 0) < 0 || (offset | 0) > len) {
          throw new (globalThis.DOMException || Error)(
            'The offset is out of range.', 'IndexSizeError');
        }
        // R205：**边界点比较重写**（spec 步骤 5-7——point 在 start 前 → -1，
        // 在 end 后 → 1，否则 0）：复用 R203 `_zwRangeBpAfter` 树序比较（旧版
        // cDP best-effort + 方向位写反 → "Must return 1 if point is after end
        // expected 1 but got -1" 310F 簇）。
        if (_zwRangeBpAfter(this.startContainer, this.startOffset | 0, node, offset | 0)) {
          return -1; // point 在 start 之前
        }
        if (_zwRangeBpAfter(node, offset | 0, this.endContainer, this.endOffset | 0)) {
          return 1; // point 在 end 之后
        }
        return 0;
      },
      // R178（js-dom M4）：`isPointInRange(node, offset)`（spec `dom-range-ispointinrange`）
      // ——root 不同返 false（不抛）；offset 超长 IndexSizeError；容器相等按 offset 区间。
      isPointInRange: function (node, offset) {
        if (node === null || node === undefined || typeof node.nodeType !== 'number') {
          throw new globalThis.TypeError(
            "Failed to execute 'isPointInRange' on 'Range': parameter 1 is not of type 'Node'.");
        }
        // R205（js-dom M4）：spec 步骤序——root 不同先返 **false**（不抛），
        // 之后 node 是 DocumentType 才抛 InvalidNodeTypeError（WPT
        // expectFalse 与 doctype-throw 两族并存：foreign doctype 走 false 路径）。
        if (this._rootOf178(this.startContainer) !== this._rootOf178(node)) return false;
        if (node.nodeType === 10) {
          throw new (globalThis.DOMException || Error)(
            'The given node is invalid.', 'InvalidNodeTypeError');
        }
        var len = this._nodeLength(node);
        if ((offset | 0) < 0 || (offset | 0) > len) {
          throw new (globalThis.DOMException || Error)(
            'The offset is out of range.', 'IndexSizeError');
        }
        if (node === this.startContainer && node === this.endContainer) {
          return (offset | 0) >= this.startOffset && (offset | 0) <= this.endOffset;
        }
        // R205（js-dom M4）：跨容器树序判定（spec 步骤 4——point 在 (start) 前
        // 或 (end) 后返 false）：复用 R203 `_zwRangeBpAfter` 边界点比较。
        //（旧版 best-effort true——WPT isPointInRange 的 "point is before start
        // or after end" 695F 簇根因。）
        if (_zwRangeBpAfter(this.startContainer, this.startOffset | 0, node, offset | 0)) {
          return false; // point 在 start 之前
        }
        if (_zwRangeBpAfter(node, offset | 0, this.endContainer, this.endOffset | 0)) {
          return false; // point 在 end 之后
        }
        return true;
      },
      // R178（js-dom M4）：`intersectsNode(node)`（spec `dom-range-intersectsnode`）——
      // root 不同返 false；node 的父 null（root 自身）→ 恒 true（range 与自身根必交）；
      // 否则按 node 起止偏移与 range 边界的区间交。
      // R205（js-dom M4）：**边界点比较重写**——node 占据边界点区间
      // [(parent, i), (parent, i+1)]；与 range [start, end] 不交 iff
      // (parent, i+1) ≤ start 或 (parent, i) ≥ end（经 R203 `_zwRangeBpAfter`
      // 树序比较）。旧版拿 node 在父中的索引 i 直接与 range 的 startOffset/
      // endOffset 比——**两者不在同一坐标系**（range 容器未必是 node 的父），
      // 跨容器形态全错（WPT intersectsNode 186F 簇）。
      intersectsNode: function (node) {
        if (node === null || node === undefined || typeof node.nodeType !== 'number') {
          throw new globalThis.TypeError(
            "Failed to execute 'intersectsNode' on 'Range': parameter 1 is not of type 'Node'.");
        }
        if (this._rootOf178(this.startContainer) !== this._rootOf178(node)) return false;
        // spec `dom-range-intersectsnode` 步骤 2：node 的 parent null（node 即
        // 根——Attr/Document 场景）→ **恒 true**（range 与自身根必交；WPT
        // "intersectsNode() with an Attr node sharing the range's root returns
        // true"——collapsed 前置判定在 spec 中不存在，是首版误加）。
        var parent = node.parentNode;
        if (!parent) return true;
        // R205：移除 collapsed 前置 false——spec（2024 版 dom-range-intersectsnode）
        // 无此步骤：collapsed range 仍与其**边界点所在节点**相交（node 占据
        // [(parent,i),(parent,i+1)]，start==end 落在其中即交）。旧 early-false 使
        // 「Node 0 paras[0] + range collapsed in firstChild」形态全错（WPT 186F）。
        var i = this._indexOf(parent, node);
        if (i < 0) return true; // 不在父视图（best-effort 保守 true）
        // R289（js-dom M4）：**严格不等**修正（WPT intersectsNode-2 Chromium
        // crbug 822510 形态）：true iff (parent,i) 严格 before end **且**
        // (parent,i+1) 严格 after start。旧版两分支用的是「(i+1) ≤ start 或
        // (i) ≥ end」的**非严格**否定——边界相接（node 首边界 == range end 或
        // node 末边界 == range start）被误判相交：range [div,0,div,1] 对 s1
        // 占据 [(div,1),(div,2)]，(div,1) vs end (div,1) 相等不交，旧版
        // after(1,1)=false 漏拦 → true（期望 false）。修：非交条件改为
        // ¬((parent,i) before end) 或 ¬((parent,i+1) after start)，其中
        // before(a,b) = after(b,a) 且相等的边界点既非 before 也非 after。
        // node 的首边界 (parent, i) 不严格在 end 之前（≥ end）→ node 整体在 range 后。
        if (!_zwRangeBpAfter(this.endContainer, this.endOffset | 0, parent, i)) {
          return false;
        }
        // node 的末边界 (parent, i+1) 不严格在 start 之后（≤ start）→ node 整体在 range 前。
        if (!_zwRangeBpAfter(parent, i + 1, this.startContainer, this.startOffset | 0)) {
          return false;
        }
        return true;
      },
      // R178（js-dom M4）：`compareBoundaryPoints(how, sourceRange)`（spec
      // `dom-range-compareboundarypoints`）——两 range root 不同 → WrongDocumentError；
      // 同 root 折叠 range 互比 → 0（WPT Attr-rooted 双 range 全 4 how 期望 0）。
      compareBoundaryPoints: function (how, sourceRange) {
        // R204（js-dom M4）：`how` 的 **WebIDL unsigned short 转换前置**（spec
        // range-compareboundarypoints 步骤 1 + WebIDL ToUint16）：ToNumber →
        // NaN/±0/±∞ → +0；否则 sign*floor(abs) mod 2^16（负数回绕）。转换结果非
        // 0-3 → NotSupportedError（WPT Range-compareBoundaryPoints 的 "-1, 4, 5,
        // NaN, ±Infinity, 0.5, 字符串形态, null/undefined/bool" 全形态断言——旧版
        // `| 0` 截断 + IndexSizeError，NaN|0=0 误判合法、-1 回绕 65535 未抛）。
        // **参数序**：how 转换先于 sourceRange 类型检查（WebIDL 参数序转换）。
        var howNum = Number(how);
        var howN;
        if (isNaN(howNum) || howNum === 0 || howNum === Infinity || howNum === -Infinity) {
          howN = 0;
        } else {
          var posInt = (howNum < 0 ? -1 : 1) * Math.floor(Math.abs(howNum));
          howN = posInt % 65536;
          if (howN < 0) howN += 65536;
        }
        if (howN !== 0 && howN !== 1 && howN !== 2 && howN !== 3) {
          throw new (globalThis.DOMException || Error)(
            'The comparison how argument is not one of START_TO_START, START_TO_END, END_TO_END or END_TO_START.', 'NotSupportedError');
        }
        if (!sourceRange || typeof sourceRange.startContainer === 'undefined') {
          throw new globalThis.TypeError(
            "Failed to execute 'compareBoundaryPoints' on 'Range': parameter 2 is not of type 'Range'.");
        }
        if (this._rootOf178(this.startContainer) !== sourceRange._rootOf178(sourceRange.startContainer)) {
          throw new (globalThis.DOMException || Error)(
            'The two ranges are in different documents.', 'WrongDocumentError');
        }
        // R288（js-dom M4）：spec 步骤 4-6 的边界点对选取 + 位置比较完整重写。
        // 旧版两缺陷（WPT 592F 簇实证）：① how=2（END_TO_END）落入默认分支成了
        // START_TO_START 语义（同容器 offset 差恒等 → "expected -1 got 0" 12F +
        // 跨容器符号对调 282F）；② 跨容器走 cDP 位（FOLLOWING/PRECEDING）——
        // 祖先/后代容器对（point 在祖先 offset 与后代树序之间）cDP 位只有树序，
        // 无 offset-vs-childIndex 比较（WPT 1,17,x [pf,0] vs [body,4] 族 56F 符号反）。
        // 修：按 spec 表选取 (this, source) 边界点对，复用 R203 `_zwRangeBpAfter`
        //（祖先 offset-vs-childIndex + 深度感知双 climb 已实现——comparePoint/
        // isPointInRange 同源）。
        // START_TO_START=0 / START_TO_END=1 / END_TO_END=2 / END_TO_START=3
        // （spec 常量：比较 (this 的 [start,end]) 与 source 的 [start,end]）。
        var thisPair178 = howN === 1 || howN === 2
          ? [this.endContainer, this.endOffset]
          : [this.startContainer, this.startOffset];
        var srcPair178 = howN === 0 || howN === 1
          ? [sourceRange.startContainer, sourceRange.startOffset]
          : [sourceRange.endContainer, sourceRange.endOffset];
        if (thisPair178[0] === srcPair178[0]) {
          return thisPair178[1] === srcPair178[1] ? 0
            : (thisPair178[1] < srcPair178[1] ? -1 : 1);
        }
        if (_zwRangeBpAfter(srcPair178[0], srcPair178[1] | 0, thisPair178[0], thisPair178[1] | 0)) {
          return -1; // this 边界点在 source 边界点之前
        }
        if (_zwRangeBpAfter(thisPair178[0], thisPair178[1] | 0, srcPair178[0], srcPair178[1] | 0)) {
          return 1; // this 边界点在 source 边界点之后
        }
        return 0;
      },
      detach: function () { /* no-op（spec 已废弃 Range.detach，保留供老库调用） */ },
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
        // R197（js-dom M4）：跨容器形态的 spec 精确化（`range-stringification`）——
        // ① start 容器是 Text → 起始切片（startOffset 起）；② 两端点间的 contained
        // Text 节点全收（文档序——按 CAC 子树 DFS 收集后过滤到 (start,end) 区间）；
        // ③ end 容器是 Text → 尾部切片（至 endOffset）。旧版取 commonAncestor 整
        // 子树文本，把端点外的兄弟内容（含 <script> 文本）也拼进结果（WPT
        // Range-stringifier 后两用例 got 含整页 script 源）。contained 判定按
        //「树序在 start 边界后 + end 边界前」——用 DFS 序 + 边界位置比较实现。
        var r197sc = this.startContainer, r197ec = this.endContainer;
        if (r197sc && r197ec && r197sc !== r197ec) {
          var _r197IsText = function (n) { return !!n && (n.nodeType === 3 || n.__zwIsText); };
          // 同一容器父下的边界比较：(ancestor, childIndex) 对——node 上行到容器父的
          // 首 child 索引链比较（spec boundary-point position 的简化：只比到共同深度）。
          var _r197Pos = function (node, offset, ancestor) {
            // 返回 [chainOrderIndex, offsetInNode]——node 相对 ancestor 的 DFS 前缀序。
            var path = [], cur = node, hops = 0;
            while (cur && cur !== ancestor && hops++ < 128) {
              var p = cur.parentNode;
              if (!p) return null;
              var kids = p.childNodes || [];
              var idx = -1;
              for (var k = 0; k < kids.length; k++) if (kids[k] === cur) { idx = k; break; }
              if (idx < 0) return null;
              path.unshift(idx);
              cur = p;
            }
            if (cur !== ancestor) return null;
            return path;
          };
          var _r197Cmp = function (pathA, offA, pathB, offB) {
            if (!pathA || !pathB) return 0; // 不可比（树形态外）→ 调用方回落
            var n = Math.min(pathA.length, pathB.length);
            for (var i = 0; i < n; i++) {
              if (pathA[i] < pathB[i]) return -1;
              if (pathA[i] > pathB[i]) return 1;
            }
            if (pathA.length === pathB.length) return offA <= offB ? -1 : 1;
            // 前缀相同：更浅（容器自身）按 offset 与子索引比——A 是 B 的祖先形态。
            if (pathA.length < pathB.length) return offA <= pathB[n] ? -1 : 1;
            return pathA[n] < offB ? -1 : 1;
          };
          var r197cac = this.commonAncestorContainer;
          if (r197cac && r197cac.childNodes) {
            // 收集 CAC 子树全部 Text（文档序 DFS），再按边界过滤。
            var r197texts = [];
            (function r197Collect(node) {
              if (_r197IsText(node)) { r197texts.push(node); return; }
              var kids = node.childNodes || [];
              for (var ci = 0; ci < kids.length; ci++) r197Collect(kids[ci]);
            })(r197cac);
            var r197res = '';
            var r197sp = _r197Pos(r197sc, null, r197cac);
            var r197ep = _r197Pos(r197ec, null, r197cac);
            // R197 fix：路径比较须**数值逐段**——字符串 join(',') 比较在 ≥10 子时
            // 错序（"10,0" < "2" 字典序 true——`10` 的 '1' < '2'），端点外的大索引
            // 子树被误收（WPT Range-stringifier got 含注入 script 文本实证）。
            var _r197PathCmp = function (a, b) {
              if (!a || !b) return 0;
              var n = Math.min(a.length, b.length);
              for (var i = 0; i < n; i++) {
                if (a[i] < b[i]) return -1;
                if (a[i] > b[i]) return 1;
              }
              if (a.length === b.length) return 0;
              return a.length < b.length ? -1 : 1; // 前缀短者在前（祖先先序）
            };
            var _r197HasPrefix = function (path, prefix) {
              if (!path || !prefix || path.length <= prefix.length) return false;
              for (var i = 0; i < prefix.length; i++) if (path[i] !== prefix[i]) return false;
              return true;
            };
            for (var ti = 0; ti < r197texts.length; ti++) {
              var tn = r197texts[ti];
              var tp = _r197Pos(tn, null, r197cac);
              if (!tp) continue;
              // 边界可见性：tp 严格在 (start, end) 开区间内才全收；等于端点容器走切片。
              var spStr = r197sp ? true : null; // R197 fix 后仅作 truthy 判定（path 存在性）
              var afterStart = !r197sp || _r197PathCmp(tp, r197sp) > 0 || _r197HasPrefix(tp, r197sp);
              var beforeEnd = !r197ep || _r197PathCmp(tp, r197ep) < 0 || _r197HasPrefix(tp, r197ep);
              // 后代包含判定：tn 在 start 容器子树内 → 由起始切片处理；在 end 容器子树内 → 尾切片处理。
              var inStartSub = _r197HasPrefix(tp, r197sp);
              var inEndSub = _r197HasPrefix(tp, r197ep);
              if (tn === r197sc || inStartSub) {
                if (tn === r197sc && _r197IsText(r197sc)) {
                  r197res += String(r197sc.nodeValue || '').slice(r197sc === r197ec ? Math.min(this.startOffset, this.endOffset) : this.startOffset);
                } else if (inStartSub && !spStr) {
                  r197res += String(tn.nodeValue || '');
                }
                // inStartSub 且容器是元素：start 之后的部分（容器内 offset 之后）——
                // 常见形态 start=(element,0) 全含 → 收全部子文本。
                else if (inStartSub && spStr) {
                  // 收集 start 容器内 offset 后的文本：容器子序 ≥ startOffset 的子树。
                  var sKids = r197sc.childNodes || [];
                  var sIdx = -1;
                  for (var sk = 0; sk < sKids.length; sk++) if (sKids[sk] === tn || (function () {
                    var sub = sKids[sk], found = false;
                    (function dig(n2) { if (n2 === tn) { found = true; return; } var kk = n2.childNodes || []; for (var jj = 0; jj < kk.length; jj++) dig(kk[jj]); })(sub);
                    return found;
                  })()) { sIdx = sk; break; }
                  if (sIdx >= (this.startOffset | 0)) r197res += String(tn.nodeValue || '');
                }
                continue;
              }
              if (tn === r197ec || inEndSub) {
                if (tn === r197ec && _r197IsText(r197ec) && r197sc !== r197ec) {
                  r197res += String(r197ec.nodeValue || '').slice(0, this.endOffset);
                } else if (inEndSub) {
                  var eKids = r197ec.childNodes || [];
                  var eIdx = -1;
                  for (var ek2 = 0; ek2 < eKids.length; ek2++) if (eKids[ek2] === tn || (function () {
                    var sub2 = eKids[ek2], found2 = false;
                    (function dig2(n3) { if (n3 === tn) { found2 = true; return; } var kk2 = n3.childNodes || []; for (var jj2 = 0; jj2 < kk2.length; jj2++) dig2(kk2[jj2]); })(sub2);
                    return found2;
                  })()) { eIdx = ek2; break; }
                  if (eIdx < (this.endOffset | 0)) r197res += String(tn.nodeValue || '');
                }
                continue;
              }
              // 纯中间节点：afterStart && beforeEnd 才收。
              if (afterStart && beforeEnd) r197res += String(tn.nodeValue || '');
            }
            return r197res;
          }
        }
        // best-effort：取 commonAncestor 子树文本（跨节点偏移不精确截取）。
        if (this.commonAncestorContainer) { var o2 = []; _descendantText(this.commonAncestorContainer, o2); return o2.join(''); }
        return '';
      },
      getBoundingClientRect: function () {
        // R34xx：文本选区几何（经 canvas measure 同一 shaping）——0 基，测试归一化。
        if (typeof _zwRangeClientRects === 'function') {
          var r = _zwRangeClientRects(this);
          if (r !== null) {
            return r.length ? _makeDomRect(r[0].x, r[0].y, r[0].width, r[0].height) : _makeDomRect(0, 0, 0, 0);
          }
        }
        return _makeDomRect(0, 0, 0, 0);
      },
      getClientRects: function () {
        // R34xx：文本选区几何——[start,end) 字形并成行 rect（与 canvas
        // getSelectionRects 行语义一致）；非注册文本 → []（既有）。
        if (typeof _zwRangeClientRects === 'function') {
          var r = _zwRangeClientRects(this);
          if (r !== null) return r;
        }
        return [];
      }
    };
    // R183（js-dom M4）：offset 活性 getter 安装——覆盖字面量的 data 槽（configurable）。
    // selectNode 形态（_mode.kind==='node'）按追踪节点现算 indexOf：移除后节点不在
    // childNodes → 两 offset 同归其插入位置（endOffset 收敛到 startOffset = spec
    // collapse-on-remove 语义；WPT Range-adopt-test 四断言）。其他形态读回写入值。
    // collapse()/setStart/setEnd 写 data 槽并清 _mode（_recalc），getter 随之回落静态值。
    try {
      var _r183IdxOf = function (parent, node) {
        var kids = parent && parent.childNodes;
        if (!kids) return -1;
        for (var _r183i = 0; _r183i < kids.length; _r183i++) if (kids[_r183i] === node) return _r183i;
        return -1;
      };
      // R183：commonAncestorContainer 活性 getter——spec「最近共同祖先容器」：
      // startContainer 的祖先链（含自身）集合，endContainer 上行首个命中。文档级
      // root（documentElement 的 parentNode = document）天然入链（WPT CAC-2 的
      // Detached Range 期望 document）。写路径（_recalc 的 best-effort 值）转入
      // _cacBase 槽；getter 优先现算，非 Node 容器回落 base。
      var _r183Ancestors = function (n) {
        var out = [], hops = 0;
        while (n && hops++ < 128) { out.push(n); n = n.parentNode; }
        return out;
      };
      Object.defineProperty(r183, 'startOffset', {
        configurable: true,
        get: function () {
          if (r183._mode && r183._mode.kind === 'node' && r183._mode.node) {
            // R191：adopt collapse——tracked node 的 ownerDocument 变更（跨文档 adopt）
            // 后 startOffset 读 0（spec live range retarget）。
            if (r183._mode.ownerDoc != null && r183._mode.node.ownerDocument !== r183._mode.ownerDoc) {
              return 0;
            }
            var i = _r183IdxOf(r183.startContainer, r183._mode.node);
            return i >= 0 ? i : (r183._startOffsetBase | 0);
          }
          return r183._startOffsetBase | 0;
        },
        set: function (v) { r183._startOffsetBase = v; },
      });
      Object.defineProperty(r183, 'endOffset', {
        configurable: true,
        get: function () {
          if (r183._mode && r183._mode.kind === 'node' && r183._mode.node) {
            if (r183._mode.ownerDoc != null && r183._mode.node.ownerDocument !== r183._mode.ownerDoc) {
              return 0; // R191：adopt collapse（同 startOffset）
            }
            var i = _r183IdxOf(r183.startContainer, r183._mode.node);
            if (i >= 0) return i + 1;
            return r183._startOffsetBase | 0; // 移除后：collapse 到 start 位（spec）
          }
          return r183._endOffsetBase | 0;
        },
        set: function (v) { r183._endOffsetBase = v; },
      });
      Object.defineProperty(r183, 'commonAncestorContainer', {
        configurable: true,
        get: function () {
          var sc = r183.startContainer, ec = r183.endContainer;
          if (!sc || !ec || typeof sc.nodeType !== 'number' || typeof ec.nodeType !== 'number') {
            return r183._cacBase != null ? r183._cacBase : null;
          }
          var chain = _r183Ancestors(sc);
          var cur = ec, hops = 0;
          while (cur && hops++ < 128) {
            for (var _r183a = 0; _r183a < chain.length; _r183a++) {
              if (chain[_r183a] === cur) return cur;
            }
            cur = cur.parentNode;
          }
          return r183._cacBase != null ? r183._cacBase : null;
        },
        set: function (v) { r183._cacBase = v; },
      });
    } catch (_e183g) {}
    // R260（js-dom M4）：**live-range 注册表**（环形缓冲 8192）——spec
    // `concept-node-replace-data` 的「for each live range whose boundary
    // point is in node, update」要求 CharacterData 变更（deleteData/
    // insertData/replaceData）同步调整活动 range 边界（WPT
    // Range-extractContents「startOffset and endOffset must always be the
    // same after extractContents()」簇：sim 侧在真浏览器经此机制把
    // (text,2→8) 折到 (2→2)，shim deleteData 不调整使双侧恒不折叠）。
    // 超容覆盖最旧（长测试序列里最旧 range 已不可达，调整 dead range
    // 无副作用）。`_zwAdjustRangesForData` 在 part03 的 CharacterData 方法
    // 面消费（跨 part 经 globalThis 交付）。
    try {
      if (!globalThis.__zwLiveRanges) globalThis.__zwLiveRanges = [];
      globalThis.__zwLiveRanges.push(r183);
      if (globalThis.__zwLiveRanges.length > 8192) globalThis.__zwLiveRanges.shift();
    } catch (_eR260lr) {}
    return r183;
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
  // js-dom M4 R42：`new Range()` 返真实 Range 实例（spec Range 有构造器，同 document.createRange()）。
  // 旧空函数 stub → `new Range().setStart` 抛 TypeError（WPT Range-attribute-nodes 等用 new Range()）。
  globalThis.Range = function Range() {
    var r = _makeRange();
    // R289（js-dom M4）：初始边界 (document, 0)——spec Range 构造器「set start to
    // (document, 0), end to (document, 0)」（与 `document.createRange()` 同款，
    // WPT Range-constructor 六断言：startContainer/endContainer === document、
    // offset 0、collapsed、CAC === document）。R183 只在 createRange 落了该初始化，
    // 构造器漏同步——startContainer 恒 null。
    try {
      r.startContainer = globalThis.document;
      r.endContainer = globalThis.document;
      r._startOffsetBase = 0;
      r._endOffsetBase = 0;
    } catch (_e289d) {}
    // R179：实例接 Range.prototype（spec 原型链；WPT node-creation-realm 的
    // `inner.Range.prototype.cloneContents.call(range)` 形态——旧字面量无原型链，
    // prototype 方法 undefined）。
    try { Object.setPrototypeOf(r, globalThis.Range.prototype); } catch (_e179r) {}
    return r;
  };
  // R179：Range.prototype 方法通道——以 _makeRange 产物为模板，把 own 方法挂到
  // prototype（转发语义：this 的同名 own 方法权威）。document.createRange() 与
  // new Range() 共用。
  (function () {
    try {
      var proto179 = globalThis.Range.prototype;
      var template179 = _makeRange();
      for (var k179 in template179) {
        if (typeof template179[k179] === 'function'
            && Object.prototype.hasOwnProperty.call(template179, k179)) {
          (function (name179) {
            if (proto179[name179]) return;
            Object.defineProperty(proto179, name179, {
              value: function () {
                if (this == null || typeof this[name179] !== 'function') {
                  throw new globalThis.TypeError(
                    "Illegal invocation - method '" + name179 + "' called on incompatible receiver");
                }
                return this[name179].apply(this, arguments);
              },
              writable: true, configurable: true, enumerable: false,
            });
          })(k179);
        }
      }
    } catch (_e179p) {}
    // R204（js-dom M4）：Range 的 **how 常量**（spec `dom-range` 接口常量——
    // START_TO_START=0 / START_TO_END=1 / END_TO_END=2 / END_TO_START=3）。缺常量
    // 使 WPT Range-compareBoundaryPoints 的合法性判定（convertedHow 与四常量比对）
    // 全部失配——所有 how 形态（含合法 0-3）都被期望抛 NotSupportedError，4728F
    // 整簇（与 compareBoundaryPoints 的 WebIDL how 转换修复同轮）。
    try {
      Object.defineProperty(globalThis.Range, 'START_TO_START', { value: 0, writable: false, enumerable: true, configurable: false });
      Object.defineProperty(globalThis.Range, 'START_TO_END', { value: 1, writable: false, enumerable: true, configurable: false });
      Object.defineProperty(globalThis.Range, 'END_TO_END', { value: 2, writable: false, enumerable: true, configurable: false });
      Object.defineProperty(globalThis.Range, 'END_TO_START', { value: 3, writable: false, enumerable: true, configurable: false });
    } catch (_eR204c) {}
  })();
  // js-dom M4 R42：`StaticRange` 构造器（spec `dom-staticrange`）——读 RangeInit dict（startContainer/
  // startOffset/endContainer/endOffset），属性 readonly，无 setStart/setEnd 等 mutable 方法。
  // WPT StaticRange-constructor：合法容器（Element/Text/PI/Comment）构造 + collapsed 派生 +
  // 非 Node 容器抛 TypeError。
  // R196（js-dom M4）：补 spec `dom-staticrange-constructor` 的两类校验——
  // ① 容器是 DocumentType/Attr（nodeType 10/2）→ InvalidNodeTypeError（WPT "Throw on
  // DocumentType or Attr container"）；② 必填成员缺失（undefined）→ TypeError（WebIDL
  // dictionary required 成员，WPT "Throw on missing or invalid arguments"——无参/缺
  // startContainer/startOffset/endContainer/endOffset/null 容器七形态）。
  // https://dom.spec.whatwg.org/#dom-staticrange-staticrange
  globalThis.StaticRange = function StaticRange(init) {
    var d = init || {};
    var sc = d.startContainer, ec = d.endContainer;
    var isNode = function (n) { return !!n && typeof n.nodeType === 'number'; };
    if (sc !== undefined && !isNode(sc)) {
      throw new globalThis.TypeError("Failed to construct 'StaticRange': member startContainer is not of type Node.");
    }
    if (ec !== undefined && !isNode(ec)) {
      throw new globalThis.TypeError("Failed to construct 'StaticRange': member endContainer is not of type Node.");
    }
    if (!isNode(sc) || !isNode(ec)) {
      throw new globalThis.TypeError("Failed to construct 'StaticRange': required member startContainer or endContainer is missing.");
    }
    // R196：offset 必填（WebIDL required unsigned long——undefined 时 ToNumber 是 NaN，
    // 真浏览器抛 TypeError；`| 0` 归 0 会静默吞掉）。
    if (d.startOffset === undefined || d.endOffset === undefined) {
      throw new globalThis.TypeError("Failed to construct 'StaticRange': required member startOffset or endOffset is missing.");
    }
    if (sc.nodeType === 10 || sc.nodeType === 2 || ec.nodeType === 10 || ec.nodeType === 2) {
      throw new (globalThis.DOMException || Error)(
        "Failed to construct 'StaticRange': The node is a DocumentType or Attr node.",
        'InvalidNodeTypeError');
    }
    var r = {};
    var so = d.startOffset | 0, eo = d.endOffset | 0;
    Object.defineProperty(r, 'startContainer', { get: function () { return sc; }, configurable: true });
    Object.defineProperty(r, 'startOffset', { get: function () { return so; }, configurable: true });
    Object.defineProperty(r, 'endContainer', { get: function () { return ec; }, configurable: true });
    Object.defineProperty(r, 'endOffset', { get: function () { return eo; }, configurable: true });
    Object.defineProperty(r, 'collapsed', {
      get: function () { return sc === ec && so === eo; },
      configurable: true
    });
    return r;
  };

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

  // R2940 `__zw_report_error(message, source, lineno, colno)`——host 侧未捕获脚本错误报告入口。
  // 宿主（tab_scripts 执行页面 <script> 出错时）调用此 hook，shim 派发 window 'error' 事件 +
  // 调 legacy window.onerror，使 Sentry / analytics / GA 等错误上报库的 hook 触发。
  // **spec 特殊处理**：window.onerror 为 legacy 5-arg 签名 (msg, source, lineno, colno, error)，非 (event)。
  // R2932 将 onerror 注册为 'error' listener（接 event 对象），故：① 先从 listener store 移除 onerror、
  // 用 legacy 签名直接调（返 true → 错误「已处理」，preventDefault 抑制默认动作）；② dispatch 'error'
  // ErrorEvent 到 window（html key）→ window.addEventListener('error') listener 触发（接 ErrorEvent）；
  // ③ 装回 onerror listener。如此 onerror 仅 legacy 调一次，不与 event 派发重复触发。
  // https://html.spec.whatwg.org/#runtime-script-errors
  globalThis.__zw_report_error = function (message, source, lineno, colno) {
    try {
      var msg = String(message == null ? '' : message);
      var src = String(source == null ? '' : source);
      var line = Number(lineno) || 0;
      var col = Number(colno) || 0;
      var ev = new ErrorEvent('error', {
        message: msg, filename: src, lineno: line, colno: col, error: null,
        bubbles: false, cancelable: true, composed: false,
      });
      var onErrFn = _winOnHandlers['error'];
      if (typeof onErrFn === 'function') {
        _globalRemoveEventListener('error', onErrFn); // 暂移除，避免 dispatch 时 onerror 被 event 形式二次触发
        try {
          if (onErrFn.call(globalThis, msg, src, line, col, null) === true) {
            ev.preventDefault(); // onerror 返 true → 错误已处理（spec：抑制默认动作，cancelable:true 故生效）
          }
        } catch (_e) {}
      }
      _dispatchToListeners(_elKey('html', null), ev, 'all', globalThis);
      if (typeof onErrFn === 'function') {
        _globalAddEventListener('error', onErrFn); // 装回
      }
    } catch (_e) {}
  };

  // R2943/R2944 `__zw_dispatch_element_event(tag, attr, absUrl, type)`——host 侧元素级 load/error 派发入口
  //（img R2943 / link R2944 / script R2944 通用）。按 URL 绝对值匹配 `__zw_query_all(tag)` 返的元素 proxy：
  // 读各元素的 `attr` 属性（src/href），相对值经 `__zw_parse_url` 解析为绝对，与 absUrl 比较，命中则用
  // **该元素自身 selector** 经 `__zw_dispatch_event` 派发（保证 listener store key 与 page JS 经
  // querySelectorAll/getElementsByTagName 获取 proxy 时一致 → onload/onerror + addEventListener 触发）。
  // 绕开 selector 歧义（不同取元素方式可能产生不同 selector）。**限制**：仅 DOM 内元素（不含动态创建）。
  globalThis.__zw_dispatch_element_event = function (tag, attr, absUrl, type) {
    try {
      if (typeof __zw_query_all !== 'function' || typeof __zw_dispatch_event !== 'function') return;
      var sels = __zw_query_all(String(tag));
      if (!sels) return;
      var list = sels.split('|').filter(Boolean);
      var pageUrl = typeof __zw_get_page_url === 'function' ? __zw_get_page_url() : '';
      var target = String(absUrl == null ? '' : absUrl);
      for (var i = 0; i < list.length; i++) {
        var sel = list[i];
        var rawVal = typeof __zw_get_attr === 'function' ? __zw_get_attr(sel, attr) : '';
        if (!rawVal) continue;
        var resolved = rawVal;
        // 相对 URL 解析为绝对（与 host 的 absUrl 同源同 base，url crate 规范化一致）。
        if (pageUrl && rawVal.indexOf('://') < 0 && typeof __zw_parse_url === 'function') {
          try {
            var parsed = JSON.parse(__zw_parse_url(rawVal, pageUrl));
            if (parsed && parsed.href) resolved = parsed.href;
          } catch (_e) {}
        }
        if (resolved === target) {
          __zw_dispatch_event(sel, type, null); // 元素自身 selector → listener key 匹配
        }
      }
    } catch (_e) {}
  };
  // FR-009：提交资源最终状态，并按元素种类派发 non-bubbling/non-cancelable 事件。
  // https://html.spec.whatwg.org/multipage/embedded-content.html#updating-the-image-data
  // https://html.spec.whatwg.org/multipage/media.html#concept-media-load-resource
  // media-elements M2：audio/video settle 成功时派 media 专有加载事件序列（headless 近似
  // 驱动——无真解码，元数据/数据即刻「可用」，事件按 spec 序同步派发）：
  //   loadstart（networkState=LOADING）→ progress（仍 LOADING）→ durationchange + loadedmetadata
  //   （readyState HAVE_METADATA，duration 定值）→ loadeddata（HAVE_CURRENT_DATA）→ canplay
  //   （HAVE_FUTURE_DATA）→ canplaythrough（HAVE_ENOUGH_DATA）；autoplay 属性存在则续派
  //   play → playing（spec autoplay 面加载完成后自动播放）。
  // https://html.spec.whatwg.org/multipage/media.html#media-elements-processing-model
  function _zwMediaFire(sel, handle, key, type) {
    try {
      if (typeof _makeEvent !== 'function') return;
      var ev = _makeEvent(type, { bubbles: false, cancelable: false });
      var invoked = false;
      try { invoked = _dispatchWithBubble(key, sel, handle, ev) !== false; } catch (_eMediaD) {}
      // on* 属性 handler 兜底（同 _mediaFireSel——handle-only 元素 on* 错位防护）。
      if (!invoked) {
        var el = (typeof _makeProxy === 'function') ? _makeProxy(sel, handle) : null;
        if (el) {
          var h = el['on' + type];
          if (typeof h === 'function') { try { h.call(el, ev); } catch (_eMediaH) {} }
        }
      }
    } catch (_eMediaEv) {}
  }
  function _zwMediaAutoplay(sel, handle, key) {
    var hasAutoplay = false;
    try {
      hasAutoplay = handle
        ? __zw_has_attr_handle(handle, 'autoplay') === '1'
        : (typeof __zw_has_attr === 'function') && __zw_has_attr(sel, 'autoplay') === '1';
    } catch (_eAuto) {}
    if (!hasAutoplay) return;
    var ms = _mediaState[key] || (_mediaState[key] = {});
    if (ms.playing) return;
    ms.playing = true;
    _zwMediaFire(sel, handle, key, 'play');
    _zwMediaFire(sel, handle, key, 'playing');
    _zwMediaFire(sel, handle, key, 'timeupdate'); // 播放推进首帧（event_timeupdate* 断言面）
  }
  function _zwMediaLoadSequence(sel, handle, key, tag) {
    var ms = _mediaState[key] || (_mediaState[key] = {});
    ms.networkState = 2; // NETWORK_LOADING——loadstart/progress 期间断言面
    _zwMediaFire(sel, handle, key, 'loadstart');
    _zwMediaFire(sel, handle, key, 'progress');
    ms.readyState = 1;
    // R3937：HAVE_NOTHING 期挂起的 seek → 元数据就绪即补跑 seek 算法（seeking +
    // seeked 异步回落 + cue active 面同步；track-cues-seeking 的 onseeked 计数链）。
    // 幂等：_zwSeekDeferred 单次消费。
    if (ms._zwSeekDeferred) {
      delete ms._zwSeekDeferred;
      ms.seeking = true;
      _zwMediaFire(sel, handle, key, 'seeking');
      _zwMediaFire(sel, handle, key, 'timeupdate');
      var _deferredSeeked = function () {
        var _dsMs = _mediaState[key];
        if (!_dsMs) return;
        _dsMs.seeking = false;
        _zwMediaFire(sel, handle, key, 'timeupdate');
        _zwMediaFire(sel, handle, key, 'seeked');
        if (typeof globalThis._zwMediaSeekSync === 'function') {
          try { globalThis._zwMediaSeekSync(key); } catch (_eDfSk) {}
        }
      };
      if (typeof setTimeout === 'function') setTimeout(_deferredSeeked, 0);
      else _deferredSeeked();
    }
    // media-playback M2a：容器时长真值优先（宿主解码器头部读取，经 _resourceStates
    // .durationMs 传入）；无真值（非 webm/探针失败）回落 headless 定值 600（无真解码；
    // 用例只断言可写/类型面）。
    var _settled = _resourceStates[key];
    if (ms.duration == null) {
      // 真值链传入毫秒 → spec 秒（dom-media-duration）；headless 定值 600 保持
      // 既有单位（历史近似值，用例只断言类型/可写面，不回改——零回归）。
      ms.duration = (_settled && _settled.durationMs != null) ? _settled.durationMs / 1000 : 600;
    }
    _zwMediaFire(sel, handle, key, 'durationchange');
    // spec resize：视频尺寸变为已知时派发（audio 无此事件）——时序在 durationchange 后、
    // loadedmetadata 前（event_order_durationchange_resize_loadedmetadata 断言面）。
    if (tag === 'video') _zwMediaFire(sel, handle, key, 'resize');
    _zwMediaFire(sel, handle, key, 'loadedmetadata');
    ms.readyState = 2;
    _zwMediaFire(sel, handle, key, 'loadeddata');
    ms.readyState = 3;
    _zwMediaFire(sel, handle, key, 'canplay');
    ms.readyState = 4;
    _zwMediaFire(sel, handle, key, 'canplaythrough');
    ms.networkState = 1; // NETWORK_IDLE——加载完成无错误（spec networkState 稳态）
    // M3 扩批 XI：suspend——「once the entire media resource has been fetched」
    //（headless：加载序列完成即全量已取；load-removes-queued-error-event 断言面）。
    // https://html.spec.whatwg.org/multipage/media.html#event-media-suspend
    _zwMediaFire(sel, handle, key, 'suspend');
    _zwMediaAutoplay(sel, handle, key);
  }
  // M2：动态 `.src=` 设置的 headless 加载模拟——runner/生产页脚本设 src 后宿主未必有
  // media fetch 通路（testharness 页面无 async_load 媒体抓取），由 shim 侧 setTimeout(0)
  // 提交状态并派事件序列（幂等：_resourceStates 已有则跳过）。handle/sel 双身份——
  // detached createElement 元素为 handle-only，key 须含 handle（_elKey(sel, handle)）。
  // media-elements M3 扩批 XI（resource-selection 族）：media load 算法同步段「await a
  // stable state」——invoke（资源选择启动）后阻塞期间 networkState = NETWORK_NO_SOURCE(3)；
  // 稳定态续段找不到候选（无 src 且无 source 子）→ NETWORK_EMPTY(0)。
  // runner/生产页面稳定态 = 当前脚本任务末（V8 perform_microtask_checkpoint）——
  // queueMicrotask 续段使「同任务内 NO_SOURCE / 下个 <script> 已 EMPTY」断言面成立。
  // 仅翻转 networkState，不派事件（无加载发生；_zwMediaScheduleLoad 负责真加载面）。
  // https://html.spec.whatwg.org/multipage/media.html#concept-media-load-algorithm
  // （synchronous section：await a stable state → 「If no candidate... set networkState
  // to NETWORK_EMPTY and return」）
  globalThis._zwMediaResourceSelect = function (sel, handle, key) {
    var _rsMs = _mediaState[key] || (_mediaState[key] = {});
    _rsMs.networkState = 3; // NETWORK_NO_SOURCE——同步段阻塞期间
    if (typeof queueMicrotask !== 'function') return;
    queueMicrotask(function () {
      var _rsPost = _mediaState[key];
      if (!_rsPost || _rsPost.networkState !== 3) return; // 已被后续加载序列/error settle 推进
      var _hasSrc = false;
      try {
        _hasSrc = handle ? __zw_has_attr_handle(handle, 'src') === '1'
          : ((typeof __zw_has_attr_lw === 'function' ? __zw_has_attr_lw(sel, 'src') : __zw_has_attr(sel, 'src')) === '1');
      } catch (_eRsH) {}
      var _hasSource = false;
      try {
        var _rsKids = (handle && _handleChildren[handle]) ? _handleChildren[handle]
          : (typeof _childNodeList === 'function' ? _childNodeList(sel, handle) : []);
        for (var _rsi = 0; _rsi < _rsKids.length; _rsi++) {
          var _rsKid = _rsKids[_rsi];
          if (_rsKid && _rsKid.nodeType === 1 && String(_rsKid.tagName || '').toLowerCase() === 'source') { _hasSource = true; break; }
        }
      } catch (_eRsK) {}
      if (!_hasSrc && !_hasSource) _rsPost.networkState = 0; // 无候选 → NETWORK_EMPTY
      else _rsPost.networkState = 2; // 有候选 → fetch 启动（headless：LOADING 面）
    });
  };
  function _zwMediaScheduleLoad(sel, handle, tag, absUrl, isEmptySrc, sourceChild) {
    var key = _elKey(sel, handle);
    if (typeof setTimeout !== 'function') return;
    var ms = _mediaState[key] || (_mediaState[key] = {});
    // M3 扩批 XI（spec dom-media-load 同步段）：media load invoke → 播放中止——paused
    // 置 true（同步面）、pending play promises reject AbortError、timeupdate + pause
    //（「If paused is false... fire timeupdate, fire pause」）。set-src-networkState
    // 断言面（play() 后 setAttribute('src') → paused === true）。
    // https://html.spec.whatwg.org/multipage/media.html#dom-media-load
    if (ms.playing) {
      ms.playing = false;
      if (ms.playPromise) {
        var _ldEntry = ms.playPromise;
        delete ms.playPromise;
        try { _ldEntry.reject(new (globalThis.DOMException || Error)('The play() request was interrupted by a call to load().', 'AbortError')); } catch (_eLdR) {}
      }
      _zwMediaFire(sel, handle, key, 'timeupdate');
      _zwMediaFire(sel, handle, key, 'pause');
    }
    ms.lastSourceChild = sourceChild || null; // load() 重调度时恢复 source 候选身份
    // M3 扩批 XI：load() 纪元——spec dom-media-load「queued tasks and pending events 被
    // 丢弃」——loadstart handler 内的 load() 使本续段余下步骤（候选 error settle）作废
    //（load-removes-queued-error-event 断言 [loadstart, loadstart, error] 序）。
    ms.loadEpoch = (ms.loadEpoch || 0) + 1;
    var _ldMyEpoch = ms.loadEpoch;
    // M3 扩批 XI：加载触发同置资源选择同步段语义（invoke 后同步 NO_SOURCE）。
    if (typeof globalThis._zwMediaResourceSelect === 'function') {
      globalThis._zwMediaResourceSelect(sel, handle, key);
    }
    // M3 扩批 XI：settle 触发时**重验候选**（spec 同步段「await a stable state」后续段——
    // stable state 前候选被移除（removeAttribute('src') / source 子被移除）→ 加载中断，
    // 不派 loadstart/不 settle；resource-selection-remove-src / -remove-source /
    // -resumes-onload 断言面）。host setTimeout 为真实线程投递（约 0ms，与后续脚本
    // 执行竞态），重验是唯一可靠的取消点。
    var _stillCandidate = function () {
      try {
        if (sourceChild != null) {
          // source 子候选：仍须是本 media 父的直接子（被移除 → 候选失效）
          var _scKids = (handle && _handleChildren[handle]) ? _handleChildren[handle]
            : (typeof _childNodeList === 'function' ? _childNodeList(sel, handle) : []);
          var _found = false;
          for (var _sci = 0; _sci < _scKids.length; _sci++) {
            if (_scKids[_sci] === sourceChild) { _found = true; break; }
          }
          return _found;
        }
        // 属性**存在性**判定（src="" 是 present-empty——合法失败候选，非「已移除」；
        // R3187 has/get 两段式——get_attr 对缺省与空值均返 ''，须 has_attr 区分）。
        var _present = handle ? __zw_has_attr_handle(handle, 'src') === '1'
          : ((typeof __zw_has_attr_lw === 'function' ? __zw_has_attr_lw(sel, 'src') : __zw_has_attr(sel, 'src')) === '1');
        if (!_present) return false; // 属性已移除 → 候选失效
        if (isEmptySrc === true) return true; // 空 src 候选：存在即继续（失败面）
        var _cur = handle ? __zw_get_attr_handle(handle, 'src') : (typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(sel, 'src') : __zw_get_attr(sel, 'src'));
        _cur = String(_cur == null ? '' : _cur).replace(/^[\x00-\x20]+/, '').replace(/[\x00-\x20]+$/, '');
        var _curAbs = _cur;
        try { if (typeof _zwResolveFetchUrl === 'function') _curAbs = _zwResolveFetchUrl(_cur); } catch (_eRsC) {}
        return _curAbs === absUrl; // 属性被改成其它 URL → 旧加载作废
      } catch (_eRsC2) { return true; }
    };
    // 空 src（剥离 C0/space 后 ''）→ spec「empty src attribute」：资源选择失败——loadstart
    // 后派 error（code=MEDIA_ERR_SRC_NOT_SUPPORTED，error IDL 面置 MediaError 实例），
    // networkState 复位 NETWORK_EMPTY、不提交资源状态（currentSrc 恒 ''）。
    // https://html.spec.whatwg.org/multipage/media.html#concept-media-load-algorithm（failed
    // with attribute 阶段：error 事件排队 + 「set the error code to MEDIA_ERR_SRC_NOT_SUPPORTED」）。
    // M3 扩批 XI：续段走 microtask（queueMicrotask）——与资源选择续段同一检查点排空，
    // 「loadstart 先于 window load」与同步段候选重验均确定性成立（host 真实线程定时器
    // 与后续脚本执行竞态，曾致 invoke-set-src 族非确定性假失败）。
    var _deferCont = function (fn) {
      if (typeof queueMicrotask === 'function') queueMicrotask(fn);
      else setTimeout(fn, 0);
    };
    if (isEmptySrc === true) {
      _deferCont(function () {
        if (!_stillCandidate()) return; // stable state 前候选已移除 → 中断
        var ms = _mediaState[key] || (_mediaState[key] = {});
        ms.networkState = 3; // NO_SOURCE——loadstart 同步段（error 在其后排队）
        _zwMediaFire(sel, handle, key, 'loadstart');
        if ((ms.loadEpoch || 0) !== _ldMyEpoch) return; // load() 已重调度 → 余下作废
        if (sourceChild != null) {
          // source 子候选失败：error 派在 source 元素上（spec「failed to find a candidate
          // resource」——候选 source 触发 onerror，父级 error 仅在全候选耗尽后面）。
          _zwSettleResourceKey(_elKey(sourceChild.__zwSelector || null, sourceChild.__zwHandle || null), sourceChild.__zwSelector || null, sourceChild.__zwHandle || null, 'source', '', 'error', 0, 0, 4);
          delete _resourceStates[key]; // 父级无状态（等待下一候选）
        } else {
          _zwSettleResourceKey(key, sel, handle, tag, '', 'error', 0, 0, 4);
        }
      });
      return;
    }
    _deferCont(function () {
      if (!_stillCandidate()) return; // 同上——stable state 前候选失效
      // media-elements M3 扩批 VIII：about:（about:blank 等）非空 src——spec 资源获取
      // 面不产出可播媒体资源（not a supported media container）→ 资源选择失败路径
      //（error 事件 + code 4，同空 src 面；video_crash_empty_src 断言 error 到达不 crash）。
      // https://html.spec.whatwg.org/multipage/media.html#concept-media-load-algorithm
      if (String(absUrl).indexOf('about:') === 0) {
        _zwSettleResourceKey(key, sel, handle, tag, '', 'error', 0, 0, 4);
        return;
      }
      _zwSettleResourceKey(key, sel, handle, tag, absUrl, 'loaded', 0, 0);
    });
  }
  function _zwSettleResourceSelector(sel, tag, url, outcome, width, height, durationMs) {
    return _zwSettleResourceKey(_elKey(sel, null), sel, null, tag, url, outcome, width, height, undefined, durationMs);
  }
  // media-elements M3 扩批 XII：data:text/vtt 最小解析 + track 资源 settle（headless
  // 近似——runner/生产页 track.src 动态设置无宿主抓取通路）。解析产物填入 track 元素
  // 关联 TextTrack 的 cue 列表并派 load 事件（TextTrackCue parsed-cue 子测断言面：
  // startTime/endTime/id/text 值面）。WEBVTT 最小面：BOM/头行 → cue 块（可选 id 行 +
  // 时间行「t1 --> t2」+ 文本行）；时间 HH:MM:SS.mmm / MM:SS.mmm。
  // https://w3c.github.io/webvtt/#webvtt-parser
  globalThis._zwParseVttDataUrl = function (url) {
    try {
      var marker = 'data:text/vtt,';
      var idx = String(url).indexOf(marker);
      if (idx < 0) return null;
      var raw = decodeURIComponent(String(url).slice(idx + marker.length));
      var lines = raw.replace(/\r\n/g, '\n').replace(/\r/g, '\n').split('\n');
      var i = 0;
      // 跳 WEBVTT 头（允许 BOM 前缀）。
      while (i < lines.length && String(lines[i]).replace(/^﻿/, '').indexOf('WEBVTT') !== 0) i++;
      i++;
      var cues = [];
      var _tParse = function (s) {
        var m = /(?:(\d+):)?(\d{1,2}):(\d{2})\.(\d{3})/.exec(String(s));
        if (!m) return NaN;
        return (m[1] ? Number(m[1]) * 3600 : 0) + Number(m[2]) * 60 + Number(m[3]) + Number(m[4]) / 1000;
      };
      while (i < lines.length) {
        var line = String(lines[i]);
        if (line === '') { i++; continue; }
        var tm = line.indexOf('-->');
        var cueId = '';
        if (tm < 0) {
          // 可能是 cue id 行——下一行应为时间行。
          cueId = line;
          i++;
          line = String(lines[i] == null ? '' : lines[i]);
          tm = line.indexOf('-->');
          if (tm < 0) { continue; }
        }
        var parts = line.split('-->');
        var st = _tParse(parts[0]);
        var et = _tParse(parts[1]);
        if (isNaN(st) || isNaN(et)) { i++; continue; }
        i++;
        var textLines = [];
        while (i < lines.length && String(lines[i]) !== '') {
          textLines.push(String(lines[i]));
          i++;
        }
        var cue = new globalThis.VTTCue(st, et, textLines.join('\n'));
        if (cueId) cue.id = cueId;
        cues.push(cue);
      }
      return cues;
    } catch (_eVtt) { return null; }
  };
  // M3 扩批 XV（2026-09-02）：WebVTT 文件解析深化（http VTT 加载用——
  // track-webvtt-* 断言族）。`_zwParseVtt(text)` → VTTCue 数组；**非 WEBVTT 头返
  // null**（调度层落 error settle）。相对 `_zwParseVttDataUrl` 的语义增补：
  // ① header 校验（BOM 剥离后以 'WEBVTT' 开头——'AWEBVTT'/'rubbish' 拒收，
  //    magic-header/no-webvtt 断言面）；
  // ② cue id 行含 '-->' 不识别为 id（cue-id-error 断言面——该行作 timings 行参与
  //    解析失败 cue 丢弃）；
  // ③ cue settings（time 行尾 `align:start position:20% line:15% vertical:rl`——
  //    settings.vtt / entities.vtt 断言面；数值 % 剥 % 号，'start'/'center'/'end'
  //    文本原样）；
  // ④ 实体解码（&amp; &lt; &gt; &lrm; &rlm; &nbsp;——entities.vtt 断言面）；
  // ⑤ 时间行解析容错（小时位缺省/超 24h 小时、空格变体——timings-hour 面）。
  globalThis._zwParseVtt = function (text) {
    try {
      var raw = String(text == null ? '' : text)
        .replace(/^﻿/, '').replace(/\r\n/g, '\n').replace(/\r/g, '\n');
      var lines = raw.split('\n');
      var i = 0;
      // header 校验：首个非空行必须以 'WEBVTT' 开头（spec webvtt-file-header-check）。
      while (i < lines.length && lines[i] === '') i++;
      if (i >= lines.length || lines[i].indexOf('WEBVTT') !== 0) return null;
      i++;
      // header 尾（空行或 header 注释/元数据行直到空行）——跳到首个空行后。
      while (i < lines.length && lines[i] !== '') i++;
      while (i < lines.length && lines[i] === '') i++;
      var cues = [];
      var _tParse = function (s) {
        s = String(s).trim();
        // spec webvtt timestamp：mm ∈ [00,59]、ss ∈ [00,59]（timings-hour-error 的
        // '00:120:00.500' 非法——宽松正则会把 '12' 错配成时位）。'\d{2}' 前导零语义：
        // '00'~'59' 之外拒收。
        var m = /^(?:(\d+):)?([0-5]\d):([0-5]\d)\.(\d{3})$/.exec(s);
        if (!m) return NaN;
        return (m[1] ? Number(m[1]) * 3600 : 0) + Number(m[2]) * 60 + Number(m[3]) + Number(m[4]) / 1000;
      };
      var _entities = function (s) {
        return String(s)
          .replace(/&amp;/g, '&').replace(/&lt;/g, '<').replace(/&gt;/g, '>')
          .replace(/&lrm;/g, '‎').replace(/&rlm;/g, '‏').replace(/&nbsp;/g, ' ');
      };
      // entities-wrong 断言面（spec cue text `<` 起始 tag/annotation 解析）：**原始**
      // '<' 开启 tag/annotation；tag 终点 = 原始 `>` / '&gt;'（实体解码出的 '>'——
      // 含实体本体一起吞）/ 其余原始 '&'，三者最近者。实体解码后 '&lt;' 产生的 '<'
      // 不是 tag 起点（entities.vtt 正例保持全文）；tag 前文本保留、tag 段全弃
      //（headless 不建 cue DOM 树）。裸 '>' 无 '<' → 纯文本。
      var _stripMarkup = function (s) {
        // 返回**原文**（实体不解码——cue.text 保持 parser 输入，spec；解码在
        // getCueAsHTML DOM 面）。终点判定（上游 Chromium 实测断言面）：原始 '>' →
        // tag 终点，后续文本保留；无原始 '>'（含 '&gt;' 实体形态——实体不终止 tag）
        // → '<' 起剩余全吞。
        var raw = String(s);
        var lt = raw.indexOf('<');
        if (lt < 0) return raw;
        var rawGt = raw.indexOf('>', lt + 1);
        if (rawGt < 0) return raw.slice(0, lt);
        return raw.slice(0, lt) + raw.slice(rawGt + 1);
      };
      var _applySettings = function (cue, s) {
        // cue settings：空格/制表分隔的 `name:value` 段（值可含 %）。已识别项写入
        // VTTCue 反射面；未识别项忽略（spec 容错）。
        // settings-bad-separation 面：**name 大小写敏感**（'Vertical:lr' 非法忽略——
        // 不命中即跳过）、`<align:end>` 尖括号包裹段非法（'<' 前缀 token 整段跳过）、
        // 裸 '-'/'/'/'|' 等无 ':' token 跳过。
        var toks = String(s).split(/[\t ]+/);
        for (var k = 0; k < toks.length; k++) {
          var tok = toks[k];
          if (!tok) continue;
          var ci = tok.indexOf(':');
          if (ci < 0) continue;
          if (tok.indexOf('<') === 0) continue; // 尖括号包裹段——非 settings（容错）
          var name = tok.slice(0, ci);
          var val = tok.slice(ci + 1);
          try {
            if (name === 'align') { cue.align = val; }
            else if (name === 'vertical') { cue.vertical = val; }
            else if (name === 'line' || name === 'position') {
              var nv = (val === 'auto') ? 'auto' : Number(String(val).replace('%', ''));
              if (nv === nv) cue[name] = nv; // NaN 不写（保留缺省）
            } else if (name === 'size') {
              var sz = Number(String(val).replace('%', ''));
              if (sz === sz) cue.size = sz;
            }
          } catch (_eVs) {}
        }
      };
      while (i < lines.length) {
        var line = lines[i];
        if (line === '') { i++; continue; }
        var tm = line.indexOf('-->');
        var cueId = '';
        if (tm < 0) {
          // 候选 id 行：**含 '-->' 不识别为 id**（cue-id-error 断言面——'-->random_id'
          // 这样的行自身就是坏 timings 行，cue 块整体丢弃）；不含 '-->' 且下一行是
          // timings 行时才作 id 候选。
          if (lines[i + 1] != null && lines[i + 1].indexOf('-->') >= 0) {
            cueId = line;
            i++;
            line = lines[i];
            tm = line.indexOf('-->');
          } else {
            i++;
            continue;
          }
        }
        // timings 行：**只按前两个 '-->' 分割**（settings-bad-separation 面 2——
        // settings 里可含字面 '-->'（` --> position:50% ...`），多余的 '-->' 属
        // settings 段）。
        var _fst = line.indexOf('-->');
        var _snd = line.indexOf('-->', _fst + 3);
        var parts = [line.substring(0, _fst), line.substring(_fst + 3, _snd < 0 ? line.length : _snd), _snd < 0 ? '' : line.substring(_snd + 3)];
        var st = _tParse(parts[0]);
        // timings 行尾 settings：第二个 --> 段内 `HH:MM:SS.mmm` 之后的部分 + 第三段
        //（若 settings 含字面 '-->' 则落在第三段——拼接回 settings）。
        var rest = (parts.length > 1 ? parts[1] : '')
          + (parts[2] ? '-->' + parts[2] : '');
        var _tMatch = /(\d+(?::\d+)*\.\d{3})/.exec(rest);
        var et = _tMatch ? _tParse(_tMatch[1]) : NaN;
        var settings = _tMatch ? rest.slice(_tMatch.index + _tMatch[0].length) : '';
        if (isNaN(st) || isNaN(et)) {
          // 坏 timings 行：仅跳过该行（spec「cue 时戳解析失败 → 忽略该 cue」——后续行
          // 可重新作为 cue 块起点。cue-id-error 断言面：'-->random_id' 坏行后的
          // '00:00:00.000 --> 00:00:30.500' 起新 cue——3 条全解析）。
          i++;
          continue;
        }
        i++;
        var textLines = [];
        while (i < lines.length && lines[i] !== '') {
          // blank-lines 面（cues-no-separation 断言——3 条 cue 期望）：cue 文本行在
          // 空行处结束；文本行内出现**新 timings 行**（含 '-->' 且时间可解析）时结束
          // 本 cue 开新 cue（spec cue text 不含 timings 形态）；裸非 timings 行
          //（如无分隔的 id 行 '2'）并入上一 cue 文本（「treated like one big cue」）。
          if (lines[i].indexOf('-->') >= 0
              && !isNaN(_tParse(String(lines[i]).split('-->')[0]))) {
            break;
          }
          textLines.push(String(lines[i]));
          i++;
        }
        // tag/annotation 截断与实体解码按 **cue 全文本**（跨行——entities-wrong 的
        // '<' tag 从首行延续到 '&' 结束，跨行吞并）。
        var cue = new globalThis.VTTCue(st, et, _stripMarkup(textLines.join('\n')));
        if (cueId) cue.id = _entities(cueId);
        if (settings) _applySettings(cue, settings);
        cues.push(cue);
      }
      return cues;
    } catch (_eVttX) { return null; }
  };
  // M3 扩批 XII：media 元素的所有 track 子 → 触发检索（_zwTrackScheduleLoad 幂等）。
  // spec：track 检索随 media 元素加载循环启动（connected 时——cues 可用性 gate 开面）。
  globalThis._zwScheduleChildTrackLoads = function (sel, handle) {
    try {
      var _sckKids = (handle && _handleChildren[handle]) ? _handleChildren[handle]
        : (typeof _childNodeList === 'function' ? _childNodeList(sel, handle) : []);
      for (var i = 0; i < _sckKids.length; i++) {
        var k = _sckKids[i];
        if (!k || k.nodeType !== 1) continue;
        var tag = String(k.tagName || '').toUpperCase();
        if (tag !== 'TRACK') continue;
        // M3 扩批 XV：mode gate——无 default 属性的 track 子 TextTrack mode 缺省
        // 'disabled'，**不自动加载**（track-default-attribute 断言「只有 default track
        // 派 onload」）。M3 扩批 XVI 实证：track-cues-* 播放推进族的 track 子均带
        // `default` 属性——kind gate（metadata 排除）非必需，恢复 XV default gate
        //（非 default caption track 自动加载会使 track-default-attribute 断言
        // 「onload 只派在 default track」失败）。
        var _sclDefault = false;
        try {
          _sclDefault = k.__zwHandle
            ? __zw_has_attr_handle(k.__zwHandle, 'default') === '1'
            : (typeof __zw_has_attr_lw === 'function' ? __zw_has_attr_lw(k.__zwSelector, 'default') : __zw_has_attr(k.__zwSelector, 'default')) === '1';
        } catch (_eSclD) {}
        if (!_sclDefault) continue;
        if (typeof globalThis._zwTrackScheduleLoad === 'function') {
          globalThis._zwTrackScheduleLoad(k.__zwSelector || null, k.__zwHandle || null);
        }
      }
    } catch (_eScl) {}
  };
  // M3 扩批 XV：track 元素视角的调度入口——父 media 元素（如有）的全部 track 子补触发
  //（静态 HTML 形态 `track.track` 访问面；幂等）。
  // 全文档面：querySelectorAll('track') 查询触发的兜底——遍历全部 media 元素补跑检索
  //（用例只查询不读 API 的静态形态；幂等——trackScheduled 去重）。
  globalThis._zwScheduleAllTrackLoads = function () {
    try {
      if (typeof __zw_query_all !== 'function') return;
      var _satSels = (__zw_query_all('audio') || '').split('|')
        .concat((__zw_query_all('video') || '').split('|')).filter(Boolean);
      for (var i = 0; i < _satSels.length; i++) {
        if (typeof globalThis._zwScheduleChildTrackLoads === 'function') {
          globalThis._zwScheduleChildTrackLoads(_satSels[i], null);
        }
      }
    } catch (_eSatl) {}
  };
  globalThis._zwScheduleParentTrackLoad = function (sel, handle) {
    try {
      if (typeof _zwParentMediaProxy !== 'function') return;
      var _spParent = _zwParentMediaProxy(sel, handle);
      if (!_spParent) return;
      if (typeof globalThis._zwScheduleChildTrackLoads === 'function') {
        globalThis._zwScheduleChildTrackLoads(_spParent.__zwSelector || null, _spParent.__zwHandle || null);
      }
    } catch (_eSptl) {}
  };
  // media-audio M3 切片 2（D1 批复 / D-WA-2 NullSink 先行）：Web Audio 最小面——
  // AudioContext 构造器 + createOscillator/createGain + destination + start/stop。
  // 宿主桥 `__zwWA*` 回调族（tab_worker/webview 构建时注入，register_webaudio_bridge_
  // callbacks 同款字符串契约）未注册时构造仍成功、节点对象语义面完整（属性反射 +
  // connect 链式），仅不产声（headless 近似——RFC §3.2 简化记录）。state 恒 'running'
  //（无 autoplay 政策域——suspended 归用户手势策略，不模拟）。
  // https://webaudio.github.io/web-audio-api/#AudioContext
  var _zwWASeq = 0;
  function _zwWANode(kind, ctxId, handle) {
    var node = Object.create(_zwWANode.prototype);
    node._zwKind = kind;
    node._zwCtx = ctxId;
    node._zwHandle = handle;
    node._zwConnected = null;
    // AudioNode 接口面（WPT audionode/destination 断言——最小面静态拓扑）：
    // 源类节点（oscillator）0 入 1 出；处理类节点（gain/stereopanner/delay/
    // biquadfilter/analyser）与 destination 1 入 1 出（ctor-gain/ctor-analyser
    // testDefaultConstructor numberOfInputs:1 断言面）；
    // channelCount 2（destination 缺省立体声，maxChannelCount ≥ 2）。
    node._zwInputs = (kind === 'oscillator') ? 0 : 1;
    node._zwOutputs = 1;
    node._zwChannelCount = 2;
    node._zwMaxChannelCount = 32;
    return node;
  }
  Object.defineProperty(_zwWANode.prototype, 'numberOfInputs', {
    get: function () { return this._zwInputs; },
    configurable: true,
  });
  Object.defineProperty(_zwWANode.prototype, 'numberOfOutputs', {
    get: function () { return this._zwOutputs; },
    configurable: true,
  });
  Object.defineProperty(_zwWANode.prototype, 'channelCount', {
    get: function () { return this._zwChannelCount; },
    set: function (v) {
      // spec AudioNode channelCount setter：0 → NotSupportedError；destination 上
      // > maxChannelCount(32) → IndexSizeError（WPT destination 断言面）；非
      // destination 节点仅 0 抛（testAudioNodeOptions {channelCount:17} 可写面）。
      var n = Number(v);
      if (n === 0) {
        throw new (globalThis.DOMException || Error)('channelCount 0 not supported.', 'NotSupportedError');
      }
      // spec AudioNode：channelCount N 夹取 [1, 32]（NC max）——0 → NotSupportedError、
      // > 32 → IndexSizeError（destination 与源节点同界——testAudioNodeOptions
      // {channelCount:99} 抛面；destination maxChannelCount 面同值）。
      if (n > 32) {
        throw new (globalThis.DOMException || Error)(
          'channelCount ' + n + ' exceeds maxChannelCount 32.', 'IndexSizeError');
      }
      this._zwChannelCount = n;
    },
    configurable: true,
  });
  Object.defineProperty(_zwWANode.prototype, 'maxChannelCount', {
    get: function () { return this._zwKind === 'destination' ? this._zwMaxChannelCount : this._zwChannelCount; },
    configurable: true,
  });
  Object.defineProperty(_zwWANode.prototype, 'channelCountMode', {
    get: function () { return this._zwChannelCountMode || 'max'; },
    set: function (v) {
      var s = String(v == null ? '' : v);
      if (s === 'max' || s === 'clamped-max' || s === 'explicit') this._zwChannelCountMode = s;
    },
    configurable: true,
  });
  Object.defineProperty(_zwWANode.prototype, 'channelInterpretation', {
    get: function () { return this._zwChannelInterpretation || 'speakers'; },
    set: function (v) {
      var s = String(v == null ? '' : v);
      if (s === 'speakers' || s === 'discrete') this._zwChannelInterpretation = s;
    },
    configurable: true,
  });
  _zwWANode.prototype.connect = function (target) {
    // spec AudioNode.connect 连接校验：非 AudioNode/AudioParam 目标 → TypeError
    //（WPT audionode「connect() method with illegal values」断言面——0/null 目标）。
    if (!target || (typeof target !== 'object') || (typeof target.connect !== 'function' && target._zwKind !== 'destination' && target._zwKind !== 'audioparam')) {
      throw new TypeError("Failed to execute 'connect' on 'AudioNode': parameter 1 is not of type 'AudioNode'.");
    }
    // spec：connect 返回目标节点（链式——osc.connect(gain).connect(destination)）。
    this._zwConnected = target || null;
    return target;
  };
  _zwWANode.prototype.disconnect = function () {
    this._zwConnected = null;
  };
  function AudioContext(options) {
    // WebIDL：无 new 调用抛 TypeError（构造器语义——同 Audio() 面但不走工厂）。
    if (!(this instanceof AudioContext)) {
      throw new TypeError("Failed to construct 'AudioContext': Please use the 'new' operator, this DOM object constructor cannot be called as a function.");
    }
    var ctxId = ++_zwWASeq;
    var self = this;
    self._zwCtxId = ctxId;
    self._zwBridge = (typeof globalThis.__zw_wa_create_osc === 'function');
    // spec BaseAudioContext.state：headless 恒 'running'（RFC §3.2）。
    self._zwState = 'running';
    self._zwSampleRate = 48000;
    // AudioContextOptions dict（spec §AudioContextOptions——latencyHint enum/
    // double + sampleRate 面；headless baseLatency 固定档：interactive ≈ 5ms）。
    // WPT audiocontextoptions 断言面：合法 latencyHint 构造不抛 + baseLatency
    // ≥ 0 + 无效 enum → TypeError。sampleRate 选项暂不反射（设备面归 CpalSink
    // 切片——上下文采样率固定 48k）。
    if (options !== undefined && options !== null && typeof options !== 'object') {
      // WebIDL dict 面非对象 → TypeError（WPT new AudioContext('latencyHint') 断言面）。
      throw new TypeError("Failed to construct 'AudioContext': The provided value is not of type 'AudioContextOptions'.");
    }
    var o = options || {};
    var _hint = (o.latencyHint !== undefined) ? o.latencyHint : 'interactive';
    var _hintNum = Number(_hint);
    if (typeof _hint === 'string') {
      var _hl = String(_hint).toLowerCase();
      if (_hl !== 'interactive' && _hl !== 'balanced' && _hl !== 'playback') {
        throw new TypeError("Failed to construct 'AudioContext': Failed to read the 'latencyHint' property from 'AudioContextOptions': The provided value '" + _hint + "' is not a valid enum value.");
      }
      self._zwBaseLatency = (_hl === 'interactive') ? 0.005 : (_hl === 'balanced' ? 0.015 : 0.04);
    } else if (isNaN(_hintNum)) {
      throw new TypeError("Failed to construct 'AudioContext': Failed to read the 'latencyHint' property from 'AudioContextOptions': The provided double value is non-finite.");
    } else {
      // double 档：baseLatency = hint 值本身（WPT 断言 high-latency 两上下文相等
      // 且 = 大 hint 值——headless 直接采用，不 clamp）。
      self._zwBaseLatency = Math.max(0, _hintNum);
    }
    // AudioContextOptions.sampleRate（spec：[3000, 768000] 外 → NotSupportedError；
    // 范围内 → 上下文采样率反射——headless 合成面以该率运行）。
    if (o.sampleRate !== undefined) {
      var _sr = Number(o.sampleRate);
      if (isNaN(_sr) || _sr < 3000 || _sr > 768000) {
        throw new (globalThis.DOMException || Error)(
          "Failed to construct 'AudioContext': sampleRate " + o.sampleRate + " is not in range [3000, 768000].", 'NotSupportedError');
      }
      self._zwSampleRate = _sr;
    }
    self._zwDestination = _zwWANode('destination', ctxId, 0);
    self._zwOscs = {};
    // close()（spec BaseAudioContext.close——state → 'closed' + settled Promise；
    // closed 后 suspend/resume reject InvalidStateError）。headless 无音频流，
    // 纯状态面。
    self._zwClose = function () {
      self._zwState = 'closed';
      return Promise.resolve();
    };
    self._zwSuspend = function () {
      if (self._zwState === 'closed') {
        return Promise.reject(new (globalThis.DOMException || Error)('Cannot suspend a closed context.', 'InvalidStateError'));
      }
      self._zwState = 'suspended';
      return Promise.resolve();
    };
    self._zwResume = function () {
      if (self._zwState === 'closed') {
        return Promise.reject(new (globalThis.DOMException || Error)('Cannot resume a closed context.', 'InvalidStateError'));
      }
      self._zwState = 'running';
      return Promise.resolve();
    };
  }
  globalThis.AudioContext = globalThis.AudioContext || AudioContext;
  Object.defineProperty(AudioContext.prototype, 'baseLatency', {
    get: function () { return this._zwBaseLatency == null ? 0.005 : this._zwBaseLatency; },
    configurable: true,
  });
  AudioContext.prototype.close = function () { return this._zwClose ? this._zwClose() : Promise.resolve(); };
  AudioContext.prototype.suspend = function () { return this._zwSuspend ? this._zwSuspend() : Promise.resolve(); };
  AudioContext.prototype.resume = function () { return this._zwResume ? this._zwResume() : Promise.resolve(); };
  // OfflineAudioContext：构造 + 节点工厂兼容面（numberOfWorkers/length/sampleRate
  // 反射；**无离线渲染**——startRendering 返 rejected promise，RFC §0 简化记录）。
  // WPT audionode-connect-return-value 等构造面用例依赖。
  function OfflineAudioContext(numberOfChannels, length, sampleRate) {
    if (!(this instanceof OfflineAudioContext)) {
      throw new TypeError("Failed to construct 'OfflineAudioContext': Please use the 'new' operator, this DOM object constructor cannot be called as a function.");
    }
    var self = this;
    self._zwCtxId = ++_zwWASeq;
    self._zwState = 'suspended';
    self._zwChannels = Number(numberOfChannels) || 1;
    self._zwLength = Number(length) || 0;
    self._zwSampleRate = Number(sampleRate) || 44100;
    self._zwDestination = _zwWANode('destination', self._zwCtxId, 0);
  }
  globalThis.OfflineAudioContext = globalThis.OfflineAudioContext || OfflineAudioContext;
  Object.defineProperty(OfflineAudioContext.prototype, 'state', {
    get: function () { return this._zwState; },
    configurable: true,
  });
  Object.defineProperty(OfflineAudioContext.prototype, 'sampleRate', {
    get: function () { return this._zwSampleRate; },
    configurable: true,
  });
  Object.defineProperty(OfflineAudioContext.prototype, 'length', {
    get: function () { return this._zwLength; },
    configurable: true,
  });
  Object.defineProperty(OfflineAudioContext.prototype, 'destination', {
    get: function () { return this._zwDestination; },
    configurable: true,
  });
  Object.defineProperty(OfflineAudioContext.prototype, 'currentTime', {
    get: function () { return 0; },
    configurable: true,
  });
  // 节点工厂共享（Online/Offline 同面——最小面节点对象与上下文类型无关）。
  // 注：createOscillator/createGain 的 AudioContext.prototype 赋值在本块**之后**
  //（7214 行附近）——此处仅引用函数对象会在赋值前拿到 undefined，工厂共享段
  // 随 createGain 定义点之后补接（见下方 _zwWAShareFactories）。
  // 离线渲染不做（RFC §0）——startRendering 恒 rejected（spec promise 面保留）。
  OfflineAudioContext.prototype.startRendering = function () {
    return Promise.reject(new (globalThis.DOMException || Error)(
      'OfflineAudioContext rendering is not supported in this build.', 'NotSupportedError'));
  };
  Object.defineProperty(AudioContext.prototype, 'state', {
    get: function () { return this._zwState; },
    configurable: true,
  });
  Object.defineProperty(AudioContext.prototype, 'sampleRate', {
    get: function () { return this._zwSampleRate; },
    configurable: true,
  });
  Object.defineProperty(AudioContext.prototype, 'destination', {
    get: function () { return this._zwDestination; },
    configurable: true,
  });
  // BaseAudioContext 面：currentTime（上下文时钟秒——headless 无真音频钟，近似
  // performance.now 换算；真值面挂 audio clock 主时钟承接）。
  Object.defineProperty(AudioContext.prototype, 'currentTime', {
    get: function () {
      try {
        return (typeof __zw_performance_now === 'function') ? __zw_performance_now() / 1000 : 0;
      } catch (_eWact) { return 0; }
    },
    configurable: true,
  });
  AudioContext.prototype.createOscillator = function () {
    var self = this;
    var node = _zwWANode('oscillator', self._zwCtxId, 0);
    var _type = 'sine';
    var _freq = 440;
    var _gain = 1.0;
    var _started = false;
    var _stopped = false;
    // 宿主桥可用时立即建 Rust 侧源（freq/type 变更经桥推）。
    if (self._zwBridge) {
      try { node._zwHandle = Number(globalThis.__zw_wa_create_osc(_type, String(_freq))) || 0; } catch (_eWao) {}
    }
    Object.defineProperty(node, 'type', {
      get: function () { return _type; },
      set: function (v) {
        var s = String(v == null ? '' : v);
        if (s !== 'sine' && s !== 'square' && s !== 'sawtooth' && s !== 'triangle') return;
        _type = s;
      },
      configurable: true,
    });
    // frequency/detune AudioParam（_zwMakeAudioParam 工厂——instanceof AudioParam
    // 面 + 非 finite value TypeError + 调度方法链式；value 变更同步宿主桥 freq）。
    var _freqParam = globalThis._zwMakeAudioParam(_freq);
    Object.defineProperty(_freqParam, 'value', {
      get: function () { return _freq; },
      set: function (v) {
        var n = Number(v);
        if (isNaN(n) || n === Infinity || n === -Infinity) {
          throw new TypeError("Failed to set the 'value' property on 'AudioParam': The provided value is non-finite.");
        }
        _freq = n;
        if (self._zwBridge && typeof globalThis.__zw_wa_set_freq === 'function') {
          try { globalThis.__zw_wa_set_freq(String(node._zwHandle), String(n)); } catch (_eWaf) {}
        }
      },
      configurable: true,
    });
    Object.defineProperty(node, 'frequency', {
      get: function () { return _freqParam; },
      configurable: true,
    });
    // detune AudioParam 占位（值恒 0——cent 偏移归后续切片）。
    var _detuneParam = globalThis._zwMakeAudioParam(0);
    Object.defineProperty(node, 'detune', {
      get: function () { return _detuneParam; },
      configurable: true,
    });
    node.start = function (when) {
      if (_started) return;
      _started = true;
      if (self._zwBridge && typeof globalThis.__zw_wa_start === 'function') {
        try { globalThis.__zw_wa_start(String(node._zwHandle), String(Math.max(0, Number(when) || 0) * 1000)); } catch (_eWas) {}
      }
    };
    node.stop = function (when) {
      if (!_started || _stopped) return;
      _stopped = true;
      if (self._zwBridge && typeof globalThis.__zw_wa_stop === 'function') {
        try { globalThis.__zw_wa_stop(String(node._zwHandle), String(Math.max(0, Number(when) || 0) * 1000)); } catch (_eWax) {}
      }
    };
    node.onended = null;
    return node;
  };
  // createGain：gain AudioParam 值面（per-source 增益经桥的 reserved 面——最小面
  // per-osc gain 在 Rust 侧由 WebAudioContext 源增益承接，桥 set-gain 归设备切片）。
  AudioContext.prototype.createGain = function () {
    var node = _zwWANode('gain', this._zwCtxId, 0);
    var _gainVal = 1.0;
    var _gainParam = globalThis._zwMakeAudioParam(1.0);
    Object.defineProperty(_gainParam, 'value', {
      get: function () { return _gainVal; },
      set: function (v) { _gainVal = Number(v); if (isNaN(_gainVal)) _gainVal = 1.0; },
      configurable: true,
    });
    Object.defineProperty(node, 'gain', {
      get: function () { return _gainParam; },
      configurable: true,
    });
    return node;
  };
  // createPeriodicWave（spec §BaseAudioContext.createPeriodicWave——real/imag
  // sequence<float> 必须同长且逐项有限；非 finite → TypeError——WPT
  // createPeriodicWaveInfiniteValuesThrows 断言面。存储面：PeriodicWave 实例）。
  AudioContext.prototype.createPeriodicWave = function (real, imag) {
    var _toFloat32 = function (src, name) {
      if (src == null || typeof src.length !== 'number') {
        throw new TypeError("Failed to execute 'createPeriodicWave' on 'AudioContext': parameter " + (name === 'real' ? 1 : 2) + " is not of type 'Float32Array'.");
      }
      var out = new Float32Array(src.length);
      for (var i = 0; i < src.length; i++) {
        var n = Number(src[i]);
        if (isNaN(n) || n === Infinity || n === -Infinity) {
          throw new TypeError("Failed to execute 'createPeriodicWave' on 'AudioContext': The provided value is non-finite.");
        }
        out[i] = n;
      }
      return out;
    };
    var r = _toFloat32(real, 'real');
    var im = _toFloat32(imag, 'imag');
    if (r.length !== im.length) {
      throw new (globalThis.DOMException || Error)(
        "Failed to execute 'createPeriodicWave' on 'AudioContext': real and imag arrays must have the same length.", 'IndexSizeError');
    }
    var ctxId = this._zwCtxId;
    return new globalThis.PeriodicWave(this, { real: Array.prototype.slice.call(r), imag: Array.prototype.slice.call(im) });
  };
  // createStereoPanner/createDelay/createBiquadFilter/createAnalyser（headless
  // 语义面——直调 builder 与构造器同对象；createDelay 缺省 maxDelayTime=1.0 spec 档）。
  AudioContext.prototype.createStereoPanner = function () { return _zwWABuildStereoPanner(this); };
  AudioContext.prototype.createDelay = function (maxDelayTime) {
    var opts = (maxDelayTime != null) ? { maxDelayTime: Number(maxDelayTime) } : undefined;
    return _zwWABuildDelay(this, opts);
  };
  AudioContext.prototype.createBiquadFilter = function () { return _zwWABuildBiquadFilter(this); };
  AudioContext.prototype.createAnalyser = function () { return _zwWABuildAnalyser(this); };
  // 节点工厂共享补接（OfflineAudioContext 面在 AudioContext 工厂定义之后——
  // 见上方 OfflineAudioContext 块内注记）。
  if (typeof globalThis.OfflineAudioContext !== 'undefined') {
    OfflineAudioContext.prototype.createOscillator = AudioContext.prototype.createOscillator;
    OfflineAudioContext.prototype.createGain = AudioContext.prototype.createGain;
    OfflineAudioContext.prototype.createStereoPanner = AudioContext.prototype.createStereoPanner;
    OfflineAudioContext.prototype.createDelay = AudioContext.prototype.createDelay;
    OfflineAudioContext.prototype.createBiquadFilter = AudioContext.prototype.createBiquadFilter;
    OfflineAudioContext.prototype.createAnalyser = AudioContext.prototype.createAnalyser;
    OfflineAudioContext.prototype.createPeriodicWave = AudioContext.prototype.createPeriodicWave;
  }
  // AudioParam 接口（spec webaudio §AudioParam——`instanceof AudioParam` 断言面；
  // 最小值面 value + 调度方法 no-op 存储。param 调度真值化归后续切片）。
  function AudioParam() {
    throw new TypeError('Illegal constructor');
  }
  globalThis.AudioParam = globalThis.AudioParam || AudioParam;
  // AudioParam 工厂（createOscillator/createGain 的 frequency/detune/gain 面共用）。
  globalThis._zwMakeAudioParam = function (initialValue) {
    var param = Object.create(globalThis.AudioParam.prototype);
    var _v = Number(initialValue) || 0;
    // spec：value setter 非 finite → TypeError（AudioParam value 面惯例；
    // WPT audioparam-exceptional-values 主断言面——NaN/Inf 拒绝）。
    Object.defineProperty(param, 'value', {
      get: function () { return _v; },
      set: function (v) {
        var n = Number(v);
        if (isNaN(n) || n === Infinity || n === -Infinity) {
          throw new TypeError("Failed to set the 'value' property on 'AudioParam': The provided value is non-finite.");
        }
        _v = n;
      },
      configurable: true,
    });
    // 调度方法参数校验面（spec webaudio §AudioParam 调度方法——WPT
    // audioparam-exceptional-values 断言：value/time 参数非 finite → TypeError；
    // 时间负值 → RangeError；exponentialRamp 0/±1e-100 值 → RangeError；
    // setValueCurve 曲线含非 finite → TypeError、时长 ≤ 0 → RangeError）。
    // https://webaudio.github.io/web-audio-api/#AudioParam
    function _zwFiniteOrThrow(v, method) {
      var n = Number(v);
      if (isNaN(n) || n === Infinity || n === -Infinity) {
        throw new TypeError("Failed to execute '" + method + "' on 'AudioParam': The provided value is non-finite.");
      }
      return n;
    }
    function _zwTimeOrThrow(v, method) {
      var n = _zwFiniteOrThrow(v, method);
      if (n < 0) throw new RangeError("Failed to execute '" + method + "' on 'AudioParam': time must be non-negative.");
      return n;
    }
    param.setValueAtTime = function (v, startTime) {
      this.value = _zwFiniteOrThrow(v, 'setValueAtTime');
      _zwTimeOrThrow(startTime, 'setValueAtTime');
      return this;
    };
    param.linearRampToValueAtTime = function (v, endTime) {
      this.value = _zwFiniteOrThrow(v, 'linearRampToValueAtTime');
      _zwTimeOrThrow(endTime, 'linearRampToValueAtTime');
      return this;
    };
    param.exponentialRampToValueAtTime = function (v, endTime) {
      var n = _zwFiniteOrThrow(v, 'exponentialRampToValueAtTime');
      // spec：exponentialRamp 目标值为 0 或次法线（±1e-100 内，指数运算下溢）→
      // RangeError（WPT [0, -1e-100, 1e-100] 断言面）。
      if (n === 0 || Math.abs(n) <= 1e-100) {
        throw new RangeError("Failed to execute 'exponentialRampToValueAtTime' on 'AudioParam': value must be non-zero and same sign.");
      }
      _zwTimeOrThrow(endTime, 'exponentialRampToValueAtTime');
      return this;
    };
    param.setTargetAtTime = function (v, startTime, timeConstant) {
      this.value = _zwFiniteOrThrow(v, 'setTargetAtTime');
      _zwTimeOrThrow(startTime, 'setTargetAtTime');
      // spec：timeConstant 严格正——负值 → RangeError（WPT (1,1,-1) 断言面）。
      var tc = _zwFiniteOrThrow(timeConstant, 'setTargetAtTime');
      if (tc <= 0) throw new RangeError("Failed to execute 'setTargetAtTime' on 'AudioParam': timeConstant must be strictly positive.");
      return this;
    };
    param.setValueCurveAtTime = function (curve, startTime, duration) {
      // 曲线序列化（WebIDL sequence<float>）后逐项非 finite → TypeError。
      try {
        var arr = Array.prototype.slice.call(curve);
        for (var i = 0; i < arr.length; i++) _zwFiniteOrThrow(arr[i], 'setValueCurveAtTime');
      } catch (_eCurve) {
        if (_eCurve instanceof TypeError) throw _eCurve;
      }
      _zwTimeOrThrow(startTime, 'setValueCurveAtTime');
      // spec：duration 严格正——0/负 → RangeError（WPT (curve,1,0)/(curve,1,-1) 断言面）。
      var d = _zwFiniteOrThrow(duration, 'setValueCurveAtTime');
      if (d <= 0) throw new RangeError("Failed to execute 'setValueCurveAtTime' on 'AudioParam': duration must be strictly positive.");
      return this;
    };
    param.cancelScheduledValues = function () { return this; };
    param.cancelAndHoldAtTime = function (t) { _zwTimeOrThrow(t, 'cancelAndHoldAtTime'); return this; };
    param.defaultValue = Number(initialValue) || 0;
    param.minValue = -3.4028235e38;
    param.maxValue = 3.4028235e38;
    return param;
  };
  // Node 构造器面（spec webaudio §OscillatorNode/GainNode——`new OscillatorNode(ctx,
  // options)` 与 `ctx.createOscillator()` 等价对象；options dict {type, frequency,
  // detune} / {gain}。非法 ctx/无 ctx → TypeError——audionodeoptions.js
  // testInvalidConstructor 断言面（new X() / new X(1) / new X(ctx, 42) 全抛）。
  // AudioNodeOptions dict 面（channelCount/channelCountMode/channelInterpretation）：
  // channelCount 0 → NotSupportedError、>max → IndexSizeError（destination 同款
  // setter 语义——testAudioNodeOptions 断言 {channelCount:17} 可写、0/99 抛）。
  function _zwWANodeCtor(nodeName, ctx, options) {
    if (!(ctx && typeof ctx === 'object' && (ctx._zwCtxId != null || typeof ctx.createGain === 'function'))) {
      throw new TypeError("Failed to construct '" + nodeName + "': parameter 1 is not of type 'BaseAudioContext'.");
    }
    if (options != null && typeof options !== 'object') {
      throw new TypeError("Failed to construct '" + nodeName + "': The provided value is not of type 'object'.");
    }
    var node;
    if (nodeName === 'OscillatorNode') node = ctx.createOscillator();
    else if (nodeName === 'GainNode') node = ctx.createGain();
    // 处理类节点：builder 直建（带 options——per-kind 专属选项在 builder 内应用；
    // 不经 ctx.createX() 工厂防工厂↔构造器互调递归）。
    else if (nodeName === 'StereoPannerNode') node = _zwWABuildStereoPanner(ctx, options);
    else if (nodeName === 'DelayNode') node = _zwWABuildDelay(ctx, options);
    else if (nodeName === 'BiquadFilterNode') node = _zwWABuildBiquadFilter(ctx, options);
    else if (nodeName === 'AnalyserNode') node = _zwWABuildAnalyser(ctx, options);
    else node = ctx.createGain();
    if (options && typeof options === 'object') {
      // spec webaudio AudioNodeOptions dict 校验（ctor 面比 setter 严——WPT
      // testAudioNodeOptions [0,99] → NotSupportedError；enum invalid → TypeError）。
      if (options.channelCount != null) {
        var cc = Number(options.channelCount);
        // spec StereoPannerNode：channelCount [1,2]（ctor dict 级约束——0/3/99 全
        // NotSupportedError，ctor-stereopanner 'test AudioNodeOptions' 断言面）；
        // 其余节点基类界 [1,32]（0 → NotSupportedError、>32 → IndexSizeError）。
        if (nodeName === 'StereoPannerNode') {
          if (cc < 1 || cc > 2) {
            throw new (globalThis.DOMException || Error)(
              'channelCount ' + cc + ' not supported for StereoPannerNode.', 'NotSupportedError');
          }
        } else if (cc === 0 || cc > 32) {
          throw new (globalThis.DOMException || Error)(
            'channelCount ' + cc + ' not supported.', 'NotSupportedError');
        }
        node.channelCount = cc;
      }
      if (options.channelCountMode != null) {
        var ccm = String(options.channelCountMode);
        // spec StereoPannerNode：mode 'max' 非法 → NotSupportedError（enum 域
        // clamped-max/explicit——ctor-stereopanner 断言面）。
        if (nodeName === 'StereoPannerNode' && ccm === 'max') {
          throw new (globalThis.DOMException || Error)(
            "Failed to construct 'StereoPannerNode': channelCountMode 'max' not supported for StereoPannerNode.", 'NotSupportedError');
        }
        if (ccm !== 'max' && ccm !== 'clamped-max' && ccm !== 'explicit') {
          throw new TypeError("Failed to construct '" + nodeName + "': Failed to read the 'channelCountMode' property from 'AudioNodeOptions': The provided value '" + ccm + "' is not a valid enum value.");
        }
        node.channelCountMode = ccm;
      }
      if (options.channelInterpretation != null) {
        var ci = String(options.channelInterpretation);
        if (ci !== 'speakers' && ci !== 'discrete') {
          throw new TypeError("Failed to construct '" + nodeName + "': Failed to read the 'channelInterpretation' property from 'AudioNodeOptions': The provided value '" + ci + "' is not a valid enum value.");
        }
        node.channelInterpretation = ci;
      }
    }
    return node;
  }
  function OscillatorNode(ctx, options) {
    var node = _zwWANodeCtor('OscillatorNode', ctx, options);
    if (options && typeof options === 'object') {
      // spec：type='custom' 必须与 periodicWave 同给——单独 custom →
      // InvalidStateError（WPT ctor 断言面）；periodicWave 非对象/非 null 声明
      // → TypeError（dict 成员类型校验）；periodicWave（PeriodicWave 实例）
      // 直接应用（type 置 custom）。
      if (options.type === 'custom' && !options.periodicWave) {
        throw new (globalThis.DOMException || Error)(
          "Failed to construct 'OscillatorNode': type 'custom' requires a periodicWave.", 'InvalidStateError');
      }
      // WebIDL：nullable 接口成员显式 null 也须类型校验（null 不是 PeriodicWave
      // ——WPT ctor {periodicWave: null} → TypeError 断言面）。
      if (options.periodicWave !== undefined
          && (typeof options.periodicWave !== 'object' || !options.periodicWave._zwReal)) {
        throw new TypeError("Failed to construct 'OscillatorNode': Failed to read the 'periodicWave' property from 'OscillatorOptions': The provided value is not of type 'PeriodicWave'.");
      }
      if (options.periodicWave) {
        node.type = 'custom';
      } else if (options.type != null) {
        node.type = options.type;
      }
      if (options.frequency != null) node.frequency.value = Number(options.frequency);
      if (options.detune != null) node.detune.value = Number(options.detune);
    }
    return node;
  }
  OscillatorNode.prototype = _zwWANode.prototype;
  globalThis.OscillatorNode = globalThis.OscillatorNode || OscillatorNode;
  function GainNode(ctx, options) {
    var node = _zwWANodeCtor('GainNode', ctx, options);
    if (options && typeof options === 'object' && options.gain != null) {
      node.gain.value = Number(options.gain);
    }
    return node;
  }
  GainNode.prototype = _zwWANode.prototype;
  globalThis.GainNode = globalThis.GainNode || GainNode;
  // ---- 处理类节点构造器族（media-audio M3 切片 2 第四批 WPT 导入支撑面）----
  // spec webaudio 各节点 `new X(ctx, options)` 与 `ctx.createX()` 等价对象；共享
  // _zwWANodeCtor（ctx 校验 + AudioNodeOptions dict 校验），per-kind 专属选项在
  // 各构造器内补校验。headless 全部为语义面（无 DSP 合成——合成面归
  // WebAudioContext 源节点，RFC §0 简化记录）。
  // ---- 处理类节点 builder 族（工厂 createX 与构造器 new X 共用的建节点逻辑；
  // 工厂直调 builder、构造器经 _zwWANodeCtor 分发 builder——两向都终结于 builder，
  // 无工厂↔构造器互调递归）。builder 内应用 per-kind 专属选项。
  // StereoPannerNode（spec §StereoPannerNode——pan AudioParam [-1,1]、
  // channelCountMode 'clamped-max' 缺省、{pan: 0.75} 选项反射——ctor-stereopanner
  // 断言面）。
  function _zwWABuildStereoPanner(ctx, options) {
    var node = _zwWANode('stereopanner', ctx._zwCtxId, 0);
    // spec StereoPannerNode：channelCountMode 缺省 'clamped-max'（基类缺省 'max'
    // 不适用——ctor-stereopanner testDefaultConstructor 断言面）。
    node._zwChannelCountMode = 'clamped-max';
    var _panVal = 0;
    var _panParam = globalThis._zwMakeAudioParam(0);
    Object.defineProperty(_panParam, 'value', {
      get: function () { return _panVal; },
      set: function (v) {
        var n = Number(v);
        if (isNaN(n) || n === Infinity || n === -Infinity) {
          throw new TypeError("Failed to set the 'value' property on 'AudioParam': The provided value is non-finite.");
        }
        _panVal = n;
      },
      configurable: true,
    });
    Object.defineProperty(node, 'pan', {
      get: function () { return _panParam; },
      configurable: true,
    });
    if (options && typeof options === 'object' && options.pan != null) {
      node.pan.value = Number(options.pan);
    }
    return node;
  }
  function StereoPannerNode(ctx, options) { return _zwWANodeCtor('StereoPannerNode', ctx, options); }
  StereoPannerNode.prototype = _zwWANode.prototype;
  globalThis.StereoPannerNode = globalThis.StereoPannerNode || StereoPannerNode;
  // DelayNode（spec §DelayNode——delayTime AudioParam [0, maxDelayTime]、
  // maxDelayTime 选项反射为 delayTime.maxValue 上界——ctor-delay
  // {delayTime: 0.5, maxDelayTime: 1.5} 断言面；缺省 maxDelayTime 1.0 spec 档）。
  function _zwWABuildDelay(ctx, options) {
    var node = _zwWANode('delay', ctx._zwCtxId, 0);
    var _delayVal = 0;
    var _maxDelay = (options && typeof options === 'object' && options.maxDelayTime != null)
      ? Number(options.maxDelayTime) : 1.0;
    if (isNaN(_maxDelay) || _maxDelay < 0) {
      throw new RangeError("Failed to construct 'DelayNode': maxDelayTime must be non-negative.");
    }
    var _delayParam = globalThis._zwMakeAudioParam(0);
    _delayParam.maxValue = _maxDelay;
    Object.defineProperty(_delayParam, 'value', {
      get: function () { return _delayVal; },
      set: function (v) {
        var n = Number(v);
        if (isNaN(n) || n === Infinity || n === -Infinity) {
          throw new TypeError("Failed to set the 'value' property on 'AudioParam': The provided value is non-finite.");
        }
        _delayVal = Math.min(Math.max(n, 0), _maxDelay);
      },
      configurable: true,
    });
    Object.defineProperty(node, 'delayTime', {
      get: function () { return _delayParam; },
      configurable: true,
    });
    if (options && typeof options === 'object' && options.delayTime != null) {
      node.delayTime.value = Number(options.delayTime);
    }
    return node;
  }
  function DelayNode(ctx, options) { return _zwWANodeCtor('DelayNode', ctx, options); }
  DelayNode.prototype = _zwWANode.prototype;
  globalThis.DelayNode = globalThis.DelayNode || DelayNode;
  // BiquadFilterNode（spec §BiquadFilterNode——type 八枚举（缺省 'lowpass'，invalid
  // 静默保留——setter 面惯例）+ Q/detune/frequency/gain 四 AudioParam 缺省
  //（1/0/350/0——ctor-biquadfilter testDefaultAttributes 断言面 + {type:'highpass',
  // frequency:512, detune:1, Q:5, gain:3} 选项反射面）。
  function _zwWABuildBiquadFilter(ctx, options) {
    var node = _zwWANode('biquadfilter', ctx._zwCtxId, 0);
    var _type = 'lowpass';
    Object.defineProperty(node, 'type', {
      get: function () { return _type; },
      set: function (v) {
        var s = String(v == null ? '' : v);
        if (s === 'lowpass' || s === 'highpass' || s === 'bandpass' || s === 'lowshelf' || s === 'highshelf' || s === 'notch' || s === 'peaking' || s === 'allpass') {
          _type = s;
        }
      },
      configurable: true,
    });
    var _q = globalThis._zwMakeAudioParam(1);
    var _detune = globalThis._zwMakeAudioParam(0);
    var _freq = globalThis._zwMakeAudioParam(350);
    var _gain = globalThis._zwMakeAudioParam(0);
    Object.defineProperty(node, 'Q', { get: function () { return _q; }, configurable: true });
    Object.defineProperty(node, 'detune', { get: function () { return _detune; }, configurable: true });
    Object.defineProperty(node, 'frequency', { get: function () { return _freq; }, configurable: true });
    Object.defineProperty(node, 'gain', { get: function () { return _gain; }, configurable: true });
    if (options && typeof options === 'object') {
      if (options.type != null) node.type = options.type;
      if (options.Q != null) node.Q.value = Number(options.Q);
      if (options.detune != null) node.detune.value = Number(options.detune);
      if (options.frequency != null) node.frequency.value = Number(options.frequency);
      if (options.gain != null) node.gain.value = Number(options.gain);
    }
    return node;
  }
  function BiquadFilterNode(ctx, options) { return _zwWANodeCtor('BiquadFilterNode', ctx, options); }
  BiquadFilterNode.prototype = _zwWANode.prototype;
  globalThis.BiquadFilterNode = globalThis.BiquadFilterNode || BiquadFilterNode;
  // AnalyserNode（spec §AnalyserNode——fftSize 缺省 2048（2 的幂，[32, 32768] 外
  // → IndexSizeError）、frequencyBinCount 只读反射（fftSize/2）、minDecibels -100 /
  // maxDecibels -30（min ≥ max → IndexSizeError）、smoothingTimeConstant 0.8
  //（[0,1] 外 → IndexSizeError）——ctor-analyser 缺省/选项/invalid 三断言面；
  // getByteTimeDomainData 等数据面无渲染缓冲不做（RFC §0）。
  function _zwWABuildAnalyser(ctx, options) {
    var node = _zwWANode('analyser', ctx._zwCtxId, 0);
    var _fftSize = 2048;
    var _minDb = -100;
    var _maxDb = -30;
    var _smoothing = 0.8;
    var _crossCheck = function () {
      if (_minDb >= _maxDb) {
        throw new (globalThis.DOMException || Error)(
          'minDecibels must be less than maxDecibels.', 'IndexSizeError');
      }
    };
    var _armed = false;
    Object.defineProperty(node, 'fftSize', {
      get: function () { return _fftSize; },
      set: function (v) {
        var n = Number(v);
        // spec：fftSize 2 的幂且 [32, 32768]——否则 IndexSizeError。
        if (n < 32 || n > 32768 || (n & (n - 1)) !== 0) {
          throw new (globalThis.DOMException || Error)(
            'fftSize ' + n + ' is not a power of two in [32, 32768].', 'IndexSizeError');
        }
        _fftSize = n;
      },
      configurable: true,
    });
    Object.defineProperty(node, 'frequencyBinCount', {
      get: function () { return _fftSize / 2; },
      configurable: true,
    });
    Object.defineProperty(node, 'minDecibels', {
      get: function () { return _minDb; },
      // ctor 选项路径（_armed=false）不触发交叉校验——spec ctor 交叉校验在全部
      // 选项应用后统一进行（ctor-analyser 'setting min/max' 断言面）；ID setter
      // 路径（_armed=true）即时校验。
      set: function (v) { _minDb = Number(v); if (_armed) _crossCheck(); },
      configurable: true,
    });
    Object.defineProperty(node, 'maxDecibels', {
      get: function () { return _maxDb; },
      set: function (v) { _maxDb = Number(v); if (_armed) _crossCheck(); },
      configurable: true,
    });
    Object.defineProperty(node, 'smoothingTimeConstant', {
      get: function () { return _smoothing; },
      set: function (v) {
        var n = Number(v);
        if (n < 0 || n > 1) {
          throw new (globalThis.DOMException || Error)(
            'smoothingTimeConstant ' + n + ' is out of range [0, 1].', 'IndexSizeError');
        }
        _smoothing = n;
      },
      configurable: true,
    });
    if (options && typeof options === 'object') {
      // spec：两 option 同给时 min/max 校验在两值都应用后进行（ctor-analyser
      // 'setting min/max' 断言面——{min:-200, max:-150} 合法、{max:-150, min:-10}
      // 抛，与给出顺序无关）。
      var _optOrder = ['fftSize', 'maxDecibels', 'minDecibels', 'smoothingTimeConstant'];
      for (var _oi = 0; _oi < _optOrder.length; _oi++) {
        var _k = _optOrder[_oi];
        if (options[_k] != null) node[_k] = options[_k];
      }
    }
    _armed = true;
    _crossCheck();
    return node;
  }
  function AnalyserNode(ctx, options) { return _zwWANodeCtor('AnalyserNode', ctx, options); }
  AnalyserNode.prototype = _zwWANode.prototype;
  globalThis.AnalyserNode = globalThis.AnalyserNode || AnalyserNode;
  // AudioBuffer（spec webaudio §AudioBuffer——独立构造器（不依赖 BaseAudioContext）：
  // length/sampleRate 必填（缺失 → TypeError）、numberOfChannels 缺省 1；
  // numberOfChannels ≥ 1 / length ≥ 1 / sampleRate ∈ [8000, 96000]（spec 正义
  // 约束——ctor-audiobuffer/audiobuffer 断言面：0 通道、0 长、100Hz →
  // NotSupportedError）；duration = length / sampleRate 反射；getChannelData(i)
  // 返回 Float32Array（越界 → IndexSizeError）。headless 存储面：零填充通道——
  // copyToChannel/copyFromChannel 随 AudioBufferSourceNode 播放面切片评估）。
  // https://webaudio.github.io/web-audio-api/#AudioBuffer
  function AudioBuffer(options) {
    if (!(this instanceof AudioBuffer)) {
      throw new TypeError("Failed to construct 'AudioBuffer': Please use the 'new' operator, this DOM object constructor cannot be called as a function.");
    }
    if (options == null || typeof options !== 'object') {
      throw new TypeError("Failed to construct 'AudioBuffer': The provided value is not of type 'AudioBufferOptions'.");
    }
    // WebIDL required members：length/sampleRate 缺失 → TypeError（dict required
    // 约束——ctor-audiobuffer 'required options' 断言面）。
    if (options.length === undefined) {
      throw new TypeError("Failed to construct 'AudioBuffer': Failed to read the 'length' property from 'AudioBufferOptions': Required member is undefined.");
    }
    if (options.sampleRate === undefined) {
      throw new TypeError("Failed to construct 'AudioBuffer': Failed to read the 'sampleRate' property from 'AudioBufferOptions': Required member is undefined.");
    }
    var _channels = (options.numberOfChannels !== undefined) ? Number(options.numberOfChannels) : 1;
    var _length = Number(options.length);
    var _sampleRate = Number(options.sampleRate);
    // spec 正义约束（非有限/越界 → NotSupportedError——ctor-audiobuffer
    // 'invalid option values' 断言面：{0 通道}/{0 长}/{100Hz}）。
    if (isNaN(_channels) || _channels < 1 || _channels > 32) {
      throw new (globalThis.DOMException || Error)(
        "Failed to construct 'AudioBuffer': numberOfChannels " + options.numberOfChannels + " is not in range [1, 32].", 'NotSupportedError');
    }
    if (isNaN(_length) || _length < 1) {
      throw new (globalThis.DOMException || Error)(
        "Failed to construct 'AudioBuffer': length " + options.length + " is not in range [1, " + 4294967295 + "].", 'NotSupportedError');
    }
    // spec sampleRate [8000, 96000]——WPT 断言面 100Hz 拒、48000/16000/54321/24576 收。
    if (isNaN(_sampleRate) || _sampleRate < 8000 || _sampleRate > 96000) {
      throw new (globalThis.DOMException || Error)(
        "Failed to construct 'AudioBuffer': sampleRate " + options.sampleRate + " is not in range [8000, 96000].", 'NotSupportedError');
    }
    var _chanData = [];
    for (var _ci = 0; _ci < _channels; _ci++) _chanData.push(new Float32Array(_length));
    Object.defineProperty(this, 'numberOfChannels', {
      get: function () { return _channels; },
      configurable: true,
    });
    Object.defineProperty(this, 'length', {
      get: function () { return _length; },
      configurable: true,
    });
    Object.defineProperty(this, 'sampleRate', {
      get: function () { return _sampleRate; },
      configurable: true,
    });
    Object.defineProperty(this, 'duration', {
      get: function () { return _length / _sampleRate; },
      configurable: true,
    });
    this.getChannelData = function (channel) {
      var i = Number(channel);
      if (isNaN(i) || i < 0 || i >= _channels || (i !== Math.floor(i))) {
        throw new (globalThis.DOMException || Error)(
          "Failed to execute 'getChannelData' on 'AudioBuffer': channel index " + channel + " is not a valid index.", 'IndexSizeError');
      }
      return _chanData[i];
    };
  }
  globalThis.AudioBuffer = globalThis.AudioBuffer || AudioBuffer;
  // PeriodicWave 接口（spec webaudio §PeriodicWave——custom waveform 容器；最小面
  // 仅构造 + real/imag 数组存储，无 FFT 合成——ctor-oscillator 断言 new
  // PeriodicWave(context, {real, imag}) 不抛 + disableNormalization 反射）。
  function PeriodicWave(context, options) {
    if (!(this instanceof PeriodicWave)) {
      throw new TypeError("Failed to construct 'PeriodicWave': Please use the 'new' operator.");
    }
    var o = (options && typeof options === 'object') ? options : {};
    this._zwReal = o.real || [1, 0];
    this._zwImag = o.imag || [0];
    this._zwDisableNormalization = !!o.disableNormalization;
  }
  Object.defineProperty(PeriodicWave.prototype, 'disableNormalization', {
    get: function () { return this._zwDisableNormalization; },
    configurable: true,
  });
  globalThis.PeriodicWave = globalThis.PeriodicWave || PeriodicWave;
  // createBufferSource/createBiquadFilter 等未实现面——spec 其它节点类型不属最小面
  //（RFC §0 不做清单），undefined 返回（调用方 try/catch 容错）。
  // track 元素 src 的 headless 加载模拟——data:text/vtt 解析填 cue + load 事件；
  // 非 data: URL 同样派 load（headless 无真字幕抓取， cues 空）。幂等：per-track 一次。
  // M3 扩批 XV（2026-09-02）：http(s) VTT 文件加载接通——同步 `__zw_fetch` 契约
  //（R115 iframe 同款）取回 VTT 文本 → `_zwParseVtt` 解析填 cue；fetch 失败或
  // 非 WEBVTT 头（`_zwParseVtt` 返 null）→ error settle（track onerror + readyState
  // ERROR——track-webvtt-magic-header no-webvtt 断言面）。无 `__zw_fetch` 宿主
  //（浏览器异步路径）回落既有 headless 面（load 恒派、cues 空——零回归）。
  globalThis._zwTrackScheduleLoad = function (sel, handle, opts) {
    var key = _elKey(sel, handle);
    if (typeof setTimeout !== 'function') return;
    var _msTrack = _mediaState[key] || (_mediaState[key] = {});
    var _isChange = !!(opts && opts.srcChange);
    if (_isChange) {
      // M3 扩批 XV：**同值变更不重载**（spec track-element-src-change stage3——设回同
      // 一 src 值不派 onload；cues 保持）。新旧绝对 URL 相等且已成功 settle → no-op。
      // （比较在读取 attr 前做：借用上次 settle 的 url。）
      var _lastState = _resourceStates ? _resourceStates[key] : null;
      var _prevUrl = (_lastState && _lastState.outcome !== 'error') ? String(_lastState.url || '') : '';
      var _curRaw = '';
      try {
        _curRaw = handle ? __zw_get_attr_handle(handle, 'src') : (typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(sel, 'src') : __zw_get_attr(sel, 'src'));
        _curRaw = String(_curRaw == null ? '' : _curRaw).replace(/^[\x00-\x20]+/, '').replace(/[\x00-\x20]+$/, '');
        if (_curRaw && typeof _zwResolveFetchUrl === 'function') _curRaw = _zwResolveFetchUrl(_curRaw);
      } catch (_eTsCu) {}
      if (_prevUrl && _prevUrl === _curRaw && _msTrack.trackScheduled) return;
      // M3 扩批 XIII：src 变更——清既有 cue（spec「track URL 变更」cue list 置空，
      // src-clear-cues 断言面）+ 重置 settle 面后按新 URL 重跑。**首调度前变更**
      //（track 从未挂 media 父/未设 src，src-clear-cues 用例形态）同样清。
      try {
        var _scTT = _elementTextTrack[key];
        if (_scTT && typeof _scTT._zwClearCues === 'function') _scTT._zwClearCues();
      } catch (_eTsCc) {}
      try { if (_resourceStates && _resourceStates[key]) delete _resourceStates[key]; } catch (_eTsRs) {}
    }
    // M3 扩批 XV：src 变更重调度——重置幂等标记后按新 URL 重跑（track-element-src-change
    // 断言面：settings.vtt → entities.vtt → settings.vtt 三段加载各派 onload）。
    if (_isChange) _msTrack.trackScheduled = false;
    if (_msTrack.trackScheduled) return;
    _msTrack.trackScheduled = true;
    var _deferCont = function (fn) {
      if (typeof queueMicrotask === 'function') queueMicrotask(fn);
      else setTimeout(fn, 0);
    };
    _deferCont(function () {
      try {
        var _raw = handle ? __zw_get_attr_handle(handle, 'src') : (typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(sel, 'src') : __zw_get_attr(sel, 'src'));
        var _abs = String(_raw == null ? '' : _raw).replace(/^[\x00-\x20]+/, '').replace(/[\x00-\x20]+$/, '');
        try { if (typeof _zwResolveFetchUrl === 'function') _abs = _zwResolveFetchUrl(_abs); } catch (_eTsR) {}
        // M3 扩批 XV：解析分派——data:text/vtt 走既有内联解析；http(s) 走同步
        // `__zw_fetch` 取文本后 `_zwParseVtt`。`_zwParseVtt` 返 null = 非 WEBVTT 头
        //（error settle）；返回数组（可为空）= 解析成功（空 VTT 文件合法）。
        var cues = null;
        var _failed = false;
        if (_abs.indexOf('data:text/vtt,') >= 0) {
          cues = (typeof globalThis._zwParseVttDataUrl === 'function') ? globalThis._zwParseVttDataUrl(_abs) : null;
        } else if ((_abs.indexOf('http://') === 0 || _abs.indexOf('https://') === 0)
            && typeof __zw_fetch === 'function'
            && typeof globalThis._zwParseVtt === 'function') {
          var _wire = '';
          try {
            _wire = String(__zw_fetch(
              'trackvtt:' + key + ':' + (++_zwTrackFetchSeq),
              'GET', _abs, '', '', '', '', '', '', '', ''
            ) || '');
          } catch (_eTvF) { _failed = true; globalThis.__zwTrackFetchDebug = 'threw:' + _eTvF.message; }
          if (!_failed) {
            if (_wire.indexOf('__zw_fetch_error') === 0) {
              _failed = true; globalThis.__zwTrackFetchDebug = 'wire:' + _wire.slice(0, 80);
            } else {
              // `__zwfr:` wire（status\x1fstatusText\x1fheadersWire\x1fbody）取 body；
              // 旧 body-only wire 兜底整串。
              var _body = _wire;
              if (_wire.indexOf('__zwfr:') === 0) {
                var _s1 = _wire.indexOf('\x1f');
                var _s2 = _wire.indexOf('\x1f', _s1 + 1);
                var _s3 = _wire.indexOf('\x1f', _s2 + 1);
                if (_s3 >= 0) _body = _wire.slice(_s3 + 1);
              }
              cues = globalThis._zwParseVtt(_body);
            }
          }
        } else {
          // 无 fetch 宿主：headless 近似——load 恒派、cues 空（既有零回归面）。
          cues = [];
        }
        if (_failed || cues === null) {
          // 非 WEBVTT 头 / fetch 失败 → error settle（spec「track loading failed」——
          // readyState ERROR + track onerror；track-webvtt-magic-header 断言面）。
          _zwSettleResourceKey(key, sel, handle, 'track', _abs, 'error', 0, 0);
          return;
        }
        if (cues.length) {
          // 填入关联 TextTrack（track.track getter 建实例）。
          var el = (typeof _makeProxy === 'function') ? _makeProxy(sel, handle) : null;
          var tt = el ? el.track : null;
          if (tt && typeof tt.addCue === 'function') {
            for (var i = 0; i < cues.length; i++) tt.addCue(cues[i]);
          }
        }
        // M3 扩批 XIII：settle 幂等已按需重置——重调度路径重新提交资源状态。
        _zwSettleResourceKey(key, sel, handle, 'track', _abs, 'loaded', 0, 0);
      } catch (_eTs) {}
    });
  };
  var _zwTrackFetchSeq = 0;
  function _zwSettleResourceKey(key, sel, handle, tag, url, outcome, width, height, errorCode, durationMs) {
    if (_resourceStates[key]) return false; // 每个资源请求只 settle / 派发一次。
    var state = {
      url: String(url), outcome: String(outcome),
      width: Math.max(0, Number(width) || 0), height: Math.max(0, Number(height) || 0),
      // media-playback M2a：容器时长真值（毫秒，宿主解码器头部读取；video 面专用）。
      // null/undefined → 语义层 _zwMediaLoadSequence 回落 headless 定值（测试零回归）。
      durationMs: durationMs == null ? null : Math.max(0, Number(durationMs) || 0),
      // errorCode 缺省 2（NETWORK_ERROR——fetch 失败路径）；空 src 资源选择失败路径
      // 传 4（MEDIA_ERR_SRC_NOT_SUPPORTED，spec「failed with attribute」步）。
      error: outcome === 'error' ? _zwMediaError(Number(errorCode) || 2, 'Error loading resource: ' + String(url)) : null
    };
    _resourceStates[key] = state;
    // media-elements M3 扩批 XI：资源选择失败（error settle）→ networkState =
    // NETWORK_NO_SOURCE(3)（spec「failed with attribute/media resource」终态——
    // 等待更多候选；resource-selection-invoke-pause-networkState 断言面）。
    // 成功 settle → _zwMediaLoadSequence 置 LOADING→IDLE（下方）。
    if ((tag === 'audio' || tag === 'video') && outcome === 'error') {
      var _seMs = _mediaState[key] || (_mediaState[key] = {});
      _seMs.networkState = 3;
    }
    var eventType = '';
    if (tag === 'img') eventType = outcome === 'error' ? 'error' : 'load';
    else if (tag === 'track') eventType = outcome === 'error' ? 'error' : 'load';
    else if ((tag === 'source' || tag === 'audio' || tag === 'video') && outcome === 'error') eventType = 'error';
    if (eventType) {
      _dispatchWithBubble(key, sel, null, _makeEvent(eventType, { bubbles: false, cancelable: false }));
    }
    // M2：media 元素资源成功加载 → 派加载事件序列（error 已在上方 error 分支派发）。
    // M3 扩批 XVI：**延后一拍**（setTimeout(0) 媒体任务队列——spec「queue a task to fire
    // 事件」）。同步派发使 settle（microtask 续段）在页面脚本注册 handler **之前**派
    // canplaythrough（Chromium 的 canplay 在数据加载 task 上，晚于当前脚本 turn——
    // track-cues-* 等用例 src= 赋值后才挂 oncanplaythrough）。延迟一拍后 handler 可达；
    // 事件内 readyState 断言不受影响（序列相对顺序不变——networkState_during_*/readyState
    // _during_* 断言面）。无 setTimeout 环境回落同步（零回归面）。
    if ((tag === 'audio' || tag === 'video') && outcome !== 'error') {
      _zwMediaLoadSequence(sel, handle, key, tag);
    }
    return true;
  }
  globalThis.__zw_commit_resource_element_state = function (tag, absUrl, outcome, width, height, durationMs) {
    try {
      tag = String(tag).toLowerCase();
      if (typeof __zw_query_all !== 'function') return;
      var sels = (__zw_query_all(tag) || '').split('|').filter(Boolean);
      var pageUrl = typeof __zw_get_page_url === 'function' ? __zw_get_page_url() : '';
      var target = String(absUrl == null ? '' : absUrl);
      for (var i = 0; i < sels.length; i++) {
        var sel = sels[i];
        var raw = typeof __zw_get_attr === 'function' ? __zw_get_attr(sel, 'src') : '';
        if (!raw && tag === 'img' && typeof __zw_get_attr === 'function') {
          var srcset = __zw_get_attr(sel, 'srcset') || '';
          raw = srcset.split(',')[0].trim().split(/\s+/)[0] || '';
        }
        if (!raw) continue;
        var resolved = raw;
        if (pageUrl && raw.indexOf('://') < 0 && raw.indexOf('data:') !== 0 &&
            typeof __zw_parse_url === 'function') {
          try {
            var parsed = JSON.parse(__zw_parse_url(raw, pageUrl));
            if (parsed && parsed.href) resolved = parsed.href;
          } catch (_e) {}
        }
        if (resolved !== target) continue;
        var committed = _zwSettleResourceSelector(sel, tag, target, outcome, width, height, durationMs);
        if (!committed || tag !== 'source') continue;
        var parent = _parentNodeFor(sel, null);
        var parentTag = parent && parent.tagName ? String(parent.tagName).toLowerCase() : '';
        if (parentTag !== 'audio' && parentTag !== 'video') continue;
        if (outcome !== 'error') {
          _zwSettleResourceSelector(parent.__zwSelector, parentTag, target, 'available', 0, 0, durationMs);
          continue;
        }
        var candidates = parent.querySelectorAll ? parent.querySelectorAll('source') : [];
        var allFailed = candidates.length > 0;
        for (var j = 0; j < candidates.length; j++) {
          var candidateState = _resourceStates[_elKey(candidates[j].__zwSelector, null)];
          if (!candidateState || candidateState.outcome !== 'error') { allFailed = false; break; }
        }
        if (allFailed) _zwSettleResourceSelector(parent.__zwSelector, parentTag, target, 'error', 0, 0, durationMs);
      }
    } catch (_e) {}
  };
  // 元素级派发便捷封装（旧 R2943 img / R2944 link / script）。
  globalThis.__zw_dispatch_img_event = function (absUrl, type) {
    __zw_commit_resource_element_state('img', absUrl, type === 'error' ? 'error' : 'loaded', 0, 0);
  };
  globalThis.__zw_dispatch_link_event = function (absHref, type) {
    __zw_dispatch_element_event('link', 'href', absHref, type);
  };
  globalThis.__zw_dispatch_script_event = function (absSrc, type) {
    __zw_dispatch_element_event('script', 'src', absSrc, type);
  };

  globalThis.__zw_dispatch_event = function(sel, type, detail) {
    var ev;
    if (type === 'submit') {
      // R2984 SubmitEvent：submitter = 按钮 proxy（detail.submitter 经 _wrapSelector）；缺省 null（Enter 隐式提交）。
      var sub = (detail && detail.submitter) ? _wrapSelector(detail.submitter) : null;
      ev = new SubmitEvent(type, { bubbles: true, cancelable: true, submitter: sub });
    } else if (type === 'compositionstart' || type === 'compositionupdate' || type === 'compositionend') {
      ev = new CompositionEvent(type, {
        bubbles: true,
        cancelable: true,
        data: (detail && detail.data) || ''
      });
    } else if ((type === 'beforeinput' || type === 'input') && detail && detail.inputType) {
      ev = new InputEvent(type, {
        bubbles: true,
        cancelable: type === 'beforeinput',
        data: detail.data,
        inputType: detail.inputType,
        isComposing: !!detail.isComposing
      });
    } else if (type === 'input') {
      ev = new InputEvent(type, { bubbles: true, cancelable: false });
    } else if (type === 'change' || type === 'focus' || type === 'blur' ||
               type === 'focusin' || type === 'focusout') {
      ev = _makeEvent(type, {
        bubbles: type === 'change' || type === 'focusin' || type === 'focusout',
        cancelable: false
      });
    } else if (detail && (detail.key || detail.code)) {
      ev = new KeyboardEvent(type, {
        bubbles: true,
        cancelable: true,
        key: detail.key || '',
        code: detail.code || detail.key || ''
      });
    } else {
      ev = _makeEvent(type, { bubbles: true, cancelable: true });
    }
    // R312（js-dom M4）：宿主派发 = UA 合成事件，isTrusted=true（spec——真实浏览器
    // 的用户输入/激活事件链全部 trusted；`__zw_dispatch_event` 只被 engine 的
    // script_gen 宿主脚本调用，页面脚本不可达——无越权面）。经 `_zwUaDispatch`
    // 印记走 UA 通道（脚本再 dispatch 同一对象时 guard 翻 false，redispatch 语义）。
    try {
      if (ev && typeof ev === 'object' && !ev.isTrusted) {
        Object.defineProperty(ev, 'isTrusted', { value: true, writable: true, configurable: true, enumerable: true });
        ev._zwUaDispatch = true;
      }
    } catch (_e312h) {}
    // R145（js-dom M4）：sel→handle identity 桥——listener 注册在 handle proxy 的
    // `_listenerStore['@'+handle]`（createElement/cloneNode 产物经 addEventListener），
    // sel-key 派发查不到（WPT pointer-event-document-move：模板 clone 的 p 上
    // pointerup listener，host 经 'p' 派发 miss）。正置反查命中 → 以 handle 形态派发
    // （`_elKey(handle)` 锚定 listener store；未命中 → 原 sel 路径，零回归）。
    var r145Handle = '';
    try {
      if (typeof __zw_handle_for_selector === 'function') r145Handle = __zw_handle_for_selector(sel) || '';
    } catch (_e145h) {}
    var r145Key = r145Handle ? _elKey(null, r145Handle) : _elKey(sel, null);
    var ok = _dispatchWithBubble(r145Key, r145Handle ? null : sel, r145Handle || null, ev);
    // R312（js-dom M4）：UA 通道印记一次性——本（宿主）dispatch 完成即消费；同一
    // 事件对象再经页面脚本 dispatchEvent 时 guard 按 legacy DOM3 语义翻
    // isTrusted=false（WPT Event-dispatch-redispatch 的 before/after 断言对）。
    try { ev._zwUaDispatch = false; } catch (_e312ua) {}
    return ok ? 'ok' : 'prevented';
  };
  // M3 扩批（2026-09-02，fixture-mounted 播放切片）：time-marches-on 钩子——宿主播放泵
  // 每 tick 调用，按**真值媒体时钟**推进全部 TextTrack 的 cue 调度（spec
  // https://html.spec.whatwg.org/multipage/media.html#time-marches-on）：
  // ① currentTime ∈ [start,end) 且未 active → enter 事件 + activeCues 计入；
  // ② active 且越界 → exit 事件（pauseOnExit → video.pause()）；
  // ③ missed cues（seek 越过全部区间）→ 不派 enter（seeking 面语义由调用方按
  //    seeking 标志控制；headless 泵按连续推进处理——cue 在上一 tick 与本 tick 间
  //    完整过区间时 enter+exit 都派）。
  // missed cue 语义注记：track-cues-missed 期望 seek 后 play 派发**跳跃** enter——
  // 本钩子以 lastMs → nowMs 区间判定（cue.start ∈ (last, now] 才 enter），天然实现。
  // 事件经 cue.dispatchEvent（EventTarget 面——onenter/onexit accessor 断言面）。
  // M3 扩批（fixture-mounted 切片 2）：seeked 后按**目标时刻**同步 cue active 面
  // （spec time-marches-on 的 seek 处理步）。目标时刻 ∈ [start,end) 的 cue 派 enter
  // （此前未 active）；此前 active 但目标时刻不在区间的 cue 静默移出（不派 exit——
  // seek 面的 active 集合重建语义；track-cues-enter-seeking / missed 断言面）。
  globalThis._zwMediaSeekSync = function (mediaKey) {
    try {
      if (typeof _mediaState === 'undefined' || typeof _elementTextTrack === 'undefined') return;
      var ms = _mediaState[mediaKey];
      if (!ms) return;
      // 拉桥真值（seek 后 currentTime 已由桥 seek 更新——registry seek_to_ms 落位）。
      if (ms.bridgeOn && ms.bridgeSrc && typeof globalThis.__zwVideoBridge === 'object') {
        try {
          var _ssBct = globalThis.__zwVideoBridge.currentTime(ms.bridgeSrc);
          if (typeof _ssBct === 'number' && isFinite(_ssBct)) ms.currentTime = _ssBct;
        } catch (_eSsB) {}
      }
      var nowMs = (typeof ms.currentTime === 'number' && isFinite(ms.currentTime)) ? ms.currentTime * 1000 : 0;
      ms._zwLastMarchMs = nowMs; // seek 落点记账（下一 tick 从此推进——跳变检测不误触）
      ms._zwMediaTimeKnown = true;
      for (var tk in _elementTextTrack) {
        var tt = _elementTextTrack[tk];
        if (!tt || !tt._zwMarchState || !tt._zwCueArrInternal) continue;
        var active = tt._zwMarchState;
        for (var ci = 0; ci < tt._zwCueArrInternal.length; ci++) {
          var cue = tt._zwCueArrInternal[ci];
          if (!cue) continue;
          var startMs = cue._zwStartTime * 1000;
          var endMs = cue._zwEndTime * 1000;
          var wasIdx = -1;
          for (var ai = 0; ai < active.length; ai++) { if (active[ai] === cue) { wasIdx = ai; break; } }
          var shouldBeActive = startMs <= nowMs && nowMs < endMs;
          if (shouldBeActive && wasIdx < 0) {
            active.push(cue);
            try { cue.dispatchEvent({ type: 'enter', target: cue, currentTarget: cue }); } catch (_eSsE) {}
          } else if (!shouldBeActive && wasIdx >= 0) {
            active.splice(wasIdx, 1); // 静默移出（seek 重建面——不派 exit）
          }
        }
      }
    } catch (_eSs) {}
  };
  globalThis._zwMediaTimeMarchesOn = function () {
    try {
      if (typeof _mediaState === 'undefined' || typeof _elementTextTrack === 'undefined') return;
      for (var key in _mediaState) {
        var ms = _mediaState[key];
        if (!ms || !ms.playing) continue;
        // 桥真值时钟优先（bridgeOn 元素主动拉取——getter 镜像只在页面读 IDL 时发生；
        // 泵 tick 无人读 IDL，须此处主动同步）。
        if (ms.bridgeOn && ms.bridgeSrc && typeof globalThis.__zwVideoBridge === 'object') {
          try {
            var _tmoBct = globalThis.__zwVideoBridge.currentTime(ms.bridgeSrc);
            if (typeof _tmoBct === 'number' && isFinite(_tmoBct)) ms.currentTime = _tmoBct;
          } catch (_eTmoB) {}
        }
        var nowMs = (typeof ms.currentTime === 'number' && isFinite(ms.currentTime)) ? ms.currentTime * 1000 : 0;
        // M3 扩批 XVIII：首拍区间基线 = 0（播放起点）而非 nowMs——旧初始化把首个 march
        // tick 的捕获区间置空（(nowMs, nowMs] 空），随后采样粒度 ~1s 时起点恰落在采样
        // 边界附近的 cue（track-cues-enter-exit 的 cue1@1.0s）被永久跳过（startMs >
        // lastMs 判定在其后每拍都为假）→ exit 缺席 → done 永不。播放起点即时间线
        // 原点：上游用例 play() 后从 0 起推进。
        var lastMs = (typeof ms._zwLastMarchMs === 'number') ? ms._zwLastMarchMs : 0;
        // seek 面检测：**时钟回退** 或 seeking 标志在位（spec time-marches-on seek 步：
        // missed cues 不派 enter；此前 active 的 cue 按目标时刻重建）。M3 扩批 XVI 修正：
        // **前进大跳不再判 seek**——桥真值时钟按泵节拍推进，tick 合并产生 >250ms 的前进
        // 跳变是常态（fixture-mounted runner 的 march 采样粒度 ~0.5-1s），前进区间的 cue
        // enter/exit 归区间捕获（_startedInGap/_endedInGap）按序补派。
        var _isSeekJump = (nowMs < lastMs) || ms.seeking === true;
        if (_isSeekJump) {
          // 清全部 active（不派 exit——seek 重建面）；seeking 进行中（seeked 未回落）
          // 则只记账不派发（目标时刻的 enter 由 seeked 后的首个 tick 补齐——起点 ≤ now
          // 且未 active 的 cue 在非跳变 tick 中 enter，满足「target 时刻 active」语义）。
          for (var tk0 in _elementTextTrack) {
            var tt0 = _elementTextTrack[tk0];
            if (tt0 && tt0._zwMarchState) tt0._zwMarchState.length = 0;
          }
          ms._zwLastMarchMs = nowMs;
          continue;
        }
        ms._zwLastMarchMs = nowMs;
        ms._zwMediaTimeKnown = true;
        // 该 media 元素 textTracks 的 track 子产物（_elementTextTrack 以 track 元素 key 存）。
        // M3 扩批 XVI：**区间捕获**——poll 间隔（tick 粒度 ~0.5-1s，见 fixture-mounted 泵）
        // 会整体跳过 <间隔 的 cue（track-cues-missed 的 1ms cue 形态）。跨 (lastMs, nowMs]
        // 区间收集事件（enter@start / exit@end），按**事件时间排序**派发（spec
        // time-marches-on 依时间序处理——上游 missed-cues 期望 enter,exit 交错对，非
        // 「全 enter 后全 exit」），再裁剪 active 集合。
        for (var tk in _elementTextTrack) {
          var tt = _elementTextTrack[tk];
          if (!tt || !tt._zwMarchState || !tt._zwCueArrInternal) continue;
          var active = tt._zwMarchState; // Array<cue>——引用即身份
          var _events = []; // [{t, type, cue}]
          for (var ci = 0; ci < tt._zwCueArrInternal.length; ci++) {
            var cue = tt._zwCueArrInternal[ci];
            if (!cue) continue;
            var startMs = cue._zwStartTime * 1000;
            var endMs = cue._zwEndTime * 1000;
            var wasIdx = -1;
            for (var ai = 0; ai < active.length; ai++) { if (active[ai] === cue) { wasIdx = ai; break; } }
            var wasActive = wasIdx >= 0;
            var shouldBeActive = startMs <= nowMs && nowMs < endMs;
            var _startedInGap = !wasActive && startMs > lastMs && startMs <= nowMs;
            var _endedInGap = wasActive && endMs > lastMs && endMs <= nowMs;
            if (shouldBeActive && !wasActive) {
              _events.push({ t: startMs, type: 'enter', cue: cue });
            } else if (_startedInGap) {
              _events.push({ t: startMs, type: 'enter', cue: cue });
              if (endMs <= nowMs) _events.push({ t: Math.max(endMs, startMs), type: 'exit', cue: cue });
            } else if (wasActive && (!shouldBeActive || _endedInGap)) {
              _events.push({ t: _endedInGap ? endMs : nowMs, type: 'exit', cue: cue });
            }
          }
          if (_events.length) {
            // 事件时间序（同刻 enter 先于 exit——同一 cue 的 start==end 零长 cue 对）。
            _events.sort(function (a, b) {
              if (a.t !== b.t) return a.t - b.t;
              if (a.type !== b.type) return a.type === 'enter' ? -1 : 1;
              return 0;
            });
            for (var ei = 0; ei < _events.length; ei++) {
              var _ev = _events[ei];
              var _cue = _ev.cue;
              var _wasIdx = -1;
              for (var ai2 = 0; ai2 < active.length; ai2++) { if (active[ai2] === _cue) { _wasIdx = ai2; break; } }
              if (_ev.type === 'enter') {
                if (_wasIdx < 0) {
                  active.push(_cue);
                  try { _cue.dispatchEvent({ type: 'enter', target: _cue, currentTarget: _cue }); } catch (_eTmo) {}
                }
              } else {
                if (_wasIdx >= 0) active.splice(_wasIdx, 1);
                // pauseOnExit 暂停**先于** exit 事件派发（spec time-marches-on 步 5：
                // cue exit 后「If paused is false... pause」的暂停须在 handler 内可
                // 同步观察——track-cues-pause-on-exit 的 onexit 内
                // assert_true(video.paused) 断言面；后置暂停使 handler 读到
                // paused=false）。handler 内的 play() 会照常续播。
                if (_cue.pauseOnExit) {
                  try {
                    ms.playing = false;
                    if (ms.bridgeOn && ms.bridgeSrc && typeof globalThis.__zwVideoBridge === 'object') {
                      globalThis.__zwVideoBridge.pause(ms.bridgeSrc);
                    }
                  } catch (_eTmoP) {}
                }
                try { _cue.dispatchEvent({ type: 'exit', target: _cue, currentTarget: _cue }); } catch (_eTmo3) {}
              }
            }
          }
        }
        // M3 扩批 XVI：ended 面——桥真值时钟走到流末（registry player Ended 态）→
        // active cue 全部派 exit（spec：ended 时 activeCues 清空、exit 逐 cue 派）+
        // paused 翻转 + timeupdate + ended（spec time-marches-on 流末处理；
        // track-cues-missed 的 onended 断言面）。幂等：ms._zwEndedDispatched 单次。
        if (ms.playing && ms.bridgeOn && ms.bridgeSrc
            && typeof globalThis.__zwVideoBridge === 'object'
            && typeof globalThis.__zwVideoBridge.isEnded === 'function'
            && !ms._zwEndedDispatched) {
          try {
            if (globalThis.__zwVideoBridge.isEnded(ms.bridgeSrc)) {
              ms._zwEndedDispatched = true;
              ms.playing = false;
              for (var tkE in _elementTextTrack) {
                var ttE = _elementTextTrack[tkE];
                if (!ttE || !ttE._zwMarchState) continue;
                var actE = ttE._zwMarchState;
                for (var ei = actE.length - 1; ei >= 0; ei--) {
                  var cueE = actE[ei];
                  actE.splice(ei, 1);
                  try { cueE.dispatchEvent({ type: 'exit', target: cueE, currentTarget: cueE }); } catch (_eTmoE0) {}
                }
              }
              // sel/handle 不在 march 作用域——key 即元素身份（_elKey 产物），
              // _dispatchWithBubble 按 key 定位；handle 键（'@h' 形态）以 null sel 走
              // handle 分支（与 _dispatchWithBubble 键语义一致）。
              _zwMediaFire(ms._zwSel || (key.charAt(0) === '@' ? null : key),
                ms._zwHandle || (key.charAt(0) === '@' ? key.slice(1) : null),
                key, 'timeupdate');
              _zwMediaFire(ms._zwSel || (key.charAt(0) === '@' ? null : key),
                ms._zwHandle || (key.charAt(0) === '@' ? key.slice(1) : null),
                key, 'ended');
            }
          } catch (_eTmoE1) {}
        }
      }
    } catch (_eTmm) {}
  };

})();

// ── R34xx：DOM 文本几何注册表（selection-rects / index-from-offset 的 DOM 对照侧）──
// created div/p + 纯文本 innerHTML → 本地文本节点 + 同一 shaping（canvas measure）
// 的 0 基文本几何。测试显式归一化绝对位置（rect.x -= parent.x / caret point 经
// gBCR.x 偏移），相对几何一致即通过（与 canvas getSelectionRects 同源）。
// 条目：{ el, handle, sel, text, node }。
var _zwTextEls = [];
// js-dom R52：el→entry Map 索引——`_zwTextEntryForEl`/`_zwUnregisterTextEl` 旧全表线性扫，
// testharness 每 subtest 6 注册 + 6 注销 × O(表) → O(n²)（GR2 探针 ap 段 460→3033ms/500 线性
// 增长根因）。数组保留顺序迭代（geometry 全表扫低频），增删查走 Map。
var _zwTextElsByEl = new Map();
function _zwTextEntryForEl(el) {
  return _zwTextElsByEl.get(el) || null;
}
function _zwRegisterTextEl(el, handle, sel, text) {
  _zwUnregisterTextEl(el);
  var node = {
    nodeType: 3, nodeName: '#text', __nv: text, textContent: text,
    length: text.length, __zwIsText: true,
    previousSibling: null, nextSibling: null,
    // R289（js-dom M4）：childNodes/children 空数组——spec CharacterData 叶子（同
    // _zwMText/doc.createTextNode/_wrapNodeEntry 三工厂；common.js nodeLength/testTree 的
    // `node.childNodes.length` 对缺字段抛 undefined.length TypeError 使整文件 setup 崩，
    // WPT Range-selectNode 主文档 textEl 域实证）。
    childNodes: [],
    children: [],
    // R51：spec ownerDocument（common.js rangeFromEndpoints 经 ownerDocument(node).createRange()）。
    ownerDocument: globalThis.document,
    // js-dom M4 R79：Node.contains / hasChildNodes / compareDocumentPosition——WPT testNodes 的
    // `paras[0].firstChild` 族（textContent= 建的本地文本节点；旧缺方法 → "reference.contains
    // is not a function"）。parentNode 由下方 defineProperty 指向 el，链路完整。
    hasChildNodes: function () { return false; },
    contains: function (other) { return _zwNodeContains(node, other); },
    compareDocumentPosition: function (other) { return _zwCompareDocumentPosition(node, other); },
    // js-dom M4 R108：文本节点 dispatchEvent（WPT Event-dispatch-click "look at parents"——
    // `textChild.dispatchEvent(new MouseEvent('click', {bubbles:true}))` 冒泡到父元素链触发
    // pre-click activation）。spec：文本节点是 EventTarget。guard + 经父 el 派发（自身无
    // listener 存储；target=父 el 与「nearest activation 元素」语义一致——activation 从
    // 派发 target 起向上找，父 el 即首站）。
    dispatchEvent: function (event) {
      globalThis._zwDispatchGuard(event);
      var parent = node.parentNode;
      if (parent && typeof parent.dispatchEvent === 'function') {
        return parent.dispatchEvent(event);
      }
      return !event._defaultPrevented;
    },
  };
  // R81：原型挂 Text.prototype（instanceof Text / CharacterData / Node——WPT Node-textContent
  // `firstChild instanceof Text` 断言；own 字段优先，原型链只补构造器身份）。
  if (globalThis.Text && globalThis.Text.prototype) {
    try { Object.setPrototypeOf(node, globalThis.Text.prototype); } catch (_eTextProto) {}
  }
  // js-dom M4 R49：data/nodeValue 可写 + CharacterData 方法——textContent=/innerHTML= 建的本地
  // 文本节点须可继续编辑（WPT takeRecords `n.firstChild.data='new data'` 发 characterData record，
  // spec target=文本节点）。写经「父 sel + child 索引 0」（SetChildText，同 R48 _wrapNodeEntry 模式）；
  // handle-only（无 sel）纯本地（旧语义）。
  var _regWrite = function (nv) {
    node.__nv = nv; node.textContent = nv; node.length = nv.length;
    if (sel && typeof __zw_set_child_text === 'function') {
      __zw_set_child_text(sel, '0', nv);
      // characterData record——经全局 notify 入口（part01 IIFE 私有的 _mo_id/_mo_notify 不在此作用域；
      // __zw_mo_notify 是其 globalThis 暴露口，未注册则跳过——record 仍由 _mo_notify 语义投递）。
      if (typeof globalThis.__zw_mo_notify_text === 'function') {
        globalThis.__zw_mo_notify_text(sel, node, node.__prevForMo != null ? node.__prevForMo : null);
      }
    }
  };
  Object.defineProperty(node, 'data', {
    get: function () { return node.__nv; },
    set: function (v) { node.__prevForMo = node.__nv; _regWrite(String(v == null ? '' : v)); },
    enumerable: true, configurable: true,
  });
  Object.defineProperty(node, 'nodeValue', {
    get: function () { return node.__nv; },
    set: function (v) { node.__prevForMo = node.__nv; _regWrite(String(v == null ? '' : v)); },
    enumerable: true, configurable: true,
  });
  node.appendData = function (s) { node.__prevForMo = node.__nv; _regWrite(node.__nv + String(s == null ? '' : s)); };
  // R260（js-dom M4）：textContent= 建的本地文本节点（paras[0].firstChild 的
  // 主体域）——live-range 边界调整同四域语义（plain/handle-proxy/parsed sel/
  // 本 textEl 域）。
  node.deleteData = function (o, c2) {
    var a = Math.max(0, o | 0), b = Math.max(0, c2 | 0);
    node.__prevForMo = node.__nv;
    try { globalThis.__zwAdjustRangesForData(node, a, b, 0); } catch (_eR260d) {}
    _regWrite(node.__nv.slice(0, a) + node.__nv.slice(a + b));
  };
  node.insertData = function (o, s) {
    var a = Math.max(0, o | 0);
    node.__prevForMo = node.__nv;
    try { globalThis.__zwAdjustRangesForData(node, a, 0, String(s == null ? '' : s).length); } catch (_eR260i) {}
    _regWrite(node.__nv.slice(0, a) + String(s == null ? '' : s) + node.__nv.slice(a));
  };
  node.replaceData = function (o, c2, s) {
    var a = Math.max(0, o | 0), b = Math.max(0, c2 | 0);
    node.__prevForMo = node.__nv;
    try { globalThis.__zwAdjustRangesForData(node, a, b, String(s == null ? '' : s).length); } catch (_eR260rp) {}
    _regWrite(node.__nv.slice(0, a) + String(s == null ? '' : s) + node.__nv.slice(a + b));
  };
  node.substringData = function (o, c2) {
    var a = Math.max(0, o | 0), b = Math.max(0, c2 | 0);
    return node.__nv.slice(a, a + b);
  };
  Object.defineProperty(node, 'parentNode', { get: function () { return el; }, enumerable: true, configurable: true });
  // R81：parentElement 仅当父为 Element（spec Node.parentElement——父非元素（document 等）→
  // null；WPT Node-properties 文本节点 parentElement 族）。
  Object.defineProperty(node, 'parentElement', { get: function () { try { return (el && el.nodeType === 1) ? el : null; } catch (_e) { return null; } }, enumerable: true, configurable: true });
  var entry = { el: el, handle: handle, sel: sel, text: text, node: node };
  _zwTextEls.push(entry);
  _zwTextElsByEl.set(el, entry);
  if (handle) _zwTextElsByHandle.set(handle, entry);
  if (sel) _zwTextElsBySel.set(sel, entry);
}
function _zwUnregisterTextEl(el) {
  var e = _zwTextElsByEl.get(el);
  if (!e) return;
  _zwTextElsByEl.delete(el);
  if (e.handle && _zwTextElsByHandle.get(e.handle) === e) _zwTextElsByHandle.delete(e.handle);
  if (e.sel && _zwTextElsBySel.get(e.sel) === e) _zwTextElsBySel.delete(e.sel);
  var i = _zwTextEls.indexOf(e);
  if (i >= 0) _zwTextEls.splice(i, 1);
}
// js-dom M4 R51c：**子树注销**——removeChild(el) 只注销 el 自身的注册文本，el 子树内元素
//（textContent= 建的本地文本视图）泄漏：WPT testharness 每 subtest `setupRangeTests()` 全量
//重建（remove testDiv + 重 createElement 6 paras），旧实现每 subtest 泄漏 6 条 `_zwTextEls`
//条目 → `_zwLocalChildNodes` 全表线性扫 → Range-mutations dataChange（~5000 subtest）O(n²)
//超时。子树经 el.childNodes 融合视图递归（本函数在 part06 顶层全局作用域，IIFE 私有的
// _handleChildren 不可达——proxy childNodes getter 已含 registry/overlay 合成）。
function _zwUnregisterTextSubtree(el) {
  if (!el) return;
  _zwUnregisterTextEl(el);
  try {
    var kids = el.childNodes;
    if (kids && kids.length) {
      for (var i = 0; i < kids.length; i++) _zwUnregisterTextSubtree(kids[i]);
    }
  } catch (_e) {}
}
// js-dom M4 R86：**移除子树的子节点物化缓存**——removeChild/remove 把注册文本视图注销
//（R51c 防泄漏）后，被移除元素的 firstChild/childNodes 须仍可读（spec：detached 子树保留
// 其子——WPT NodeIterator-removal：remove paras[0] 后 `paras[0].firstChild` 期望 #text，
// 旧注销后 null → 用例 setup 直接崩 "Cannot read properties of null"）。注销前把融合视图
// 快照入 _zwDetachedChildren（handle 键），childNodes/firstChild/lastChild 读路径在
// 注册表 miss 后回落此缓存。防泄漏：512 软上限（与 _zwChildBaseCache 同款）。
var _zwDetachedChildren = new Map();
function _zwMaterializeDetachedChildren(el) {
  try {
    var kids = el.childNodes;
    if (!kids || !kids.length) return;
    var key = el.__zwHandle;
    if (!key) return;
    if (_zwDetachedChildren.size > 512) _zwDetachedChildren.clear();
    _zwDetachedChildren.set(key, Array.prototype.slice.call(kids));
  } catch (_e) {}
}
function _zwDetachedChildrenOf(handle) {
  if (!handle) return null;
  return _zwDetachedChildren.get(handle) || null;
}
// R87：物化缓存剔除单个子（removeChild 文本子路径——物化的移除前视图含 removed，
// spec 要求父视图不再含；handle 节点的本地 childNodes 读路径回落此缓存）。
function _zwDetachChildFromCache(handle, child) {
  var kids = _zwDetachedChildren.get(handle);
  if (!kids || !kids.length) return;
  var i = kids.indexOf(child);
  if (i >= 0) kids.splice(i, 1);
}
// 本地 childNodes（handle 元素无 sel 时读注册表）
// R52：handle/sel 双 Map 索引（同 _zwTextElsByEl——childNodes/firstChild/lastChild 每读全表
// 扫的 O(n²) 修复）。注册/注销同步维护。
var _zwTextElsByHandle = new Map();
var _zwTextElsBySel = new Map();
function _zwLocalChildNodes(sel, handle) {
  var e = (handle && _zwTextElsByHandle.get(handle)) || (sel && _zwTextElsBySel.get(sel)) || null;
  return e ? [e.node] : null;
}
// 临时 measure context（缓存——与页面 canvas 同共享 registry）
var _zwMeasureCtxHandle = null;
// 经 canvas measure 取文本 0 基字形几何 { width, rects: [[l,t,r,b]...] }。
function _zwTextElGeometry(text, font, direction, spacing) {
  if (typeof __zw_canvas_op !== 'function') return null;
  try {
    if (!_zwMeasureCtxHandle) {
      var id = String(__zw_canvas_op('0', 'getContext2d', '1000', '100'));
      if (!id || id.charAt(0) === '!') return null;
      _zwMeasureCtxHandle = id;
    }
    if (font) __zw_canvas_op(_zwMeasureCtxHandle, 'setFont', String(font));
    if (direction) __zw_canvas_op(_zwMeasureCtxHandle, 'setDirection', String(direction));
    if (spacing) __zw_canvas_op(_zwMeasureCtxHandle, 'setLetterSpacing', String(spacing));
    var wire = String(__zw_canvas_op(_zwMeasureCtxHandle, 'measureText', String(text)));
    var parts = wire.split('|');
    var p0 = (parts[0] || '').split(',');
    var width = parseFloat(p0[0]) || 0;
    var asc = parseFloat(p0[5]) || 0, desc = parseFloat(p0[6]) || 0;
    var rects = [];
    var pens = [];
    if (parts[1]) {
      var gs = parts[1].split(';');
      for (var gi = 0; gi < gs.length; gi++) {
        var gv = gs[gi].split(','); // pen,l,t,r,b
        var gpen = parseFloat(gv[0]) || 0;
        var gl = parseFloat(gv[1]) || 0, gt = parseFloat(gv[2]) || 0;
        var gr = parseFloat(gv[3]) || 0, gb = parseFloat(gv[4]) || 0;
        // 保留全部字形（含 0 墨迹空格——索引须与 getSelectionRects 的 glyphs 对齐；
        // 合并时 0 尺寸不影响 min/max）。
        rects.push([gl, gt, gr, gb]);
        // R34xx：字形原点并行数组——caretPositionFromPoint 的 glyph 中点规则
        //（与 getIndexFromOffset 同：中点 = 相邻原点中点，末字形 = 与文本右缘中点）。
        pens.push(gpen);
      }
    }
    return { width: width, ascent: asc, descent: desc, rects: rects, pens: pens };
  } catch (_e) { return null; }
}
function _zwTextElStyle(entry) {
  var font = '', direction = 'ltr', spacing = '';
  try {
    if (entry.el && entry.el.style && typeof entry.el.style.font === 'string' && entry.el.style.font) font = String(entry.el.style.font);
    if (entry.el && entry.el.style && typeof entry.el.style.direction === 'string' && entry.el.style.direction) direction = String(entry.el.style.direction);
    if (entry.el && entry.el.style && typeof entry.el.style.letterSpacing === 'string' && entry.el.style.letterSpacing) spacing = String(entry.el.style.letterSpacing);
  } catch (_e) {}
  return { font: font, direction: direction, spacing: spacing };
}
// Range.getClientRects：范围 [start,end) 字形并成行 rect（单行 → 1 个 rect，与
// Chromium getClientRects 行语义一致）；非注册文本 → null（调用方落 []）。
function _zwRangeClientRects(range) {
  if (!range || !range.startContainer) return null;
  var node = range.startContainer;
  var entry = null;
  for (var i = 0; i < _zwTextEls.length; i++) {
    if (_zwTextEls[i].node === node) { entry = _zwTextEls[i]; break; }
  }
  if (!entry) return null;
  var st = _zwTextElStyle(entry);
  var geom = _zwTextElGeometry(entry.text, st.font, st.direction, st.spacing);
  if (!geom) return null;
  var start = range.startOffset | 0, end = range.endOffset | 0;
  if (start > end) { var t = start; start = end; end = t; }
  if (end <= start || start >= geom.rects.length) return [];
  var l = Infinity, t2 = Infinity, r = -Infinity, b = -Infinity;
  var any = false;
  for (var i2 = start; i2 < end && i2 < geom.rects.length; i2++) {
    var g = geom.rects[i2];
    l = Math.min(l, g[0]); t2 = Math.min(t2, g[1]);
    r = Math.max(r, g[2]); b = Math.max(b, g[3]);
    any = true;
  }
  if (!any) return [];
  // R34xx：y/height 用字体 em 盒（与 API getSelectionRects 一致——baselines 断言
  // top=-ascent/bottom=+descent；主测试只比 x/width/height，双侧一致即通过）。
  return [new DOMRect(l, -(geom.ascent || 0), r - l, (geom.ascent || 0) + (geom.descent || 0))];
}
// 注册文本元素的 bounding rect（0 基，x/y=0——测试归一化绝对位置）
function _zwTextElBoundingRect(sel, handle) {
  for (var i = 0; i < _zwTextEls.length; i++) {
    var e = _zwTextEls[i];
    if ((handle && e.handle === handle) || (sel && e.sel === sel)) {
      var st = _zwTextElStyle(e);
      var geom = _zwTextElGeometry(e.text, st.font, st.direction, st.spacing);
      if (!geom) return null;
      return new DOMRect(0, 0, geom.width, (geom.ascent || 30) + (geom.descent || 0));
    }
  }
  return null;
}
// document.caretPositionFromPoint(x, y)：命中注册文本元素字形 → { offsetNode, offset }。
function _zwCaretFromPoint(x, y) {
  for (var i = 0; i < _zwTextEls.length; i++) {
    var e = _zwTextEls[i];
    var st = _zwTextElStyle(e);
    var geom = _zwTextElGeometry(e.text, st.font, st.direction, st.spacing);
    if (!geom) continue;
    // R34xx：y 为 div 相对（text_y = gBCR.y + height/2）——转基线坐标（基线在
    // ascent 处）：y' = y - ascent；垂直命中判断（单行文本恒中）。
    var yBase = y - (geom.ascent || 0);
    var vHit = false;
    for (var vi = 0; vi < geom.rects.length; vi++) {
      var gv = geom.rects[vi]; // [l, t, r, b]
      if (gv[2] > gv[0] && gv[3] > gv[1] && yBase >= gv[1] && yBase <= gv[3]) { vHit = true; break; }
    }
    if (!vHit) continue;
    // 边界语义与 getIndexFromOffset 一致（ltr：字形中点 < x 的字形数——中点 = 相邻
    // 原点中点、末字形与文本右缘中点；index-from-offset-edge-cases 的 a_width/2+1 →
    // 1 等；rtl：左缘 > x 的字形数）——字形间隙返回边界索引（非 null）。
    var cnt = 0;
    var rtl = (st.direction === 'rtl');
    for (var gi = 0; gi < geom.rects.length; gi++) {
      var g = geom.rects[gi]; // [l, t, r, b]
      if (g[2] <= g[0] && g[3] <= g[1]) continue;
      if (rtl) {
        if (g[0] > x) cnt++;
      } else {
        var nextPen = (gi + 1 < geom.pens.length) ? geom.pens[gi + 1] : geom.width;
        var center = (geom.pens[gi] + nextPen) / 2;
        if (center < x) cnt++;
      }
    }
    return { offsetNode: e.node, offset: cnt };
  }
  return null;
}
