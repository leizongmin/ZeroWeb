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
    // 重入断环（qCV 走查 forecolor 跨元素 peel 形实证）：extraneous 判定链
    // precedes → 爬升 → blockStartPoint(parent, brIdx+1) → isVisible(同 br)
    // → 本函数 无界互递归（RangeError）。in-flight 印记 + boundary 探测跳过
    // in-flight br（见 _zwJusIsBlockStartPoint）：precedes 转而回看更前节点，
    // 语义不变、递归有界。
    if (br.__zwExProbeInFlight) return false;
    br.__zwExProbeInFlight = true;
    var r;
    try {
      r = _zwJusPrecedesLineBreak(br);
    } finally {
      br.__zwExProbeInFlight = false;
    }
    return r;
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
    // 含界判定就地展开（2026-11-09 F6b 尾选区 delete 空转定谳）：spec
    // contained 语义为含界（起点不早于 range 起点、终点不晚于 range 终点
    // ——w3c editing spec #contained），端界归一到元素边界后与端界等值的
    // 节点（`<sub>[bar` 起点≡壳界、`baz]` 终点≡宿主界）按严格版会被排除
    // 致空转。就地含界比较使收集面归 spec 口径；_zwJusIsContained 严格版
    // 保留给其余调用面（:1314/:2189/part01f:1573 行为不变）。
    // stop=null 卫兵：end 界落宿主链顶（NextNodeDescendants 走根）时迭代
    // 至树尽——此后无节点，界外溢收集不存在。
    while (node && (!stop || _zwJusIsBefore(node, stop)) && guard++ < 65536) {
      if (_zwJusBpCompare(node, 0, range.startContainer, range.startOffset) >= 0
          && _zwJusBpCompare(node, _zwJusNodeLength(node),
              range.endContainer, range.endOffset) <= 0
          && cond(node)) {
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
    // extraneous 判定在途的 br 不作 boundary 候选（断 extraneous ↔ isVisible
    // 互递归，配对印记见 _zwJusIsExtraneousLineBreak）——该子级视为不存在，
    // 调用方（precedes/follows 回看）继续考察更前/更后节点取真实语义。
    var cand = (node.childNodes && offset - 1 >= 0 && offset - 1 < node.childNodes.length)
      ? node.childNodes[offset - 1] : null;
    return (!node.parentNode && offset === 0)
      || (cand && !cand.__zwExProbeInFlight
          && _zwJusIsVisible(cand)
          && (_zwJusIsBlockNode(cand) || _zwJusIsHtmlElement(cand, 'br')));
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

    // 原始 start 端点（li 臂/do-while 改写 startNode 前捕获；list-wide 纯壳
    // 门判分支归属用）。
    var origStart = startNode;
    // 原始 end 端点（list-wide block start 边界排除臂判几何用，改写前捕获）。
    var origEndN = endNode;
    var origEndO = endOffset;

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

    if (chromeTight === 'list-wide') {
      // toggleLists enable 局部档（q16 转换主流程 Chrome delta）。end 界：
      // vendored blockExtend 忠实语义——纯 at-length 爬升（无跨端关系守卫，
      // :373 内壳 end 必须跨出 innerUL 才能让外壳入含域），宿主为顶护栏
      // （:43/:48 尾文本吸收归 flat-model 臂，range 越顶会过度吸收）。
      // Chrome delta（语料权威，div 边界 compare 族 iul :803/:809）：原始
      // end 恰落 block start 边界（offset0 沿首子链爬升到最近 block 仍在
      // offset0）且该 block 无列表祖先、非宿主、非起始块时，该 block 不入
      // 含域——end 钳到 block 前（语料明注 "don't listify the last empty
      // <div> if only its start boundary is in the range"；对照 :806
      // end=offset3 含域、:800 start 侧对称位不触发、两语料 census 仅
      // :803/:809 该几何；eb1 空体形 start=end 同块不触发）。
      var exB = origEndN, exO = origEndO, gEx = 0;
      while (exO === 0 && exB && exB.parentNode && exB !== host
          && !_zwJusIsBlockNode(exB) && gEx++ < 256) {
        exO = _zwJusNodeIndex(exB);
        exB = exB.parentNode;
      }
      if (exO === 0 && exB && exB !== host && exB.parentNode
      && _zwJusIsBlockNode(exB)
      && !_zwJusIsHtmlElement(exB, ['ol', 'ul', 'li', 'td', 'th'])) {
        var sbX = origStart, gSx = 0;
        while (sbX && sbX.parentNode && sbX !== host
        && !_zwJusIsBlockNode(sbX) && gSx++ < 256) {
          sbX = sbX.parentNode;
        }
        var laX = exB, hasListX = false, gLx = 0;
        while (laX && laX !== host && gLx++ < 256) {
          if (_zwJusIsHtmlElement(laX, ['li', 'ol', 'ul'])) { hasListX = true; break; }
          laX = laX.parentNode;
        }
        if (exB !== sbX && !hasListX) {
          endNode = exB.parentNode;
          endOffset = _zwJusNodeIndex(exB);
        }
      }
      while (endOffset === _zwJusNodeLength(endNode) && endNode && endNode.parentNode
          && endNode !== host) {
        endOffset = 1 + _zwJusNodeIndex(endNode);
        endNode = endNode.parentNode;
      }
      var shellsExited = 0;
      var shellsPure = true;
      var peekN = startNode, peekO = startOffset;
      while (peekO === 0 && peekN && peekN.parentNode && peekN !== host) {
        if (_zwJusIsHtmlElement(peekN, ['ol', 'ul'])) {
          shellsExited++;
          // 纯壳门：被退出壳的直属 li 子若存在不含原始选区 start 端点的
          // 分支（兄弟分支残留），整壳分裂语义（Chrome 保形）依赖 start
          // 相等剔除前序兄弟——不提交爬升（<ul><li>foo<li>[bar]<li>baz
          // </ul> 类；对照 :373 纯壳链/单分支整壳换型）。判据用 origStart
          // （li 臂/do-while 改写前的原始端点），非当前 startNode。
          var pk = peekN.firstChild;
          while (pk) {
            if (_zwJusIsHtmlElement(pk, 'li')
            && !_zwJusIsAncestorOf(pk, origStart) && pk !== origStart) {
              shellsPure = false;
            }
            pk = pk.nextSibling;
          }
        }
        peekO = _zwJusNodeIndex(peekN);
        peekN = peekN.parentNode;
      }
      if (shellsExited > 0 && shellsPure) {
        startNode = peekN;
        startOffset = peekO;
      }
    } else if (chromeTight) {
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
    // 嵌套簇 B2（2026-11-12）：bare 文本子（sel 域解析 wrapper / 工厂产物，无 handle）
    // 入 **handle 壳**（createElement 产物容器）——appendChild trap 对该形态只做本地
    // `_handleChildren` 记账（R84 臂）不发 host mutation，壳内容在 host 侧永久为空
    //（嵌套 wrap 语料 `<i>foo</i>` 壳空根因）。摘挂已完成（上方 removeChild 的
    // RemoveChildAt 已先出批），此处补「CreateTextNode + AppendChildByHandle」净效果
    // 等价发射：host 侧以新句柄文本子落地（同批 Create 注册后 append 解析）。本函数
    // 只服务编辑 move/wrap 流（innerHTML 解析 / cloneNode / shadow 装配不经此处），
    // 发射面天然限于编辑域。
    if (node && node.nodeType === 3 && !node.__zwHandle && newParent.__zwHandle
        && typeof __zw_create_text === 'function' && typeof __zw_append_child_handle === 'function') {
      try {
        var _b2h = __zw_create_text(String(node.data != null ? node.data : (node.__nv || '')));
        if (_b2h) __zw_append_child_handle(newParent.__zwHandle, _b2h);
      } catch (_eJb2) {}
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
    // 返回值：fresh np 场景返回 host 内重定位后的新包元素（原 np 引用已脱挂，
    // list 簇 enable 主循环依赖返回值继续套壳——裸 li 实证），复用包返回原元素。
    var wEl = wrapper;
    try {
      if (rng && wrapper) {
        if (wEl.parentNode !== host) {
          var wantOpen = _zwJusSerializeOpenTag(wrapper);
          // 孪生定位位优先：wrapper 恰占重建子列 firstIdx 位（prefix 前缀 =
          // 成员首索引，html 拼装序即 DOM 序）；位不可靠（解析回流/结构怪形）
          // 再按 open tag 串扫描兜底。串扫描首命中在同 open-tag 前缀面取错
          // 孪生（div 边界 :775 刀双 li 形：liEl 指向 liFooTwin，主循环
          // sublist 重锚后重复引用同节点、真包装 liBarTwin 永远 bare 实证）。
          var posW = host.childNodes[firstIdx];
          if (posW && posW.nodeType === 1
          && _zwJusSerializeOpenTag(posW) === wantOpen) {
            wEl = posW;
          } else {
            for (k = 0; k < host.childNodes.length; k++) {
              var cN = host.childNodes[k];
              if (cN.nodeType === 1 && _zwJusSerializeOpenTag(cN) === wantOpen) {
                wEl = cN; break;
              }
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
    return wEl && wEl.parentNode === host ? wEl : wrapper;
  }

  // plainSkipViaHtml（Chrome delta，formatBlock 专用）：host 直子文本成员走
  // plain wrap 不走 ViaHtml 整段重写——`<p>[foo<p>bar]` + `<p>` 双成员同 tag 时
  // ViaHtml 的序列化重写会使未处理成员游离复制倒序（语料 :1410 三重 p 录音铁证；
  // formatBlock 的 skip 环已在 nodeList 预剔同 tag 成员，plain wrap 到位即净零变更）。
  function _zwJusWrap(nodeList, siblingCriteria, newParentInstructions,
      plainSkipViaHtml) {
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
      if (!plainSkipViaHtml && nodeList[i].nodeType === 3 && nodeList[i].parentNode
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

  // —— Chrome delta 原语（J-A/J-D 祖先壳改写 + 邻接壳吸收 + 包缘 ws 吞入）——
  // 权威面 = 语料 justify*.js 录制期望（真实 Chrome 行为），非 spec 算法——
  // vendored 参考实现同页直调不产出该形（⑥ 族 census 定谳）。

  // 对齐载具元素：align attr（任意值）或 style text-align 声明或 center 标签
  //（与 ① 清理 pass 收集条件同判，另出常函数供祖先链走查/邻接壳判定复用）。
  function _zwJusIsAlignCarrierEl(el) {
    if (!el || el.nodeType !== 1) return false;
    if (_zwJusIsHtmlElement(el, 'center')) return true;
    if (el.getAttribute && el.getAttribute('align') != null) return true;
    var st = el.getAttribute && el.getAttribute('style');
    return !!st && /(?:^|;)\s*text-align\s*:/i.test(String(st));
  }

  // 纯 ws 文本节点（含空串）。
  function _zwJusIsWsOnlyText(node) {
    return !!node && node.nodeType === 3
      && /^[ \t\n\f\r]*$/.test(String(node.nodeValue || ''));
  }

  // style attr 内 text-align 声明覆写（其余声明保留、无则追加、串空摘 attr）。
  // 录制形 `style="text-align:center"` 单声明与值覆写均落此串（无空格序列化）。
  function _zwJusSetInlineTextAlign(el, value) {
    var st = el.getAttribute && el.getAttribute('style');
    var out;
    if (st) {
      var decls = String(st).split(';');
      var kept = [];
      var replaced = false;
      for (var i = 0; i < decls.length; i++) {
        if (/(?:^|\s*)text-align\s*:/i.test(decls[i])) {
          if (!replaced) { kept.push('text-align:' + value); replaced = true; }
          continue;
        }
        if (decls[i].replace(/\s+/g, '') !== '') kept.push(decls[i]);
      }
      if (!replaced) kept.push('text-align:' + value);
      out = kept.join(';').replace(/^;+|;+$/g, '');
    } else {
      out = 'text-align:' + value;
    }
    if (out === '') {
      try { el.removeAttribute('style'); } catch (_eJsita1) {}
    } else {
      try { el.setAttribute('style', out); } catch (_eJsita2) {}
    }
  }

  // 祖先壳改写门控：壳子节点全被含（部分覆盖选区不触发——justifyleft
  // `<div align=left><p>[foo]<p>bar}` 录制原样不动/跨值壳内层包裹）且壳内
  // 无非 editable 节点（contenteditable=false 边界形归既有劈分路径）。
  function _zwJusShellRewritable(shell, rng) {
    var kids = shell.childNodes ? [].slice.call(shell.childNodes) : [];
    for (var i = 0; i < kids.length; i++) {
      if (!_zwJusIsContained(kids[i], rng)) return false;
    }
    var allOk = true;
    var walk = function (node) {
      var c = node.firstChild;
      while (c && allOk) {
        if (!_zwJusIsEditable(c)) { allOk = false; return; }
        walk(c);
        c = c.nextSibling;
      }
    };
    walk(shell);
    return allOk;
  }

  // 相邻同值载具壳折叠：s 紧随（至多隔一纯 ws 文本）同值 div 壳时，ws 与壳子
  // 节点并入 s、壳移除——录制 `<div align="center"><p>foo</p> <p>bar</p>
  // <p>baz</p></div>` 形（justifycenter [7]/justifyright 无 ws 三节点行）。
  function _zwJusCollapseAlignedShell(s, alignment) {
    var guard = 0;
    while (s && s.parentNode && guard++ < 64) {
      var nx = s.nextSibling;
      var ws = null;
      if (_zwJusIsWsOnlyText(nx)) { ws = nx; nx = nx.nextSibling; }
      if (!nx || nx.nodeType !== 1 || _zwJusTag(nx) !== 'div'
          || !_zwJusIsAlignCarrierEl(nx)
          || _zwJusGetAlignmentValue(nx) !== alignment) {
        return;
      }
      if (ws) _zwJusMovePreservingRanges(ws, s, -1);
      while (nx.firstChild) {
        _zwJusMovePreservingRanges(nx.firstChild, s, -1);
      }
      try { nx.parentNode.removeChild(nx); } catch (_eJcol) {}
    }
  }

  globalThis.__zw_ce_justify_apply = function (alignment) {
    var range = _zwJusActiveRange();
    if (!range) return false;

    // 查询面宿主 stash（apply 时点 startContainer 上行链完好——wrap 前节点）：
    // 后续 queryCommandValue/State 若端点为 setter 产物孤儿（parentNode null，
    // 爬链找宿主即断），`_zwJusQueryRoot` 以此兜底下钻。换代写覆盖，陈旧 stash
    // 下钻不中 → 链 null → 保留直读缺省，无副作用。
    (function () {
      var n = range.startContainer, g = 0;
      while (n && g++ < 256) {
        if (_zwJusIsEditingHost(n)) { globalThis.__zwJusLastHost = n; break; }
        n = n.parentNode;
      }
    })();

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
    // Chrome delta（J-A/J-D 对齐载具祖先壳改写）：① 含域无载具（祖先壳不入含
    // 域——{} 边界选区形除外，彼时走 ① 既有路径）且选区端点同在某载具祖先壳
    // 内、壳内容全被含、壳内全 editable 时，改写壳形态并收口。录制口径：
    // - 目标≠left：壳归一 div style text-align=目标——div 摘 align attr 后
    //   覆写/追加声明；center 换代 div 加声明；未知元素（quasit 族）摘 attr
    //   后新 div 外裹。justifycenter/right/full 九祖先形（align/style × 5 值
    //   + center）全落此臂，值==目标也换形（justifycenter [5] align=center →
    //   style 形）。
    // - 目标=left（缺省值）：壳剥对齐（摘 attr/剔声明）后按 ① 同判解包提升
    //   （无属性 div 解包；center 换代后解包）——justifyleft 九祖先形全落此臂。
    if (elementList.length === 0) {
      var shellJ = null;
      var ancJ = range.startContainer;
      var gJ = 0;
      while (ancJ && gJ++ < 128) {
        if (_zwJusIsEditingHost(ancJ)) break;
        if (ancJ.nodeType === 1 && _zwJusIsAlignCarrierEl(ancJ)) {
          shellJ = ancJ;
          break;
        }
        ancJ = ancJ.parentNode;
      }
      var endOkJ = false;
      if (shellJ) {
        endOkJ = newRange.endContainer === shellJ
          || _zwJusIsAncestorOf(shellJ, newRange.endContainer);
      }
      if (shellJ && endOkJ && _zwJusShellRewritable(shellJ, newRange)) {
        var tagJ = _zwJusTag(shellJ);
        if (alignment === 'left') {
          try { shellJ.removeAttribute('align'); } catch (_eJalp) {}
          var stJ = shellJ.getAttribute && shellJ.getAttribute('style');
          if (stJ && /(?:^|;)\s*text-align\s*:/i.test(String(stJ))) {
            var declsJ = String(stJ).split(';');
            var keptJ = [];
            for (var dJ = 0; dJ < declsJ.length; dJ++) {
              if (/(?:^|\s*)text-align\s*:/i.test(declsJ[dJ])) continue;
              if (declsJ[dJ].replace(/\s+/g, '') !== '') keptJ.push(declsJ[dJ]);
            }
            var outJ = keptJ.join(';').replace(/^;+|;+$/g, '');
            if (outJ === '') {
              try { shellJ.removeAttribute('style'); } catch (_eJstp1) {}
            } else {
              try { shellJ.setAttribute('style', outJ); } catch (_eJstp2) {}
            }
          }
          if (_zwJusIsHtmlElement(shellJ, ['div', 'span']) || tagJ === 'center') {
            var elJ = tagJ === 'center' ? _zwJusSetTagName(shellJ, 'div') : shellJ;
            if (elJ.attributes && !elJ.attributes.length) {
              _zwJusRemovePreservingDescendants(elJ);
            }
          }
        } else if (tagJ === 'center') {
          var ndJ = _zwJusSetTagName(shellJ, 'div');
          _zwJusSetInlineTextAlign(ndJ, alignment);
        } else if (tagJ === 'div') {
          try { shellJ.removeAttribute('align'); } catch (_eJald) {}
          _zwJusSetInlineTextAlign(shellJ, alignment);
        } else {
          try { shellJ.removeAttribute('align'); } catch (_eJalu) {}
          var wJ = document.createElement('div');
          try { wJ.setAttribute('style', 'text-align:' + alignment); } catch (_eJwu) {}
          if (shellJ.parentNode) {
            shellJ.parentNode.insertBefore(wJ, shellJ);
            _zwJusMovePreservingRanges(shellJ, wJ, 0);
          }
        }
        return true;
      }
    }
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

    // Chrome delta（J-A 邻接壳吸收）：目标≠left 且选区块与同值 div 载具壳相邻
    //（中间至多一个纯 ws 文本节点）时，块移入壳内（ws 随迁入壳）、紧随同值壳
    // 折叠并入——录制 `<div align="center"><p>foo</p> <p>bar</p></div>`（前邻
    // ws 形）/ `<div align="right"><p>foo</p><p>[bar]</p><p>baz</p></div>`（无
    // ws + 后随壳折叠形）。center 壳不吸收（`<center><p>foo</center> <p>[bar]`
    // 录制新建 div）。目标=left 时选区块对齐已命中不入 nodeList，本臂自然不触
    // 发（justifyleft 邻接形录制原样不动）。
    if (alignment !== 'left') {
      for (var bI = nodeList.length - 1; bI >= 0; bI--) {
        var bJ2 = nodeList[bI];
        if (!bJ2 || bJ2.nodeType !== 1 || !_zwJusIsHtmlElement(bJ2)) continue;
        var parJ2 = bJ2.parentNode;
        if (!parJ2) continue;
        var pvJ = bJ2.previousSibling;
        var pvWsJ = null;
        if (_zwJusIsWsOnlyText(pvJ)) { pvWsJ = pvJ; pvJ = pvJ.previousSibling; }
        if (pvJ && pvJ.nodeType === 1 && _zwJusTag(pvJ) === 'div'
            && _zwJusIsAlignCarrierEl(pvJ)
            && _zwJusGetAlignmentValue(pvJ) === alignment
            && pvJ.parentNode === parJ2 && _zwJusIsEditable(pvJ)) {
          if (pvWsJ) _zwJusMovePreservingRanges(pvWsJ, pvJ, -1);
          _zwJusMovePreservingRanges(bJ2, pvJ, -1);
          _zwJusCollapseAlignedShell(pvJ, alignment);
          nodeList.splice(bI, 1);
          continue;
        }
        var nxJ = bJ2.nextSibling;
        var nxWsJ = null;
        if (_zwJusIsWsOnlyText(nxJ)) { nxWsJ = nxJ; nxJ = nxJ.nextSibling; }
        if (nxJ && nxJ.nodeType === 1 && _zwJusTag(nxJ) === 'div'
            && _zwJusIsAlignCarrierEl(nxJ)
            && _zwJusGetAlignmentValue(nxJ) === alignment
            && nxJ.parentNode === parJ2 && _zwJusIsEditable(nxJ)) {
          _zwJusMovePreservingRanges(bJ2, nxJ, 0);
          if (nxWsJ) _zwJusMovePreservingRanges(nxWsJ, nxJ, 1);
          _zwJusCollapseAlignedShell(nxJ, alignment);
          nodeList.splice(bI, 1);
        }
      }
    }

    // ③ 连续兄弟 run 逐段 wrap（sibling criteria：仅 align attr = alignment 或
    // 单声明 style text-align = alignment 的 div 可复用；新父 = div style 带值）。
    var guard3 = 0;
    while (nodeList.length && guard3++ < 4096) {
      var sublist = [nodeList.shift()];
      while (nodeList.length
          && nodeList[0] === sublist[sublist.length - 1].nextSibling) {
        sublist.push(nodeList.shift());
      }
      var freshJ = false;
      var wrappedJ = _zwJusWrap(sublist,
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
          freshJ = true;
          var np = document.createElement('div');
          // Chrome 样式序列化无空格形（录制 `style="text-align:center"`）。
          try { np.setAttribute('style', 'text-align:' + alignment); } catch (_eJnp) {}
          return np;
        });
      // Chrome delta（J-E 包缘 ws 吞入）：新建包 div 吞并紧邻纯 ws 文本兄弟——
      // 录制 `<div style="text-align:center"> <p>bar</p></div>`（前缘）/
      // `<div style="text-align:center"><p>foo</p> </div>`（后缘）形；复用壳
      // 与既有 DOM 形不动（freshJ 仅新父臂置位）。
      if (freshJ && wrappedJ) {
        var pvWJ = wrappedJ.previousSibling;
        if (_zwJusIsWsOnlyText(pvWJ)) _zwJusMovePreservingRanges(pvWJ, wrappedJ, 0);
        var nxWJ = wrappedJ.nextSibling;
        if (_zwJusIsWsOnlyText(nxWJ)) _zwJusMovePreservingRanges(nxWJ, wrappedJ, -1);
      }
    }
    return true;
  };

  // —— 查询面：justify 族 Indeterm/State/Value 一次延伸共享算——
  // spec Indeterm：可见 editable 无子节点叶集合中「有命中且有未命中」→ true；
  // State：叶非空且全命中；Value：首叶对齐值（无叶 → "left"）。
  // https://w3c.github.io/editing/execCommand.html#the-justifycenter-command
  //
  // 对齐读数 uplink 断裂兜底（zz-jrprobe span-caret 实证）：wrap 走 `host.innerHTML=`
  // setter（sanctioned 写路径——裸 DOM 搬运不回流读数），setter 产物子 wrapper 是
  // 孤儿——div.childNodes 可枚举（nKids=2）而 child.parentNode=null、兄弟链 null，
  // `_zwJusGetAlignmentValue` 的 parentNode 爬链与 `_zwJusIsContained` 的 BpCompare
  // 祖先链双双失效 → 查询面 Value 恒 'left' / State 恒 false（justifycenter 同形
  // 127 例、right/full 同构）。兜底只对「直读返缺省 'left' 且上行链不可达宿主」的
  // 节点触发：自宿主下钻 DFS 合成祖先链，读链上显式提示（style text-align / align
  // 属性映射集 / center 标签）；链上无显式提示 → null（保留直读缺省——真左对齐
  // 不受影响）。健康路径可达性爬链 O(depth) 即返回，无 DFS 成本。
  function _zwJusQueryRoot(range) {
    var n = range.startContainer;
    var g = 0;
    while (n && g++ < 256) {
      if (_zwJusIsEditingHost(n)) return n;
      n = n.parentNode;
    }
    var stash = globalThis.__zwJusLastHost;
    return stash && _zwJusIsEditingHost(stash) ? stash : null;
  }
  function _zwJusChainDown(root, target) {
    if (root === target) return [root];
    var kids = root.childNodes;
    if (!kids) return null;
    for (var i = 0; i < kids.length; i++) {
      var sub = _zwJusChainDown(kids[i], target);
      if (sub) { sub.unshift(root); return sub; }
    }
    return null;
  }
  // 合成链读数（镜像 `_zwJusGetAlignmentValue` 判序，链替代 parentNode 爬行）：
  // 自内向外（target 端先）跳过非元素/inline 显示节点 → 最近块级元素 computed
  // text-align；缺字面值（''/start/end）时继续沿链外扩做显式提示补读（style
  // text-align / align 属性映射集 / center 标签——`_zwJusAlignmentHint` 的
  // parentNode 爬行对孤儿不可用，链外扩等价替代）。无提示 → null。
  function _zwJusAvFromChain(chain) {
    var bi = -1;
    for (var i = chain.length - 1; i >= 0; i--) {
      var nd = chain[i];
      if (!nd || nd.nodeType !== 1) continue;
      if (['inline', 'none'].indexOf(_zwJusCs(nd, 'display')) !== -1) continue;
      bi = i;
      var resolved = _zwJusCs(nd, 'textAlign')
        .replace(/^-(moz|webkit)-/, '')
        .replace(/^auto$/, 'start');
      if (['center', 'justify', 'left', 'right'].indexOf(resolved) !== -1) return resolved;
      break;
    }
    if (bi < 0) return null;
    for (var j = bi; j >= 0; j--) {
      var hn = chain[j];
      if (!hn || hn.nodeType !== 1) continue;
      var own = hn.getAttribute && hn.getAttribute('style');
      if (own) {
        var m = /(?:^|;)\s*text-align\s*:\s*([^;\s]+)/i.exec(String(own));
        if (m) {
          var tv = String(m[1]).toLowerCase();
          if (['center', 'justify', 'left', 'right'].indexOf(tv) !== -1) return tv;
        }
      }
      var t = _zwJusTag(hn);
      if (t === 'center') return 'center';
      if (t !== 'table' && _zwJusAlignMapped[t] && hn.getAttribute) {
        var al = hn.getAttribute('align');
        if (al != null) {
          var av = String(al).trim().toLowerCase();
          if (av === 'left' || av === 'right' || av === 'justify') return av;
          if (av === 'center' || av === 'middle') return 'center';
        }
      }
    }
    return null;
  }
  function _zwJusQueryAlignment(root, node, directAv) {
    if (directAv !== 'left' || !root || !node) return directAv;
    var up = node, reachable = false, g = 0;
    while (up && g++ < 256) {
      if (up === root) { reachable = true; break; }
      up = up.parentNode;
    }
    if (reachable) return directAv;
    var chain = _zwJusChainDown(root, node);
    if (!chain) return directAv;
    var av = _zwJusAvFromChain(chain);
    return av == null ? directAv : av;
  }

  globalThis.__zw_ce_justify_query_face = function (alignment) {
    var range = _zwJusActiveRange();
    if (!range) return { indeterm: false, state: false, value: '' };
    var ext = _zwJusBlockExtend(range, true);
    var leaves = _zwJusGetContainedNodes(ext, function (node) {
      return _zwJusIsVisible(node)
        && _zwJusIsEditable(node)
        && (node.childNodes ? node.childNodes.length : 0) === 0;
    });
    var qroot = _zwJusQueryRoot(range);
    var hit = 0, miss = 0, value = 'left';
    for (var i = 0; i < leaves.length; i++) {
      var av = _zwJusQueryAlignment(qroot, leaves[i], _zwJusGetAlignmentValue(leaves[i]));
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
      if (scQ) value = _zwJusQueryAlignment(qroot, scQ, _zwJusGetAlignmentValue(scQ));
      return { indeterm: false, state: value === alignment, value: value };
    }
    return {
      indeterm: hit > 0 && miss > 0,
      state: hit > 0 && miss === 0,
      value: value
    };
  };

  // ============================================================
  // editing-compat style-with-value 簇（2026-11-01）：plain 元素四属性计算值
  // 解析器（getComputedStyle 的 host miss 回落）。
  //
  // 缘由：editing/run 语料的查询与比较机制是 vendored tests.js 直呼其实现函数
  //（myQueryCommandValue → commands.*.value() → getEffectiveCommandValue →
  // getComputedStyle(node)[relevantCssProperty]，另有 getLegacyFontSize 运行时
  // 探针、normalizeColor 色归一探针）——全部流经我方 globalThis.getComputedStyle。
  // host 通道只覆盖 sel/handle 注册元素（probe t16：setupDiv 内容物全 M 形，
  // fontSize/color/fontFamily/backgroundColor 恒 ''；display 有 UA 表回落故
  // isVisible 不受扰）。本解析器按 CSS 级联语义 JS 侧补齐四属性：
  //   inline style → font 表现提示（size/color/face/bgcolor，HTML 渲染节
  //   https://html.spec.whatwg.org/multipage/rendering.html#fonts-and-colors ）→
  //   继承走查（color/font-size/font-family 继承；background-color 非继承——
  //   首个非透明声明即停，透明继续上行，等价 getEffectiveCommandValue 的
  //   transparent 爬升组合）。无源返空——gCS 通道保 P9-9/R5107 合成源契约
  //   （无规则即空，englib p99 钉测）；UA 初值（color rgb(0, 0, 0)、
  //   font-family serif、font-size 16px、background-color 透明）由值面消费点
  //   __zw_ce_value_query 填（probe tD/tE 与语料裸文本期望）。
  // 序列化对齐 Chrome 计算值口径（state/indeterm 面做原始串等值比较，不归一）：
  //   named/hex/hsl → rgb(r, g, b)；alpha≠1 → rgba(r, g, b, a)；
  //   transparent → rgba(0, 0, 0, 0)；currentColor → 沿父链继承解析；
  //   font-size 关键字表 px 底数取 Chrome 缺省字号标度（xx-small 9px …
  //   xxx-large 48px，probe tC 实证；font[size] attr 1-7 → x-small…xxx-large）。
  // FIXME: 仅逗号式 rgb()/rgba()/hsl()/hsla()；空格式语法 `rgb(0 0 255 / .5)`
  //   与 font 简写（font: …）未解析。relative size 关键字（smaller/larger）与
  //   rem 未实现（语料无消费面）。
  // FIXME: CSS 系统色（canvas/buttontext 等）与 color-mix() 等现代语法未覆盖。
  var _zwPlainNamedColor = {
    aliceblue: '240, 248, 255', antiquewhite: '250, 235, 215', aqua: '0, 255, 255',
    aquamarine: '127, 255, 212', azure: '240, 255, 255', beige: '245, 245, 220',
    bisque: '255, 228, 196', black: '0, 0, 0', blanchedalmond: '255, 235, 205',
    blue: '0, 0, 255', blueviolet: '138, 43, 226', brown: '165, 42, 42',
    burlywood: '222, 184, 135', cadetblue: '95, 158, 160', chartreuse: '127, 255, 0',
    chocolate: '210, 105, 30', coral: '255, 127, 80', cornflowerblue: '100, 149, 237',
    cornsilk: '255, 248, 220', crimson: '220, 20, 60', cyan: '0, 255, 255',
    darkblue: '0, 0, 139', darkcyan: '0, 139, 139', darkgoldenrod: '184, 134, 11',
    darkgray: '169, 169, 169', darkgreen: '0, 100, 0', darkgrey: '169, 169, 169',
    darkkhaki: '189, 183, 107', darkmagenta: '139, 0, 139',
    darkolivegreen: '85, 107, 47', darkorange: '255, 140, 0', darkorchid: '153, 50, 204',
    darkred: '139, 0, 0', darksalmon: '233, 150, 122', darkseagreen: '143, 188, 143',
    darkslateblue: '72, 61, 139', darkslategray: '47, 79, 79', darkslategrey: '47, 79, 79',
    darkturquoise: '0, 206, 209', darkviolet: '148, 0, 211', deeppink: '255, 20, 147',
    deepskyblue: '0, 191, 255', dimgray: '105, 105, 105', dimgrey: '105, 105, 105',
    dodgerblue: '30, 144, 255', firebrick: '178, 34, 34', floralwhite: '255, 250, 240',
    forestgreen: '34, 139, 34', fuchsia: '255, 0, 255', gainsboro: '220, 220, 220',
    ghostwhite: '248, 248, 255', gold: '255, 215, 0', goldenrod: '218, 165, 32',
    gray: '128, 128, 128', green: '0, 128, 0', greenyellow: '173, 255, 47',
    grey: '128, 128, 128', honeydew: '240, 255, 240', hotpink: '255, 105, 180',
    indianred: '205, 92, 92', indigo: '75, 0, 130', ivory: '255, 255, 240',
    khaki: '240, 230, 140', lavender: '230, 230, 250', lavenderblush: '255, 240, 245',
    lawngreen: '124, 252, 0', lemonchiffon: '255, 250, 205', lightblue: '173, 216, 230',
    lightcoral: '240, 128, 128', lightcyan: '224, 255, 255',
    lightgoldenrodyellow: '250, 250, 210', lightgray: '211, 211, 211',
    lightgreen: '144, 238, 144', lightgrey: '211, 211, 211', lightpink: '255, 182, 193',
    lightsalmon: '255, 160, 122', lightseagreen: '32, 178, 170',
    lightskyblue: '135, 206, 250', lightslategray: '119, 136, 153',
    lightslategrey: '119, 136, 153', lightsteelblue: '176, 196, 222',
    lightyellow: '255, 255, 224', lime: '0, 255, 0', limegreen: '50, 205, 50',
    linen: '250, 240, 230', magenta: '255, 0, 255', maroon: '128, 0, 0',
    mediumaquamarine: '102, 221, 170', mediumblue: '0, 0, 205',
    mediumorchid: '186, 85, 211', mediumpurple: '147, 112, 219',
    mediumseagreen: '60, 179, 113', mediumslateblue: '123, 104, 238',
    mediumspringgreen: '0, 250, 154', mediumturquoise: '72, 209, 204',
    mediumvioletred: '199, 21, 133', midnightblue: '25, 25, 112',
    mintcream: '245, 255, 250', mistyrose: '255, 228, 225', moccasin: '255, 228, 181',
    navajowhite: '255, 222, 173', navy: '0, 0, 128', oldlace: '253, 245, 230',
    olive: '128, 128, 0', olivedrab: '107, 142, 35', orange: '255, 165, 0',
    orangered: '255, 69, 0', orchid: '218, 112, 214', palegoldenrod: '238, 232, 170',
    palegreen: '152, 251, 152', paleturquoise: '175, 238, 238',
    palevioletred: '219, 112, 147', papayawhip: '255, 239, 213',
    peachpuff: '255, 218, 185', peru: '205, 133, 63', pink: '255, 192, 203',
    plum: '221, 160, 221', powderblue: '176, 224, 230', purple: '128, 0, 128',
    red: '255, 0, 0', rosybrown: '188, 143, 143', royalblue: '65, 105, 225',
    saddlebrown: '139, 69, 19', salmon: '250, 128, 114', sandybrown: '244, 164, 96',
    seagreen: '46, 139, 87', seashell: '255, 245, 238', sienna: '160, 82, 45',
    silver: '192, 192, 192', skyblue: '135, 206, 235', slateblue: '106, 90, 205',
    slategray: '112, 128, 144', slategrey: '112, 128, 144', snow: '255, 250, 250',
    springgreen: '0, 255, 127', steelblue: '70, 130, 180', tan: '210, 180, 140',
    teal: '0, 128, 128', thistle: '216, 191, 216', tomato: '255, 99, 71',
    turquoise: '64, 224, 208', violet: '238, 130, 238', wheat: '245, 222, 179',
    white: '255, 255, 255', whitesmoke: '245, 245, 245', yellow: '255, 255, 0',
    yellowgreen: '154, 205, 50'
  };
  // font[size] attr → CSS 绝对字号关键字（HTML 渲染节映射）；px 底数 = Chrome
  // 缺省字号标度（probe tB/tC：10/13/16/18/24/32/48px；xx-small 9px 同证）。
  var _zwPlainFontAttrKw = { 1: 'x-small', 2: 'small', 3: 'medium', 4: 'large',
    5: 'x-large', 6: 'xx-large', 7: 'xxx-large' };
  var _zwPlainFontKwPx = { 'xx-small': 9, 'x-small': 10, small: 13, medium: 16,
    large: 18, 'x-large': 24, 'xx-large': 32, 'xxx-large': 48 };
  // UA stylesheet font-family 元素规则（HTML 渲染节 15.3.1.1——`tt, code, kbd,
  // samp { font-family: monospace }`；pre/listing/xmp/plaintext 同款。元素选择器
  // 规则 = 每个命中元素的声明，级联中位于继承之上——walk 逐元素内层优先命中）。
  var _zwPlainUaMono = { tt: 1, code: 1, kbd: 1, samp: 1, pre: 1, listing: 1,
    xmp: 1, plaintext: 1 };
  // inline style 属性文本的朴素声明扫描（style IDL 缺位形态的兜底——pending 形
  // 无 .style）。返 { prop: value }（prop kebab 小写、value trim）。文本半边
  // 独立导出（cssom 写回刀 2026-11-11：_zwVaSetStyleProp 读核验需按 attr 权威
  // 面文本扫描——getAttribute 接 CSSOM replay 后元素入参面不再恒等 attr 权威）。
  function _zwPlainStyleDeclsText(st) {
    var out = {};
    var parts = String(st == null ? '' : st).split(';');
    for (var i = 0; i < parts.length; i++) {
      var seg = parts[i];
      var at = seg.indexOf(':');
      if (at <= 0) continue;
      var k = seg.slice(0, at).trim().toLowerCase();
      if (k) out[k] = seg.slice(at + 1).trim();
    }
    return out;
  }
  function _zwPlainStyleDecls(el) {
    var out = {};
    try {
      var st = String(el.getAttribute && el.getAttribute('style') || '');
      out = _zwPlainStyleDeclsText(st);
    } catch (_ePD) {}
    return out;
  }
  function _zwPlainDecl(el, prop) {
    // 优先 style IDL（wrapper/M 形均有；getPropertyValue 自带 kebab 归一与
    // 末次声明语义），缺位回落属性文本扫描。
    try {
      if (el.style && typeof el.style.getPropertyValue === 'function') {
        var v = el.style.getPropertyValue(prop);
        if (v) {
          // raw 影子优先（css-color-4 §missing：declared 写面归一消解
          // `rgb(128 none none)` → `rgb(128, 0, 0)` 是 valid corpus 规范化面，
          // computed corpus 期望 `color(srgb 0.50196078 none none)` 保 none——
          // part03 _zwRawByHandle 影子在归一变形时存原值，本读取点取回交
          // part11-computed-color 管线；其余属性/形态影子为空，走归一形）。
          if (typeof el.style.__zwRawDecl === 'function') {
            var rv = el.style.__zwRawDecl(prop);
            if (rv) return String(rv).trim();
          }
          return String(v).trim();
        }
      }
    } catch (_ePST) {}
    return _zwPlainStyleDecls(el)[prop] || '';
  }
  // 作者样式表扁平声明快照（2026-11-05 ③'-c）：host gCS 桥对 <link> 作者样式表
  // 盲（语料 #purple qV-before got 黑实证——合成源契约 P9-9/R5107 面），plain
  // 走查此前只见 inline/表现提示/UA 臂。editing 语料 reset.css 全规则为无后代
  // 组合器的简单选择器（元素/类/id/伪类 :link,:visited），据此做有界扁平匹配。
  // 页生命周期快照：document.styleSheets getter 每访问即走桥面全量重查（part06
  // live DOM 语义），而 __zwPlainComputed 位于 apply/qV 热走查（每节点每步调
  // 用——shim trap 域成本不可见，R350 教训），故首用构建、页生命周期复用；语料
  // 壳页均为静态样式表（runConformanceTest 不改 sheet）。表/规则数封顶防病态页；
  // 修改样式表的页面不经本缓存面（cssom 公开视图不走此路径，语义不变）。
  var _zwPlainAuthorRules = null;
  function _zwPlainAuthorSnapshot() {
    if (_zwPlainAuthorRules) return _zwPlainAuthorRules;
    var out = [];
    try {
      var sheets = globalThis.document ? globalThis.document.styleSheets : null;
      for (var si = 0; sheets && si < sheets.length && si < 16; si++) {
        var rules = null;
        try { rules = sheets[si].cssRules; } catch (_ePASn) { rules = null; }
        if (!rules) continue;
        for (var ri = 0; ri < rules.length && out.length < 256; ri++) {
          var r = rules[ri];
          // 仅 style 规则（selectorText+style 双在；@media/@import 等无此二者）。
          if (!r || !r.selectorText || !r.style) continue;
          out.push({ sel: String(r.selectorText), st: r.style,
            text: String(r.style.cssText || '') });
        }
      }
    } catch (_ePAS) {}
    _zwPlainAuthorRules = out;
    return out;
  }
  function _zwPlainAuthorMatches(selText, el) {
    var sels = String(selText).split(',');
    var tag = String(el.tagName || '').toLowerCase();
    var id = '';
    try { id = el.id ? String(el.id) : ''; } catch (_ePAMi) {}
    var cls = [];
    try {
      cls = el.className ? String(el.className).split(/\s+/) : [];
    } catch (_ePAMc) {}
    for (var i = 0; i < sels.length; i++) {
      var s = sels[i].trim().toLowerCase();
      if (!s) continue;
      // 有界域：简单选择器（元素/.类/#id/:link/:visited）；含组合器/属性/通配/
      // 其他伪类的选择器一律不匹配（越界宁可漏配不误配——reset.css 无此形态）。
      var bare = s.replace(/:link|:visited/g, '');
      if (!bare || /[\s>+~\[*:]/.test(bare)) continue;
      var m = /^([a-z][a-z0-9]*)?((?:[.#][a-z0-9_-]+)*)(:(?:link|visited))?$/.exec(s);
      if (!m) continue;
      if (m[1] && m[1] !== tag) continue;
      var ok = true;
      var parts = m[2] ? m[2].match(/[.#][a-z0-9_-]+/g) || [] : [];
      for (var pi = 0; pi < parts.length; pi++) {
        if (parts[pi].charAt(0) === '#') {
          if (id !== parts[pi].slice(1)) { ok = false; break; }
        } else if (cls.indexOf(parts[pi].slice(1)) === -1) { ok = false; break; }
      }
      if (!ok) continue;
      if (m[3]) {
        // :link/:visited 同域近似（语料 reset.css 双伪类同声明 blue；:visited
        // 历史不可查——与 UA a[href] 臂同口径）。
        if (tag !== 'a') continue;
        try {
          if (!el.getAttribute || el.getAttribute('href') == null) continue;
        } catch (_ePAMl) { continue; }
      }
      return true;
    }
    return false;
  }
  function _zwPlainAuthorDecl(el, prop) {
    // 末次匹配规则胜（同 specificity 文档序语义；语料 reset.css 无同属性竞争
    // 规则，全排序不需要）。声明读取：style IDL getPropertyValue 优先（kebab
    // 归一 + 无序列化损耗），cssText 文本扫描兜底。
    var rules = _zwPlainAuthorSnapshot();
    var found = '';
    for (var i = 0; i < rules.length; i++) {
      if (!_zwPlainAuthorMatches(rules[i].sel, el)) continue;
      var v = '';
      try {
        if (rules[i].st && typeof rules[i].st.getPropertyValue === 'function') {
          v = String(rules[i].st.getPropertyValue(prop) || '').trim();
        }
      } catch (_ePADg) {}
      if (!v && rules[i].text) {
        var m = new RegExp('(?:^|;)\\s*' + prop + '\\s*:\\s*([^;]+)', 'i')
          .exec(rules[i].text);
        if (m) v = m[1].trim();
      }
      if (v) found = v;
    }
    return found;
  }
  // hsl(h, s%, l%) → rgb 序列化值（CSS Color 3 算法；h 角度可负可超 360）。
  function _zwPlainHslToRgb(h, s, l) {
    h = ((h % 360) + 360) % 360;
    s /= 100; l /= 100;
    var c = (1 - Math.abs(2 * l - 1)) * s;
    var x = c * (1 - Math.abs((h / 60) % 2 - 1));
    var m = l - c / 2;
    var r = 0, g = 0, b = 0;
    if (h < 60) { r = c; g = x; } else if (h < 120) { r = x; g = c; }
    else if (h < 180) { g = c; b = x; } else if (h < 240) { g = x; b = c; }
    else if (h < 300) { r = x; b = c; } else { r = c; b = x; }
    return [Math.round((r + m) * 255), Math.round((g + m) * 255), Math.round((b + m) * 255)];
  }
  // 颜色声明 → Chrome 计算值序列化（rgb(r, g, b) / rgba(r, g, b, a)）。
  // 无 alpha 或 alpha=1 → rgb() 形（Chrome 序列化口径）；解析失败返 ''。
  function _zwPlainColorSpec(val, el) {
    // 注释剥离（css-syntax 注释=空白——'/**/transparent' inline 语料实证：
    // 不剥则解析失败落入继承爬升，transparent 语义翻红）。
    var s0 = String(val == null ? '' : val);
    var v = (s0.indexOf('/*') < 0 ? s0 : s0.replace(/\/\*[\s\S]*?\*\//g, ' ')).trim();
    if (!v) return '';
    var low = v.toLowerCase();
    if (low === 'transparent') return 'rgba(0, 0, 0, 0)';
    if (low === 'currentcolor') {
      // currentColor = 继承的 color 计算值（沿父链解析；缺省黑）。
      var p = el && el.parentNode;
      while (p && p.nodeType === 1) {
        var pv = globalThis.__zwPlainComputed(p, 'color');
        if (pv) return pv;
        p = p.parentNode;
      }
      return 'rgb(0, 0, 0)';
    }
    if (Object.prototype.hasOwnProperty.call(_zwPlainNamedColor, low)) {
      return 'rgb(' + _zwPlainNamedColor[low] + ')';
    }
    if (v.charAt(0) === '#') {
      var hex = v.slice(1);
      if (/^[0-9a-fA-F]{3}$/.test(hex)) {
        hex = hex.charAt(0) + hex.charAt(0) + hex.charAt(1) + hex.charAt(1)
          + hex.charAt(2) + hex.charAt(2);
      }
      if (!/^[0-9a-fA-F]{6}$/.test(hex)) return '';
      return 'rgb(' + parseInt(hex.slice(0, 2), 16) + ', '
        + parseInt(hex.slice(2, 4), 16) + ', ' + parseInt(hex.slice(4, 6), 16) + ')';
    }
    var m = /^(rgba?|hsla?)\(([^)]*)\)$/i.exec(v);
    if (!m) return '';
    var fn = m[1].toLowerCase();
    var nums = m[2].split(',').map(function (s) { return s.trim(); });
    if (nums.length < 3) return '';
    var comps = [];
    for (var i = 0; i < 3; i++) {
      var t = nums[i];
      var isPct = /%$/.test(t);
      var n = parseFloat(t);
      if (isNaN(n)) return '';
      if (fn === 'rgb' || fn === 'rgba') {
        comps.push(isPct ? Math.round(n * 255 / 100) : Math.round(n));
      } else {
        comps.push(n);
      }
    }
    var rgbv = (fn === 'rgb' || fn === 'rgba')
      ? comps : _zwPlainHslToRgb(comps[0], comps[1], comps[2]);
    if (fn === 'rgba' || fn === 'hsla') {
      var a = parseFloat(nums[3]);
      if (isNaN(a)) return '';
      if (a >= 1) return 'rgb(' + rgbv.join(', ') + ')';
      return 'rgba(' + rgbv.join(', ') + ', ' + String(a) + ')';
    }
    return 'rgb(' + rgbv.join(', ') + ')';
  }
  // font-size px 解析（数字 = 浏览器缺省标度 16px 基准）。不可解析（含
  // smaller/larger、rem、简写）返 NaN → 调用方视作无声明。
  function _zwPlainFontSizePx(el) {
    var decl = _zwPlainDecl(el, 'font-size');
    var m = /^([+-]?[0-9]*\.?[0-9]+)([a-z%]*)$/.exec(String(decl).trim());
    if (m) {
      var n = parseFloat(m[1]);
      var u = m[2];
      if (u === 'px') return n;
      if (u === 'pt') return n * 96 / 72;
      if (u === 'pc') return n * 16;
      if (u === 'in') return n * 96;
      if (u === 'cm') return n * 96 / 2.54;
      if (u === 'mm') return n * 96 / 25.4;
      if (u === 'em' || u === '%') {
        var par = el.parentNode;
        var base = 16;
        var guard = 0;
        while (par && par.nodeType === 1 && guard++ < 256) {
          var pv = _zwPlainFontSizePx(par);
          if (!isNaN(pv)) { base = pv; break; }
          par = par.parentNode;
        }
        return u === 'em' ? n * base : n * base / 100;
      }
      return NaN;
    }
    var kw = _zwPlainFontKwPx[String(decl).trim().toLowerCase()];
    if (kw) return kw;
    // font[size] 表现提示（HTML 渲染节）。
    try {
      var sa = el.getAttribute && el.getAttribute('size');
      if (sa != null) {
        // 相对值 ±N：相对父链 rendered size 档位 ±N（HTML 渲染节 relative
        // size 映射——`<font size=+1>` 语料 qV fontsize 期望档 4 实证；裸
        // parseInt('+1') 会错当档位 1 = x-small）。基准档 = 父链首个可解析
        // font-size 过 legacy 中点表（缺省 16px → 档 3），结果 clamp 1..7。
        var mRel = /^([+-])([1-6])$/.exec(String(sa).trim());
        if (mRel) {
          var basePx = 16;
          var par = el.parentNode;
          var guardRel = 0;
          while (par && par.nodeType === 1 && guardRel++ < 256) {
            var pp = _zwPlainFontSizePx(par);
            if (!isNaN(pp)) { basePx = pp; break; }
            par = par.parentNode;
          }
          var lg = parseInt(_zwValLegacyFontSize(basePx + 'px'), 10);
          if (isNaN(lg)) lg = 3;
          lg += (mRel[1] === '+' ? 1 : -1) * parseInt(mRel[2], 10);
          lg = Math.min(7, Math.max(1, lg));
          return _zwPlainFontKwPx[_zwPlainFontAttrKw[lg]];
        }
        if (Object.prototype.hasOwnProperty.call(_zwPlainFontAttrKw, String(parseInt(sa, 10)))) {
          return _zwPlainFontKwPx[_zwPlainFontAttrKw[String(parseInt(sa, 10))]];
        }
      }
    } catch (_ePSA) {}
    // UA stylesheet 元素规则（HTML 渲染节 15.3.1：big{font-size:larger}/
    // small{font-size:smaller}）——engine UA 样式表无该规则，host gCS 恒
    // 缺省 16px。larger/smaller = 父链 rendered size 过 legacy 中点表定档
    // ±1（与 font[size] ±N 相对臂同口径），clamp 1..7。font[size] 表现提示
    // （author 源）级联在上，故后置。
    try {
      var tg = (el.tagName && String(el.tagName).toLowerCase()) || '';
      if (tg === 'big' || tg === 'small') {
        var baseUa = 16;
        var parUa = el.parentNode;
        var guardUa = 0;
        while (parUa && parUa.nodeType === 1 && guardUa++ < 256) {
          var pvUa = _zwPlainFontSizePx(parUa);
          if (!isNaN(pvUa)) { baseUa = pvUa; break; }
          parUa = parUa.parentNode;
        }
        var lgUa = parseInt(_zwValLegacyFontSize(baseUa + 'px'), 10);
        if (isNaN(lgUa)) lgUa = 3;
        lgUa += tg === 'big' ? 1 : -1;
        lgUa = Math.min(7, Math.max(1, lgUa));
        return _zwPlainFontKwPx[_zwPlainFontAttrKw[lgUa]];
      }
    } catch (_ePSB) {}
    return NaN;
  }
  // 入口：prop 为 kebab-case；仅应答四属性，其余返 ''（调用方维持旧行为）。
  // 作者样式表声明查询出口（part11 var() 展开消费——页级 `html { --x: ... }`
  // 声明臂；_zwPlainAuthorDecl 通用 prop，custom property 命名直达）。
  globalThis.__zwPlainAuthorDecl = function (el, prop) {
    try { return _zwPlainAuthorDecl(el, prop) || ''; } catch (_ePADx) { return ''; }
  };

  globalThis.__zwPlainComputed = function (elt, prop, qvMode) {
    if (!elt || elt.nodeType !== 1) return '';
    var guard = 0;
    var node = elt;
    if (prop === 'font-size') {
      // 继承：自 elt 上行首个可解析声明（或 font[size] 提示）；无源返空。
      var px = NaN;
      while (node && node.nodeType === 1 && guard++ < 256) {
        px = _zwPlainFontSizePx(node);
        if (!isNaN(px)) break;
        node = node.parentNode;
      }
      if (isNaN(px)) return '';
      // Chrome 序列化：非整值保 3 位小数（20pt → 26.667px，probe tC）。
      var r3 = Math.round(px * 1000) / 1000;
      return String(r3) + 'px';
    }
    if (prop === 'opacity') {
      // DC-2 次批（css-color-4 § opacity computed）：非继承——只读自身 inline
      // 声明（语料 test_computed_value 全走 inline set）；number/%/calc()/
      // min()/max()/clamp() 经 __zwOpacityComputed（clamp [0,1]）序列化；
      // 无声明返初始值 '1'（css-position 邻族初始值回落同形）。
      var opRaw = _zwPlainDecl(elt, 'opacity');
      if (opRaw && typeof globalThis.__zwOpacityComputed === 'function') {
        try {
          var opV = globalThis.__zwOpacityComputed(opRaw);
          if (opV !== '') return opV;
        } catch (_eOpC) {}
        return '';
      }
      return '1';
    }
    if (prop === 'position-anchor' || prop === 'position-visibility'
        || prop === 'position-try-order' || prop === 'position-try-fallbacks'
        || prop === 'anchor-name' || prop === 'anchor-scope') {
      // wptfyi-css 组 F：css-anchor-position-1 anchor 族声明面 computed——全族
      // 非继承（position-area 先例同构）：只读自身 inline 声明，computed =
      // as-specified（canonicalize 折叠后恒等）；空声明返属性初始值
      //（position-anchor normal / position-visibility anchors-visible /
      // position-try-order normal / position-try-fallbacks·anchor-name·
      // anchor-scope none）。CSS-wide keyword 计算期语义（CSS Cascading 4
      // §6-§7）：initial/unset/revert/revert-layer → 初始值；inherit 强制取
      // 父链首个非 css-wide 声明的计算值（§7.3 对非继承属性亦生效——
      // assert_not_inherited inherit 步钉）。折叠面单点在
      // dom_bindings::anchor_props::computed_value（grid G1 单实现双通道同型，
      // 经 __zw_anchor_computed 回调薄委托）。
      // FIXME: 作者样式表声明臂未接（_zwPlainDecl 只读 inline；语料全 inline）。
      // https://drafts.csswg.org/css-anchor-position-1/
      var _zwAnchorCv = function (p, raw) {
        try { return globalThis.__zw_anchor_computed(p, String(raw)) || ''; }
        catch (_eAC) { return ''; }
      };
      var aRaw = _zwPlainDecl(elt, prop);
      if (!aRaw) return _zwAnchorCv(prop, '');
      var aS = String(aRaw).trim().toLowerCase();
      if (aS === 'inherit') {
        node = elt.parentNode; guard = 0;
        while (node && node.nodeType === 1 && guard++ < 256) {
          var aInh = _zwPlainDecl(node, prop);
          if (aInh) {
            var aInhS = String(aInh).trim().toLowerCase();
            if (aInhS === 'inherit' || aInhS === 'initial' || aInhS === 'unset'
                || aInhS === 'revert' || aInhS === 'revert-layer') {
              node = node.parentNode;
              continue;
            }
            return _zwAnchorCv(prop, aInh);
          }
          node = node.parentNode;
        }
        return _zwAnchorCv(prop, '');
      }
      if (aS === 'initial' || aS === 'unset' || aS === 'revert'
          || aS === 'revert-layer') {
        return _zwAnchorCv(prop, '');
      }
      return _zwAnchorCv(prop, aRaw);
    }
    if (prop === 'position-try') {
      // 组 F：position-try 简写 computed = 两长手 inline 声明合成（order=normal
      // 省略 / 双初始折叠 'none'——join 规则与 dom_bindings::anchor_props::
      // join_position_try 同形，一致性由语料 round-trip 钉）。双长手均无声明返
      // ''（gCS 通道无源空串契约）。css-wide 简写写面拆分时双长手同 keyword
      //（Rust split_position_try css-wide 臂），此处按长手各读。
      var _zwTryJoin = function (o, f) {
        if (o === 'normal' && f === 'none') return 'none';
        if (o === 'normal') return f;
        if (f === 'none') return o + ' none';
        return o + ' ' + f;
      };
      var tO = _zwPlainDecl(elt, 'position-try-order');
      var tF = _zwPlainDecl(elt, 'position-try-fallbacks');
      if (!tO && !tF) return '';
      var oS = String(tO || 'normal').trim().toLowerCase();
      var fS = String(tF || 'none').trim().toLowerCase();
      if (oS === 'initial' || oS === 'unset' || oS === 'revert'
          || oS === 'revert-layer') oS = 'normal';
      else if (oS === 'inherit') oS = 'normal'; // 简写级联近似（语料全 inline 直写）
      if (fS === 'initial' || fS === 'unset' || fS === 'revert'
          || fS === 'revert-layer') fS = 'none';
      else if (fS === 'inherit') fS = 'none';
      return _zwTryJoin(oS, fS);
    }
    if (prop === 'position-area') {
      // wptfyi-css 组F：css-anchor-position-1 §position-area 计算值——非继承
      //（position-area-computed assert_not_inherited 锚），只读自身 inline 声明；
      // 折叠管线（逻辑 long→short 同形塌缩、span-all 槽位消歧 `start span-all` →
      // block-start / `span-all start` → inline-start、center 槽位归位）单点在
      // dom_bindings::position_area::computed_value（grid G1 单实现双通道同型，
      // 经 __zw_position_area_computed 回调薄委托）。无声明返初始值 'none'。
      // CSS-wide keyword 计算期语义（CSS Cascading 4 §6-§7）：initial/unset/
      // revert/revert-layer → 属性初始值 'none'（非继承且 UA 层无 position-area
      // 声明可回退——assert_initial 钉）；inherit 强制取父链首个非 css-wide 声明
      // 的计算值（§7.3 对非继承属性亦生效——assert_not_inherited inherit 步钉）。
      // 回显原文曾撞 host 'initial'/'inherit' 兜底字面量（paprobe 残余 2F 根因）。
      // FIXME: 作者样式表声明臂未接（_zwPlainDecl 只读 inline；语料两页全 inline）。
      var paRaw = _zwPlainDecl(elt, 'position-area');
      if (!paRaw) return 'none';
      var paS = String(paRaw).trim().toLowerCase();
      if (paS === 'inherit') {
        node = elt.parentNode; guard = 0;
        while (node && node.nodeType === 1 && guard++ < 256) {
          var paInh = _zwPlainDecl(node, 'position-area');
          if (paInh) {
            var paInhS = String(paInh).trim().toLowerCase();
            if (paInhS === 'inherit' || paInhS === 'initial' || paInhS === 'unset'
                || paInhS === 'revert' || paInhS === 'revert-layer') {
              node = node.parentNode;
              continue;
            }
            if (typeof globalThis.__zw_position_area_computed === 'function') {
              try {
                var paCV = globalThis.__zw_position_area_computed(String(paInh));
                if (paCV !== '') return paCV;
              } catch (_ePACI) {}
            }
            return '';
          }
          node = node.parentNode;
        }
        return 'none';
      }
      if (paS === 'initial' || paS === 'unset' || paS === 'revert'
          || paS === 'revert-layer') {
        return 'none';
      }
      if (typeof globalThis.__zw_position_area_computed === 'function') {
        try {
          var paV = globalThis.__zw_position_area_computed(String(paRaw));
          if (paV !== '') return paV;
        } catch (_ePAC) {}
      }
      return '';
    }
    if (prop === 'color-scheme') {
      // color-scheme 级联声明读取（css-color-adjust-1 §color-scheme：继承属性）——
      // 供 light-dark()/system color 求值方合成 used scheme（part11
      // _zwCCUsedSchemeDark 消费）。inline → 作者臂（qvMode）→ 沿父链首声明，
      // 声明原文返回（含 normal/only token——token 判定归消费方，与
      // style-system parse_color_scheme_dark 同口径：无 light/dark token 的
      // 声明回落文档 scheme，corpus 页无 meta color-scheme 时保守 light）。
      node = elt; guard = 0;
      while (node && node.nodeType === 1 && guard++ < 256) {
        var csRaw = _zwPlainDecl(node, prop);
        if (!csRaw && qvMode) {
          try { csRaw = _zwPlainAuthorDecl(node, prop); } catch (_eCSAu) { csRaw = ''; }
        }
        if (csRaw) {
          var csS = String(csRaw).replace(/\/\*[\s\S]*?\*\//g, ' ').trim();
          if (csS) return csS;
        }
        node = node.parentNode;
      }
      return '';
    }
    if (prop === 'color' || prop === 'font-family'
        || prop === 'background-color') {
      var isBg = prop === 'background-color';
      node = elt; guard = 0;
      while (node && node.nodeType === 1 && guard++ < 256) {
        var raw = _zwPlainDecl(node, prop);
        if (!raw && qvMode) {
          // 作者样式表规则（③'-c）：作者 normal 声明 > 表现提示 > UA 规则——
          // 插在 inline decl 之后、font 提示臂之前。仅 qvMode（值面通道）开：
          // apply 面 ownFacePre/parFacePre 等价分组消费方对近端作者声明可见性
          // 敏感（ba[r ba] compare 修复前绿、#purple 臂开实测 z/ba 序 scramble；
          // pre/listing fontname compare 同证 bazfoo 跨块并段）——作者臂在
          // apply 面的等价分组语义属 ③'-a/b/d 同窗实施，本刀不动。
          try { raw = _zwPlainAuthorDecl(node, prop); } catch (_ePAAu) { raw = ''; }
        }
        var tagL = String(node.tagName || '').toLowerCase();
        if (!raw && tagL === 'font') {
          try {
            // 表现提示：font[color]/font[face]/font[bgcolor]（HTML 渲染节）。
            var attrName = prop === 'color' ? 'color'
              : (prop === 'font-family' ? 'face' : 'bgcolor');
            var av = node.getAttribute(attrName);
            if (av != null) raw = String(av);
            // legacy 属性色无 '#' hex 形态补 '#'（HTML 渲染节 body 属性映射
            // 的 legacy parse——font color="0000ff" 语料期望 rgb(0, 0, 255)；
            // inline style 同形非法，此处仅属性臂消费面）。
            if (raw && raw.charAt(0) !== '#'
                && (/^[0-9a-fA-F]{6}$/.test(raw) || /^[0-9a-fA-F]{3}$/.test(raw))) {
              raw = '#' + raw;
            }
          } catch (_ePFA) {}
        }
        if (!raw && prop === 'color' && tagL === 'a') {
          // UA stylesheet a:link（HTML 渲染节 15.3.1——`a:link { color: -webkit-link }`，
          // Chrome 计算值 rgb(0, 0, 255)）：仅带 href 属性命中（:link 域）；
          // 无 href 不着色。语料 `<a href>foo[bar]baz</a>` qV forecolor before 实证。
          try {
            if (node.getAttribute && node.getAttribute('href') != null) {
              return 'rgb(0, 0, 255)';
            }
          } catch (_eUAlink) {}
        }
        if (!raw && prop === 'font-family' && _zwPlainUaMono[tagL]) {
          return 'monospace';
        }
        if (raw) {
          if (prop === 'font-family') return raw;
          // DC-2 次批：现代色函数（hsl/rgb 现代空格语法、none 分量、hwb/lab
          // 族/color()/color-mix()/相对色）——`_zwPlainColorSpec` 只认 legacy
          // 形（named/hex/comma rgb·hsla），返空才落 part11 管线（零漂移：
          // legacy 输出口径不变）。无效/不可解析返 ''（同无源路径）。
          // currentcolor 例外前置：旧臂爬升不含作者样式表（gCS 通道 qvMode
          // 不开）读数恒黑；part11 currentcolor 臂带作者臂（WPT computed 页
          // #container{color:...} 锚——named-color 页实证）。
          var rawLow = raw.length > 32 ? '' : raw.toLowerCase();
          var isCC = rawLow.indexOf('/*') < 0 ? rawLow === 'currentcolor' : rawLow.replace(/\/\*[\s\S]*?\*\//g, ' ').trim() === 'currentcolor';
          var cv = '';
          if (!isCC) cv = _zwPlainColorSpec(raw, node);
          if (!cv && typeof globalThis.__zwComputedColor === 'function') {
            // 第三参属性上下文：currentcolor 在 color 属性自身走父链（自引用
            // =继承），background-color 等先试本元素（css-color-4 §5.3）。
            try { cv = globalThis.__zwComputedColor(raw, node, prop); } catch (_eCC) { cv = ''; }
          }
          if (cv) {
            // background-color：透明声明视作无（transparent 爬升等价形——
            // getEffectiveCommandValue 的 while 组合语义不变）。
            if (!isBg || cv !== 'rgba(0, 0, 0, 0)') return cv;
          }
        }
        node = node.parentNode;
      }
      // 无源返空——gCS 通道保 P9-9/R5107 合成源契约（无规则即空）；UA 初值
      // 由值面消费点 __zw_ce_value_query 填（englib p99 外部 CSS 钉测回归定谳）。
      return '';
    }
    return '';
  };

  // ============================================================
  // editing-compat style-with-value 簇（2026-11-01）：document.queryCommand*
  // 值面/Indeterm 面——五标准行内值命令（fontsize/fontname/forecolor/
  // backcolor/hilitecolor）。
  //
  // 归因（tests.js conformance 比较列实证）：before/after qV 断言读数走
  // **document.queryCommandValue**（tests.js:5595/5627 browser 列），非 vendored
  // myQueryCommandValue（后者只服务 spec 列）——引擎面此前恒 ''。算法按参考实现
  // 移植（tests/wpt-runner/wpt-data/editing/include/implementation.js）：
  //   ① effectively contained 文档序首个 formattable node（:1807/:1997），无则
  //     active range startContainer（standardInlineValueCommand value 生成块
  //     :8477-8495）；② getEffectiveCommandValue（:2069）——TEXT 归父、
  //     backcolor/hilitecolor transparent 爬升、其余 gCS relevantCssProperty
  //     （无声明时 UA 初值在此消费点填——gCS 通道保 P9-9 合成源契约返空）；
  //   ③ fontsize → getLegacyFontSize（:3309）中点映射（UA 缺省标度
  //     10/13/16/18/24/32/48px，probe tB/tC 实证）；
  //   ④ Indeterm = 含域 formattable 有效值 ≥2 个互异（:8470 生成块）。
  // 选区经 globalThis.getSelection 单例（同 justify 查询面）。
  // FIXME: 值 override 机制未接——vendored myExecCommand 的 override 写私有闭包
  //   map，引擎面不可见；collapsed 选区（无 contained formattable）apply 后
  //   Chrome 期望值来自其原生 override（DOM 不变仍返新值），我方 DOM 走查返
  //   走查值，该族 qV-after 挂账迭代。
  function _zwValIsEffectivelyContained(node, range) {
    if (range.collapsed) return false;
    if (_zwJusIsContained(node, range)) return true;
    if (node === range.startContainer && node.nodeType === 3
        && _zwJusNodeLength(node) !== range.startOffset) return true;
    if (node === range.endContainer && node.nodeType === 3
        && range.endOffset !== 0) return true;
    if (_zwJusHasKids(node)
        && (! _zwJusIsAncestorOf(node, range.startContainer)
            || range.startContainer.nodeType !== 3
            || range.startOffset === 0)
        && (! _zwJusIsAncestorOf(node, range.endContainer)
            || range.endContainer.nodeType !== 3
            || range.endOffset === _zwJusNodeLength(range.endContainer))) {
      for (var i = 0; i < node.childNodes.length; i++) {
        if (!_zwValIsEffectivelyContained(node.childNodes[i], range)) return false;
      }
      return true;
    }
    return false;
  }
  function _zwValGetAllEffectivelyContained(range, condition) {
    var node = range.startContainer;
    var guard = 0;
    while (node && guard++ < 256 && _zwValIsEffectivelyContained(node.parentNode, range)) {
      node = node.parentNode;
    }
    var stop = _zwJusNextNodeDescendants(range.endContainer);
    var out = [];
    // stop=null 卫兵（同 :399 F6b 先例；2026-11-12 嵌套簇刀 D2 臂）：end 界落
    // 文档链顶（endContainer 为末节点 → NextNodeDescendants 走根返 null）时
    // isBefore(x, null) 恒假 → 走查空转 → contained 集空 → 六 format 命令落
    // override-only 臂（qS override 翻转、DOM 零变更）——嵌套簇两枚 #[ignore]
    // 探针 r4 定谳（nested-probe-r4.log：containedAll=0 + selval-return）。
    // 真页面选区终点即文档末节点为合法形态，迭代至树尽。
    while (node && (!stop || _zwJusIsBefore(node, stop)) && guard++ < 65536) {
      if (_zwValIsEffectivelyContained(node, range) && condition(node)) {
        out.push(node);
      }
      node = _zwJusNextNode(node);
    }
    return out;
  }
  // formattable 可见面 Chrome delta：空白-only 文本节点的 collapsed 判定补前向
  // 走查（vendored isCollapsedWhitespaceNode :912-1001 双向；_zwJusCollapsed-
  // WhitespaceNode 结构化重写只保留后向——`<p>[foo</p> <p>bar]</p>` 族
  // indeterm-after 期望 false 实证裸空白不进 formattable 集）。仅 formattable
  // 消费面取双向语义；结构面（block boundary、follows/precedesLineBreak）维持
  // 既有单向语义不动（wss/ilb/join 基线锁定）。
  // https://w3c.github.io/editing/execCommand.html#collapsed-whitespace-node
  function _zwJusFormattableVisible(node) {
    if (node.nodeType !== 3) return _zwJusIsVisible(node);
    if (_zwJusIsCollapsedWhitespaceNode(node)) return false;
    if (!_zwJusIsWhitespaceNode(node)) return true;
    var ancestor = node.parentNode;
    while (ancestor && !_zwJusIsBlockNode(ancestor) && ancestor.parentNode) {
      ancestor = ancestor.parentNode;
    }
    if (!ancestor) return false;
    var stop = _zwJusNextNodeDescendants(ancestor);
    var reference = node;
    var guard = 0;
    while (reference !== stop && guard++ < 4096) {
      reference = _zwJusNextNode(reference);
      if (!reference) break;
      if (_zwJusIsBlockNode(reference) || _zwJusIsHtmlElement(reference, 'br')) return false;
      if ((reference.nodeType === 3 && !_zwJusIsWhitespaceNode(reference))
          || _zwJusIsHtmlElement(reference, 'img')) break;
    }
    return true;
  }
  function _zwValIsFormattable(node) {
    // OPTIMIZATION：类型门槛前置（纯合取词换序，真值不变）——qV 走查逐节点
    // 调用，绝大多数节点（span/div/font…）非 text/img/br，先短路免掉
    // FormattableVisible → isVisible 的全子树下行；重排前 multitest 大树
    // 每查询同步开销爆炸（30 分钟 0 行出账，br extraneous 断环后走查不再
    // 被 RangeError 快速中止，全量走完暴露了这层成本）。
    if (!(node.nodeType === 3 || _zwJusIsHtmlElement(node, ['img', 'br']))) {
      return false;
    }
    return _zwJusIsEditable(node) && _zwJusFormattableVisible(node);
  }
  // qI 普查面（Chrome delta，④ fontsize qI-after 收口）：extraneous br 不作
  // 可见性过滤——Chrome qI 普查含选区内未包裹 br（fontsize 语料
  // `<p>[foo<p><br><p>bar]` 双变体记录 qI after=true：br 继承缺省值与包裹
  // span 混合；vendored isFormattableNode 的 isVisible 冗余 br 过滤与 Chrome
  // 录音相悖，语料权威）。qV 值面口径保持 _zwValIsFormattable 过滤不动。
  // br 短路免掉 FormattableVisible 下行（trap 域成本同 _zwValIsFormattable 门）。
  // https://w3c.github.io/editing/execCommand.html#indeterminate
  function _zwValIsFormattableQI(node) {
    if (_zwJusIsHtmlElement(node, 'br')) return _zwJusIsEditable(node);
    return _zwValIsFormattable(node);
  }
  // getLegacyFontSize 中点表（Chrome UA 缺省标度下界 px；上界 = 下位 +1 档）。
  var _zwValLegacyBounds = [10, 13, 16, 18, 24, 32];
  function _zwValLegacyFontSize(pxStr) {
    var m = /^([0-9]+(\.[0-9]+)?)(px)?$/.exec(String(pxStr == null ? '' : pxStr).trim());
    if (!m) return '';
    var px = parseFloat(m[1]);
    for (var i = 0; i < 6; i++) {
      // 中点 = (下界[i] + 下界[i+1]) / 2；下界[7] = 48。
      var next = i < 5 ? _zwValLegacyBounds[i + 1] : 48;
      if (px < (_zwValLegacyBounds[i] + next) / 2) return String(i + 1);
    }
    return '7';
  }
  function _zwValEffectiveValue(node, cmd) {
    if (!node) return null;
    if (node.nodeType !== 1) {
      var p = node.parentNode;
      if (!p || p.nodeType !== 1) return null;
      node = p;
    }
    if (cmd === 'backcolor' || cmd === 'hilitecolor') {
      // transparent 爬升（参考实现 :2107-2120）；本仓 backgroundColor 解析器
      // 已含等价爬升，此处保留双保险（空串继续上行）。
      var g = 0;
      while (node && node.nodeType === 1 && g++ < 256) {
        var bg = _zwJusCs(node, 'background-color');
        if (bg && bg !== 'rgba(0, 0, 0, 0)' && bg !== 'transparent') return bg;
        node = node.parentNode;
      }
      return 'rgba(0, 0, 0, 0)';
    }
    if (cmd === 'bold' || cmd === 'italic' || cmd === 'underline'
        || cmd === 'strikethrough' || cmd === 'subscript' || cmd === 'superscript') {
      // DC-2 P2 apply 面（2026-11-11）：六 inline format 命令 effective 值
      //（vendored getEffectiveCommandValue :2069 同构——sub/sup inline 链、
      // decoration 祖先走查、weight/style resolved 读）经 P1 读面载体
      // _zwFmtEffectiveValue（UA 元素表补 host gCS 盲区，h1-h6/th 已入表）。
      // 无源填 CSS 初值：bold 400 / italic normal（vendored gCS 恒返——等值
      // 门依赖 400≡normal 早退，置 '' 会对裸文本误发 span）；decoration 无线
      // 返 null、sub/sup 无标签返 null（同 vendored null 契约）。
      var fv = _zwFmtEffectiveValue(cmd, node);
      if (fv == null || fv === '') {
        if (cmd === 'bold') return '400';
        if (cmd === 'italic') return 'normal';
        return null;
      }
      return String(fv);
    }
    var prop = cmd === 'fontsize' ? 'font-size'
      : (cmd === 'fontname' ? 'font-family' : 'color');
    var v;
    if (cmd === 'fontname' || cmd === 'forecolor') {
      // plain 走查优先（2026-11-05 ③'-c/⑤）：host gCS 桥对 <link> 作者样式表
      // 与 legacy font face 提示均盲（forecolor #purple qV-before got 黑 /
      // fontname pre 族 qV-after got serif 实证）；plain 走查含 inline、作者
      // 规则、font 提示、UA 臂，即完整近端级联。host 读数回落保留（P9-9
      // 通道口径），fontsize 档位面维持 host 优先不动（getLegacyFontSize
      // 消费 host 16px 缺省口径）。
      try { v = __zwPlainComputed(node, prop, true); } catch (_eEvPV) { v = ''; }
      if (!v) v = _zwJusCs(node, prop);
    } else {
      v = _zwJusCs(node, prop);
    }
    if (cmd === 'fontsize' && (v === '' || v === '16px')) {
      // host gCS 通道无 UA stylesheet 元素规则（big/small——HTML 渲染节
      // 15.3.1），big/small 子树读数恒 16px 缺省 → forceValue 等值早退跳包
      // （big/small 语料期望内层包裹实证）+ qV 档位错档（small 得 3 期望
      // 2）。host 读数为空或缺省时改 plain 走查补 UA 面（plain 同时见声明/
      // 表现提示/UA 规则；host 非 16px 可分辨值优先保 P9-9 通道口径）。
      var pv = '';
      try { pv = __zwPlainComputed(node, 'font-size'); } catch (_eEvP) {}
      if (pv && pv !== '16px') return pv;
    }
    if (v === '') {
      // 无声明时 UA 初值（Chrome computed 口径，probe tD/tE 与语料裸行期望）——
      // gCS 通道保 P9-9 合成源契约返空，初值在此值面消费点填；fontsize 16px
      // 过 legacy 中点映射落 '3'（medium）。
      v = cmd === 'fontname' ? 'serif'
        : (cmd === 'forecolor' ? 'rgb(0, 0, 0)' : '16px');
    }
    // F6b r2（2026-11-09 r1 批 ABORT 定谳）：sub/sup UA 0.83em 缩放在 host gCS
    // 与 plain 走查双通道盲读（style-system UA 表 sub,sup{font-size:0.83em}
    // shim 侧不可见）——C1/C2 等桶溶解位 qV("fontsize") 回落缺省 16px 过中点
    // 映射得 '3'，语料/Chrome computed 语义 16×0.83=13.28px→'2'。仅缺省回落位
    // 且**无显式 font[size] 载体**时缩放：载体在场（C3/C4 `<font size=3><sub>`
    // /A/N 族）的 16px 是 attr 派生真值（legacy '3'），盲读同串不同源，近端
    // 不可分辨故按载体存在性分派；作者 px 声明路径（plain 非 16px 直返）不触。
    // r1 全量 P2F 矩阵实证影响面恰 C1/C2 四断言行（C3/C4 载体读数、N 族无
    // fontsize qV 断言均不受扰）。
    // https://w3c.github.io/editing/execCommand.html#value-overrides-and-query
    if (cmd === 'fontsize' && v === '16px') {
      var scN = node;
      var gS = 0;
      var hasCarrier = false;
      var inScript = false;
      while (scN && scN.nodeType === 1 && gS++ < 256) {
        var tnS = String(scN.tagName || '').toLowerCase();
        if (tnS === 'sub' || tnS === 'sup') inScript = true;
        if (tnS === 'font' && scN.hasAttribute && scN.hasAttribute('size')) hasCarrier = true;
        if (typeof _zwJusIsEditingHost === 'function' && _zwJusIsEditingHost(scN)) break;
        scN = scN.parentNode;
      }
      if (inScript && !hasCarrier) v = (16 * 0.83) + 'px';
    }
    return v;
  }
  // 查询入口：返 { value, indeterm }（命中五命令）或 null（调用方回落）。
  globalThis.__zw_ce_value_query = function (cmd) {
    var five = { fontsize: 1, fontname: 1, forecolor: 1, backcolor: 1, hilitecolor: 1 };
    if (!five[String(cmd)]) return null;
    // 值 override（part01d，2026-11-01 style-with-value 簇实应用轮）：collapsed
    // 选区 execCommand 记账、选区变动即作废（reset 在读数侧）。fontsize 的
    // legacy 映射在导出侧完成（part01d _zwVaLegacyValueOf 两段式——part01c
    // _zwValLegacyFontSize 只吃 px 串，关键字 override 会错转），此处直返
    //（vendored myQueryCommandValue :655-663 同序）。
    try {
      if (typeof globalThis.__zw_ce_value_override_value === 'function') {
        var ov = globalThis.__zw_ce_value_override_value(String(cmd));
        if (ov !== undefined) {
          return { value: ov == null ? '' : String(ov), indeterm: false };
        }
      }
    } catch (_eOvQ) {}
    try {
      // 选区单例经 globalThis.getSelection（_getSelection 声明域不覆盖本文件
      // 挂载点——同 justify 查询面取区惯用法）。
      var sel = globalThis.getSelection ? globalThis.getSelection() : null;
      var rng = sel && sel._ranges && sel._ranges.length > 0 ? sel._ranges[0] : null;
      if (!rng) return { value: '', indeterm: false };
      // qI 普查独立于 qV 值面（单遍走查，④ 收口）：普查用 _zwValIsFormattableQI
      // （含 extraneous br）；qV ref = 普查序首个通过 qV 过滤的节点——两走查
      // 同文档序且 qV 过滤集 ⊆ 普查集（仅差 extraneous br），首过滤节点与旧
      // 全量过滤走查 nodes[0] 恒等（qV 首节点口径零漂移）。
      var nodes = _zwValGetAllEffectivelyContained(rng, _zwValIsFormattableQI);
      var indeterm = false;
      if (nodes.length > 1) {
        var first = _zwValEffectiveValue(nodes[0], cmd);
        for (var i = 1; i < nodes.length; i++) {
          if (_zwValEffectiveValue(nodes[i], cmd) !== first) { indeterm = true; break; }
        }
      }
      var ref = null;
      for (var i = 0; i < nodes.length; i++) {
        if (_zwValIsFormattable(nodes[i])) { ref = nodes[i]; break; }
      }
      if (!ref) ref = rng.startContainer;
      var v = _zwValEffectiveValue(ref, cmd);
      if (cmd === 'fontsize') v = _zwValLegacyFontSize(v);
      return { value: v == null ? '' : String(v), indeterm: indeterm };
    } catch (_eVQ) {
      return { value: '', indeterm: false };
    }
  };

  // ============================================================
  // editing-compat format state 簇（2026-11-11）：六 inline format 命令
  //（bold/italic/underline/strikethrough/subscript/superscript）的
  // queryCommandState/Indeterm 读面（wptfyi-editing DC-2 P1 共享路径）。
  //
  // 归因（DC-2 triage：er8/er9 纯 state 失败 632 例与 innerHTML 失败零共现
  // ——DOM 已对、读出错了）：此前 queryCommandState = 选区起点祖先链
  // tagName 匹配（part06 R3254-M3 切片 4 近似），与 spec state 语义三处
  // 差距即主因：① computed style 判定缺失（style span/元素语义族读不出，
  // u/s 跨样式族方向性误报）；② 判定域错位（spec = 含域 formattable 全量
  // 命中，非起点单点——跨段落/ce=false 混合选区族）；③ Indeterm 恒 false
  //（混合选区族）。
  //
  // 算法按 spec 移植（https://w3c.github.io/editing/docs/execCommand/）：
  //   - state（inline command activated values）：含域（effectively
  //     contained）formattable node 全命中 activated values；无 contained
  //     → active range start node 读法。
  //   - Indeterm：四 format 命令 part-not-all；sub/sup = ≥1 target 值且
  //     ≥1 他值、或任一 "mixed"。
  //   - effective command value：Text 归父；bold/italic = relevant CSS
  //     property 解析值（font-weight/font-style，等价词 bold≡700）；
  //     underline/strikethrough = node 或任一祖先 text-decoration 解析值
  //     含 underline/line-through（非继承属性，逐元素 own 值）；
  //     subscript/superscript = inline 祖先链 sub/sup 标签计数
  //     （both→"mixed"；Gecko 语义定谳，非 vertical-align）。
  //   - activated values：bold {bold,600,700,800,900}（600 cutoff，Chrome
  //     口径）；italic {italic,oblique}；underline {underline}；
  //     strikethrough {line-through}；subscript {subscript}；
  //     superscript {superscript}。
  // CSS 解析与值面同构（host gCS 对 plain 元素 miss，probe t16）：host
  // 优先、miss 落 JS 走查——inline style → UA 元素表（HTML 渲染节 15.3.1：
  // b,strong bold / i,em,cite,dfn,var,address italic / u,ins underline /
  // s,strike,del line-through；a[href] 同 a:link underline 臂先例）→ 继承
  // 走查（font-weight/font-style 继承）。
  // FIXME: 作者样式表规则臂 u/s decoration 面已接（2026-11-11
  //   _zwFmtDecorationOf——reset.css 类选择器载体 state 读），bold/italic
  //   值面未接；h1-h6/th 加粗 UA 面未建；font 简写（font: bold …）与
  //   bolder/lighter 相对关键字未解析。
  // state override（2026-11-11 P3）：写面 part01d _zwVaSetSelectionValue 无
  //   formattable 臂（__zw_ce_value_state_override 读数导出）；本面入口
  //   override 先行（读序 ⓪），collapsed exec typing 态另有 part06
  //   __zwTypingTags 兜底（读序 override 之后）。
  var _zwFmtUaWeight = { b: '700', strong: '700',
    h1: '700', h2: '700', h3: '700', h4: '700', h5: '700', h6: '700', th: '700' };
  var _zwFmtUaStyle = { i: 'italic', em: 'italic', cite: 'italic',
    dfn: 'italic', 'var': 'italic', address: 'italic' };
  // text-decoration 逐元素 own 值表（非继承；HTML 渲染节 15.3.1）。
  function _zwFmtUaDecoration(tagL, hasHref) {
    if (tagL === 'u' || tagL === 'ins') return 'underline';
    if (tagL === 's' || tagL === 'strike' || tagL === 'del') return 'line-through';
    if (tagL === 'a' && hasHref) return 'underline';
    return '';
  }
  // 继承链 resolved prop：inline style → 作者表 → host gCS → UA 元素表 →
  // 父链上行（font-weight/font-style 继承属性；无源返空，不填初值——state
  // 比较面自然 false，同值面无源返空契约）。own 声明臂在 host 读前的级联
  // 依据：继承属性下元素自身声明（inline/作者/UA）恒胜继承值，而 host gCS
  // 读数混入父链继承（`.notbold{font-weight:normal}` 载体在 <b> 内 host 读
  // bold 继承致 state 读 true 误判，bold.js notbold 簇 14F 实证），故
  // inline/作者先行；与 _zwFmtDecorationOf 的 host 先行序不矛盾——
  // text-decoration 非继承，host 读仅 own 值无继承污染。
  // 作者臂（2026-11-12 wptfyi-editing DC-2 bold 值面批）：host gCS 对
  // <link> 作者样式表盲、inline/UA 表不含类选择器声明——复用 ③'-c 作者
  // 快照匹配机具（同款有界简单选择器域），级联序作者 > UA、inline > 作者。
  function _zwFmtResolvedProp(elt, camel, kebab, uaTab) {
    var node = elt;
    var guard = 0;
    while (node && node.nodeType === 1 && guard++ < 256) {
      var d = '';
      try { d = _zwPlainDecl(node, kebab) || ''; } catch (_eFpd) { d = ''; }
      if (!d) {
        try { d = _zwPlainAuthorDecl(node, kebab) || ''; } catch (_eFpa) { d = ''; }
      }
      if (d) return String(d);
      var v = '';
      try { v = _zwJusCs(node, camel); } catch (_eFcs) { v = ''; }
      if (v) return String(v);
      if (uaTab) {
        var tagL = String(node.tagName || '').toLowerCase();
        d = uaTab[tagL] || '';
      }
      if (d) return String(d);
      node = node.parentNode;
    }
    return '';
  }
  // 单元素 text-decoration own 值（host → inline → 作者表 → UA 表；非继承
  // 不爬升）。作者臂（2026-11-11 wptfyi-editing DC-2 ①）：host gCS 对
  // <link> 作者样式表盲、inline/UA 表不含类选择器声明——reset.css
  // `.line-through/.underline` 载体 span state 读 false 误应用外裹（strike/
  // underline 语料 8 P2F 实证）。复用 ③'-c 作者快照匹配机具（同款有界简单
  // 选择器域）；级联序作者 > UA、inline > 作者，插在 inline 后 UA 前。
  function _zwFmtDecorationOf(el) {
    var v = '';
    try { v = _zwJusCs(el, 'textDecoration'); } catch (_eFtd) { v = ''; }
    if (v) return String(v);
    var d = '';
    try { d = _zwPlainDecl(el, 'text-decoration') || ''; } catch (_eFtd2) { d = ''; }
    if (d) return d;
    // 长手回退：text-decoration 与 text-decoration-line 同一逻辑属性
    //（Chrome computed 由 line/style/color 合成；Site A 溶解换形只写长手
    // 形，strikethrough.js:492/:532 录形），state 读不得因只写长手而盲。
    var dL = '';
    try { dL = _zwPlainDecl(el, 'text-decoration-line') || ''; } catch (_eFtdL) { dL = ''; }
    if (dL) return dL;
    var dA = '';
    try { dA = _zwPlainAuthorDecl(el, 'text-decoration') || ''; } catch (_eFtdA) { dA = ''; }
    if (dA) return dA;
    var tagL = String(el.tagName || '').toLowerCase();
    var href = null;
    try { href = el.getAttribute && el.getAttribute('href'); } catch (_eFhr) {}
    return _zwFmtUaDecoration(tagL, href != null);
  }
  // effective command value（六命令；返回 string 或 null）。
  function _zwFmtEffectiveValue(cmd, node) {
    if (!node) return null;
    if (node.nodeType !== 1) {
      var p = node.parentNode;
      if (!p || p.nodeType !== 1) return null;
      node = p;
    }
    if (cmd === 'subscript' || cmd === 'superscript') {
      var gS = 0;
      var subHit = false;
      var supHit = false;
      var nS = node;
      while (nS && _zwJusIsInlineNode(nS) && gS++ < 256) {
        var tnS = String(nS.tagName || '').toLowerCase();
        if (tnS === 'sub') subHit = true;
        else if (tnS === 'sup') supHit = true;
        nS = nS.parentNode;
      }
      if (subHit && supHit) return 'mixed';
      if (subHit) return 'subscript';
      if (supHit) return 'superscript';
      return null;
    }
    if (cmd === 'underline' || cmd === 'strikethrough') {
      var want = cmd === 'underline' ? 'underline' : 'line-through';
      var gD = 0;
      var nD = node;
      while (nD && nD.nodeType === 1 && gD++ < 256) {
        var dv = _zwFmtDecorationOf(nD);
        if (dv && String(dv).toLowerCase().indexOf(want) >= 0) return want;
        nD = nD.parentNode;
      }
      return null;
    }
    var isBold = cmd === 'bold';
    var v = _zwFmtResolvedProp(node,
      isBold ? 'fontWeight' : 'fontStyle',
      isBold ? 'font-weight' : 'font-style',
      isBold ? _zwFmtUaWeight : _zwFmtUaStyle);
    if (!v) return null;
    return String(v).toLowerCase();
  }
  function _zwFmtActivated(cmd, v) {
    if (v == null) return false;
    if (cmd === 'bold') {
      return ['bold', '600', '700', '800', '900']
        .indexOf(String(v).toLowerCase()) >= 0;
    }
    if (cmd === 'italic') return v === 'italic' || v === 'oblique';
    return v === (cmd === 'underline' ? 'underline'
      : cmd === 'strikethrough' ? 'line-through'
      : cmd === 'subscript' ? 'subscript' : 'superscript');
  }
  // 激活值判定导出（part01d state override 写面消费）——vendored
  // setStateOverride 的 inlineCommandActivatedValues 命中判定（:2974-2976），
  // 激活值表单点复用（bold 600 cutoff Chrome delta 同此）。
  globalThis.__zw_ce_format_activated = _zwFmtActivated;
  // 查询入口：返 { state, indeterm }（命中六命令）或 null（调用方回落）。
  // formattable 普查复用值面 _zwValGetAllEffectivelyContained +
  // _zwValIsFormattableQI（含 extraneous br 的 Chrome delta 口径）。
  globalThis.__zw_ce_format_query = function (cmd) {
    var c = String(cmd == null ? '' : cmd).toLowerCase();
    var six = { bold: 1, italic: 1, underline: 1, strikethrough: 1,
      subscript: 1, superscript: 1 };
    if (!six[c]) return null;
    // spec myQueryCommandState（vendored :609-621）读序：state override 已设
    // → state 直返 override（写面 part01d _zwVaSetSelectionValue 无
    // formattable 臂）；indeterm 不走 override（vendored
    // myQueryCommandIndeterm :593-605 纯 DOM）——qI 与 qS 同消费本入口，故
    // override 只覆盖返回值 .state，不得提前 return（swcss 跨 span
    // qI-after 两例回归实证，2026-11-11）。
    var ovStateQ = undefined;
    try {
      ovStateQ = (typeof globalThis.__zw_ce_value_state_override === 'function')
        ? globalThis.__zw_ce_value_state_override(c) : undefined;
    } catch (_eFqOv) {}
    try {
      var sel = globalThis.getSelection ? globalThis.getSelection() : null;
      var rng = sel && sel._ranges && sel._ranges.length > 0 ? sel._ranges[0] : null;
      if (!rng) {
        return { state: ovStateQ !== undefined ? !!ovStateQ : false,
          indeterm: false };
      }
      var isSubSup = c === 'subscript' || c === 'superscript';
      var nodes = _zwValGetAllEffectivelyContained(rng, _zwValIsFormattableQI);
      var hit = 0;
      var tgt = 0;
      var mixed = false;
      var i;
      for (i = 0; i < nodes.length; i++) {
        var v = _zwFmtEffectiveValue(c, nodes[i]);
        if (_zwFmtActivated(c, v)) hit++;
        if (isSubSup) {
          if (v === (c === 'subscript' ? 'subscript' : 'superscript')) tgt++;
          else if (v === 'mixed') mixed = true;
        }
      }
      var state = nodes.length
        ? hit === nodes.length
        : _zwFmtActivated(c, _zwFmtEffectiveValue(c, rng.startContainer));
      var indeterm = isSubSup
        ? (tgt > 0 && tgt < nodes.length) || mixed
        : hit > 0 && hit < nodes.length;
      if (ovStateQ !== undefined) state = !!ovStateQ;
      return { state: state, indeterm: indeterm };
    } catch (_eFq) {
      return null;
    }
  };
