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
    querySelector: function(sel) {
      var hit = __zw_query_match(sel);
      return hit ? _wrapSelector(hit) : null;
    },
    getElementById: function(id) {
      return globalThis.document.querySelector('#' + id);
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
      var all = __zw_query_all(sel);
      if (!all) return _zwMakeCollection([], false);
      return _zwMakeCollection(all.split('|').filter(Boolean).map(_wrapSelector), false);
    },
    getElementsByClassName: function(cls) {
      // R3019：honor `this` for cross-document use（DOMPurify 等库 getElementsByClassName.call(parsedDoc, cls)
      // 须查 parsedDoc 而非页面 document）。this === 页面 document 时走页面 DOM；否则委托 this.querySelectorAll。
      // R3033：返 HTMLCollection（item + namedItem），包 _zwMakeCollection(arr, true)。
      if (this && this !== globalThis.document && typeof this.querySelectorAll === 'function') {
        return _zwMakeCollection(this.querySelectorAll('.' + cls), true);
      }
      return _zwMakeCollection(globalThis.document.querySelectorAll('.' + cls), true);
    },
    getElementsByTagName: function(tag) {
      // R3019：honor `this` for cross-document use（DOMPurify _initDocument 经 getElementsByTagName.call(doc,'body')[0]
      // 取 parsed doc 的 body——旧实现恒查页面 document 致 DOMPurify 清洗空页面 body 返 ""）。
      // R3033：返 HTMLCollection（item + namedItem），包 _zwMakeCollection(arr, true)。
      if (this && this !== globalThis.document && typeof this.querySelectorAll === 'function') {
        return _zwMakeCollection(this.querySelectorAll(String(tag)), true);
      }
      return _zwMakeCollection(globalThis.document.querySelectorAll(tag), true);
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
    // R3023：`document.createAttribute(name)`——建 Attr 节点（nodeType 2，value=''）。供 setAttributeNode /
    // element.attributes.setNamedItem(attr) 用法（属性库 / 序列化库高频）。真 Attr 实例（经 _zwMakeAttr，
    // 含 localName/namespaceURI=null/prefix=null/specified/ownerElement=null 全字段，非 plain {name,value}）。
    createAttribute: function(name) {
      return _zwMakeAttr(name, '', null);
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
        pagetransitionevent: globalThis.PageTransitionEvent,
        clipboardevent: globalThis.ClipboardEvent,
        dragevent: globalThis.DragEvent,
        errorevent: globalThis.ErrorEvent,
      };
      var Ctor = (map[t] && typeof map[t] === 'function') ? map[t] : globalThis.Event;
      // 构造器接收 (type, options)；createEvent 返**空 type** 事件（initEvent/initCustomEvent 设 type）。
      return new Ctor('');
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
      }
      return true;
    },
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
    // + createDocument/createHTMLDocument（R3013：返 queryable detached Document——body.innerHTML setter +
    // querySelector 族经 __zw_parse_html_query 查解析树，jQuery/DOMPurify feature-detect / 模板引擎可用）。
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
      if (String(type) === 'pageshow') _maybeFirePageShow(); // R2931：首次 pageshow listener → _defer 派发一次
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
  // R2932 `window.dispatchEvent`——window 为 EventTarget（spec 有 dispatchEvent）。复用 window listener
  // 派发路径（_elKey('html', null) + 'all' phase），返 `!defaultPrevented`（spec）。使合成事件可测 on* handler。
  globalThis.dispatchEvent = function(event) {
    if (!event || typeof event.type !== 'string') return true;
    if (!event.target) event.target = globalThis;
    return _dispatchToListeners(_elKey('html', null), event, 'all', globalThis);
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
  var _fontsReadyResolve = null;
  var _fontFaceSetFaces = []; // FontFace 对象列表（add/delete 管理；values/forEach/size/迭代反映）
  var _fontFaceSet = {
    status: 'loaded', // 'loading' | 'loaded'（headless 简化：初始即 loaded，settle 时不改）
    onloading: null, onloadingdone: null, onloadingerror: null,
    ready: new Promise(function (resolve) { _fontsReadyResolve = resolve; }),
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
  // R2949 FontFace——单字体面（CSS Font Loading API face 层，补全 R2947 set 层）。
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
        __zw_load_font(self.family, self._src, id, weightNum, isItalic);
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
        for (var j = kids.length - 1; j >= 0; j--) {
          try { if (typeof kids[j].remove === 'function') kids[j].remove(); } catch (_e) {}
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
  // 元素级派发便捷封装（R2943 img / R2944 link / script）。
  globalThis.__zw_dispatch_img_event = function (absUrl, type) {
    __zw_dispatch_element_event('img', 'src', absUrl, type);
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
