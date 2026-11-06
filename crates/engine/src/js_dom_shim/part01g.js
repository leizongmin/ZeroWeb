// part01g（2026-11-07 editing-compat F2 后续：四命令零应用族 apply 面）：
// formatBlock / insertHorizontalRule / insertImage / indent 的 action 忠实移植
// + formatBlock value 读面。此前四命令仅 _zwExecCmdInputType 有名（事件照派、
// execCommand 返 true、supported/enabled=true），apply 面缺失 → DOM 零变更
//（multitest 837F 普查 Family A 241 行 + formatblock/indent/inserthorizontalrule/
// insertimage 四独立语料页同源；formatblock qV-after 48 行 exp 'div' got '' 属
// 同族下游——apply 缺失致包裹 div 未物化）。挂 part01f 后同闭包复用 _zwJus*（
// blockExtend/wrap/contained/位置机具）、_zwList*（splitParent/fixDisallowed
// Ancestors/normalizeSublists/indentation 判据）、_zwDel*（deleteSelection/
// recordValues/restoreValues）、_zwIpara*（single-line container 判据）、
// _zwVa*（active range）；part06 经 globalThis.__zw_ce_fb_apply/__zw_ce_fb_value/
// __zw_ce_indent_apply/__zw_ce_hr_apply/__zw_ce_img_apply 单点调用。
// vendored removeAllRanges/addRange 选区重绑舞蹈在本仓是无效面（part01d
// _zwVaSetSelectionValue 先例：sel._ranges[0] 即 active range 本体，重绑丢端点
// → 零变异），故对 range 原位写、收尾用 sel.collapse。
// https://w3c.github.io/editing/execCommand.html#the-formatblock-command
// https://w3c.github.io/editing/execCommand.html#the-inserthorizontalrule-command
// https://w3c.github.io/editing/execCommand.html#the-insertimage-command
// https://w3c.github.io/editing/execCommand.html#the-indent-command
// https://w3c.github.io/editing/execCommand.html#indent-nodes
// https://w3c.github.io/editing/execCommand.html#prohibited-paragraph-child

  // "A formattable block name is "address", "dd", "div", "dt", "h1", ...,
  // "h6", "p", or "pre"."
  // Chrome delta（宿主豁免同族口径）：真实 Blink 名单另收 8 个
  // sectioning/blockquote 名——语料录音 `<article>/<aside>/<blockquote>/<footer>/
  // <header>/<hgroup>/<nav>/<section>` exec ret=true 且树真 wrap（compare 行
  // 刀前全红＝录音树已变），而 `<del>/<dl>/<ins>/<li>/<ol>/<ul>/<quasit>` 录音
  // ret=[false] 维持拒。spec 白名单（implementation.js :6577-6580）不含这 8 名。
  var _zwFbqFormattableNames = ['address', 'article', 'aside', 'blockquote',
      'dd', 'div', 'dt', 'footer', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
      'header', 'hgroup', 'nav', 'p', 'pre', 'section'];

  // "A non-list single-line container is an HTML element whose local name is
  // one of ..."（全版 single-line container 另含 li/dt/dd，用 _zwIpara 判据）。
  function _zwFbqIsNonListSingleLineContainer(node) {
    return _zwJusIsHtmlElement(node, ['address', 'div', 'h1', 'h2', 'h3',
        'h4', 'h5', 'h6', 'listing', 'p', 'pre', 'xmp']);
  }

  // node 的后代中存在 prohibited paragraph child（getDescendants 不含自身；
  // 手爬 childNodes 栈，spec getDescendants(node).some(isProhibitedParagraphChild)）。
  function _zwFbqHasProhibitedParagraphChildDescendant(node) {
    var stack = [];
    var i;
    for (i = 0; node.childNodes && i < node.childNodes.length; i++) {
      stack.push(node.childNodes[i]);
    }
    while (stack.length) {
      var n = stack.pop();
      if (_zwListIsProhibitedParagraphChild(n)) return true;
      for (i = 0; n.childNodes && i < n.childNodes.length; i++) {
        stack.push(n.childNodes[i]);
      }
    }
    return false;
  }

  // start/end 两边界点的最低公共祖先（indent action 的
  // activeRange.commonAncestorContainer 面；本仓 Range 无该 getter）。
  function _zwFbqCommonAncestor(a, b) {
    var n = a;
    var g = 0;
    while (n && g++ < 256) {
      if (_zwJusIsAncestorOf(n, b)) return n;
      n = n.parentNode;
    }
    return null;
  }

  // formatBlock 值归一 + 白名单判定（action 与前置校验共用）："<div>" 剥壳 +
  // ASCII 小写 + formattableBlockNames 白名单。前置校验在事件前拒绝（part06
  // value_validate 臂同款 Chrome 口径：零事件、execCommand 返 false——语料
  // `<del>/<dl>/<ins>/<li>/<ol>/<ul>/<quasit>` 变体 exec return [false] + 树不变
  // 录音实证）。
  function _zwFbqNormalizeBlockValue(value) {
    var v = String(value == null ? '' : value);
    if (/^<.*>$/.test(v)) v = v.slice(1, -1);
    return v.toLowerCase();
  }

  globalThis.__zw_ce_fb_value_valid = function (value) {
    return _zwFbqFormattableNames.indexOf(_zwFbqNormalizeBlockValue(value)) !== -1;
  };

  // insertImage 值前置校验："If value is the empty string, return false."（语料
  // `foo[bar]baz` + [""] 变体 exec return [false] + 树不变录音实证）。
  globalThis.__zw_ce_img_value_valid = function (value) {
    return String(value == null ? '' : value) !== '';
  };

  // formatBlock action。https://w3c.github.io/editing/execCommand.html#
  // the-formatblock-command （vendored implementation.js :6580-6698）。
  globalThis.__zw_ce_fb_apply = function (value) {
    // "If value begins with a "<" character and ends with a ">" character,
    // remove the first and last characters from it." / "Let value be converted
    // to ASCII lowercase." / "If value is not a formattable block name, return
    // false."（非法值已被 part06 前置校验挡在事件前，此处为直接调用防御。）
    var v = _zwFbqNormalizeBlockValue(value);
    if (_zwFbqFormattableNames.indexOf(v) === -1) return false;

    var range = _zwVaGetActiveRange();
    if (!range) return true;

    // "Block-extend the active range, and let new range be the result."
    var newRange = _zwJusBlockExtend(range, false);
    throw new Error('FBQ-STAGE-1');

    // "For each node node contained in new range, append node to node list
    // if it is editable, ... node is either a non-list single-line container
    // or an allowed child of "p" or a dd or dt, and node is not the ancestor
    // of a prohibited paragraph child."
    var nodeList = _zwJusGetContainedNodes(newRange, function (node) {
      return _zwJusIsEditable(node)
        && (_zwFbqIsNonListSingleLineContainer(node)
            || _zwJusIsAllowedChild(node, 'p')
            || _zwJusIsHtmlElement(node, ['dd', 'dt']))
        && !_zwFbqHasProhibitedParagraphChildDescendant(node);
    });
    throw new Error('FBQ-STAGE-2 nodes=' + nodeList.length);

    // "Record the values of node list, and let values be the result."
    var values = _zwDelRecordValues(nodeList);

    // "For each node in node list, while node is the descendant of an
    // editable HTML element in the same editing host, whose local name is a
    // formattable block name, and which is not the ancestor of a prohibited
    // paragraph child, split the parent of the one-node list consisting of
    // node."
    // Chrome delta（宿主豁免）：spec 判据含宿主自身（getEditingHostOf(host)=host
    // → inSameEditingHost 真 → 裸宿主 text 的 formattable 祖先即宿主 → 拆宿主成
    // 双层 contenteditable）。录音铁证：multitest `"foo[]bar"` + formatblock div
    // exp `<div>foo[]bar</div>` 单层（宿主内直接 wrap 物化）——Chrome 不拆宿主，
    // 祖先爬升跳过 editing host。
    var i;
    for (i = 0; i < nodeList.length; i++) {
      var node = nodeList[i];
      var keep = true;
      while (keep) {
        keep = false;
        var anc = node.parentNode;
        var g = 0;
        while (anc && g++ < 256) {
          if (!_zwJusIsEditingHost(anc)
              && _zwJusIsEditable(anc)
              && _zwListInSameEditingHost(anc, node)
              && _zwJusIsHtmlElement(anc, _zwFbqFormattableNames)
              && !_zwFbqHasProhibitedParagraphChildDescendant(anc)) {
            var prevP = node.parentNode;
            _zwListSplitParent([node]);
            // 进度防御：splitParent plain 域拷贝 no-op（身份不迁）即停，防
            // while 重拆同一祖先克隆循环（同 part01e 允许子臂防御，fbq 轮）。
            if (node.parentNode !== prevP) keep = true;
            break;
          }
          anc = anc.parentNode;
        }
      }
    }

    // "Restore the values from values."
    _zwDelRestoreValues(values);
    throw new Error('FBQ-STAGE-3');

    // "While node list is not empty: ..."（single-line container 解包臂 ∨
    // 连续 sibling 聚 sublist 后 wrap）。
    while (nodeList.length) {
      var sublist;
      // "If the first member of node list is a single-line container: ..."
      if (_zwIparaIsSingleLineContainer(nodeList[0])) {
        sublist = Array.prototype.slice.call(nodeList[0].childNodes);
        var vals2 = _zwDelRecordValues(sublist);
        // "Remove the first member of node list from its parent, preserving
        // its descendants."
        _zwJusRemovePreservingDescendants(nodeList[0]);
        _zwDelRestoreValues(vals2);
        nodeList.shift();
      } else {
        sublist = [nodeList.shift()];
        // "While node list is not empty, and the first member of node list
        // is the nextSibling of the last member of sublist, and the first
        // member of node list is not a single-line container, and the last
        // member of sublist is not a br, ..."
        while (nodeList.length
            && nodeList[0] === sublist[sublist.length - 1].nextSibling
            && !_zwIparaIsSingleLineContainer(nodeList[0])
            && !_zwJusIsHtmlElement(sublist[sublist.length - 1], 'br')) {
          sublist.push(nodeList.shift());
        }
      }
      // "Wrap sublist. If value is "div" or "p", sibling criteria returns
      // false; otherwise it returns true for an HTML element with local name
      // value and no attributes, and false otherwise. ... Then fix disallowed
      // ancestors of the result."
      var sibCrit = (['div', 'p'].indexOf(v) !== -1)
        ? function () { return false; }
        : function (n) {
            return _zwJusIsHtmlElement(n, v)
              && n.attributes && n.attributes.length === 0;
          };
      _zwListFixDisallowedAncestors(_zwJusWrap(sublist, sibCrit,
        function () { return document.createElement(v); }));
    }
    return true;
  };

  // formatBlock value 读面（vendored implementation.js :6773-6805）：block-extend
  // 后首个可见无子 editable 叶子向上爬至首个 formattable ancestor 的子位，终判
  // 该节点本身。语料形判定与 Chrome 录音一致：裸宿主 caret（"foo[]bar"）→ 叶子
  // 爬升停在 host（formattable）→ 叶子非元素 → ""；apply 物化包裹 div 后 → "div"。
  // https://w3c.github.io/editing/execCommand.html#the-formatblock-command
  globalThis.__zw_ce_fb_value = function () {
    var range = _zwVaGetActiveRange();
    if (!range) return '';
    var newRange = _zwJusBlockExtend(range, false);
    // "Let node be the first visible editable node that is contained in new
    // range and has no children."
    var nodes = _zwJusCollectContainedNodes(newRange, function (node) {
      return _zwJusIsVisible(node)
        && _zwJusIsEditable(node)
        && !_zwJusHasKids(node);
    }, false);
    if (!nodes.length) return '';
    var node = nodes[0];
    // "While node's parent is editable and in the same editing host as node,
    // and node is not an HTML element whose local name is a formattable
    // block name, set node to its parent."
    var g = 0;
    while (node.parentNode && g++ < 256
        && _zwJusIsEditable(node.parentNode)
        && _zwListInSameEditingHost(node, node.parentNode)
        && !_zwJusIsHtmlElement(node, _zwFbqFormattableNames)) {
      node = node.parentNode;
    }
    // "If node is an editable HTML element whose local name is a formattable
    // block name, and node is not the ancestor of a prohibited paragraph
    // child, return node's local name, converted to ASCII lowercase."
    if (_zwJusIsEditable(node)
        && _zwJusIsHtmlElement(node, _zwFbqFormattableNames)
        && !_zwFbqHasProhibitedParagraphChildDescendant(node)) {
      return _zwJusTag(node);
    }
    return '';
  };

  // indentNodes（vendored implementation.js :5482-5517）。ol/ul 父臂同型
  // tag-wrap ∨ blockquote 包裹（simple indentation element 判据合并既有壳）。
  // CSS-mode（styleWithCSS=true → margin div 臂）spec 仅在 isSimpleIndentation
  // Element 合并面上有分叉，wrap 新建臂恒 blockquote；Chrome 录音（indent 语料
  // qV after=''、blockquote 形）与 spec 臂一致，margin-div 臂未移植（FIXME）。
  function _zwFbqIndentNodes(nodeList) {
    // "If node list is empty, do nothing and abort these steps."
    if (!nodeList.length) return;
    var firstNode = nodeList[0];
    // "If first node's parent is an ol or ul: ... wrap node list ... tag ..."
    if (_zwJusIsHtmlElement(firstNode.parentNode, ['ol', 'ul'])) {
      var tag = _zwJusTag(firstNode.parentNode);
      _zwJusWrap(nodeList,
        function (node) { return _zwJusIsHtmlElement(node, tag); },
        function () { return firstNode.ownerDocument.createElement(tag); });
      return;
    }
    // "Wrap node list, with sibling criteria returning true for a simple
    // indentation element ... new parent ... createElement("blockquote")..."
    var newParent = _zwJusWrap(nodeList,
      function (node) { return _zwListIsSimpleIndentationElement(node); },
      function () { return firstNode.ownerDocument.createElement('blockquote'); });
    // "Fix disallowed ancestors of new parent."
    _zwListFixDisallowedAncestors(newParent);
  }

  // indent action（vendored implementation.js :7052-7120）。
  globalThis.__zw_ce_indent_apply = function () {
    var range = _zwVaGetActiveRange();
    if (!range) return true;
    // "Let items be a list of all lis that are ancestor containers of the
    // active range's start and/or end node"（树序：end 爬升、start 爬升、
    // common ancestor 爬升三段各 unshift，终序为树序）。
    var cca = _zwFbqCommonAncestor(range.startContainer, range.endContainer);
    var items = [];
    var n = range.endContainer;
    while (n && n !== cca) {
      if (_zwJusIsHtmlElement(n, 'li')) items.unshift(n);
      n = n.parentNode;
    }
    n = range.startContainer;
    while (n && n !== cca) {
      if (_zwJusIsHtmlElement(n, 'li')) items.unshift(n);
      n = n.parentNode;
    }
    n = cca;
    var g = 0;
    while (n && g++ < 256) {
      if (_zwJusIsHtmlElement(n, 'li')) items.unshift(n);
      n = n.parentNode;
    }
    // "For each item in items, normalize sublists of item."
    var i;
    for (i = 0; i < items.length; i++) {
      _zwListNormalizeSublists(items[i]);
    }

    // "Block-extend the active range, and let new range be the result."
    var newRange = _zwJusBlockExtend(range, false);
    // "For each node node contained in new range, if node is editable and is
    // an allowed child of "div" or "ol" ..., append node to node list."
    var nodeList = _zwJusGetContainedNodes(newRange, function (node) {
      return _zwJusIsEditable(node)
        && (_zwJusIsAllowedChild(node, 'div')
            || _zwJusIsAllowedChild(node, 'ol'));
    });

    // "If the first visible member of node list is an li whose parent is an
    // ol or ul: ... If sibling is an li, normalize sublists of sibling."
    var vis = nodeList.filter(function (x) { return _zwJusIsVisible(x); });
    if (_zwJusIsHtmlElement(vis[0], 'li')
        && _zwJusIsHtmlElement(vis[0].parentNode, ['ol', 'ul'])) {
      var sib = vis[0].previousSibling;
      // spec isInvisible 爬升；null 守卫（根位 prev 链尽）。
      while (sib && _zwJusIsInvisible(sib)) {
        sib = sib.previousSibling;
      }
      if (_zwJusIsHtmlElement(sib, 'li')) {
        _zwListNormalizeSublists(sib);
      }
    }

    // "While node list is not empty: ... remove the first member ... append
    // ... While the first member of node list is the nextSibling of the last
    // member of sublist ... Indent sublist."
    while (nodeList.length) {
      var sublist = [nodeList.shift()];
      while (nodeList.length
          && nodeList[0] === sublist[sublist.length - 1].nextSibling) {
        sublist.push(nodeList.shift());
      }
      _zwFbqIndentNodes(sublist);
    }
    return true;
  };

  // 选区边界点插入 element（spec range insertNode 的本仓展开面）：Text 起点
  // offset-0 直插前方 ∨ splitText 后插 tail 前 ∨ 元素起点按子 index 插。
  function _zwFbqInsertAtRangeStart(el) {
    var range = _zwVaGetActiveRange();
    var sn = range.startContainer;
    var so = range.startOffset;
    if (sn.nodeType === 3) {
      if (so === 0) {
        sn.parentNode.insertBefore(el, sn);
      } else {
        var tail = sn.splitText(so);
        sn.parentNode.insertBefore(el, tail);
      }
      return;
    }
    sn.insertBefore(el, sn.childNodes[so] || null);
  }

  // insertHorizontalRule action（vendored implementation.js :7141-7230）。
  globalThis.__zw_ce_hr_apply = function () {
    var range = _zwVaGetActiveRange();
    if (!range) return true;
    // "While start offset is 0 and start node's parent is not null, ..."
    var startNode = range.startContainer;
    var startOffset = range.startOffset;
    var endNode = range.endContainer;
    var endOffset = range.endOffset;
    var g = 0;
    while (startOffset === 0 && startNode.parentNode && g++ < 256) {
      startOffset = _zwJusNodeIndex(startNode);
      startNode = startNode.parentNode;
    }
    g = 0;
    while (endOffset === _zwJusNodeLength(endNode) && endNode.parentNode
        && g++ < 256) {
      endOffset = 1 + _zwJusNodeIndex(endNode);
      endNode = endNode.parentNode;
    }
    // "Call collapse(...)/extend(...) on the Selection"——本仓 collapse 直改
    // sel._ranges[0] 同对象（deleteSelection 同款），setStart/setEnd 同义冗余
    // 保留 spec 形。
    var sel = globalThis.getSelection ? globalThis.getSelection() : null;
    if (sel) {
      sel.collapse(startNode, startOffset);
      sel.extend(endNode, endOffset);
    }
    range.setStart(startNode, startOffset);
    range.setEnd(endNode, endOffset);

    // "Delete the selection, with block merging false."
    _zwDelDeleteSelection({ blockMerging: false });
    range = _zwVaGetActiveRange();
    if (!range) return true;

    // "If the active range's start node is neither editable nor an editing
    // host, return true."
    if (!_zwJusIsEditable(range.startContainer)
        && !_zwJusIsEditingHost(range.startContainer)) {
      return true;
    }

    // Text 边界重锚两臂（offset-0 → 父 index；offset-len → 父 1+index）。
    if (range.startContainer.nodeType === 3 && range.startOffset === 0) {
      var pn = range.startContainer.parentNode;
      var po = _zwJusNodeIndex(range.startContainer);
      if (sel) sel.collapse(pn, po);
      range.setStart(pn, po);
      range.collapse(true);
    }
    if (range.startContainer.nodeType === 3
        && range.startOffset === _zwJusNodeLength(range.startContainer)) {
      var pn2 = range.startContainer.parentNode;
      var po2 = 1 + _zwJusNodeIndex(range.startContainer);
      if (sel) sel.collapse(pn2, po2);
      range.setStart(pn2, po2);
      range.collapse(true);
    }

    // "Let hr be the result of calling createElement("hr") ... Run
    // insertNode(hr) on the active range."
    var hr = document.createElement('hr');
    _zwFbqInsertAtRangeStart(hr);

    // "Fix disallowed ancestors of hr."
    _zwListFixDisallowedAncestors(hr);

    // "Run collapse() on the Selection, with first argument hr's parent and
    // the second argument equal to one plus hr's index."
    if (hr.parentNode) {
      var hp = hr.parentNode;
      var hi = 1 + _zwJusNodeIndex(hr);
      if (sel) sel.collapse(hp, hi);
      range.setStart(hp, hi);
      range.collapse(true);
    }
    return true;
  };

  // insertImage action（vendored implementation.js :7316-7367）。
  globalThis.__zw_ce_img_apply = function (value) {
    // "If value is the empty string, return false."
    var v = String(value == null ? '' : value);
    if (v === '') return false;

    // "Delete the selection, with strip wrappers false."
    _zwDelDeleteSelection({ stripWrappers: false });
    var range = _zwVaGetActiveRange();
    if (!range) return true;

    // "If the active range's start node is neither editable nor an editing
    // host, return true."
    if (!_zwJusIsEditable(range.startContainer)
        && !_zwJusIsEditingHost(range.startContainer)) {
      return true;
    }

    // "If range's start node is a block node whose sole child is a br, and
    // its start offset is 0, remove its start node's child from it."
    if (_zwJusIsBlockNode(range.startContainer)
        && range.startContainer.childNodes.length === 1
        && _zwJusIsHtmlElement(range.startContainer.firstChild, 'br')
        && range.startOffset === 0) {
      range.startContainer.removeChild(range.startContainer.firstChild);
    }

    // "Let img be ... createElement("img") ... setAttribute("src", value) ...
    // Run insertNode(img) on the range."
    var img = document.createElement('img');
    img.setAttribute('src', v);
    _zwFbqInsertAtRangeStart(img);

    // "Run collapse() on the Selection, with first argument equal to the
    // parent of img and the second argument equal to one plus the index of
    // img."（vendored 的 removeAllRanges/addRange 舞蹈在本仓是无效面，见文件头。）
    if (img.parentNode) {
      var ip = img.parentNode;
      var ii = 1 + _zwJusNodeIndex(img);
      range.setStart(ip, ii);
      range.setEnd(ip, ii);
      var sel2 = globalThis.getSelection ? globalThis.getSelection() : null;
      if (sel2) sel2.collapse(ip, ii);
    }
    return true;
  };
