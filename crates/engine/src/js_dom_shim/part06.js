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
      if (node === root) { orderPos = 0; return; }
      for (var i = 0; i < order.length; i++) { if (order[i] === node) { orderPos = i; return; } }
      orderPos = -1;
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
                currentNodeVal = node; syncOrderPosTo(node); idx = accepted.indexOf(node); return node;
              }
              if (r === 2) {
                sibling = node.previousSibling; // REJECT → 跳过子树
              } else {
                sibling = node.lastChild || node.previousSibling; // SKIP/0 → 先入子树尾
              }
            }
            node = node.parentNode;
            if (!node || node === root) return null;
            var rp = check(node);
            if (rp === 1) { currentNodeVal = node; syncOrderPosTo(node); idx = accepted.indexOf(node); return node; }
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
      var q = String(sel);
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
      if (!all) return _zwMakeCollection([], false);
      return _zwMakeCollection(all.split('|').filter(Boolean).map(_wrapSelector), false);
    },
    getElementsByClassName: function(cls) {
      // R3019：honor `this` for cross-document use（DOMPurify 等库 getElementsByClassName.call(parsedDoc, cls)
      // 须查 parsedDoc 而非页面 document）。this === 页面 document 时走页面 DOM；否则委托 this.querySelectorAll。
      // R3033：返 HTMLCollection（item + namedItem），包 _zwMakeCollection(arr, true)。
      // R50：liveSpec——同步脚本内 append/remove 后集合 lazy 重查（matches 按 class 判定归属）。
      if (this && this !== globalThis.document && typeof this.querySelectorAll === 'function') {
        return _zwMakeCollection(this.querySelectorAll('.' + cls), true);
      }
      var clsStr = String(cls);
      return _zwMakeCollection(globalThis.document.querySelectorAll('.' + cls), true, {
        matches: function (el) {
          try { return !!el && (' ' + (el.className || '') + ' ').indexOf(' ' + clsStr + ' ') !== -1; } catch (_e) { return false; }
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
      return _zwMakeCollection(_zwFilterByTagNameNS(_zwDocAllElements(), _r120Tag, undefined), true,
        { matches: _zwLiveMatchesFor(_r120Tag, undefined) });
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
      if (/[\s>]/.test(_q)) {
        _throwDom('InvalidCharacterError', 'The string contains invalid characters.');
      }
      if (_pre === null) {
        if (!_zwIsNameStartChar(Array.from(_q)[0])) {
          _throwDom('InvalidCharacterError', 'The string contains invalid characters.');
        }
      } else {
        var _locChars = Array.from(_loc);
        if (!_locChars.length || !_zwIsNameStartChar(_locChars[0])) {
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
      var isHtmlDoc = !(typeof this.contentType === 'string' && this.contentType.indexOf('html') < 0);
      var n = isHtmlDoc ? t.replace(/[A-Z]/g, function (c) { return String.fromCharCode(c.charCodeAt(0) + 32); }) : t;
      return _zwMakeAttr(n, '', null);
    },
    // R3024：`document.createAttributeNS(ns, qualifiedName)`——建命名空间 Attr（SVG/MathML/xlink）。
    // 解析 qualifiedName 的 `prefix:local`，设 namespaceURI/prefix/localName（区别 createAttribute 的 null ns）。
    // 值 ''，ownerElement=null（游离）。返 Attr instanceof Attr（经 _zwMakeAttr 的 Object.create(Attr.prototype)）。
    createAttributeNS: function(ns, qualifiedName) {
      var q = String(qualifiedName);
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
    // `document.createProcessingInstruction(target, data)`（js-dom M4，spec `dom-document-createprocessinginstruction`）——
    // PI 节点（nodeType 7，target/data/nodeName=target）。spec 校验在调用点同步抛 DOMException（与 native
    // dom_bindings factories.rs 对齐：① target 须合法 Name production ② data 不得含 `?>`，违则
    // InvalidCharacterError）。合法经 host `__zw_create_processing_instruction`（apply 时 doc.create_processing_instruction）。
    createProcessingInstruction: function(target, data) {
      var t = String(target == null ? '' : target);
      var d = String(data == null ? '' : data);
      // spec 步骤 2：target 须合法 Name（复用 R3 createElement 校验 helper）。
      if (!_zwIsValidQualifiedName(t)) {
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
        // 期望表实证宽松——''/'1foo'/'edi:`'/'edi:<'/{/} 全 pass；仅含空白或 '>' throw
        // INVALID_CHARACTER_ERR——与元素 Name 产线不同，doctype 名仅禁此二类）。
        var _r130Dq = String(qualifiedName == null ? '' : qualifiedName);
        if (/[\s>]/.test(_r130Dq)) {
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
      var dt = {
        nodeType: 10,
        name: 'html',
        nodeName: 'html',
        publicId: '',
        systemId: '',
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
        contains: function (other) { return _zwNodeContains(dt, other); },
        compareDocumentPosition: function (other) { return _zwCompareDocumentPosition(dt, other); },
        // R81：主文档 doctype 导航面（WPT Node-properties doctype.nextSibling 期望 html——
        // document.childNodes = [doctype, html]；firstChild/lastChild/parentElement 恒 null）。
        get firstChild() { return null; },
        get lastChild() { return null; },
        get parentElement() { return null; },
        get previousSibling() { return null; },
        get nextSibling() { return (globalThis.document && globalThis.document.documentElement) || null; },
      };
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
    get childNodes() {
      // R79 注记：曾「不含 doctype」与 WPT oracle previousNode 遍历世界对齐（html.previousSibling
      // 快照恒 null）。R81 spec 纠正：真浏览器 document.childNodes = [doctype, html]（WPT
      // Node-properties document.childNodes.length 期望 2、childNodes[0] 为 DocumentType）。
      // html.previousSibling 仍走 __zw_sibling_nodes 快照（R80 后 R79 的遍历一致性问题已由
      // JS 侧 _zwCompareDocumentPosition 链式判定取代，不依赖该子序）。
      // R87：_docDtorRemoved（removeChild(doctype) 本地标记）时剔除 doctype（恢复段
      // insertBefore 还原）。
      if (this._docDtorRemoved) return [_wrapSelector('html')];
      return [this.doctype, _wrapSelector('html')];
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
        return c;
      }
      if (c && c.__zwSelector && typeof __zw_remove === 'function') {
        if (globalThis._zwNotifyIteratorsRemove) {
          try { globalThis._zwNotifyIteratorsRemove(c); } catch (_e87e) {}
        }
        try { __zw_remove(c.__zwSelector); } catch (_e2) {}
        if (typeof _zwMarkRemoved === 'function') _zwMarkRemoved(c.__zwSelector);
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
      // R105：document target 的 touch/wheel 族默认 passive（spec default-passive-value）。
      _listenerStore[key][t].push({ fn: fn, capture: _optCapture(opts), once: _optOnce(opts), tgt: 'doc',
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
  globalThis.window = globalThis;
  globalThis.addEventListener = _globalAddEventListener;
  globalThis.removeEventListener = _globalRemoveEventListener;
  // R2932 `window.dispatchEvent`——window 为 EventTarget（spec 有 dispatchEvent）。R40 改经
  // `_dispatchWithBubble(…, 'win')`：window 为 target（AT_TARGET 只触发 tgt='win' 槽位注册，含
  // window.addEventListener + on* handler 注册），path = [window]，返 `!defaultPrevented`（spec）。
  globalThis.dispatchEvent = function(event) {
    // R106：spec 入口守卫（同 document.dispatchEvent）。
    globalThis._zwDispatchGuard(event);
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
  globalThis.window.attachEvent = function(type, fn) {
    _attachEventForKey(_elKey('html', null), type, fn);
  };
  globalThis.window.detachEvent = function(type, fn) {
    _detachEventForKey(_elKey('html', null), type, fn);
  };
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
  [
    'afterprint', 'beforeprint', 'beforeunload', 'hashchange', 'languagechange', 'message', 'messageerror',
    'offline', 'online', 'pagehide', 'pageshow', 'popstate', 'rejectionhandled', 'storage', 'unhandledrejection',
    'unload', 'load', 'error', 'resize', 'scroll', 'focus', 'blur',
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

  // 构造 Range（document.createRange / selectNode* 等用）。**已知限制**：① toString 精确覆盖 selectNode/
  // selectNodeContents（整节点子树文本）+ 同文本节点 setStart/setEnd（slice 偏移）；其余 setStart/setEnd
  // 组合 best-effort 取 commonAncestor 子树文本（跨节点偏移不精确截取）；② deleteContents/extractContents/
  // insertNode/cloneContents/surroundContents（R2929/R2930）经既有 mutation-emitting proxy
  //（remove/insertBefore/appendChild/cloneNode）真实变更——精确覆盖 start==end 元素容器的 offset 区间
  //（selectNode/selectNodeContents 后），sel/handle 子均支持；surroundContents 精确落位仅在覆盖块延伸到容器
  // 末尾（selectNodeContents 包整元素内容），非尾部 best-effort 落末尾；跨容器/文本节点部分切片仍 best-effort；
  // ③ getBoundingClientRect/getClientRects 返空（无 layout 选择几何）；④ 无真 live。
  function _makeRange() {
    return {
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
      // 旧初版把 Attr 一并拒绝 → "at offset 0 is allowed" 误伤。Offset 校验：仅当 length 可判定时
      //（detached/handle-only proxy childNodes 恒空但树存在——children 视图缺失时 childNodes.length===0
      // 与真 length 无法区分 → 对元素容器放宽不抛，保既有用例不回归；文本/注释/PI data 可判定仍精确校验）。
      setStart: function (node, off) {
        if (!node || typeof node.nodeType !== 'number' || node.nodeType === 10) {
          throw new globalThis.DOMException('The given node is invalid.', 'InvalidNodeTypeError');
        }
        var o = off | 0;
        if (o < 0) throw new globalThis.DOMException('The given offset is out of bounds.', 'IndexSizeError');
        if (node.nodeType !== 1 && o > this._nodeLength(node)) {
          throw new globalThis.DOMException('The given offset is out of bounds.', 'IndexSizeError');
        }
        this.startContainer = node; this.startOffset = o; this._recalc(); return this;
      },
      setEnd: function (node, off) {
        if (!node || typeof node.nodeType !== 'number' || node.nodeType === 10) {
          throw new globalThis.DOMException('The given node is invalid.', 'InvalidNodeTypeError');
        }
        var o = off | 0;
        if (o < 0) throw new globalThis.DOMException('The given offset is out of bounds.', 'IndexSizeError');
        if (node.nodeType !== 1 && o > this._nodeLength(node)) {
          throw new globalThis.DOMException('The given offset is out of bounds.', 'IndexSizeError');
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
        this.startContainer = sp; this.startOffset = i;
        this.endContainer = sp; this.endOffset = i + 1;
        this.commonAncestorContainer = sp; this.collapsed = false; this._mode = { node: node, kind: 'node' };
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
      // R2929：收集 range 覆盖的「顶层子节点」（start==end 元素容器 + offset 区间）。selectNode/
      // selectNodeContents 后的精确情况。跨容器 / 文本节点部分切片 → null（toString 已处理文本 slice）。
      _coveredChildren: function () {
        if (!this.startContainer || this.startContainer !== this.endContainer) return null;
        var sc = this.startContainer;
        if (sc.nodeType !== 1 && !sc.tagName) return null; // 非元素容器（文本切片）→ defer
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
        var f = globalThis.document.createDocumentFragment();
        var kids = this._coveredChildren();
        if (kids) {
          for (var i = 0; i < kids.length; i++) {
            try { f.appendChild(kids[i].cloneNode(true)); } catch (_e) {}
          }
          for (var j = kids.length - 1; j >= 0; j--) {
            try { if (typeof kids[j].remove === 'function') kids[j].remove(); } catch (_e) {}
          }
          this.collapse(true);
        }
        return f;
      },
      cloneContents: function () {
        // R2929：真实子树克隆（cloneNode deep）到 fragment。元素容器 + offset 区间精确；
        // 跨容器/文本节点容器回落文本（既有 best-effort）。
        var f = globalThis.document.createDocumentFragment();
        var kids = this._coveredChildren();
        if (kids) {
          for (var i = 0; i < kids.length; i++) {
            try { f.appendChild(kids[i].cloneNode(true)); } catch (_e) {}
          }
        } else {
          var t = this.toString();
          if (t) f.appendChild(globalThis.document.createTextNode(t));
        }
        return f;
      },
      insertNode: function (node) {
        // 在 startContainer 的 startOffset 位置插入 node（created 节点）。off < 子数 → insertBefore(ref)，否则
        // appendChild。复用既有 insertBefore/appendChild（emit mutation）。返回 node（spec）。
        if (!node || !this.startContainer) return node;
        try {
          var kids = this.startContainer.childNodes;
          var off = this.startOffset | 0;
          if (kids && off < kids.length && kids[off]) {
            this.startContainer.insertBefore(node, kids[off]);
          } else {
            this.startContainer.appendChild(node);
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
        var kids = this._coveredChildren();
        if (kids === null) return; // 跨容器/文本切片 defer
        if (kids.length === 0) { this.insertNode(newParent); return; }
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
        for (var j = 0; j < kids.length; j++) {
          var _rprev = null, _rnext = null;
          try {
            _rprev = kids[j].previousSibling || null;
            _rnext = kids[j].nextSibling || null;
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
        try { this.startContainer.appendChild(newParent); } catch (_e) {}
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
  globalThis.Range = function Range() { return _makeRange(); };
  // js-dom M4 R42：`StaticRange` 构造器（spec `dom-staticrange`）——读 RangeInit dict（startContainer/
  // startOffset/endContainer/endOffset），属性 readonly，无 setStart/setEnd 等 mutable 方法。
  // WPT StaticRange-constructor：合法容器（Element/Text/PI/Comment）构造 + collapsed 派生 +
  // 非 Node 容器抛 TypeError。
  globalThis.StaticRange = function StaticRange(init) {
    var d = init || {};
    var sc = d.startContainer, ec = d.endContainer;
    var isNode = function (n) { return !!n && typeof n.nodeType === 'number'; };
    if (!isNode(sc) || !isNode(ec)) {
      throw new globalThis.TypeError("StaticRangeInit containers must be Nodes");
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
  function _zwSettleResourceSelector(sel, tag, url, outcome, width, height) {
    var key = _elKey(sel, null);
    if (_resourceStates[key]) return false; // 每个资源请求只 settle / 派发一次。
    var state = {
      url: String(url), outcome: String(outcome),
      width: Math.max(0, Number(width) || 0), height: Math.max(0, Number(height) || 0),
      error: outcome === 'error' ? _zwMediaError(2, 'Error loading resource: ' + String(url)) : null
    };
    _resourceStates[key] = state;
    var eventType = '';
    if (tag === 'img') eventType = outcome === 'error' ? 'error' : 'load';
    else if (tag === 'track') eventType = outcome === 'error' ? 'error' : 'load';
    else if ((tag === 'source' || tag === 'audio' || tag === 'video') && outcome === 'error') eventType = 'error';
    if (eventType) {
      _dispatchWithBubble(key, sel, null, _makeEvent(eventType, { bubbles: false, cancelable: false }));
    }
    return true;
  }
  globalThis.__zw_commit_resource_element_state = function (tag, absUrl, outcome, width, height) {
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
        var committed = _zwSettleResourceSelector(sel, tag, target, outcome, width, height);
        if (!committed || tag !== 'source') continue;
        var parent = _parentNodeFor(sel, null);
        var parentTag = parent && parent.tagName ? String(parent.tagName).toLowerCase() : '';
        if (parentTag !== 'audio' && parentTag !== 'video') continue;
        if (outcome !== 'error') {
          _zwSettleResourceSelector(parent.__zwSelector, parentTag, target, 'available', 0, 0);
          continue;
        }
        var candidates = parent.querySelectorAll ? parent.querySelectorAll('source') : [];
        var allFailed = candidates.length > 0;
        for (var j = 0; j < candidates.length; j++) {
          var candidateState = _resourceStates[_elKey(candidates[j].__zwSelector, null)];
          if (!candidateState || candidateState.outcome !== 'error') { allFailed = false; break; }
        }
        if (allFailed) _zwSettleResourceSelector(parent.__zwSelector, parentTag, target, 'error', 0, 0);
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
    var ok = _dispatchWithBubble(_elKey(sel, null), sel, null, ev);
    return ok ? 'ok' : 'prevented';
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
  node.deleteData = function (o, c2) {
    var a = Math.max(0, o | 0), b = Math.max(0, c2 | 0);
    node.__prevForMo = node.__nv; _regWrite(node.__nv.slice(0, a) + node.__nv.slice(a + b));
  };
  node.insertData = function (o, s) {
    var a = Math.max(0, o | 0);
    node.__prevForMo = node.__nv; _regWrite(node.__nv.slice(0, a) + String(s == null ? '' : s) + node.__nv.slice(a));
  };
  node.replaceData = function (o, c2, s) {
    var a = Math.max(0, o | 0), b = Math.max(0, c2 | 0);
    node.__prevForMo = node.__nv;
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
