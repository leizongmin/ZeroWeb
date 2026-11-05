// part01e（2026-11-01 editing-compat F1 list 命令簇；2026-11-05 f2 残余面）：
// execCommand insertorderedlist/insertunorderedlist 实应用引擎 + qState/
// qIndeterm 读面（toggleLists/getSelectionListState 参考实现移植）。挂 part01d
// 后同闭包复用 _zwJus* 机具（blockExtend/containedNodes/wrap/
// movePreservingRanges/setTagName/followsLineBreak 系）与 part01f
// _zwDelRecordValues/_zwDelRestoreValues（vendored :2385/:2427 忠实版）；list
// 域 isAllowedChild 走本文件 _zwListIsAllowedChild 忠实尾段（part01d
// _zwVaIsAllowedChild 是值命令启发式，无 per-parent 表/pair 表，只服务值域）。
// part06 经 globalThis.__zw_ce_list_toggle / __zw_ce_list_state 单点调用，
// 事件口径同 justify 臂（beforeinput 已由事件序派发、input 走既有尾部）。
// 规范：https://w3c.github.io/editing/execCommand.html#the-insertorderedlist-
// command（toggleLists :5690 / getSelectionListState :3949）。
// f2 收口（iloop/illist 语料 dd/dt 聚合、嵌套归并、样式保全、缩进合并面）：
// ② normalizeSublists + items 祖先容器归并（vendored :3904/:5704）；③
// recordValues/restoreValues 跨父移动样式保全（disable/toConvert/otherTagName
// 三位点）；④ isSimpleIndentationElement 缩进合并新父臂（vendored :3775/:5922）。
// f3 收口（q9-q11 门禁批；P2F 17→9→0 收敛链）：_zwListMovePreservingRanges
// plain 拷贝重锚变体（splitParent/normalizeSublists/outdent 移动点）、
// _zwListRetagPreservingSelection（fd-retag/outdent-attr 臂选区路径快照回写，
// 修 setTagName detached-repl 窗口锚塌缩）、normalize 前奏 disable 单面门
//（enable 录制保留 li 内嵌列表形）、disable 出口端点身份回植 +
// enable-retag 交集钳制（qInd/qS 恢复选区读面）。
// FIXME（切片范围，语料面挂后续切片）：① outdentNode 仅 ol/ul 臂（simple/
// indentation 元素臂 + ancestorList 缩进臂挂后续）；⑤ blockExtend 单行容器
// 边界面（foo<br>bar 跨行 enable 的 br 落位形，vendored blockExtend 边界臂
// 未移植）；⑥ qState/qIndeterm 读面残余（语料 readface 67F 族）。

  function _zwListIsIndentationElement(node) {
    if (!_zwJusIsHtmlElement(node)) return false;
    var tn = String(node.tagName || '').toUpperCase();
    if (tn === 'BLOCKQUOTE') return true;
    if (tn !== 'DIV') return false;
    // 近似检查（vendored 同注）：style 属性串含 margin 族即视缩进元素——
    // 物化面 style IDL 无索引枚举面，属性串解析口径等价。
    var st = String(node.getAttribute && node.getAttribute('style') || '');
    return /(^|;)\s*(-[a-z]+-)?margin/i.test(st);
  }

  // isSimpleIndentationElement（vendored :3775 忠实移植）：缩进元素 + 全属性
  // 为 HTML ns 的 style/dir + style 声明属性名全带 margin/border/padding 前缀。
  // 物化面 style IDL 无索引枚举（同上注）——属性串逐声明拆名，属性名正则与
  // vendored node.style[i] 同式（无冒号破声明 CSSOM 丢弃，跳过）。
  function _zwListIsSimpleIndentationElement(node) {
    if (!_zwListIsIndentationElement(node)) return false;
    var attrs = node.attributes;
    var n = attrs && attrs.length ? attrs.length : 0;
    for (var i = 0; i < n; i++) {
      var a = attrs[i];
      var nsOk = true;
      try {
        var ns = a.namespaceURI;
        nsOk = ns === null || ns === 'http://www.w3.org/1999/xhtml';
      } catch (_eNs) { nsOk = true; }
      if (!nsOk || ['style', 'dir'].indexOf(a.name) === -1) return false;
    }
    var st = '';
    try { st = String(node.getAttribute('style') || ''); } catch (_eSt) { st = ''; }
    var decls = st.split(';');
    for (var d = 0; d < decls.length; d++) {
      var decl = decls[d];
      if (!/^[ \t\n\f\r]*$/.test(decl)) {
        var ci = decl.indexOf(':');
        if (ci === -1) continue;
        var prop = decl.slice(0, ci).replace(/^[ \t\n\f\r]+/, '').replace(/[ \t\n\f\r]+$/, '');
        if (!/^(-[a-z]+-)?(margin|border|padding)/.test(prop)) return false;
      }
    }
    return true;
  }

  function _zwListEditingHostOf(node) {
    var n = node;
    var g = 0;
    while (n && g++ < 256) {
      if (_zwJusIsEditingHost(n)) return n;
      n = n.parentNode;
    }
    return null;
  }

  function _zwListInSameEditingHost(node1, node2) {
    var h1 = _zwListEditingHostOf(node1);
    return !!h1 && h1 === _zwListEditingHostOf(node2);
  }

  // list 域移动原语（part01c _zwJusMovePreservingRanges 的 plain 重锚变体）：
  // shim 桥对 plain（parser 产）子做序列化拷贝不保身份，「端点在 node 内随动」
  // 的补偿假设对真移动成立、对拷贝不成立——removeChild 后原节点脱离，锚死亡
  //（q9 P2F 代表面实证：foo@0 起锚死→选区塌 bar@3→qS 读 true）。故移动后
  // 若 node 身份未落 newParent（拷贝路径），把 node 内端点容器沿子索引路径
  // 迁到插入产物同位副本。真移动（shim 节点）路径与 part01c 版逐语义一致。
  // FIXME：part01c 版修复 plain 重锚后（item-4 域）此处本地件可回收。
  function _zwListMovePreservingRanges(node, newParent, newIndex) {
    if (!node || !newParent) return;
    var idx = newIndex === -1 ? (newParent.childNodes ? newParent.childNodes.length : 0) : newIndex;
    var oldParent = node.parentNode;
    var oldIndex = _zwJusNodeIndex(node);

    var rng = null;
    try {
      var sel = globalThis.getSelection ? globalThis.getSelection() : null;
      rng = sel && sel._ranges && sel._ranges.length > 0 ? sel._ranges[0] : null;
    } catch (_eJselL) {}
    var sc = rng ? rng.startContainer : null;
    var so = rng ? rng.startOffset : 0;
    var ec = rng ? rng.endContainer : null;
    var eo = rng ? rng.endOffset : 0;

    // 端点在 node 内（同或后代）→ 随动不补偿（spec movePreservingRanges 1181）。
    var epInNode = function (n) {
      return n === node || _zwJusIsAncestorOf(node, n);
    };
    if (rng) {
      if (!epInNode(sc)) {
        if (sc === newParent && so > idx) so++;
        if (sc === oldParent && (so === oldIndex || so === oldIndex + 1)) {
          sc = newParent; so += idx - oldIndex;
        } else if (sc === oldParent && so > oldIndex + 1) {
          so--;
        }
      }
      if (!epInNode(ec)) {
        if (ec === newParent && eo > idx) eo++;
        if (ec === oldParent && (eo === oldIndex || eo === oldIndex + 1)) {
          ec = newParent; eo += idx - oldIndex;
        } else if (ec === oldParent && eo > oldIndex + 1) {
          eo--;
        }
      }
    }

    // plain 摘挂纪律（R54xx）：先摘原父再插（桥拷贝不摘原父）。
    if (oldParent && typeof oldParent.removeChild === 'function') {
      try { oldParent.removeChild(node); } catch (_eJrmL) {}
    }
    var ref = newParent.childNodes && idx < newParent.childNodes.length
      ? newParent.childNodes[idx] : null;
    try {
      if (ref) newParent.insertBefore(node, ref);
      else newParent.appendChild(node);
    } catch (_eJinsL) {
      try { newParent.appendChild(node); } catch (_eJinsL2) {}
    }

    // 拷贝路径判别：真移动后 node.parentNode === newParent；桥拷贝后原节点
    // 脱离（parentNode null）。此时端点容器若在 node 内，沿子索引路径迁到
    // newParent 里的插入产物（ref 前一位 / 尾子）同位副本。
    if (rng && node.parentNode !== newParent) {
      var moved = ref && ref.previousSibling ? ref.previousSibling : newParent.lastChild;
      var remap = function (n) {
        if (n === node) return moved;
        var path = [];
        var w = n;
        while (w && w !== node && w.parentNode) { path.unshift(_zwJusNodeIndex(w)); w = w.parentNode; }
        if (w !== node) return null;
        var m = moved;
        for (var pi = 0; pi < path.length && m; pi++) m = m.childNodes ? m.childNodes[path[pi]] : null;
        return m;
      };
      var nsc = remap(sc);
      if (nsc) sc = nsc;
      var nec = remap(ec);
      if (nec) ec = nec;
    }

    if (rng) {
      try {
        rng.setStart(sc, so);
        rng.setEnd(ec, eo);
      } catch (_eJrngL) {}
    }
  }

  // splitParent（vendored :5058 忠实移植）：nodeList 整体提到原父之前/之后。
  // 尾段独在臂（!firstIn && lastIn）逆序后移 + 尾 br 保全后 abort；首段缺位臂
  // 克隆壳承接头部；中段通用路径：成员前移 + 首尾 br 保全 + 空壳删除。
  // 值录还（vendored 同构）在调用位点（disable/toConvert/otherTagName 臂），
  // splitParent 本体不录还。
  function _zwListSplitParent(nodeList) {
    var originalParent = nodeList[0].parentNode;
    if (!_zwJusIsEditable(originalParent) || !originalParent.parentNode) return;
    var firstIn = nodeList.indexOf(originalParent.firstChild) !== -1;
    var lastIn = nodeList.indexOf(originalParent.lastChild) !== -1;
    if (firstIn) _zwJusRemoveExtraneousLineBreaksBefore(originalParent);
    var followsLb = firstIn && _zwJusFollowsLineBreak(originalParent);
    var precedesLb = lastIn && _zwJusPrecedesLineBreak(originalParent);
    if (!firstIn && lastIn) {
      for (var ri = nodeList.length - 1; ri >= 0; ri--) {
        _zwListMovePreservingRanges(nodeList[ri], originalParent.parentNode, 1 + _zwJusNodeIndex(originalParent));
      }
      if (precedesLb && !_zwJusPrecedesLineBreak(nodeList[nodeList.length - 1])) {
        nodeList[nodeList.length - 1].parentNode.insertBefore(document.createElement('br'), nodeList[nodeList.length - 1].nextSibling);
      }
      _zwJusRemoveExtraneousLineBreaksAtTheEndOf(originalParent);
      return;
    }
    if (!firstIn) {
      var clonedParent = originalParent.cloneNode(false);
      try { originalParent.removeAttribute('id'); } catch (_eRid) {}
      originalParent.parentNode.insertBefore(clonedParent, originalParent);
      while (nodeList[0].previousSibling) {
        _zwListMovePreservingRanges(originalParent.firstChild, clonedParent, clonedParent.childNodes.length);
      }
    }
    for (var mi = 0; mi < nodeList.length; mi++) {
      _zwListMovePreservingRanges(nodeList[mi], originalParent.parentNode, _zwJusNodeIndex(originalParent));
    }
    if (followsLb && !_zwJusFollowsLineBreak(nodeList[0])) {
      nodeList[0].parentNode.insertBefore(document.createElement('br'), nodeList[0]);
    }
    if (_zwJusIsInlineNode(nodeList[nodeList.length - 1])
    && !_zwJusIsHtmlElement(nodeList[nodeList.length - 1], 'br')
    && _zwJusIsHtmlElement(originalParent.firstChild, 'br')
    && !_zwJusIsInlineNode(originalParent)) {
      originalParent.removeChild(originalParent.firstChild);
    }
    if (!originalParent.hasChildNodes()) {
      originalParent.parentNode.removeChild(originalParent);
      if (precedesLb && !_zwJusPrecedesLineBreak(nodeList[nodeList.length - 1])) {
        nodeList[nodeList.length - 1].parentNode.insertBefore(document.createElement('br'), nodeList[nodeList.length - 1].nextSibling);
      }
    } else {
      _zwJusRemoveExtraneousLineBreaksBefore(originalParent);
    }
    if (!nodeList[nodeList.length - 1].nextSibling
    && nodeList[nodeList.length - 1].parentNode) {
      _zwJusRemoveExtraneousLineBreaksAtTheEndOf(nodeList[nodeList.length - 1].parentNode);
    }
  }

  // isAllowedChild（vendored :1629 忠实移植，F1 dd/dt 聚合面根因修复）：
  // part01d _zwVaIsAllowedChild 是值命令域启发式（无 per-parent switch 表、无
  // dd/dt/li 归属检查、无 pair 表），list 域尾段照录——dd/dt 只入 dl、li 只入
  // ol/ul、同名 pair 拒绝。items 聚合谓词与 fixDisallowedAncestors 同源走此。
  // prohibited 名单 vendored :701 照录（与 part01d +main 的 Chrome delta 名单
  // 分开，list 域语料权威 = vendored）。
  // https://w3c.github.io/editing/execCommand.html#allowed-children
  var _zwListProhibitedParagraphChildNames = ['address', 'article', 'aside',
    'blockquote', 'caption', 'center', 'col', 'colgroup', 'dd', 'details',
    'dir', 'div', 'dl', 'dt', 'fieldset', 'figcaption', 'figure', 'footer',
    'form', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'header', 'hgroup', 'hr', 'li',
    'listing', 'menu', 'nav', 'ol', 'p', 'plaintext', 'pre', 'section',
    'summary', 'table', 'tbody', 'td', 'tfoot', 'th', 'thead', 'tr', 'ul',
    'xmp'];

  function _zwListIsAllowedChild(child, parent_) {
    var i;
    // table 族父的非空白 Text 子 → false。
    var tableFamily = ['colgroup', 'table', 'tbody', 'tfoot', 'thead', 'tr'];
    if ((tableFamily.indexOf(parent_) !== -1 || _zwJusIsHtmlElement(parent_, tableFamily))
    && child && typeof child === 'object' && child.nodeType === 3
    && !/^[ \t\n\f\r]*$/.test(String(child.data))) {
      return false;
    }
    // script/style/plaintext/xmp 父的非 Text 子 → false。
    var rawText = ['script', 'style', 'plaintext', 'xmp'];
    if ((rawText.indexOf(parent_) !== -1 || _zwJusIsHtmlElement(parent_, rawText))
    && (typeof child !== 'object' || child.nodeType !== 3)) {
      return false;
    }
    // Document/DocumentFragment/DocumentType 子 → false。
    if (child && typeof child === 'object'
    && (child.nodeType === 9 || child.nodeType === 11 || child.nodeType === 10)) {
      return false;
    }
    // HTML 元素子 → 本地名；非字符串子（Text/Comment 等）→ true。
    var childName = null;
    if (_zwJusIsHtmlElement(child)) childName = _zwJusTag(child);
    if (childName === null) return true;
    // HTML 元素 parent：祖先链三检（a∈a / prohibited∈inline-contents / hN∈hN）。
    if (_zwJusIsHtmlElement(parent_)) {
      var ancestor = parent_;
      var guard = 0;
      while (ancestor && guard++ < 256) {
        if (childName === 'a' && _zwJusIsHtmlElement(ancestor, 'a')) return false;
        if (_zwListProhibitedParagraphChildNames.indexOf(childName) !== -1
        && _zwVaIsElementWithInlineContents(ancestor)) {
          return false;
        }
        if (/^h[1-6]$/.test(childName)
        && _zwJusIsHtmlElement(ancestor)
        && /^h[1-6]$/.test(_zwJusTag(ancestor))) {
          return false;
        }
        ancestor = ancestor.parentNode;
      }
      parent_ = _zwJusTag(parent_);
    }
    // Element/DocumentFragment parent 对象（非 HTML 元素）→ true；非字符串非
    // 元素对象（含 null）→ false。
    if (parent_ && typeof parent_ === 'object'
    && (parent_.nodeType === 1 || parent_.nodeType === 11)) {
      return true;
    }
    if (typeof parent_ !== 'string') return false;
    // per-parent switch 表（vendored :1689）。
    switch (parent_) {
      case 'colgroup':
        return childName === 'col';
      case 'table':
        return ['caption', 'col', 'colgroup', 'tbody', 'td', 'tfoot', 'th', 'thead', 'tr'].indexOf(childName) !== -1;
      case 'tbody':
      case 'thead':
      case 'tfoot':
        return ['td', 'th', 'tr'].indexOf(childName) !== -1;
      case 'tr':
        return ['td', 'th'].indexOf(childName) !== -1;
      case 'dl':
        return ['dt', 'dd'].indexOf(childName) !== -1;
      case 'dir':
      case 'ol':
      case 'ul':
        return ['dir', 'li', 'ol', 'ul'].indexOf(childName) !== -1;
      case 'hgroup':
        return /^h[1-6]$/.test(childName);
    }
    // 文档骨架名 → false。
    if (['body', 'caption', 'col', 'colgroup', 'frame', 'frameset', 'head',
    'html', 'tbody', 'td', 'tfoot', 'th', 'thead', 'tr'].indexOf(childName) !== -1) {
      return false;
    }
    // dd/dt 只入 dl；li 只入 ol/ul（F1 dd/dt 聚合缺失根因）。
    if ((childName === 'dd' || childName === 'dt') && parent_ !== 'dl') return false;
    if (childName === 'li' && parent_ !== 'ol' && parent_ !== 'ul') return false;
    // 同名 pair 拒绝表（vendored :1723）。
    var inlineContents = (typeof _zwVaInlineContents !== 'undefined' && _zwVaInlineContents) || [];
    var pairTable = [
      [['a'], ['a']],
      [['dd', 'dt'], ['dd', 'dt']],
      [['h1', 'h2', 'h3', 'h4', 'h5', 'h6'], ['h1', 'h2', 'h3', 'h4', 'h5', 'h6']],
      [['li'], ['li']],
      [['nobr'], ['nobr']],
      [inlineContents, _zwListProhibitedParagraphChildNames],
      [['td', 'th'], ['caption', 'col', 'colgroup', 'tbody', 'td', 'tfoot', 'th', 'thead', 'tr']],
    ];
    for (i = 0; i < pairTable.length; i++) {
      if (pairTable[i][0].indexOf(parent_) !== -1
      && pairTable[i][1].indexOf(childName) !== -1) {
        return false;
      }
    }
    return true;
  }

  // retag 选区保持包装（f3 knife A）：_zwJusSetTagName 换壳对 plain 子做序列化
  // 移动时端点容器对象身份灭失（实测 probe TRACE：foo@0 端点被改写到 bar@3）。
  // 改壳前录端点相对路径（child-index 链；壳内子序与子树结构换壳不变），改壳
  // 后按路径重解析重设 range。端点在壳外时身份不受影响，不触碰。
  function _zwListPathFrom(root, container, offset) {
    var idxs = [];
    var c = container;
    var g = 0;
    while (c && c !== root && c.parentNode && g++ < 256) {
      idxs.unshift(_zwJusNodeIndex(c));
      c = c.parentNode;
    }
    return { idxs: c === root ? idxs : null, offset: offset };
  }
  function _zwListResolvePath(root, path) {
    if (!path.idxs) return null;
    var c = root;
    for (var i = 0; i < path.idxs.length; i++) {
      c = c.childNodes[path.idxs[i]];
      if (!c) return null;
    }
    return { c: c, o: path.offset };
  }
  function _zwListContainsNode(root, n) {
    var g = 0;
    while (n && n !== root && g++ < 256) n = n.parentNode;
    return !!n;
  }
  // 节点仍在当前文档树中（爬到顶须为 document）——disable 出口身份回植的
  // 存活判定，detached 子树不回植。
  function _zwListNodeInDocument(n) {
    if (!n) return false;
    var top = n;
    var g = 0;
    while (top.parentNode && g++ < 256) top = top.parentNode;
    return top === document;
  }
  // 边界 offset 按容器类型钳界：文本按字符长度、元素按子数。
  function _zwListClampOffset(node, off) {
    var len = node.nodeType === 3 ? (node.nodeValue || '').length
      : (node.childNodes ? node.childNodes.length : 0);
    return off > len ? len : off;
  }
  // 端点是否落在列表内容内：容器自身为列表壳（(ul,2) 尾沿形）按外部处理，
  // 容器的某层父为 li（其父链命中 ol/ul）才算内容内。
  function _zwListInListContent(n) {
    if (!n) return false;
    var g = 0;
    var p = n;
    while (p && g++ < 256) {
      if (_zwJusIsHtmlElement(p.parentNode, ['ol', 'ul'])) return true;
      p = p.parentNode;
    }
    return false;
  }
  // 端点相对 item 的路径类别（前奏归并几何门，q15 修正语义）：从 n 沿 parent
  // 链爬升，仅在 item 可达时分类——先达 item → 'direct'（端点在 item 直属
  // 内容）；先命中 ol/ul 再达 item → 'list'（端点在 item 的子列表内，列表
  // 介于端点与 item 之间）；item 不在祖先链上（可能途中撞到无关列表，如
  // straddle 端点撞外层壳）→ 'outside'，不得误判 'list'。
  function _zwListEndpointPathKind(n, item) {
    var g = 0;
    var p = n;
    var inList = false;
    while (p && g++ < 256) {
      if (p === item) return inList ? 'list' : 'direct';
      if (_zwJusIsHtmlElement(p, ['ol', 'ul'])) inList = true;
      p = p.parentNode;
    }
    return 'outside';
  }
  function _zwListRetagPreservingSelection(node, newTag, extraRanges) {
    var sel = globalThis.getSelection ? globalThis.getSelection() : null;
    var rng = sel && sel._ranges && sel._ranges.length > 0 ? sel._ranges[0] : null;
    var pathS = null, pathE = null;
    if (rng && _zwListContainsNode(node, rng.startContainer)) {
      pathS = _zwListPathFrom(node, rng.startContainer, rng.startOffset);
    }
    if (rng && _zwListContainsNode(node, rng.endContainer)) {
      pathE = _zwListPathFrom(node, rng.endContainer, rng.endOffset);
    }
    // 附加 range 保持（f4）：toggle 流的 newRange（blockExtend 产物）start 端
    // 常为被换壳元素自身（li 臂产 [ol@0..]），换壳后容器 detached → 下游
    // getContainedNodes 收空 → 吸收臂永不触发（语料 iul:33/iol:43 单列表
    // 吸收形实测）。Chrome 原生 Range 随 DOM 变更自动吸附（壳 → 同位新壳），
    // 此处对附加 range 的壳内端点做同路径捕获重锚。
    var extras = [];
    if (extraRanges) {
      for (var ei = 0; ei < extraRanges.length; ei++) {
        var er = extraRanges[ei];
        if (!er) continue;
        var epS = _zwListContainsNode(node, er.startContainer)
          ? _zwListPathFrom(node, er.startContainer, er.startOffset) : null;
        var epE = _zwListContainsNode(node, er.endContainer)
          ? _zwListPathFrom(node, er.endContainer, er.endOffset) : null;
        extras.push({ r: er, s: epS, e: epE,
          sc: er.startContainer, so: er.startOffset,
          ec: er.endContainer, eo: er.endOffset });
      }
    }
    var newNode = _zwJusSetTagName(node, newTag);
    if (newNode && rng && (pathS || pathE)) {
      var ns = pathS ? _zwListResolvePath(newNode, pathS) : { c: rng.startContainer, o: rng.startOffset };
      var ne = pathE ? _zwListResolvePath(newNode, pathE) : { c: rng.endContainer, o: rng.endOffset };
      if (ns && ne) {
        try { rng.setStart(ns.c, ns.o); rng.setEnd(ne.c, ne.o); } catch (_eRp) {}
      }
    }
    for (var ei2 = 0; ei2 < extras.length; ei2++) {
      var xo = extras[ei2];
      if (!newNode) continue;
      if (xo.s) {
        var xs = _zwListResolvePath(newNode, xo.s);
        if (xs) {
          try {
            xo.r.setStart(xs.c, xs.o);
          } catch (_eRs) {}
        }
      } else if (_zwListNodeInDocument(xo.sc) &&
          (xo.r.startContainer !== xo.sc || xo.r.startOffset !== xo.so)) {
        // 壳外端点：setTagName 为先插后删的替换（净子数不变），引擎
        // pre-remove 活范围钩子按纯删除收缩 offset，端点被误前移；回写
        // 入口快照（按容器钳界）抵消该误收缩。
        try {
          xo.r.setStart(xo.sc, _zwListClampOffset(xo.sc, xo.so));
        } catch (_eRr) {}
      }
      if (xo.e) {
        var xe = _zwListResolvePath(newNode, xo.e);
        if (xe) {
          try {
            xo.r.setEnd(xe.c, xe.o);
          } catch (_eRe) {}
        }
      } else if (_zwListNodeInDocument(xo.ec) &&
          (xo.r.endContainer !== xo.ec || xo.r.endOffset !== xo.eo)) {
        try {
          xo.r.setEnd(xo.ec, _zwListClampOffset(xo.ec, xo.eo));
        } catch (_eRf) {}
      }
    }
    return newNode;
  }

  // fixDisallowedAncestors（vendored :3816 忠实移植）：双臂——「无同宿主允许
  // 祖先」臂（dd/dt → 无属性 dl wrap；禁排子 → 缺省单行容器 setTagName 递归 +
  // 新壳内禁排孙逐个录还拆出）与「允许于某祖先但非直父」拆父臂（录还包
  // while-splitParent）。
  function _zwListFixDisallowedAncestors(node) {
    if (!_zwJusIsEditable(node)) return;
    var anc = node.parentNode;
    var ok = false;
    var g = 0;
    while (anc && g++ < 256) {
      if (_zwListInSameEditingHost(node, anc) && _zwListIsAllowedChild(node, anc)) { ok = true; break; }
      anc = anc.parentNode;
    }
    if (!ok) {
      if (_zwJusIsHtmlElement(node, ['dd', 'dt'])) {
        _zwJusWrap([node], function (sib) {
          var hasAttr = false;
          try { hasAttr = sib.attributes && sib.attributes.length > 0; } catch (_eA) { hasAttr = true; }
          return _zwJusIsHtmlElement(sib, 'dl') && !hasAttr;
        }, function () { return document.createElement('dl'); });
        return;
      }
      var host = _zwListEditingHostOf(node);
      if (!host || !_zwListIsAllowedChild('p', host)) return;
      var sep = globalThis.__zwDefaultParagraphSeparator === 'p' ? 'p' : 'div';
      if (!_zwListIsProhibitedParagraphChild(node)) return;
      node = _zwListRetagPreservingSelection(node, sep);
      _zwListFixDisallowedAncestors(node);
      // 新壳内禁排孙录还拆出（vendored :3873-3887）。
      var kids = [];
      var kc = node.firstChild;
      while (kc) { kids.push(kc); kc = kc.nextSibling; }
      for (var ki = 0; ki < kids.length; ki++) {
        if (!_zwListIsProhibitedParagraphChild(kids[ki])) continue;
        var kv = _zwDelRecordValues([kids[ki]]);
        _zwListSplitParent([kids[ki]]);
        _zwDelRestoreValues(kv);
      }
      return;
    }
    // 允许子臂（vendored :3886-3896）：允许于同宿主某祖先但非直父 → 逐层拆父。
    var values = _zwDelRecordValues([node]);
    var pg = 0;
    while (node.parentNode && !_zwListIsAllowedChild(node, node.parentNode) && pg++ < 256) {
      _zwListSplitParent([node]);
    }
    _zwDelRestoreValues(values);
  }

  function _zwListTagOf(node) {
    return node && node.nodeType === 1 ? String(node.tagName || '').toLowerCase() : '';
  }

  function _zwListIsProhibitedParagraphChild(node) {
    var list = (typeof _zwVaProhibitedParagraphChildren !== 'undefined' && _zwVaProhibitedParagraphChildren) || null;
    if (!list) return false;
    return list.indexOf(_zwListTagOf(node)) !== -1;
  }

  // 真共同祖先：物化 Range.commonAncestorContainer 是 best-effort
  //（=startContainer，part06 _recalc）——items 祖先爬升须真实 CAC。
  function _zwListCommonAncestor(a, b) {
    var n = b;
    var g = 0;
    while (n && g++ < 256) {
      var m = a;
      var g2 = 0;
      while (m && g2++ < 256) {
        if (m === n) return n;
        m = m.parentNode;
      }
      n = n.parentNode;
    }
    return null;
  }

  // normalizeSublists（vendored :3904 忠实移植）：li 内 ol/ul 子归并外提——
  // 尾部 ol/ul 子提回父层紧随 item；中间 ws-text/inline 子聚入 item 后新插的
  // 兄弟 li。list 域 items 前奏与缩进合并新父臂两处调用。
  function _zwListNormalizeSublists(item) {
    if (!_zwJusIsHtmlElement(item, 'li')
    || !_zwJusIsEditable(item)
    || !_zwJusIsEditable(item.parentNode)) {
      return;
    }
    var newItem = null;
    var g = 0;
    while (g++ < 4096) {
      var hasSub = false;
      for (var si = 0; si < item.childNodes.length; si++) {
        if (_zwJusIsHtmlElement(item.childNodes[si], ['ol', 'ul'])) { hasSub = true; break; }
      }
      if (!hasSub) break;
      var child = item.lastChild;
      if (_zwJusIsHtmlElement(child, ['ol', 'ul'])
      || (!newItem && child.nodeType === 3 && /^[ \t\n\f\r]*$/.test(String(child.data)))) {
        newItem = null;
        _zwListMovePreservingRanges(child, item.parentNode, 1 + _zwJusNodeIndex(item));
      } else {
        if (!newItem) {
          newItem = document.createElement('li');
          item.parentNode.insertBefore(newItem, item.nextSibling);
        }
        _zwListMovePreservingRanges(child, newItem, 0);
      }
    }
  }

  // getSelectionListState（vendored :3949）：blockExtend 后 contained 节点
  // 列表判定 ol/ul/mixed。
  function _zwListSelectionState() {
    var range = _zwJusActiveRange();
    if (!range) return 'none';
    var newRange = _zwJusBlockExtend(range, false);
    var nodeList = _zwJusGetContainedNodes(newRange, function (node) {
      return _zwJusIsEditable(node)
        && !_zwListIsIndentationElement(node)
        && (_zwJusIsHtmlElement(node, ['ol', 'ul'])
        || _zwJusIsHtmlElement(node.parentNode, ['ol', 'ul'])
        || _zwListIsAllowedChild(node, 'li'));
    });
    if (!nodeList.length) return 'none';
    function hasDesc(node, tag) {
      try { return node.querySelector && node.querySelector(tag) ? true : false; } catch (_eQ) { return false; }
    }
    function everyOf(tag) {
      return nodeList.every(function (node) {
        var p = node.parentNode;
        return _zwJusIsHtmlElement(node, tag)
          || _zwJusIsHtmlElement(p, tag)
          || (_zwJusIsHtmlElement(p, 'li') && _zwJusIsHtmlElement(p.parentNode, tag));
      }) && !nodeList.some(function (node) {
        return _zwJusIsHtmlElement(node, tag === 'ol' ? 'ul' : 'ol') || hasDesc(node, tag === 'ol' ? 'ul' : 'ol');
      });
    }
    if (everyOf('ol')) return 'ol';
    if (everyOf('ul')) return 'ul';
    var hasOl = nodeList.some(function (node) {
      var p = node.parentNode;
      return _zwJusIsHtmlElement(node, 'ol') || _zwJusIsHtmlElement(p, 'ol') || hasDesc(node, 'ol')
        || (_zwJusIsHtmlElement(p, 'li') && _zwJusIsHtmlElement(p.parentNode, 'ol'));
    });
    var hasUl = nodeList.some(function (node) {
      var p = node.parentNode;
      return _zwJusIsHtmlElement(node, 'ul') || _zwJusIsHtmlElement(p, 'ul') || hasDesc(node, 'ul')
        || (_zwJusIsHtmlElement(p, 'li') && _zwJusIsHtmlElement(p.parentNode, 'ul'));
    });
    if (hasOl && hasUl) return 'mixed';
    if (hasOl) return 'mixed ol';
    if (hasUl) return 'mixed ul';
    return 'none';
  }

  // outdentNode ol/ul 臂（vendored :5022 切片）：disable 模式整列表撤壳主案
  //（toggle-off 语料面）。摘 reversed/start/type 后 preserve-descendants 撤壳 +
  // 子 fixDisallowedAncestors + 值录还。
  // FIXME：simple/indentation 元素臂、ancestorList 缩进臂未移植。
  function _zwListOutdentListNode(node) {
    if (!_zwJusIsEditable(node)) return;
    if (!_zwJusIsHtmlElement(node, ['ol', 'ul'])) return;
    try {
      node.removeAttribute('reversed');
      node.removeAttribute('start');
      node.removeAttribute('type');
    } catch (_eRa) {}
    var children = [];
    var ck = node.firstChild;
    while (ck) { children.push(ck); ck = ck.nextSibling; }
    var hasAttr = false;
    try { hasAttr = node.attributes && node.attributes.length > 0; } catch (_eHb) { hasAttr = true; }
    if (hasAttr && !_zwJusIsHtmlElement(node.parentNode, ['ol', 'ul'])) {
      _zwListRetagPreservingSelection(node, 'div');
    } else {
      // removePreservingDescendants + record/restore（vendored ol/ul 臂
      // 「otherwise」分支：值先录、撤壳、再还）。
      var outValues = _zwDelRecordValues(children);
      var pp = node.parentNode;
      var idx = _zwJusNodeIndex(node);
      for (var i = 0; i < children.length; i++) {
        _zwListMovePreservingRanges(children[i], pp, idx + i);
      }
      pp.removeChild(node);
      _zwDelRestoreValues(outValues);
    }
    for (var ci = 0; ci < children.length; ci++) {
      try { _zwListFixDisallowedAncestors(children[ci]); } catch (_eFd2) {}
    }
  }

  // toggleLists（vendored :5690）：enable 全流程 + disable outdent 臂（ol/ul
  // 切片）移植。选区保持依赖 _zwJusWrap/_zwListMovePreservingRanges（plain 重锚）。
  function _zwListToggle(tagName) {
    var range = _zwJusActiveRange();
    if (!range) return;
    var mode = _zwListSelectionState() === tagName ? 'disable' : 'enable';
    // tag 常量小写（_zwJusIsHtmlElement 经 _zwJusTag 小写比较，大写常量恒 miss
    // ——ol→ul 换型恒等实证）。
    var utag = tagName;
    var otherTagName = tagName === 'ol' ? 'ul' : 'ol';

    // 端点身份捕获（Chrome delta，语料权威）：动作前录 (textNode, offset)。
    // setTagName/splitParent 均按 movePreservingRanges 逐子迁移，文本节点身份
    // 存活；disable 出口按身份回植（语料 straddle 族期望恢复跨区选区——q10
    // 探针 i5-straddle sel=@2..@2 塌缩实证，跨区恢复后读面 [li, div] 混排给
    // qInd=true 与语料一致）；enable-retag 用交集钳制（见 toConvert 位点），
    // 不走此回植（全跨度回植会混入壳外文本，absorb 行 qInd 期望 false）。
    var capS = { n: range.startContainer, o: range.startOffset };
    var capE = { n: range.endContainer, o: range.endOffset };

    // items 前奏（vendored :5704-5727）：range 端点祖先容器 li 逐个归并子列表
    // ——end 侧爬到真 CAC 止、start 侧爬到根，unshift 成树序外先内后。
    // Chrome delta（语料权威）：归并仅 disable 面执行。enable 录制保留 li 内
    // 嵌列表换壳形（<ul><li>[foo<ul><li>bar]</ul><li>baz</ul> iol 期望嵌套+
    // 分裂），vendored 无条件 items.forEach 归并与 enable 录制不符；disable
    // 录制（li 内嵌列表外提合并形）才需要归并。语料权威面优先。
    var items = [];
    var cac = _zwListCommonAncestor(range.startContainer, range.endContainer);
    var ac = range.endContainer;
    var gI = 0;
    while (ac && ac !== cac && gI++ < 256) {
      if (_zwJusIsHtmlElement(ac, 'li')) items.unshift(ac);
      ac = ac.parentNode;
    }
    for (ac = range.startContainer; ac && gI++ < 512; ac = ac.parentNode) {
      if (_zwJusIsHtmlElement(ac, 'li')) items.unshift(ac);
    }
    // normalize 前奏模式×几何双门（语料权威，q9 判读面 + q14/q15 census）：
    // disable 无条件归并（top-level 合并形 `<div>foo</div><ol>…<ol>…` 依赖，
    // q10 判读）；enable 按选区几何逐 item 门控——起点在 item 直属内容且
    // 终点未离开该 item（直属内容或其子列表内）的形状跳过归并：跨入形
    // `[foo<ol><li>bar]`（q9 P2F 17 行族，:233/:448 expected 保嵌套换壳）
    // 与单 li 直选含嵌套尾随形 `[foo]<ol><li>bar</ol>…`（q15 census 八面
    // 中四面，:407/:418/:463/:473 expected 换壳 li 保嵌套子列表/尾随文本，
    // Chrome 录制不 lift）。其余形状归并（起点在子列表内：subset-of-N :178
    // 外提换壳形；终点离开 item：straddle-extra :458 外提后整体换壳分裂
    // 形——vendored lift+换型+splitParent+wrap 全流程模拟即 expected）。
    // 单门含两者：无条件归并重引 17 行 P2F 且回引 q15 四面、全关丢失外提行。
    if (mode === 'disable') {
      for (var ii = 0; ii < items.length; ii++) {
        _zwListNormalizeSublists(items[ii]);
      }
    } else {
      for (var ii = 0; ii < items.length; ii++) {
        if (_zwListEndpointPathKind(range.startContainer, items[ii]) === 'direct'
        && _zwListEndpointPathKind(range.endContainer, items[ii]) !== 'outside') {
          continue;
        }
        _zwListNormalizeSublists(items[ii]);
      }
    }

    var newRange = _zwJusBlockExtend(range, mode === 'enable' ? 'list-wide' : false);

    // enable：对 other tag 名可编辑元素换型或并入邻接同名列表。
    if (mode === 'enable') {
      var retaggedFinal = null;
      // ARM-B' 命中壳记录（:478 族 wholesale 档第二半）：壳内成员已随整壳
      // retag 处理完毕，后置 nodeList 平铺阶段必须排除壳子树——否则首分支
      // 直属文本以 allowedChild(li) 混入平铺重排，inline-wrap-li + 缩进合并
      // default-wrap 臂把 foo lift 进嵌套列表（split-lift 错形，DBG 探针
      // nodeList/gather/sublistArm 三点实证）。
      var shellArm = null;
      var toConvert = _zwJusCollectContainedNodes(newRange, function (node) {
        return _zwJusIsEditable(node) && _zwJusIsHtmlElement(node, otherTagName);
      }, false);
      // Chrome delta（语料权威，q16 直选跨嵌套形）：start 直属分支 li 内容
      // （path kind direct）、end 跨入同一分支的嵌套子列表（list）且最外层
      // 壳仅一 li 子时，部分含域收集不到的顶层壳前置整壳换型（:478 expected
      // 双层换壳保尾随文本；对照 :470 list→list 分裂保壳、:448 双 li 残留
      // UL 分出、:458 跨分支 straddle——li 子数与端点几何三分判据）。
      var wS = range.startContainer, wE = range.endContainer, stLi = null, enLi = null, gW = 0;
      while (wS && gW++ < 256) {
        if (_zwJusIsHtmlElement(wS, 'li')) { stLi = wS; break; }
        wS = wS.parentNode;
      }
      gW = 0;
      while (wE && gW++ < 256) {
        if (_zwJusIsHtmlElement(wE, 'li')) { enLi = wE; break; }
        wE = wE.parentNode;
      }
      if (stLi && enLi) {
        var shell = null, walker = stLi, gS2 = 0;
        while (walker && gS2++ < 256) {
          walker = walker.parentNode;
          if (!walker || !_zwJusIsHtmlElement(walker, ['ol', 'ul'])) break;
          shell = walker;
        }
        var branchS = stLi, branchE = enLi, gS3 = 0, gS4 = 0;
        while (branchS.parentNode && branchS.parentNode !== shell && gS3++ < 256) branchS = branchS.parentNode;
        while (branchE.parentNode && branchE.parentNode !== shell && gS4++ < 256) branchE = branchE.parentNode;
        if (shell && branchS === branchE
        && _zwListEndpointPathKind(range.startContainer, branchS) === 'direct'
        && _zwListEndpointPathKind(range.endContainer, branchE) === 'list') {
          var wLic = 0, wK = shell.firstChild;
          while (wK) {
            if (_zwJusIsHtmlElement(wK, 'li')) wLic++;
            wK = wK.nextSibling;
          }
          if (wLic === 1 && _zwJusIsEditable(shell)
          && _zwJusIsHtmlElement(shell, otherTagName)
          && toConvert.indexOf(shell) === -1) {
            toConvert.unshift(shell);
            shellArm = shell;
          }
        }
      }
      for (var ci = 0; ci < toConvert.length; ci++) {
        var list = toConvert[ci];
        // Chrome delta（语料权威，q15 census 四面）：嵌套在被换壳 li 内、且
        // 选区两端点均不在列表内的子列表不换型（:407/:418/:463/:473 expected
        // 换壳 li 保嵌套 UL 原型；对照 :448 选区跨入嵌套 `[foo<ol><li>bar]`
        // 换型、:458 外提后列表升 sibling 位 parent 非 li 换型——三分判据）。
        if (_zwJusIsHtmlElement(list.parentNode, 'li')
        && !_zwListContainsNode(list, range.startContainer)
        && !_zwListContainsNode(list, range.endContainer)) {
          continue;
        }
        var prevL = list.previousSibling, nextL = list.nextSibling;
        if ((_zwJusIsEditable(prevL) && _zwJusIsHtmlElement(prevL, utag))
        || (_zwJusIsEditable(nextL) && _zwJusIsHtmlElement(nextL, utag))) {
          retaggedFinal = _zwJusIsEditable(prevL) && _zwJusIsHtmlElement(prevL, utag) ? prevL : nextL;
          var children = [];
          var ck = list.firstChild;
          while (ck) { children.push(ck); ck = ck.nextSibling; }
          var kidsValues = _zwDelRecordValues(children);
          _zwListSplitParent(children);
          _zwJusWrap(children, function (node) { return _zwJusIsHtmlElement(node, utag); }, function () { return document.createElement(utag); });
          _zwDelRestoreValues(kidsValues);
        } else {
          var retagged = _zwListRetagPreservingSelection(list, utag.toLowerCase(), [newRange]);
          if (retagged) retaggedFinal = retagged;
          // shellArm 重锚（:478 族 wholesale 档收尾）：setTagName 为先插后删
          // 替换，换型后原壳节点 detached——nodeList 过滤须持新壳引用（批9 v1
          // 锚旧壳过滤恒空，probe q16-2 零位移实证）；批10 重锚后壳子树成员
          // （首分支直属文本）不再混入平铺重排，_zwJusWrap 邻接 utag 同壳
          // 收养臂无从触发，split-lift 消除。
          if (retagged && list === shellArm) {
            shellArm = retagged;
          }
          // Chrome delta（语料权威，absorb 行）：enable-retag 后活动选区与换
          // 壳子树求交——端点在壳内保留原位（身份随逐子迁移存活），壳外钳到
          // 壳内最深文本端。Chrome retag 后选区留在 li 内（qInd=false 确定
          // 读），全跨度回植会混入壳外文本给 mixed（探针 qIndA=true 实证）。
          if (retagged) {
            var rngC = _zwJusActiveRange();
            if (rngC) {
              var sIn = _zwListContainsNode(retagged, rngC.startContainer);
              var eIn = _zwListContainsNode(retagged, rngC.endContainer);
              if (!sIn || !eIn) {
                try {
                  if (!sIn) {
                    var fC = retagged;
                    while (fC.firstChild) fC = fC.firstChild;
                    rngC.setStart(fC.parentNode, _zwJusNodeIndex(fC));
                  }
                  if (!eIn) {
                    var lC = retagged;
                    while (lC.lastChild) lC = lC.lastChild;
                    rngC.setEnd(lC.parentNode, 1 + _zwJusNodeIndex(lC));
                  }
                } catch (_eCl) {}
              }
            }
          }
        }
      }
    }

    // node list：blockExtend 后 contained 可编辑（ol/ul 或其子或 li 允许子）。
    var nodeList = _zwJusGetContainedNodes(newRange, function (node) {
      return _zwJusIsEditable(node)
        && !_zwListIsIndentationElement(node)
        && (_zwJusIsHtmlElement(node, ['ol', 'ul'])
        || _zwJusIsHtmlElement(node.parentNode, ['ol', 'ul'])
        || _zwListIsAllowedChild(node, 'li'));
    });
    if (mode === 'enable') {
      nodeList = nodeList.filter(function (node) {
        if (shellArm && (node === shellArm || _zwListContainsNode(shellArm, node))) {
          return false;
        }
        return !_zwJusIsHtmlElement(node, ['ol', 'ul'])
          || _zwJusIsHtmlElement(node.parentNode, ['ol', 'ul']);
      });
    }

    if (mode === 'disable') {
      while (nodeList.length) {
        var dsub = [nodeList.shift()];
        if (!dsub[0].parentNode) continue;
        if (_zwJusIsHtmlElement(dsub[0], utag)) {
          try { _zwListOutdentListNode(dsub[0]); } catch (_eOd) {}
          continue;
        }
        while (nodeList.length
        && nodeList[0] === dsub[dsub.length - 1].nextSibling
        && !_zwJusIsHtmlElement(nodeList[0], utag)) {
          dsub.push(nodeList.shift());
        }
        var dValues = _zwDelRecordValues(dsub);
        _zwListSplitParent(dsub);
        for (var di = 0; di < dsub.length; di++) {
          try { _zwListFixDisallowedAncestors(dsub[di]); } catch (_eFd) {}
        }
        _zwDelRestoreValues(dValues);
      }
      // disable 出口端点身份回植：双端点文本节点仍在文档中时按捕获 offset
      // 原位重设（offset 越界钳到容器长度）；任一端点身份灭失则保持现行为
      // （marker/path 恢复结果），不用半新半旧混搭。
      if (_zwListNodeInDocument(capS.n) && _zwListNodeInDocument(capE.n)) {
        try {
          var soR = _zwListClampOffset(capS.n, capS.o);
          var eoR = _zwListClampOffset(capE.n, capE.o);
          range.setStart(capS.n, soR);
          range.setEnd(capE.n, eoR);
        } catch (_eRs) {}
      }
      return;
    }

    // enable 主循环：连续兄弟聚 sublist → p/div 换型 li / li·ol·ul 直收 /
    // inline 段 wrap 成 li。
    while (nodeList.length) {
      var sublist = [];
      while (nodeList.length
      && (!sublist.length
      || nodeList[0] === sublist[sublist.length - 1].nextSibling)) {
        if (_zwJusIsHtmlElement(nodeList[0], ['p', 'div'])) {
          sublist.push(_zwListRetagPreservingSelection(nodeList[0], 'li'));
          nodeList.shift();
        } else if (_zwJusIsHtmlElement(nodeList[0], ['li', 'ol', 'ul'])) {
          sublist.push(nodeList.shift());
        } else {
          var nodesToWrap = [];
          while (!nodesToWrap.length
          || (nodeList.length
          && nodeList[0] === nodesToWrap[nodesToWrap.length - 1].nextSibling
          && _zwJusIsInlineNode(nodeList[0])
          && _zwJusIsInlineNode(nodesToWrap[nodesToWrap.length - 1])
          && !_zwJusIsHtmlElement(nodesToWrap[nodesToWrap.length - 1], 'br'))) {
            nodesToWrap.push(nodeList.shift());
          }
          var liEl = _zwJusWrap(nodesToWrap, function () { return false; }, function () { return document.createElement('li'); });
          if (liEl) sublist.push(liEl);
        }
      }

      if (_zwJusIsHtmlElement(sublist[0].parentNode, utag)
      || sublist.every(function (node) { return _zwJusIsHtmlElement(node, ['ol', 'ul']); })) {
        continue;
      }

      if (_zwJusIsHtmlElement(sublist[0].parentNode, otherTagName)) {
        var xoValues = _zwDelRecordValues(sublist);
        _zwListSplitParent(sublist);
        _zwJusWrap(sublist, function (node) { return _zwJusIsHtmlElement(node, utag); }, function () { return document.createElement(utag); });
        _zwDelRestoreValues(xoValues);
        continue;
      }

      // 缩进合并新父臂（vendored :5922-5960）：sublist[0].parentNode 是可编辑
      // simple 缩进元素且其 previousSibling 是可编辑 utag 元素 → newParent =
      // 归并后该 utag 的 lastChild（非可编辑 utag 则新插一个），wrap 的
      // isBefore 判定落 append 臂；否则恒新壳。
      _zwJusWrap(sublist, function (node) { return _zwJusIsHtmlElement(node, utag); }, function () {
        if (!_zwJusIsEditable(sublist[0].parentNode)
        || !_zwListIsSimpleIndentationElement(sublist[0].parentNode)
        || !_zwJusIsEditable(sublist[0].parentNode.previousSibling)
        || !_zwJusIsHtmlElement(sublist[0].parentNode.previousSibling, utag)) {
          return document.createElement(utag);
        }
        var list = sublist[0].parentNode.previousSibling;
        _zwListNormalizeSublists(list.lastChild);
        if (!_zwJusIsEditable(list.lastChild)
        || !_zwJusIsHtmlElement(list.lastChild, utag)) {
          list.appendChild(document.createElement(utag));
        }
        return list.lastChild;
      });
    }

    // enable 出口列表内收钳（q12 判读 + 探针里程碑证据）：吸收流经 wrap 的
    // flat-model HTML 重写臂（editing host 直子文本成员），换壳列表与端点文
    // 本身份整体灭失重建（探针 barAlive=false、rF=UL/- detached），入口身份
    // 回植与 retaggedFinal 锚定均不可行；端点被引擎重锚到列表壳边界（UL@2
    // 实证），选中区脱离 li 内容使 qState-after 翻红。按树位收钳：端点容器
    // 自身为列表壳时（重锚落壳形）直接钳入该壳最深首/尾端（吸收后树位 =
    // 语料 after 树 marker 全 li 内跨度）；容器在列表壳外时回退换壳列表
    // （flat 臂未触发、列表存活的常形，与 mid-toggle 交集钳制同型收口）；
    // 端点已在列表内容内则不动（多列表/换型行保位）。
    {
      var rngX = _zwJusActiveRange();
      if (rngX) {
        try {
          var tS = _zwListInListContent(rngX.startContainer) ? null
            : (_zwJusIsHtmlElement(rngX.startContainer, ['ol', 'ul'])
            && rngX.startContainer.parentNode ? rngX.startContainer
            : (retaggedFinal && _zwListNodeInDocument(retaggedFinal) ? retaggedFinal : null));
          if (tS) {
            var fF = tS;
            while (fF.firstChild) fF = fF.firstChild;
            rngX.setStart(fF.parentNode, _zwJusNodeIndex(fF));
          }
          var tE = _zwListInListContent(rngX.endContainer) ? null
            : (_zwJusIsHtmlElement(rngX.endContainer, ['ol', 'ul'])
            && rngX.endContainer.parentNode ? rngX.endContainer
            : (retaggedFinal && _zwListNodeInDocument(retaggedFinal) ? retaggedFinal : null));
          if (tE) {
            var lF = tE;
            while (lF.lastChild) lF = lF.lastChild;
            rngX.setEnd(lF.parentNode, 1 + _zwJusNodeIndex(lF));
          }
        } catch (_eXc) {}
      }
    }
  }

  globalThis.__zw_ce_list_toggle = function (tagName) {
    try { _zwListToggle(tagName === 'ol' ? 'ol' : 'ul'); } catch (_eLt) {}
  };

  // qState 布尔读面（Chrome 录制契约直读，双面拟合 P10：before 286/286 全
  // 拟合、after 278/286 恒改善）：两端点最近列表祖先存在且 tag 恰为本命令
  // 型，且两端点最近 li 无异型嵌套子列表。同型嵌套不翻 false（dps 行
  // <ol><li>[foo<ol> 录 true 实证）；异型嵌套翻 false（<ol><li>[foo<ul> 录
  // false 实证）。跨列表+同型嵌套面语料内部矛盾（:343 before 录 true vs
  // :478 after 录 false，同树同选区），无状态读面不可双全——按 before 全保
  // 取舍，跨壳族留待选区恢复耦合刀。mixed 判定仅归 indeterm。
  // 规范：https://w3c.github.io/editing/execCommand.html#querycommandstate()
  function _zwListQueryStateBool(cmd) {
    var want = cmd === 'insertorderedlist' ? 'ol' : 'ul';
    var opp = want === 'ol' ? 'ul' : 'ol';
    var range = null;
    try { range = _zwJusActiveRange(); } catch (_eAr) { return false; }
    if (!range) return false;
    function endpointOk(container) {
      var node = container;
      var list = null;
      var li = null;
      while (node) {
        if (!list && _zwJusIsHtmlElement(node, ['ol', 'ul'])) list = node;
        if (!li && _zwJusIsHtmlElement(node, 'li')) li = node;
        node = node.parentNode;
      }
      if (!list || !_zwJusIsHtmlElement(list, want)) return false;
      if (li) {
        try {
          if (li.querySelector(opp)) return false;
        } catch (_eQ) {}
      }
      return true;
    }
    try {
      return endpointOk(range.startContainer) && endpointOk(range.endContainer);
    } catch (_eEo) { return false; }
  }

  // qState/qIndeterm 读面（vendored commands.insertorderedlist/unorderedlist
  // state/indeterm）：state = _zwListQueryStateBool（端点直读，见上）；
  // indeterm = /^mixed( ol)?$/（ol）/ /^mixed( ul)?$/（ul）。
  globalThis.__zw_ce_list_state = function (cmd) {
    var isOl = cmd === 'insertorderedlist';
    var st = null;
    try { st = _zwListSelectionState(); } catch (_eLs) { st = null; }
    return {
      state: _zwListQueryStateBool(cmd),
      indeterm: st === 'mixed' || st === (isOl ? 'mixed ol' : 'mixed ul')
    };
  };
