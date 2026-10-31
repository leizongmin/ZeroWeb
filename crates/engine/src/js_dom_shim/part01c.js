  // ============================================================
  // editing-compat justify 簇（2026-10-31）：execCommand justify* 实应用引擎。
  //
  // 此前 _zwExecCmdInputType 含 justifycenter/full/left/right 四命令（beforeinput/
  // input 事件照派）但零变更臂，run 语料 justify 族 12,108F 为最大单命令块。
  // 本文件按 editing spec 参考实现（tests/wpt-runner/wpt-data/editing/include/
  // implementation.js 的 justifySelection + 依赖 helper 子集）移植：
  //   ① blockExtend（块延伸——含 li 臂与 offset==0 尾爬升，祖先容器由此入含域，
  //     center/div align 多行行是此步拉开与 naive 实现的差距）；
  //   ② 对齐清理 pass（element list：去 align attr / 去 style text-align /
  //     无属性 div/span/center 摘壳 / center 带 attr 改名 div）；
  //   ③ 包裹 pass（node list：editable + allowed child of div + alignment 值≠
  //     目标 → 连续兄弟 run wrap 进新 div style="text-align:…"）。
  // 对齐值经 getComputedStyle().textAlign 真级联读数（center/td/th/caption 命中
  // UA 或表现提示），div/p align attr 提示本仓级联未映射（style-system 表现提示
  // 无 div 臂——渲染流工作面不越界），在 _zwJusGetAlignmentValue 内 JS 侧补读
  // （computed 非 center/justify/left/right 时沿祖先补 align attr——级联继承等价形）。
  // 选区保持：move/wrap 全程经 _zwJusMovePreservingRanges 边界点补偿回写
  // sel._ranges[0]（对象身份不变），命令后 queryCommandState/Value 断言读同一
  // 选区（spec wrap 语义「preserving ranges」）。
  // plain 节点摘挂纪律（R54xx 同域）：跨父移动一律先 oldParent.removeChild 再
  // 插入——shim 桥 appendChild 对 plain 子做序列化拷贝不摘原父。
  // 挂载位缘由：part01b/part02 边界在 crypto.subtle 对象字面量内部（字节精确
  // 切分残留），无顶层语句边界可插，故经 js_dom_bridge.rs 挂 part06 尾（外层
  // IIFE 内干净语句边界）；选区单例经 globalThis.getSelection 取用（_getSelection
  // 声明域不覆盖本文件挂载点）。part06 经 globalThis.__zw_ce_justify_* 单点调用。
  // https://w3c.github.io/editing/execCommand.html#justifycommand
  // https://w3c.github.io/editing/execCommand.html#block-extending-a-range
  // ============================================================

  // —— 基础件：树序/索引/长度（参考 implementation.js 216-326，以 parentNode 链
  // 实现边界点比较，不经 compareDocumentPosition——plain 语料节点少一层桥面）——

  function _zwJusTag(node) {
    return node && node.nodeType === 1 && node.tagName
      ? String(node.tagName).toLowerCase() : '';
  }

  // 参考实现 isHtmlElement 的 tag 匹配面（namespace 检查省略——shim 无 namespace
  // 标记，语料面全 HTML。FIXME：svg/math 命名面接通时补 namespaceURI 判定）。
  function _zwJusIsHtmlElement(node, tags) {
    if (!node || node.nodeType !== 1) return false;
    var t = _zwJusTag(node);
    if (typeof tags === 'undefined') return true;
    if (typeof tags === 'string') tags = [tags];
    for (var i = 0; i < tags.length; i++) if (tags[i] === t) return true;
    return false;
  }

  function _zwJusNodeIndex(node) {
    var ret = 0;
    var n = node;
    while (n && n.previousSibling) { ret++; n = n.previousSibling; }
    return ret;
  }

  // node length（spec：Text/Comment → data.length；其余 → childNodes.length）。
  function _zwJusNodeLength(node) {
    if (!node) return 0;
    if (node.nodeType === 3 || node.nodeType === 8 || node.__zwIsText) {
      return node.length != null ? node.length : String(node.nodeValue || '').length;
    }
    return node.childNodes ? node.childNodes.length : 0;
  }

  function _zwJusHasKids(node) {
    return !!(node && node.childNodes && node.childNodes.length);
  }

  function _zwJusIsAncestorOf(anc, desc) {
    var n = desc;
    while (n) {
      if (n.parentNode === anc) return true;
      n = n.parentNode;
    }
    return false;
  }

  // DOM Range 边界点 (node, offset) 全序比较：-1 A<B / 0 相等 / 1 A>B。
  // spec dom-range boundary point position 语义（参考 getPosition 216-301 的
  // parentNode 链重写）：同节点比 offset；祖先链经共同祖先的子索引定序，
  // 端点落在祖先上时按 (祖先, offset) 与路径子的 index 裁决。
  function _zwJusBpCompare(nA, oA, nB, oB) {
    if (nA === nB) return oA < oB ? -1 : (oA > oB ? 1 : 0);
    if (_zwJusIsAncestorOf(nA, nB)) {
      var c1 = nB;
      while (c1.parentNode !== nA) c1 = c1.parentNode;
      var i1 = _zwJusNodeIndex(c1);
      return oA <= i1 ? -1 : 1;
    }
    if (_zwJusIsAncestorOf(nB, nA)) {
      var c2 = nA;
      while (c2.parentNode !== nB) c2 = c2.parentNode;
      var i2 = _zwJusNodeIndex(c2);
      // oB == i2：端界恰在路径子起点，A（该子内部）严格在其后。
      return oB <= i2 ? 1 : -1;
    }
    // 共同祖先（非互祖先）：两侧路径子 index 定序。失根（跨树）→ A<B 兜底
    //（语料面单文档不现；FIXME：跨 document 选区语义接通时补 DISCONNECTED 面）。
    var pA = nA, pB = nB;
    while (pA.parentNode && !_zwJusIsAncestorOf(pA.parentNode, nB)) pA = pA.parentNode;
    while (pB.parentNode && pB.parentNode !== pA.parentNode) pB = pB.parentNode;
    if (!pA.parentNode || !pB.parentNode || pA.parentNode !== pB.parentNode) return -1;
    var ia = _zwJusNodeIndex(pA), ib = _zwJusNodeIndex(pB);
    return ia < ib ? -1 : (ia > ib ? 1 : 0);
  }

  function _zwJusIsBefore(node1, node2) {
    return _zwJusBpCompare(node1, 0, node2, 0) < 0;
  }

  function _zwJusPreviousNode(node) {
    if (node.previousSibling) {
      node = node.previousSibling;
      while (_zwJusHasKids(node)) node = node.lastChild;
      return node;
    }
    return node.parentNode && node.parentNode.nodeType === 1 ? node.parentNode : null;
  }

  function _zwJusNextNodeDescendants(node) {
    while (node && !node.nextSibling) node = node.parentNode;
    return node ? node.nextSibling : null;
  }

  function _zwJusNextNode(node) {
    return _zwJusHasKids(node) ? node.firstChild : _zwJusNextNodeDescendants(node);
  }

  // —— CSS 读数（getComputedStyle 真级联；异常回落空串）——
  // display 读数经 WeakMap 记忆化（OPTIMIZATION：isVisible/isBlockNode 链每行数百次
  // 触发 getComputedStyle 全级联桥面调用，run 语料千行块 600s+ 撞墙钟——display 非
  // 继承属性且节点移动不变，缓存安全；text-align/white-space 受继承与移动影响不缓存）。
  var _zwJusDisplayCache = new WeakMap();

  function _zwJusCs(node, prop) {
    if (!node || node.nodeType !== 1 || !globalThis.getComputedStyle) return '';
    if (prop === 'display') {
      if (_zwJusDisplayCache.has(node)) return _zwJusDisplayCache.get(node);
      var dv = '';
      try {
        var csD = globalThis.getComputedStyle(node);
        dv = csD ? String(csD.display || '') : '';
      } catch (_eJcsD) {}
      _zwJusDisplayCache.set(node, dv);
      return dv;
    }
    try {
      var cs = globalThis.getComputedStyle(node);
      return cs ? String(cs[prop] || '') : '';
    } catch (_eJcs) { return ''; }
  }

  // spec block node：display 解析值 ∉ {inline, inline-block, inline-table, none}
  // 的 Element，或 Document/DocumentFragment。
  // https://w3c.github.io/editing/execCommand.html#block-node
  function _zwJusIsBlockNode(node) {
    if (!node) return false;
    if (node.nodeType === 9 || node.nodeType === 11) return true;
    if (node.nodeType !== 1) return false;
    return ['inline', 'inline-block', 'inline-table', 'none']
      .indexOf(_zwJusCs(node, 'display')) === -1;
  }

  function _zwJusIsInlineNode(node) {
    return !!node && !_zwJusIsBlockNode(node);
  }

  // 父链 white-space 读数：computed white-space 缺（级联面未含）时按 style attr
  // + pre 族 UA 近似（pre/xmp/listing/plaintext → pre，其余 normal）。
  // FIXME：style-system white-space 计算值全量接通后改为直读。
  function _zwJusWhiteSpaceOf(el) {
    var v = _zwJusCs(el, 'whiteSpace');
    if (v) return v;
    var own = el.getAttribute && el.getAttribute('style');
    if (own) {
      var m = /(?:^|;)\s*white-space\s*:\s*([^;\s]+)/i.exec(String(own));
      if (m) return m[1].toLowerCase();
    }
    var t = _zwJusTag(el);
    return (t === 'pre' || t === 'xmp' || t === 'listing' || t === 'plaintext')
      ? 'pre' : 'normal';
  }

  function _zwJusIsWhitespaceNode(node) {
    if (!node || node.nodeType !== 3) return false;
    var data = String(node.nodeValue || '');
    if (data === '') return true;
    if (!/^[\t\n\r ]+$/.test(data) && !/^[\t\r ]+$/.test(data)) return false;
    var p = node.parentNode;
    if (!p || p.nodeType !== 1) return false;
    var ws = _zwJusWhiteSpaceOf(p);
    return (/^[\t\n\r ]+$/.test(data) && (ws === 'normal' || ws === 'nowrap'))
      || (/^[\t\r ]+$/.test(data) && ws === 'pre-line');
  }

  function _zwJusHasDisplayNoneAncestor(node) {
    var n = node;
    while (n) {
      if (n.nodeType === 1 && _zwJusCs(n, 'display') === 'none') return true;
      n = n.parentNode;
    }
    return false;
  }

  // collapsed whitespace node（参考实现 912-1001 结构化重写）：whitespace 节点且
  // 前向无可见文字/图片（前一路径遇块或 br 即折行 → 折叠）。
  function _zwJusIsCollapsedWhitespaceNode(node) {
    if (!_zwJusIsWhitespaceNode(node)) return false;
    if (String(node.nodeValue || '') === '') return true;
    if (_zwJusHasDisplayNoneAncestor(node)) return true;
    var ancestor = node.parentNode;
    while (ancestor && !_zwJusIsBlockNode(ancestor) && ancestor.parentNode) {
      ancestor = ancestor.parentNode;
    }
    var reference = node;
    var guard = 0;
    while (reference && reference !== ancestor && guard++ < 4096) {
      reference = _zwJusPreviousNode(reference);
      if (!reference) return true;
      if (_zwJusIsBlockNode(reference) || _zwJusIsHtmlElement(reference, 'br')) return true;
      if ((reference.nodeType === 3 && !_zwJusIsWhitespaceNode(reference))
          || _zwJusIsHtmlElement(reference, 'img')) break;
    }
    return false;
  }

  // —— br 行断语义：offsetHeight 渲染探针不可用（shim 无布局读数），结构近似——
  // collapsed line break ≈ 折行后行内无他内容（follows line break 且后向块内无
  // 可见兄弟内容）；extraneous line break ≈ 折行前无他内容（precedes line break）
  // 且非 li 唯一子。FIXME：布局读数接通后换 spec 探针版。
  // https://w3c.github.io/editing/execCommand.html#collapsed-line-break

  function _zwJusFollowsLineBreak(node) {
    var offset = 0;
    var n = node;
    var guard = 0;
    while (n && !_zwJusIsBlockBoundaryPoint(n, offset) && guard++ < 256) {
      if (offset - 1 >= 0 && n.childNodes && offset - 1 < n.childNodes.length
          && _zwJusIsVisible(n.childNodes[offset - 1])) return false;
      if (offset === 0 || !n.childNodes || !n.childNodes.length) {
        offset = _zwJusNodeIndex(n);
        n = n.parentNode;
      } else {
        n = n.childNodes[offset - 1];
        offset = _zwJusNodeLength(n);
      }
    }
    return !!n;
  }

  function _zwJusPrecedesLineBreak(node) {
    var offset = node ? _zwJusNodeLength(node) : 0;
    var n = node;
    var guard = 0;
    while (n && !_zwJusIsBlockBoundaryPoint(n, offset) && guard++ < 256) {
      if (n.childNodes && offset < n.childNodes.length
          && _zwJusIsVisible(n.childNodes[offset])) return false;
      if (offset === _zwJusNodeLength(n) || !n.childNodes || !n.childNodes.length) {
        offset = _zwJusNodeIndex(n) + 1;
        n = n.parentNode;
      } else {
        n = n.childNodes[offset];
        offset = 0;
      }
    }
    return !!n;
  }

  function _zwJusIsCollapsedLineBreak(br) {
    if (!_zwJusIsHtmlElement(br, 'br')) return false;
    if (!_zwJusFollowsLineBreak(br)) return false;
    var n = br.nextSibling;
    var guard = 0;
    while (n && guard++ < 256) {
      if (_zwJusIsVisible(n)) return false;
      n = n.nextSibling;
    }
    return true;
  }

  function _zwJusIsExtraneousLineBreak(br) {
    if (!_zwJusIsHtmlElement(br, 'br')) return false;
    if (_zwJusIsHtmlElement(br.parentNode, 'li')
        && br.parentNode.childNodes && br.parentNode.childNodes.length === 1) return false;
    return _zwJusPrecedesLineBreak(br);
  }

  // spec visible：block node / 非折叠空白文本 / img / 非冗余 br / 有可见后代；
  // display:none 祖先链除外。
  // https://w3c.github.io/editing/execCommand.html#visible
  function _zwJusIsVisible(node) {
    if (!node) return false;
    if (_zwJusHasDisplayNoneAncestor(node)) return false;
    if (_zwJusIsBlockNode(node)
        || (node.nodeType === 3 && !_zwJusIsCollapsedWhitespaceNode(node))
        || _zwJusIsHtmlElement(node, 'img')
        || (_zwJusIsHtmlElement(node, 'br') && !_zwJusIsExtraneousLineBreak(node))) {
      return true;
    }
    if (_zwJusHasKids(node)) {
      for (var i = 0; i < node.childNodes.length; i++) {
        if (_zwJusIsVisible(node.childNodes[i])) return true;
      }
    }
    return false;
  }

  function _zwJusIsInvisible(node) {
    return !!node && !_zwJusIsVisible(node);
  }

  // —— editing host / editable（参考实现 715-757；contenteditable 反射经 attr
  // 读数：''/true/plaintext-only = true 态，false = false 态，缺 = 继承）——
  // https://w3c.github.io/editing/execCommand.html#editable

  function _zwJusCeState(el) {
    if (!el || el.nodeType !== 1 || !el.getAttribute) return null;
    var a = el.getAttribute('contenteditable');
    if (a === null || a === undefined) return null;
    var v = String(a).toLowerCase();
    if (v === '' || v === 'true' || v === 'plaintext-only') return 'true';
    if (v === 'false') return 'false';
    return null;
  }

  function _zwJusIsEditingHost(node) {
    return !!node && node.nodeType === 1 && _zwJusCeState(node) === 'true';
  }

  function _zwJusIsEditable(node, _depth) {
    if (!node) return false;
    var depth = _depth || 0;
    if (depth > 128) return false;
    if (_zwJusIsEditingHost(node)) return false;
    if (node.nodeType === 1 && _zwJusCeState(node) === 'false') return false;
    var p = node.parentNode;
    if (!p) return false;
    if (!_zwJusIsEditingHost(p) && !_zwJusIsEditable(p, depth + 1)) return false;
    return node.nodeType === 1
      ? _zwJusIsHtmlElement(node)
      : _zwJusIsHtmlElement(p);
  }

  // —— containment（参考实现 254-411）——

  function _zwJusGetRoot(node) {
    var n = node;
    while (n && n.parentNode) n = n.parentNode;
    return n;
  }

  function _zwJusIsContained(node, range) {
    if (!node || !range) return false;
    if (_zwJusGetRoot(node) !== _zwJusGetRoot(range.startContainer)) return false;
    return _zwJusBpCompare(node, 0, range.startContainer, range.startOffset) > 0
      && _zwJusBpCompare(node, _zwJusNodeLength(node), range.endContainer, range.endOffset) < 0;
  }

  // condition 命中即收，且收后跳过其后代（getContainedNodes 语义）；条件忽略时
  // 全收（getAllContainedNodes 用途当前无调用方，保留对照参考以备 indent 簇）。
  function _zwJusCollectContainedNodes(range, condition, stopAtMatch) {
    var cond = condition || function () { return true; };
    var node = range.startContainer;
    if (_zwJusHasKids(node) && range.startOffset < node.childNodes.length) {
      node = node.childNodes[range.startOffset];
    } else if (range.startOffset === _zwJusNodeLength(node)) {
      node = _zwJusNextNodeDescendants(node);
    } else {
      node = _zwJusNextNode(node);
    }
    var stop = range.endContainer;
    if (_zwJusHasKids(stop) && range.endOffset < stop.childNodes.length) {
      stop = stop.childNodes[range.endOffset];
    } else {
      stop = _zwJusNextNodeDescendants(stop);
    }
    var nodeList = [];
    var guard = 0;
    while (node && stop && _zwJusIsBefore(node, stop) && guard++ < 65536) {
      if (_zwJusIsContained(node, range) && cond(node)) {
        nodeList.push(node);
        if (stopAtMatch) { node = _zwJusNextNodeDescendants(node); continue; }
      }
      node = _zwJusNextNode(node);
    }
    return nodeList;
  }

  function _zwJusGetContainedNodes(range, condition) {
    return _zwJusCollectContainedNodes(range, condition, true);
  }

  // —— block-extend（参考实现 4193-4284 忠实移植 + Chrome 行为差收口）——
  // FIXME（Chrome delta）：spec 尾爬升无条件爬到最近块边界祖先，在 corpus 页
  // 形状（宿主为 body 第 2 子）对 foo[]bar 产出 BODY@2..DIV@1 空含域（vendored
  // 参考实现同页直调实证），而录制期望（真实 Chrome）把当前行/当前块收入含域。
  // 故分两档：
  // - 默认（清理 pass）：尾爬升仅当对侧端界已逃出当前容器子树时继续（祖先容器
  //   center/带 align div 须入含域以供清理；caret 行不得误收祖先），再夹回宿主。
  // - chromeTight（wrap/query pass）：不做尾爬升，端界落在闭合容器（td/th/li）
  //   或宿主的祖先上时下移到该容器内容边界——wrap 面只含当前行/当前 cell 内容。
  // https://w3c.github.io/editing/execCommand.html#block-extending-a-range

  function _zwJusIsBlockStartPoint(node, offset) {
    return (!node.parentNode && offset === 0)
      || (offset - 1 >= 0 && node.childNodes && offset - 1 < node.childNodes.length
          && _zwJusIsVisible(node.childNodes[offset - 1])
          && (_zwJusIsBlockNode(node.childNodes[offset - 1])
              || _zwJusIsHtmlElement(node.childNodes[offset - 1], 'br')));
  }

  function _zwJusIsBlockEndPoint(node, offset) {
    return (!node.parentNode && offset === _zwJusNodeLength(node))
      || (node.childNodes && offset < node.childNodes.length
          && _zwJusIsVisible(node.childNodes[offset])
          && _zwJusIsBlockNode(node.childNodes[offset]));
  }

  function _zwJusIsBlockBoundaryPoint(node, offset) {
    return _zwJusIsBlockStartPoint(node, offset) || _zwJusIsBlockEndPoint(node, offset);
  }

  // 闭合容器（Chrome 块扩展的行基点）：td/th/li——cell 与列表项终止块爬升
  // （Blink enclosingBlock 语义），限宿主子树内。
  function _zwJusClosestClosure(node, host) {
    var n = node;
    while (n) {
      if (n === host) return null;
      if (_zwJusIsHtmlElement(n, ['td', 'th', 'li'])) return n;
      n = n.parentNode;
    }
    return null;
  }

  // Chrome 收口：端界落在闭合容器/宿主的严格祖先上时下移到其内容边界。
  function _zwJusClampToClosure(node, offset, host, closure, end) {
    if (closure && node !== closure && _zwJusIsAncestorOf(node, closure)) {
      return [closure, end ? _zwJusNodeLength(closure) : 0];
    }
    if (host && _zwJusIsAncestorOf(node, host)) {
      return [host, end ? _zwJusNodeLength(host) : 0];
    }
    return [node, offset];
  }

  function _zwJusBlockExtend(range, chromeTight) {
    var startNode = range.startContainer;
    var startOffset = range.startOffset;
    var endNode = range.endContainer;
    var endOffset = range.endOffset;

    // Chrome 收口基点：编辑宿主 + 两侧闭合容器。
    var host = null;
    var hn = startNode;
    while (hn) {
      if (_zwJusIsEditingHost(hn)) { host = hn; break; }
      hn = hn.parentNode;
    }
    var startClosure = _zwJusClosestClosure(startNode, host);
    var endClosure = _zwJusClosestClosure(endNode, host);

    // li 臂：start/end 所在最近 li 整行收编。
    var li = null;
    var n = startNode;
    while (n) {
      if (_zwJusIsHtmlElement(n, 'li')) { li = n; break; }
      n = n.parentNode;
    }
    if (li) { startOffset = _zwJusNodeIndex(li); startNode = li.parentNode; }
    if (!_zwJusIsBlockStartPoint(startNode, startOffset)) {
      var g1 = 0;
      do {
        if (startOffset === 0) {
          startOffset = _zwJusNodeIndex(startNode);
          startNode = startNode.parentNode;
        } else {
          startOffset--;
        }
        if (_zwJusIsBlockBoundaryPoint(startNode, startOffset)) break;
      } while (startNode && g1++ < 256);
    }

    li = null;
    n = endNode;
    while (n) {
      if (_zwJusIsHtmlElement(n, 'li')) { li = n; break; }
      n = n.parentNode;
    }
    if (li) { endOffset = 1 + _zwJusNodeIndex(li); endNode = li.parentNode; }
    if (!_zwJusIsBlockEndPoint(endNode, endOffset)) {
      var g2 = 0;
      do {
        if (endOffset === _zwJusNodeLength(endNode)) {
          endOffset = 1 + _zwJusNodeIndex(endNode);
          endNode = endNode.parentNode;
        } else {
          endOffset++;
        }
        if (_zwJusIsBlockBoundaryPoint(endNode, endOffset)) break;
      } while (endNode && g2++ < 256);
    }

    if (chromeTight) {
      var ctS = _zwJusClampToClosure(startNode, startOffset, host, startClosure, false);
      startNode = ctS[0]; startOffset = ctS[1];
      var ctE = _zwJusClampToClosure(endNode, endOffset, host, endClosure, true);
      endNode = ctE[0]; endOffset = ctE[1];
    } else {
      // 条件尾爬升：对侧端界仍在当前容器子树内（或同容器）则停，否则继续上爬。
      while (startOffset === 0 && startNode && startNode.parentNode
          && startNode !== endNode && !_zwJusIsAncestorOf(startNode, endNode)) {
        startOffset = _zwJusNodeIndex(startNode);
        startNode = startNode.parentNode;
      }
      while (endOffset === _zwJusNodeLength(endNode) && endNode && endNode.parentNode
          && endNode !== startNode && !_zwJusIsAncestorOf(endNode, startNode)) {
        endOffset = 1 + _zwJusNodeIndex(endNode);
        endNode = endNode.parentNode;
      }
      var ccS = _zwJusClampToClosure(startNode, startOffset, host, null, false);
      startNode = ccS[0]; startOffset = ccS[1];
      var ccE = _zwJusClampToClosure(endNode, endOffset, host, null, true);
      endNode = ccE[0]; endOffset = ccE[1];
    }

    var newRange = document.createRange();
    newRange.setStart(startNode, startOffset);
    newRange.setEnd(endNode, endOffset);
    return newRange;
  }

  // —— 对齐值（参考实现 4037-4077 + div/p align attr 提示 JS 侧补读）——
  // spec alignment value：沿父链跳过 inline/none，首个块级元素读 computed
  // text-align（-moz/-webkit- 前缀剥除，auto→start）；start/end 按方向性映射。
  // 补读缘由：本仓 style-system 表现提示无 div/p align 臂（渲染流工作面），
  // computed 缺 {center,justify,left,right} 字面值时沿祖先链补 style attr 显式
  // text-align → 非 table 元素 align attr（HTML4 §15.1.2 提示语义，table align
  // = margin 不入 text-align——与 chromium UA 同判）→ center 元素。
  // https://w3c.github.io/editing/execCommand.html#alignment-value
  function _zwJusGetAlignmentValue(node) {
    while (node && (node.nodeType !== 1
        || ['inline', 'none'].indexOf(_zwJusCs(node, 'display')) !== -1)) {
      node = node.parentNode;
    }
    if (!node || node.nodeType !== 1) return 'left';

    var resolved = _zwJusCs(node, 'textAlign')
      .replace(/^-(moz|webkit)-/, '')
      .replace(/^auto$/, 'start');

    if (resolved === 'start' || resolved === 'end') {
      // Chrome computed 继承形：start/end = 未命中显式对齐的初始值；祖先链有
      // 显式提示（style text-align / center 标签 / align 属性）时按提示——wrap
      // 后 td 内文本继承外层包 div 的 center（语料 {<table>} 选区行录制口径）。
      var hintedStart = _zwJusAlignmentHint(node);
      if (hintedStart) return hintedStart;
      var rtl = false;
      var el = node;
      while (el && el.nodeType === 1) {
        var dir = el.getAttribute ? el.getAttribute('dir') : null;
        if (dir != null) { rtl = String(dir).toLowerCase() === 'rtl'; break; }
        el = el.parentNode;
      }
      if (resolved === 'start') return rtl ? 'right' : 'left';
      return rtl ? 'left' : 'right';
    }
    if (['center', 'justify', 'left', 'right'].indexOf(resolved) !== -1) {
      return resolved;
    }

    var hinted = _zwJusAlignmentHint(node);
    if (hinted) return hinted;
    return 'left';
  }

  // 表现提示补读（沿祖先链——级联继承等价形；computed 已命中时不可达此处）。
  // align 属性只认 HTML § rendering 的对齐映射元素集——quasit 等未知元素的
  // align 不产生 CSS text-align（语料 <quasit align=right><p> 录制 left）。
  var _zwJusAlignMapped = { div: 1, p: 1, h1: 1, h2: 1, h3: 1, h4: 1, h5: 1,
    h6: 1, caption: 1, center: 1, legend: 1, tbody: 1, thead: 1, tfoot: 1,
    tr: 1, td: 1, th: 1, col: 1, colgroup: 1 };
  function _zwJusAlignmentHint(node) {
    var anc = node;
    var guard = 0;
    while (anc && anc.nodeType === 1 && guard++ < 128) {
      var own = anc.getAttribute && anc.getAttribute('style');
      if (own) {
        var m = /(?:^|;)\s*text-align\s*:\s*([^;\s]+)/i.exec(String(own));
        if (m) {
          var tv = m[1].toLowerCase();
          if (['center', 'justify', 'left', 'right'].indexOf(tv) !== -1) return tv;
        }
      }
      var t = _zwJusTag(anc);
      if (t === 'center') return 'center';
      if (t !== 'table' && anc.getAttribute && _zwJusAlignMapped[t]) {
        var al = anc.getAttribute('align');
        if (al != null) {
          var av = String(al).trim().toLowerCase();
          if (av === 'left' || av === 'right' || av === 'justify') return av;
          if (av === 'center' || av === 'middle') return 'center';
        }
      }
      anc = anc.parentNode;
    }
    return null;
  }

  // —— 选区保持移动（参考实现 movePreservingRanges 1181-1264）——
  // 边界点补偿只作用于活动选区（sel._ranges[0]——shim 选区单 range 面；spec 多
  // range 选区本仓未开）。plain 摘挂：先 oldParent.removeChild 再插（R54xx 纪律，
  // shim 桥 appendChild 对 plain 子做序列化拷贝不摘原父）。
  function _zwJusMovePreservingRanges(node, newParent, newIndex) {
    if (!node || !newParent) return;
    var idx = newIndex === -1 ? (newParent.childNodes ? newParent.childNodes.length : 0) : newIndex;
    var oldParent = node.parentNode;
    var oldIndex = _zwJusNodeIndex(node);

    var rng = null;
    try {
      var sel = globalThis.getSelection ? globalThis.getSelection() : null;
      rng = sel && sel._ranges && sel._ranges.length > 0 ? sel._ranges[0] : null;
    } catch (_eJsel) {}
    var sc = rng ? rng.startContainer : null;
    var so = rng ? rng.startOffset : 0;
    var ec = rng ? rng.endContainer : null;
    var eo = rng ? rng.endOffset : 0;

    // 端点在 node 内（同或后代）→ 随动不补偿。
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

    if (oldParent && typeof oldParent.removeChild === 'function') {
      try { oldParent.removeChild(node); } catch (_eJrm) {}
    }
    var ref = newParent.childNodes && idx < newParent.childNodes.length
      ? newParent.childNodes[idx] : null;
    try {
      if (ref) newParent.insertBefore(node, ref);
      else newParent.appendChild(node);
    } catch (_eJins) {
      try { newParent.appendChild(node); } catch (_eJins2) {}
    }

    if (rng) {
      try {
        rng.setStart(sc, so);
        rng.setEnd(ec, eo);
      } catch (_eJrng) {}
    }
  }

  // spec remove preserving descendants（参考 5191-5197 + splitParent 子搬运）：
  // 子逐个 movePreservingRanges 到父的 node 原位，随后摘壳。
  function _zwJusRemovePreservingDescendants(node) {
    var parent = node.parentNode;
    if (!parent) return;
    var kids = node.childNodes ? [].slice.call(node.childNodes) : [];
    if (!kids.length) {
      try { parent.removeChild(node); } catch (_eJrm0) {}
      return;
    }
    var idx = _zwJusNodeIndex(node);
    for (var i = 0; i < kids.length; i++) {
      _zwJusMovePreservingRanges(kids[i], parent, idx + i);
    }
    try { parent.removeChild(node); } catch (_eJrm1) {}
  }

  // spec set tag name（参考 1266-1302）：center→div 保属性保子。
  // https://w3c.github.io/editing/execCommand.html#set-the-tag-name
  function _zwJusSetTagName(el, newName) {
    if (_zwJusTag(el) === newName) return el;
    if (!el.parentNode) return el;
    var repl = document.createElement(newName);
    var attrs = el.attributes || [];
    var i;
    for (i = 0; i < attrs.length; i++) {
      try { repl.setAttribute(attrs[i].name, attrs[i].value); } catch (_eJat) {}
    }
    var kids = el.childNodes ? [].slice.call(el.childNodes) : [];
    for (i = 0; i < kids.length; i++) {
      _zwJusMovePreservingRanges(kids[i], repl, -1);
    }
    el.parentNode.insertBefore(repl, el);
    _zwJusMovePreservingRangesAttrSafe(repl, el);
    return repl;
  }

  // el → repl 的选区端点迁移（setTagName 专用：端点落在被换壳元素内时迁入 repl
  // 同位——spec splitParent 链覆盖该面，此处按语料面做同位迁移近似。
  // FIXME：端点在 center 元素自身上的选区（run 语料不现）精确语义接通时补）。
  function _zwJusMovePreservingRangesAttrSafe(fromEl, deadEl) {
    try { fromEl.parentNode.removeChild(deadEl); } catch (_eJrm2) {}
    var sel = globalThis.getSelection ? globalThis.getSelection() : null;
    var rng = sel && sel._ranges && sel._ranges.length > 0 ? sel._ranges[0] : null;
    if (!rng) return;
    try {
      if (rng.startContainer === deadEl) rng.setStart(fromEl, rng.startOffset);
      if (rng.endContainer === deadEl) rng.setEnd(fromEl, rng.endOffset);
    } catch (_eJrng2) {}
  }

  // —— 冗余行断清理（参考 1304-1376，isVisible/isExtraneous 近似件同上标注）——

  function _zwJusRemoveExtraneousLineBreaksBefore(node) {
    var ref = node.previousSibling;
    if (!ref) return;
    while (_zwJusHasKids(ref)) ref = ref.lastChild;
    var guard = 0;
    while (_zwJusIsInvisible(ref) && !_zwJusIsExtraneousLineBreak(ref)
           && ref !== node.parentNode && guard++ < 256) {
      ref = _zwJusPreviousNode(ref);
    }
    if (_zwJusIsEditable(ref) && _zwJusIsExtraneousLineBreak(ref)) {
      try { ref.parentNode.removeChild(ref); } catch (_eJrb) {}
    }
  }

  function _zwJusRemoveExtraneousLineBreaksAtTheEndOf(node) {
    var ref = node;
    while (_zwJusHasKids(ref)) ref = ref.lastChild;
    var guard = 0;
    while (_zwJusIsInvisible(ref) && !_zwJusIsExtraneousLineBreak(ref)
           && ref !== node && guard++ < 256) {
      ref = _zwJusPreviousNode(ref);
    }
    if (_zwJusIsEditable(ref) && _zwJusIsExtraneousLineBreak(ref)) {
      while (_zwJusIsEditable(ref.parentNode) && _zwJusIsInvisible(ref.parentNode)) {
        ref = ref.parentNode;
      }
      try { ref.parentNode.removeChild(ref); } catch (_eJre) {}
    }
  }

  // —— wrap（参考 1377-1568 忠实移植：尾 br 收编 / 隐形兄弟收编 / 复用既对齐
  // div / br 插入臂 / 空原父摘除 / 后邻合并臂）——
  // https://w3c.github.io/editing/execCommand.html#wrapping-a-list-of-nodes

  // flat 模型边界（editing host 直子文本成员专用）：宿主直子文本 model-backed，
  // 裸 DOM 搬运不回流 innerHTML 读数（getter 走 pending 模型；zz-jcprobe w3/w5
  // 实证 insert+move 后读数仍旧），故整段经 host.innerHTML setter 重写（同
  // _zwExecCmdApplyFormat splice 口径——setter 即建 pending 节点）。复用兄弟包
  // （align/style div）时并入其内容 + br 分隔（录制形 `foo<br>[bar]`）；选区
  // 端点锚在成员文本内 → 映射到包内副本同累积偏移。
  function _zwJusEscapeHtmlText(s) {
    return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  }

  function _zwJusSerializeOpenTag(el) {
    var s = '<' + _zwJusTag(el);
    var attrs = el.attributes || [];
    for (var i = 0; i < attrs.length; i++) {
      s += ' ' + attrs[i].name + '="'
        + String(attrs[i].value).replace(/&/g, '&amp;').replace(/"/g, '&quot;') + '"';
    }
    return s + '>';
  }

  function _zwJusKidHtml(node) {
    if (node.nodeType === 3) return _zwJusEscapeHtmlText(String(node.nodeValue || ''));
    if (node.nodeType === 8) return '<!--' + String(node.nodeValue || '') + '-->';
    return String(node.outerHTML || '');
  }

  function _zwJusTextLenOf(root) {
    var n = 0;
    (function walk(x) {
      if (x.nodeType === 3) { n += String(x.nodeValue || '').length; return; }
      for (var i = 0; i < x.childNodes.length; i++) walk(x.childNodes[i]);
    })(root);
    return n;
  }

  // 包内累积文本偏移 → (文本节点, 节点内偏移)（DFS 文本后代序；超尾 → 末文本尾）。
  function _zwJusLocateTextOffset(root, want) {
    var lastText = null;
    var acc = 0;
    (function walk(x) {
      if (acc > want || lastText) return;
      if (x.nodeType === 3) {
        var len = String(x.nodeValue || '').length;
        if (acc + len >= want) { lastText = [x, want - acc]; acc += len; return; }
        acc += len;
        lastText = null;
        return;
      }
      for (var i = 0; i < x.childNodes.length; i++) walk(x.childNodes[i]);
    })(root);
    return lastText;
  }

  function _zwJusWrapViaHtml(host, members, siblingCriteria, newParentInstructions) {
    var k;
    var kids = [];
    for (k = 0; k < host.childNodes.length; k++) kids.push(host.childNodes[k]);
    var firstIdx = -1, lastIdx = -1;
    for (k = 0; k < kids.length; k++) {
      if (members.indexOf(kids[k]) !== -1) {
        if (firstIdx < 0) firstIdx = k;
        lastIdx = k;
      }
    }
    if (firstIdx < 0) return null;
    var memberHtml = '';
    for (k = firstIdx; k <= lastIdx; k++) memberHtml += _zwJusKidHtml(kids[k]);
    var prefix = '', suffix = '';
    for (k = 0; k < firstIdx; k++) prefix += _zwJusKidHtml(kids[k]);
    for (k = lastIdx + 1; k < kids.length; k++) suffix += _zwJusKidHtml(kids[k]);

    // 选区端点在成员文本内 → 记包内容累积偏移（重建后映射回副本）。
    var rng = null;
    try {
      var selW = globalThis.getSelection ? globalThis.getSelection() : null;
      rng = selW && selW._ranges && selW._ranges.length > 0 ? selW._ranges[0] : null;
    } catch (_eJws) {}
    var sc = rng ? rng.startContainer : null, so = rng ? rng.startOffset : 0;
    var ec = rng ? rng.endContainer : null, eo = rng ? rng.endOffset : 0;
    var cum = 0, startRel = -1, endRel = -1;
    for (k = firstIdx; k <= lastIdx; k++) {
      var mk = kids[k];
      if (mk === sc) startRel = cum + so;
      if (mk === ec) endRel = cum + eo;
      if (mk.nodeType === 3) cum += String(mk.nodeValue || '').length;
    }

    var anyBr = false;
    for (k = firstIdx; k <= lastIdx; k++) {
      if (_zwJusIsHtmlElement(kids[k], 'br')) { anyBr = true; break; }
    }
    var html, wrapper = null, reusePrev = false;
    var prevSib = firstIdx > 0 ? kids[firstIdx - 1] : null;
    var nextSib = lastIdx + 1 < kids.length ? kids[lastIdx + 1] : null;
    if (prevSib && prevSib.nodeType === 1 && _zwJusIsEditable(prevSib)
        && siblingCriteria(prevSib)) {
      var pHtml = String(prevSib.outerHTML || '');
      var pClose = pHtml.lastIndexOf('</' + _zwJusTag(prevSib) + '>');
      if (pClose < 0) return null;
      wrapper = prevSib; reusePrev = true;
      html = prefix + pHtml.slice(0, pClose) + (anyBr ? '' : '<br>') + memberHtml
        + pHtml.slice(pClose) + suffix;
    } else if (nextSib && nextSib.nodeType === 1 && _zwJusIsEditable(nextSib)
        && siblingCriteria(nextSib)) {
      var nHtml = String(nextSib.outerHTML || '');
      var nOpen = nHtml.indexOf('>') + 1;
      if (nOpen <= 0) return null;
      wrapper = nextSib;
      html = prefix + nHtml.slice(0, nOpen) + memberHtml + (anyBr ? '' : '<br>')
        + nHtml.slice(nOpen) + suffix;
    } else {
      var np = newParentInstructions();
      if (!np) return null;
      wrapper = np;
      html = prefix + _zwJusSerializeOpenTag(np) + memberHtml
        + '</' + _zwJusTag(np) + '>' + suffix;
    }
    host.innerHTML = html;

    // 选区映射：重建后按 open tag 串匹配定位新包（fresh np 已脱挂）；复用包
    // 直接用原元素。端点偏移 → 包内文本副本同位。
    try {
      if (rng && wrapper) {
        var wEl = wrapper;
        if (wEl.parentNode !== host) {
          var wantOpen = _zwJusSerializeOpenTag(wrapper);
          for (k = 0; k < host.childNodes.length; k++) {
            var cN = host.childNodes[k];
            if (cN.nodeType === 1 && _zwJusSerializeOpenTag(cN) === wantOpen) {
              wEl = cN; break;
            }
          }
        }
        if (wEl && wEl.parentNode === host) {
          if (reusePrev && startRel >= 0) startRel += _zwJusTextLenOf(wrapper);
          if (reusePrev && endRel >= 0) endRel += _zwJusTextLenOf(wrapper);
          var locS = startRel >= 0 ? _zwJusLocateTextOffset(wEl, startRel) : null;
          var locE = endRel >= 0 ? _zwJusLocateTextOffset(wEl, endRel) : null;
          var nRange = document.createRange();
          if (locS && locE) {
            nRange.setStart(locS[0], locS[1]);
            nRange.setEnd(locE[0], locE[1]);
          } else if (locS) {
            nRange.setStart(locS[0], locS[1]);
            nRange.collapse(true);
          } else {
            nRange.setStart(host, host.childNodes.length);
            nRange.collapse(true);
          }
          rng.setStart(nRange.startContainer, nRange.startOffset);
          rng.setEnd(nRange.endContainer, nRange.endOffset);
        }
      }
    } catch (_eJwm) {}
    return wrapper;
  }

  function _zwJusWrap(nodeList, siblingCriteria, newParentInstructions) {
    var i;
    var anyBr = false;
    for (i = 0; i < nodeList.length; i++) {
      if (_zwJusIsHtmlElement(nodeList[i], 'br')) { anyBr = true; break; }
    }
    if (!anyBr) {
      var allInvisible = true;
      for (i = 0; i < nodeList.length; i++) {
        if (_zwJusIsVisible(nodeList[i])) { allInvisible = false; break; }
      }
      if (allInvisible) return null;
    }
    if (!nodeList[0].parentNode) return null;
    if (_zwJusIsInlineNode(nodeList[nodeList.length - 1])
        && !_zwJusIsHtmlElement(nodeList[nodeList.length - 1], 'br')
        && _zwJusIsHtmlElement(nodeList[nodeList.length - 1].nextSibling, 'br')) {
      nodeList.push(nodeList[nodeList.length - 1].nextSibling);
    }
    while (_zwJusIsInvisible(nodeList[0].previousSibling)) {
      nodeList.unshift(nodeList[0].previousSibling);
    }
    while (_zwJusIsInvisible(nodeList[nodeList.length - 1].nextSibling)) {
      nodeList.push(nodeList[nodeList.length - 1].nextSibling);
    }

    // FIXME（Chrome delta 残余族，justifycenter 语料 ~57 形状行 wrap DOM 未达录制）：
    // 跨 cell 选区行（data-start=1 data-end=2 只 wrap 首 cell 的不对称）、相邻同
    // 对齐 div 三节点合并行（`<div center>foo</div>[bar]<div center>baz</div>` →
    // 单一 center div）、tbody/tr-attr selection 行、整表选择行部分变体。
    // flat 模型边界臂：editing host 直子文本成员 → innerHTML setter 整段重写。
    for (i = 0; i < nodeList.length; i++) {
      if (nodeList[i].nodeType === 3 && nodeList[i].parentNode
          && _zwJusIsEditingHost(nodeList[i].parentNode)) {
        return _zwJusWrapViaHtml(nodeList[i].parentNode, nodeList,
          siblingCriteria, newParentInstructions);
      }
    }

    var newParent;
    if (_zwJusIsEditable(nodeList[0].previousSibling)
        && siblingCriteria(nodeList[0].previousSibling)) {
      newParent = nodeList[0].previousSibling;
    } else if (_zwJusIsEditable(nodeList[nodeList.length - 1].nextSibling)
        && siblingCriteria(nodeList[nodeList.length - 1].nextSibling)) {
      newParent = nodeList[nodeList.length - 1].nextSibling;
    } else {
      newParent = newParentInstructions();
    }
    if (!newParent) return null;

    if (!newParent.parentNode) {
      nodeList[0].parentNode.insertBefore(newParent, nodeList[0]);
      // 边界点补偿：端点 = (新父的父, 新父插入位) → offset+1。
      try {
        var sel0 = globalThis.getSelection ? globalThis.getSelection() : null;
        var rng0 = sel0 && sel0._ranges && sel0._ranges.length > 0 ? sel0._ranges[0] : null;
        if (rng0) {
          var ni = _zwJusNodeIndex(newParent);
          if (rng0.startContainer === newParent.parentNode
              && rng0.startOffset === ni) {
            rng0.setStart(rng0.startContainer, rng0.startOffset + 1);
          }
          if (rng0.endContainer === newParent.parentNode
              && rng0.endOffset === ni) {
            rng0.setEnd(rng0.endContainer, rng0.endOffset + 1);
          }
        }
      } catch (_eJwp) {}
    }

    var originalParent = nodeList[0].parentNode;

    if (_zwJusIsBefore(newParent, nodeList[0])) {
      if (!_zwJusIsInlineNode(newParent)
          && _zwJusIsInlineNode(_zwJusLastVisibleChild(newParent))
          && _zwJusIsInlineNode(_zwJusFirstVisibleMember(nodeList))
          && !_zwJusIsHtmlElement(newParent.lastChild, 'br')) {
        try { newParent.appendChild(document.createElement('br')); } catch (_eJbr1) {}
      }
      for (i = 0; i < nodeList.length; i++) {
        _zwJusMovePreservingRanges(nodeList[i], newParent, -1);
      }
    } else {
      if (!_zwJusIsInlineNode(newParent)
          && _zwJusIsInlineNode(_zwJusFirstVisibleChild(newParent))
          && _zwJusIsInlineNode(_zwJusLastVisibleMember(nodeList))
          && !_zwJusIsHtmlElement(nodeList[nodeList.length - 1], 'br')) {
        try { newParent.insertBefore(document.createElement('br'), newParent.firstChild); } catch (_eJbr2) {}
      }
      for (i = nodeList.length - 1; i >= 0; i--) {
        _zwJusMovePreservingRanges(nodeList[i], newParent, 0);
      }
    }

    if (_zwJusIsEditable(originalParent) && !_zwJusHasKids(originalParent)) {
      try { originalParent.parentNode.removeChild(originalParent); } catch (_eJop) {}
    }

    if (_zwJusIsEditable(newParent.nextSibling)
        && siblingCriteria(newParent.nextSibling)) {
      if (!_zwJusIsInlineNode(newParent)
          && _zwJusIsInlineNode(newParent.lastChild)
          && _zwJusIsInlineNode(newParent.nextSibling.firstChild)
          && !_zwJusIsHtmlElement(newParent.lastChild, 'br')) {
        try { newParent.appendChild(document.createElement('br')); } catch (_eJbr3) {}
      }
      var guard = 0;
      while (_zwJusHasKids(newParent.nextSibling) && guard++ < 4096) {
        _zwJusMovePreservingRanges(newParent.nextSibling.firstChild, newParent, -1);
      }
      try { newParent.parentNode.removeChild(newParent.nextSibling); } catch (_eJmg) {}
    }
    _zwJusRemoveExtraneousLineBreaksBefore(newParent);
    _zwJusRemoveExtraneousLineBreaksAtTheEndOf(newParent);
    return newParent;
  }

  function _zwJusLastVisibleChild(el) {
    var kids = el.childNodes || [];
    for (var i = kids.length - 1; i >= 0; i--) {
      if (_zwJusIsVisible(kids[i])) return kids[i];
    }
    return undefined;
  }

  function _zwJusFirstVisibleChild(el) {
    var kids = el.childNodes || [];
    for (var i = 0; i < kids.length; i++) {
      if (_zwJusIsVisible(kids[i])) return kids[i];
    }
    return undefined;
  }

  function _zwJusFirstVisibleMember(list) {
    for (var i = 0; i < list.length; i++) {
      if (_zwJusIsVisible(list[i])) return list[i];
    }
    return undefined;
  }

  function _zwJusLastVisibleMember(list) {
    for (var i = list.length - 1; i >= 0; i--) {
      if (_zwJusIsVisible(list[i])) return list[i];
    }
    return undefined;
  }

  // —— allowed child（参考 1572-1712；本模块只以字符串 "div" 为 parent 调用，
  // 通用面保留）——
  // https://w3c.github.io/editing/execCommand.html#allowed-children
  var _zwJusProhibitedParagraphNames = ['address', 'article', 'aside',
    'blockquote', 'caption', 'center', 'col', 'colgroup', 'dd', 'details',
    'dir', 'div', 'dl', 'dt', 'fieldset', 'figcaption', 'figure', 'footer',
    'form', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'header', 'hgroup', 'hr', 'li',
    'listing', 'menu', 'nav', 'ol', 'p', 'plaintext', 'pre', 'section',
    'summary', 'table', 'tbody', 'td', 'tfoot', 'th', 'thead', 'tr', 'ul',
    'xmp'];
  var _zwJusInlineContentsNames = ['a', 'abbr', 'b', 'bdi', 'bdo', 'cite',
    'code', 'dfn', 'em', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'i', 'kbd',
    'mark', 'p', 'pre', 'q', 'rp', 'rt', 'ruby', 's', 'samp', 'small', 'span',
    'strong', 'sub', 'sup', 'u', 'var', 'acronym', 'listing', 'strike', 'xmp',
    'big', 'blink', 'font', 'marquee', 'nobr', 'tt'];

  function _zwJusIsAllowedChild(child, parentName) {
    var parent_ = parentName;
    if ((['colgroup', 'table', 'tbody', 'tfoot', 'thead', 'tr']
         .indexOf(parent_) !== -1)
        && child && child.nodeType === 3
        && !/^[ \t\n\f\r]*$/.test(String(child.nodeValue || ''))) {
      return false;
    }
    if ((['script', 'style', 'plaintext', 'xmp'].indexOf(parent_) !== -1)
        && (!child || child.nodeType !== 3)) {
      return false;
    }
    if (child && (child.nodeType === 9 || child.nodeType === 11
        || child.nodeType === 10)) {
      return false;
    }
    if (_zwJusIsHtmlElement(child)) child = _zwJusTag(child);
    if (typeof child !== 'string') return true;

    switch (parent_) {
      case 'colgroup': return child === 'col';
      case 'table': return ['caption', 'col', 'colgroup', 'tbody', 'td',
        'tfoot', 'th', 'thead', 'tr'].indexOf(child) !== -1;
      case 'tbody': case 'thead': case 'tfoot':
        return ['td', 'th', 'tr'].indexOf(child) !== -1;
      case 'tr': return ['td', 'th'].indexOf(child) !== -1;
      case 'dl': return ['dt', 'dd'].indexOf(child) !== -1;
      case 'dir': case 'ol': case 'ul':
        return ['dir', 'li', 'ol', 'ul'].indexOf(child) !== -1;
      case 'hgroup': return /^h[1-6]$/.test(child);
    }

    if (['body', 'caption', 'col', 'colgroup', 'frame', 'frameset', 'head',
         'html', 'tbody', 'td', 'tfoot', 'th', 'thead', 'tr']
        .indexOf(child) !== -1) {
      return false;
    }
    if ((child === 'dd' || child === 'dt') && parent_ !== 'dl') return false;
    if (child === 'li' && parent_ !== 'ol' && parent_ !== 'ul') return false;

    var noPairs = [
      [['a'], ['a']],
      [['dd', 'dt'], ['dd', 'dt']],
      [['h1', 'h2', 'h3', 'h4', 'h5', 'h6'], ['h1', 'h2', 'h3', 'h4', 'h5', 'h6']],
      [['li'], ['li']],
      [['nobr'], ['nobr']],
      [_zwJusInlineContentsNames, _zwJusProhibitedParagraphNames],
      [['td', 'th'], ['caption', 'col', 'colgroup', 'tbody', 'td', 'tfoot',
        'th', 'thead', 'tr']]
    ];
    for (var i = 0; i < noPairs.length; i++) {
      if (noPairs[i][0].indexOf(parent_) !== -1
          && noPairs[i][1].indexOf(child) !== -1) {
        return false;
      }
    }
    return true;
  }

  // —— 主入口：justifySelection（参考 5972-6081 忠实移植）——
  // 活动选区 = shim 选区首 range；无选区/端点缺 → 不应用（execCommand 臂事件序
  // 已派，返回 false 仅记变更未做）。
  // https://w3c.github.io/editing/execCommand.html#justify-the-selection
  function _zwJusActiveRange() {
    var sel = globalThis.getSelection ? globalThis.getSelection() : null;
    var rng = sel && sel._ranges && sel._ranges.length > 0 ? sel._ranges[0] : null;
    return rng && rng.startContainer && rng.endContainer ? rng : null;
  }

  globalThis.__zw_ce_justify_apply = function (alignment) {
    var range = _zwJusActiveRange();
    if (!range) return false;

    // ① 块延伸 + 对齐清理 pass（element list：align attr / style text-align /
    // center 元素；Contained 全收含嵌套——参考 getAllContainedNodes）。
    var newRange = _zwJusBlockExtend(range);
    var elementList = _zwJusCollectContainedNodes(newRange, function (node) {
      if (node.nodeType !== 1 || !_zwJusIsEditable(node)) return false;
      var styleAttr = node.getAttribute && node.getAttribute('style');
      var hasTa = styleAttr
        && /(?:^|;)\s*text-align\s*:/i.test(String(styleAttr));
      return (!!node.getAttribute && node.getAttribute('align') !== null
          && node.getAttribute('align') !== undefined)
        || !!hasTa
        || _zwJusIsHtmlElement(node, 'center');
    }, false);
    var i;
    for (i = 0; i < elementList.length; i++) {
      var element = elementList[i];
      try { element.removeAttribute('align'); } catch (_eJal) {}
      // style attr text-align unset（参考 element.style.textAlign = ""——attr 串
      // 直改等价形：声明剔除后空 → 摘 style attr）。
      var st = element.getAttribute && element.getAttribute('style');
      if (st && /(?:^|;)\s*text-align\s*:/i.test(String(st))) {
        var decls = String(st).split(';');
        var kept = [];
        for (var d = 0; d < decls.length; d++) {
          if (/(?:^|\s*)text-align\s*:/i.test(decls[d])) continue;
          if (decls[d].replace(/\s+/g, '') !== '') kept.push(decls[d]);
        }
        var out = kept.join(';').replace(/^;+|;+$/g, '');
        if (out === '') {
          try { element.removeAttribute('style'); } catch (_eJst1) {}
        } else {
          try { element.setAttribute('style', out); } catch (_eJst2) {}
        }
      }
      if (_zwJusIsHtmlElement(element, ['div', 'span', 'center'])
          && element.attributes && !element.attributes.length) {
        _zwJusRemovePreservingDescendants(element);
      }
      if (_zwJusIsHtmlElement(element, 'center')
          && element.attributes && element.attributes.length) {
        _zwJusSetTagName(element, 'div');
      }
    }

    // ② 重块延伸 + node list（editable + allowed child of div + alignment 值≠
    // 目标；Contained 收即跳后代）——chromeTight：wrap 面只含当前行/cell。
    newRange = _zwJusBlockExtend(range, true);
    var nodeList = _zwJusGetContainedNodes(newRange, function (node) {
      return _zwJusIsEditable(node)
        && _zwJusIsAllowedChild(node, 'div')
        && _zwJusGetAlignmentValue(node) !== alignment;
    });

    // ③ 连续兄弟 run 逐段 wrap（sibling criteria：仅 align attr = alignment 或
    // 单声明 style text-align = alignment 的 div 可复用；新父 = div style 带值）。
    var guard3 = 0;
    while (nodeList.length && guard3++ < 4096) {
      var sublist = [nodeList.shift()];
      while (nodeList.length
          && nodeList[0] === sublist[sublist.length - 1].nextSibling) {
        sublist.push(nodeList.shift());
      }
      _zwJusWrap(sublist,
        function (node) {
          if (!_zwJusIsHtmlElement(node, 'div')) return false;
          var attrs = node.attributes || [];
          for (var k = 0; k < attrs.length; k++) {
            var an = String(attrs[k].name).toLowerCase();
            if (an === 'align') {
              if (String(attrs[k].value).toLowerCase() !== alignment) return false;
            } else if (an === 'style') {
              var sa = String(attrs[k].value);
              var decls = sa.split(';');
              var n = 0, ta = '';
              for (var d2 = 0; d2 < decls.length; d2++) {
                var dd = decls[d2].trim();
                if (dd === '') continue;
                n++;
                var m = /^text-align\s*:\s*(.+)$/i.exec(dd);
                if (m) ta = m[1].trim().toLowerCase();
              }
              if (!(n === 1 && ta === alignment)) return false;
            } else {
              return false;
            }
          }
          return true;
        },
        function () {
          var np = document.createElement('div');
          // Chrome 样式序列化无空格形（录制 `style="text-align:center"`）。
          try { np.setAttribute('style', 'text-align:' + alignment); } catch (_eJnp) {}
          return np;
        });
    }
    return true;
  };

  // —— 查询面：justify 族 Indeterm/State/Value 一次延伸共享算——
  // spec Indeterm：可见 editable 无子节点叶集合中「有命中且有未命中」→ true；
  // State：叶非空且全命中；Value：首叶对齐值（无叶 → "left"）。
  // https://w3c.github.io/editing/execCommand.html#the-justifycenter-command
  globalThis.__zw_ce_justify_query_face = function (alignment) {
    var range = _zwJusActiveRange();
    if (!range) return { indeterm: false, state: false, value: '' };
    var ext = _zwJusBlockExtend(range, true);
    var leaves = _zwJusGetContainedNodes(ext, function (node) {
      return _zwJusIsVisible(node)
        && _zwJusIsEditable(node)
        && (node.childNodes ? node.childNodes.length : 0) === 0;
    });
    var hit = 0, miss = 0, value = 'left';
    for (var i = 0; i < leaves.length; i++) {
      var av = _zwJusGetAlignmentValue(leaves[i]);
      if (i === 0) value = av;
      if (av === alignment) hit++; else miss++;
    }
    if (!leaves.length) {
      // Chrome 查询锚（caret 面）：contained 叶为空（端点在容器边界、wrap 后
      // DOM 全在非叶块内——blockExtend 夹取到宿主后无叶后代）→ 锚到选区起点
      // 所在块：元素容器位取前一可见子（无则后一子），文本容器即所在块。
      var scQ = range.startContainer;
      if (scQ && scQ.nodeType === 1) {
        var anchorQ = scQ.childNodes[range.startOffset - 1]
          || scQ.childNodes[range.startOffset];
        if (anchorQ) scQ = anchorQ;
      }
      if (scQ) value = _zwJusGetAlignmentValue(scQ);
      return { indeterm: false, state: value === alignment, value: value };
    }
    return {
      indeterm: hit > 0 && miss > 0,
      state: hit > 0 && miss === 0,
      value: value
    };
  };

