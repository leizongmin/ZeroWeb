// part01f（2026-11-02 editing-compat F6 delete/forwarddelete 撤壳边界）：
// execCommand delete/forwarddelete 实应用引擎——deleteSelection + 两 action 的
// 忠实移植（参考 implementation.js :4550-5058 / :6193-6815 / :6815-7052），替换
// part06 flat 模型（同文本节点 nodeValue splice、跨容器 defer）。挂 part01e 后
// 同闭包复用 _zwJus*（位置/可见性/搬移机具）、_zwVa*（specified value /
// pushDown / forceValue / override 存储）、_zwList*（splitParent /
// inSameEditingHost）；part06 经 globalThis.__zw_ce_delete_apply 单点调用。
// Chrome delta（g16-multitest D1 失败族实证，见控制面 F6 节）：strip-wrappers
// 阶段**留壳**——原 startContainer 上行 inline 包裹链上的幸存壳不摘（打字续壳：
// 无 a 时链上最外层壳；有 a 时最外层 <a>，链上其余壳照摘），nodeList 相含阶段
// 不豁免（[foo<a>bar]</a>baz 期望整体移除）。语料期望样例：
//   foo<b>[bar</b>baz] → foo<b>a</b>；foo<u><a>[bar</a></u>baz] → foo<a>a</a>；
//   foo<font color=blue><a>[bar</a></font>baz] → foo<a>a</a>；
//   [foo<a>bar]</a>baz → abaz（相含 a 连 subtree 整体移除，无壳）。
// 残余挂账（本轮不做）：Blink TypingStyle（font color/size 删除后重发——brown/
// #0000FF/size delta 族仍 Fail）；delete action 的 li/dt/dd 首子臂与 indentation
// 元素臂（依赖 normalizeSublists/outdentNode，未移植，FIXME 标注）。
// https://w3c.github.io/editing/execCommand.html#deleting-the-selection
// https://w3c.github.io/editing/execCommand.html#the-delete-command
// https://w3c.github.io/editing/execCommand.html#the-forwarddelete-command

  // —— 位置工具（getPosition 经 part01c BpCompare 总序映射；vendored :254 递归
  // 换位在总序比较子下不需要）——
  function _zwDelGetPosition(nA, oA, nB, oB) {
    var c = _zwJusBpCompare(nA, oA, nB, oB);
    return c === 0 ? 'equal' : (c < 0 ? 'before' : 'after');
  }

  // —— 等价点四件套（vendored :4079-4160）——
  function _zwDelGetNextEquivalentPoint(node, offset) {
    if (_zwJusNodeLength(node) == 0) return null;
    if (offset == _zwJusNodeLength(node) && node.parentNode && _zwJusIsInlineNode(node)) {
      return [node.parentNode, 1 + _zwJusNodeIndex(node)];
    }
    if (0 <= offset && offset < node.childNodes.length
        && _zwJusNodeLength(node.childNodes[offset]) != 0
        && _zwJusIsInlineNode(node.childNodes[offset])) {
      return [node.childNodes[offset], 0];
    }
    return null;
  }

  function _zwDelGetPreviousEquivalentPoint(node, offset) {
    if (_zwJusNodeLength(node) == 0) return null;
    if (offset == 0 && node.parentNode && _zwJusIsInlineNode(node)) {
      return [node.parentNode, _zwJusNodeIndex(node)];
    }
    if (0 <= offset - 1 && offset - 1 < node.childNodes.length
        && _zwJusNodeLength(node.childNodes[offset - 1]) != 0
        && _zwJusIsInlineNode(node.childNodes[offset - 1])) {
      return [node.childNodes[offset - 1], _zwJusNodeLength(node.childNodes[offset - 1])];
    }
    return null;
  }

  function _zwDelGetFirstEquivalentPoint(node, offset) {
    var prev;
    while ((prev = _zwDelGetPreviousEquivalentPoint(node, offset))) {
      node = prev[0]; offset = prev[1];
    }
    return [node, offset];
  }

  function _zwDelGetLastEquivalentPoint(node, offset) {
    var next;
    while ((next = _zwDelGetNextEquivalentPoint(node, offset))) {
      node = next[0]; offset = next[1];
    }
    return [node, offset];
  }

  // —— canonical space sequence（vendored :5204）——
  function _zwDelCanonicalSpaceSequence(n, nonBreakingStart, nonBreakingEnd) {
    if (n == 0) return '';
    if (n == 1 && !nonBreakingStart && !nonBreakingEnd) return ' ';
    if (n == 1) return '\xa0';
    var buffer = '';
    var repeatedPair = nonBreakingStart ? '\xa0 ' : ' \xa0';
    while (n > 3) { buffer += repeatedPair; n -= 2; }
    if (n == 3) {
      buffer += !nonBreakingStart && !nonBreakingEnd ? ' \xa0 '
        : nonBreakingStart && !nonBreakingEnd ? '\xa0\xa0 '
        : !nonBreakingStart && nonBreakingEnd ? ' \xa0\xa0'
        : '\xa0 \xa0';
    } else {
      buffer += !nonBreakingStart && !nonBreakingEnd ? '\xa0 '
        : nonBreakingStart && !nonBreakingEnd ? '\xa0 '
        : !nonBreakingStart && nonBreakingEnd ? ' \xa0'
        : '\xa0\xa0';
    }
    return buffer;
  }

  // —— canonicalize whitespace（vendored :5265-5470；deleteData/insertData 以
  // nodeValue splice 等价——本仓 shim 文本变异 idiom，引擎侧 SetChildText 同
  // 流转）。parentNode 空守卫：deleteSelection 中段 canonicalize 位点可能已被
  // nodeList 阶段摘除父链（vendored 依赖真浏览器 gCS(null) 不抛，本仓显式断臂）。
  function _zwDelCanonicalizeWhitespace(node, offset, fixCollapsedSpace) {
    if (fixCollapsedSpace === undefined) fixCollapsedSpace = true;
    if (!_zwJusIsEditable(node) && !_zwJusIsEditingHost(node)) return;
    var startNode = node, startOffset = offset;
    while (true) {
      if (0 <= startOffset - 1 && startNode.childNodes[startOffset - 1]
          && _zwListInSameEditingHost(startNode, startNode.childNodes[startOffset - 1])) {
        startNode = startNode.childNodes[startOffset - 1];
        startOffset = _zwJusNodeLength(startNode);
      } else if (startOffset == 0 && !_zwJusFollowsLineBreak(startNode)
          && startNode.parentNode
          && _zwListInSameEditingHost(startNode, startNode.parentNode)) {
        startOffset = _zwJusNodeIndex(startNode);
        startNode = startNode.parentNode;
      } else if (startNode.nodeType == 3 && startNode.parentNode
          && ['pre', 'pre-wrap'].indexOf(_zwJusWhiteSpaceOf(startNode.parentNode)) == -1
          && startOffset != 0
          && /[ \xa0]/.test(String(startNode.nodeValue || '').charAt(startOffset - 1))) {
        startOffset--;
      } else {
        break;
      }
    }
    var endNode = startNode, endOffset = startOffset;
    var length = 0;
    var collapseSpaces = startOffset == 0 && _zwJusFollowsLineBreak(startNode);
    while (true) {
      if (endOffset < endNode.childNodes.length && endNode.childNodes[endOffset]
          && _zwListInSameEditingHost(endNode, endNode.childNodes[endOffset])) {
        endNode = endNode.childNodes[endOffset];
        endOffset = 0;
      } else if (endOffset == _zwJusNodeLength(endNode) && !_zwJusPrecedesLineBreak(endNode)
          && endNode.parentNode
          && _zwListInSameEditingHost(endNode, endNode.parentNode)) {
        endOffset = 1 + _zwJusNodeIndex(endNode);
        endNode = endNode.parentNode;
      } else if (endNode.nodeType == 3 && endNode.parentNode
          && ['pre', 'pre-wrap'].indexOf(_zwJusWhiteSpaceOf(endNode.parentNode)) == -1
          && endOffset != _zwJusNodeLength(endNode)
          && /[ \xa0]/.test(String(endNode.nodeValue || '').charAt(endOffset))) {
        if (fixCollapsedSpace && collapseSpaces
            && String(endNode.nodeValue || '').charAt(endOffset) == ' ') {
          var dFix = String(endNode.nodeValue || '');
          endNode.nodeValue = dFix.slice(0, endOffset) + dFix.slice(endOffset + 1);
          continue;
        }
        collapseSpaces = String(endNode.nodeValue || '').charAt(endOffset) == ' ';
        endOffset++;
        length++;
      } else {
        break;
      }
    }
    if (fixCollapsedSpace) {
      while (_zwDelGetPosition(startNode, startOffset, endNode, endOffset) == 'before') {
        if (0 <= endOffset - 1 && endOffset - 1 < endNode.childNodes.length
            && endNode.childNodes[endOffset - 1]
            && _zwListInSameEditingHost(endNode, endNode.childNodes[endOffset - 1])) {
          endNode = endNode.childNodes[endOffset - 1];
          endOffset = _zwJusNodeLength(endNode);
        } else if (endOffset == 0 && endNode.parentNode
            && _zwListInSameEditingHost(endNode, endNode.parentNode)) {
          endOffset = _zwJusNodeIndex(endNode);
          endNode = endNode.parentNode;
        } else if (endNode.nodeType == 3 && endNode.parentNode
            && ['pre', 'pre-wrap'].indexOf(_zwJusWhiteSpaceOf(endNode.parentNode)) == -1
            && endOffset == _zwJusNodeLength(endNode)
            && String(endNode.nodeValue || '').slice(-1) == ' '
            && _zwJusPrecedesLineBreak(endNode)) {
          endOffset--;
          length--;
          var dEnd = String(endNode.nodeValue || '');
          endNode.nodeValue = dEnd.slice(0, endOffset) + dEnd.slice(endOffset + 1);
        } else {
          break;
        }
      }
    }
    var replacementWhitespace = _zwDelCanonicalSpaceSequence(length,
      startOffset == 0 && _zwJusFollowsLineBreak(startNode),
      endOffset == _zwJusNodeLength(endNode) && _zwJusPrecedesLineBreak(endNode));
    while (_zwDelGetPosition(startNode, startOffset, endNode, endOffset) == 'before') {
      if (startOffset < startNode.childNodes.length) {
        startNode = startNode.childNodes[startOffset];
        startOffset = 0;
      } else if (startNode.nodeType != 3 || startOffset == _zwJusNodeLength(startNode)) {
        startOffset = 1 + _zwJusNodeIndex(startNode);
        startNode = startNode.parentNode;
      } else {
        var element = replacementWhitespace.charAt(0);
        replacementWhitespace = replacementWhitespace.slice(1);
        if (element != String(startNode.nodeValue || '').charAt(startOffset)) {
          var dRep = String(startNode.nodeValue || '');
          startNode.nodeValue = dRep.slice(0, startOffset) + element + dRep.slice(startOffset + 1);
        }
        startOffset++;
      }
    }
  }

  // —— collapsed block prop（vendored :1033-1060；collapsed line break /
  // extraneous line break 复用 part01c 语料锁定面）——
  function _zwDelIsCollapsedBlockProp(node) {
    if (_zwJusIsCollapsedLineBreak(node) && !_zwJusIsExtraneousLineBreak(node)) {
      return true;
    }
    if (!node || !_zwJusIsInlineNode(node) || node.nodeType != 1) return false;
    var hasCollapsedBlockPropChild = false;
    for (var i = 0; i < node.childNodes.length; i++) {
      if (!_zwJusIsInvisible(node.childNodes[i])
          && !_zwDelIsCollapsedBlockProp(node.childNodes[i])) {
        return false;
      }
      if (_zwDelIsCollapsedBlockProp(node.childNodes[i])) hasCollapsedBlockPropChild = true;
    }
    return hasCollapsedBlockPropChild;
  }

  // —— block node of（vendored :3812：inline 上行至首块）——
  function _zwDelGetBlockNodeOf(node) {
    while (node && _zwJusIsInlineNode(node)) node = node.parentNode;
    return node;
  }

  // —— 含自身祖先链（树序，vendored :102）——
  function _zwDelGetInclusiveAncestors(node) {
    var out = [], n = node;
    while (n) { out.unshift(n); n = n.parentNode; }
    return out;
  }

  // —— recordValues / restoreValues（vendored :2385/:2427；specified value /
  // pushDown / forceValue / equivalentValues 复用 part01d 语料锁定面）——
  function _zwDelRecordValues(nodeList) {
    var values = [];
    var cmds = ['subscript', 'bold', 'fontname', 'fontsize', 'forecolor',
      'hilitecolor', 'italic', 'strikethrough', 'underline'];
    for (var i = 0; i < nodeList.length; i++) {
      for (var j = 0; j < cmds.length; j++) {
        var command = cmds[j];
        var ancestor = nodeList[i];
        if (ancestor && ancestor.nodeType != 1) ancestor = ancestor.parentNode;
        while (ancestor && ancestor.nodeType == 1
            && _zwVaSpecifiedValue(ancestor, command) === null) {
          ancestor = ancestor.parentNode;
        }
        if (ancestor && ancestor.nodeType == 1) {
          values.push([nodeList[i], command, _zwVaSpecifiedValue(ancestor, command)]);
        } else {
          values.push([nodeList[i], command, null]);
        }
      }
    }
    return values;
  }

  function _zwDelRestoreValues(values) {
    for (var i = 0; i < values.length; i++) {
      var node = values[i][0], command = values[i][1], value = values[i][2];
      var ancestor = node;
      if (!ancestor || ancestor.nodeType != 1) ancestor = ancestor ? ancestor.parentNode : null;
      while (ancestor && ancestor.nodeType == 1
          && _zwVaSpecifiedValue(ancestor, command) === null) {
        ancestor = ancestor.parentNode;
      }
      if (value === null && ancestor && ancestor.nodeType == 1) {
        _zwVaPushDownValues(node, command, null);
      } else if ((ancestor && ancestor.nodeType == 1
          && !_zwVaEquivalentValues(command, _zwVaSpecifiedValue(ancestor, command), value))
          || ((!ancestor || ancestor.nodeType != 1) && value !== null)) {
        _zwVaForceValue(node, command, value);
      }
    }
  }

  // —— recordCurrentStatesAndValues / restoreStatesAndValues（vendored
  // :4381/:4450）。restore 的非 collapsed 臂需要命令 action 派发（boolean →
  // typing 标签 toggle、string → 值重应用），当前不可达：deleteSelection 全部
  // restore 位点均已先 collapse（isEffectivelyContained 对 collapsed 恒 false
  // → node null → 返回）。FIXME: TypingStyle 轮接手补 action 派发面。
  function _zwDelFirstFormattableContained() {
    var rng = _zwVaGetActiveRange();
    if (!rng) return null;
    var contained = _zwValGetAllEffectivelyContained(rng, function (n) {
      return _zwValIsFormattable(n);
    });
    return contained.length ? contained[0] : null;
  }

  // inlineCommandActivatedValues（vendored :3130/:3475/:3603/:3635/:3668/:3685）。
  var _zwDelActivatedValues = {
    bold: ['bold', '600', '700', '800', '900'],
    italic: ['italic', 'oblique'],
    strikethrough: ['line-through'],
    subscript: ['subscript'],
    superscript: ['superscript'],
    underline: ['underline']
  };

  function _zwDelRecordCurrentStatesAndValues() {
    var overrides = [];
    var node = _zwDelFirstFormattableContained();
    if (!node) return overrides;
    overrides.push(['createlink', _zwValEffectiveValue(node, 'createlink')]);
    var bools = ['bold', 'italic', 'strikethrough', 'subscript', 'superscript', 'underline'];
    for (var i = 0; i < bools.length; i++) {
      overrides.push([bools[i],
        _zwDelActivatedValues[bools[i]].indexOf(_zwValEffectiveValue(node, bools[i])) != -1]);
    }
    // fontname/fontsize/forecolor/hilitecolor 取 queryCommandValue 面（vendored
    // commands[command].value() 同义——本仓值查询面 __zw_ce_value_query）。
    var vals = ['fontname', 'fontsize', 'forecolor', 'hilitecolor'];
    for (var j = 0; j < vals.length; j++) {
      var v = '';
      try {
        v = (typeof globalThis.__zw_ce_value_query === 'function')
          ? globalThis.__zw_ce_value_query(vals[j]) : '';
      } catch (_eQv) { v = ''; }
      overrides.push([vals[j], v]);
    }
    overrides.push(['fontsize', _zwValEffectiveValue(node, 'fontsize')]);
    return overrides;
  }

  function _zwDelRestoreStatesAndValues(overrides) {
    var node = _zwDelFirstFormattableContained();
    if (!node) return;
    // FIXME: 非 collapsed 恢复臂（boolean → typing 标签 toggle、string → 值
    // 重应用、createlink/fontsize 特例）——当前调用域全为 collapsed（见上注）。
  }

  // —— deleteSelection 主体（vendored :4550-5058）——flags {blockMerging,
  // stripWrappers, direction}，缺省 true/true/forward。
  function _zwDelDeleteSelection(flags) {
    if (flags === undefined) flags = {};
    var blockMerging = 'blockMerging' in flags ? Boolean(flags.blockMerging) : true;
    var stripWrappers = 'stripWrappers' in flags ? Boolean(flags.stripWrappers) : true;
    var direction = 'direction' in flags ? flags.direction : 'forward';

    if (!_zwVaGetActiveRange()) return;

    _zwDelCanonicalizeWhitespace(_zwVaGetActiveRange().startContainer,
      _zwVaGetActiveRange().startOffset);
    _zwDelCanonicalizeWhitespace(_zwVaGetActiveRange().endContainer,
      _zwVaGetActiveRange().endOffset);

    var start = _zwDelGetLastEquivalentPoint(_zwVaGetActiveRange().startContainer,
      _zwVaGetActiveRange().startOffset);
    var startNode = start[0], startOffset = start[1];
    var end = _zwDelGetFirstEquivalentPoint(_zwVaGetActiveRange().endContainer,
      _zwVaGetActiveRange().endOffset);
    var endNode = end[0], endOffset = end[1];

    if (_zwDelGetPosition(endNode, endOffset, startNode, startOffset) !== 'after') {
      var sel0 = globalThis.getSelection();
      if (direction == 'forward') {
        if (sel0.rangeCount) sel0.collapseToStart();
        _zwVaGetActiveRange().collapse(true);
      } else {
        if (sel0.rangeCount) sel0.collapseToEnd();
        _zwVaGetActiveRange().collapse(false);
      }
      return;
    }

    if (startNode.nodeType == 3 && startOffset == 0) {
      startOffset = _zwJusNodeIndex(startNode);
      startNode = startNode.parentNode;
    }
    if (endNode.nodeType == 3 && endOffset == _zwJusNodeLength(endNode)) {
      endOffset = 1 + _zwJusNodeIndex(endNode);
      endNode = endNode.parentNode;
    }

    // 本仓 Selection.collapse/extend 直改唯一 range（_ranges[0] 同对象族），
    // vendored 随后的 setStart/setEnd 为同义冗余面，不重复。
    var sel1 = globalThis.getSelection();
    sel1.collapse(startNode, startOffset);
    sel1.extend(endNode, endOffset);

    var startBlock = _zwVaGetActiveRange().startContainer;
    while (_zwListInSameEditingHost(startBlock, startBlock.parentNode)
        && _zwJusIsInlineNode(startBlock)) {
      startBlock = startBlock.parentNode;
    }
    if ((!_zwJusIsBlockNode(startBlock) && !_zwJusIsEditingHost(startBlock))
        || !_zwVaIsAllowedChild('span', startBlock)
        || _zwJusIsHtmlElement(startBlock, ['td', 'th'])) {
      startBlock = null;
    }

    var endBlock = _zwVaGetActiveRange().endContainer;
    while (_zwListInSameEditingHost(endBlock, endBlock.parentNode)
        && _zwJusIsInlineNode(endBlock)) {
      endBlock = endBlock.parentNode;
    }
    if ((!_zwJusIsBlockNode(endBlock) && !_zwJusIsEditingHost(endBlock))
        || !_zwVaIsAllowedChild('span', endBlock)
        || _zwJusIsHtmlElement(endBlock, ['td', 'th'])) {
      endBlock = null;
    }

    var overrides = _zwDelRecordCurrentStatesAndValues();

    if (startNode == endNode && _zwJusIsEditable(startNode)
        && startNode.nodeType == 3) {
      var dSame = String(startNode.nodeValue || '');
      startNode.nodeValue = dSame.slice(0, startOffset) + dSame.slice(endOffset);
      _zwDelCanonicalizeWhitespace(startNode, startOffset, false);
      var selSame = globalThis.getSelection();
      if (direction == 'forward') {
        if (selSame.rangeCount) selSame.collapseToStart();
        _zwVaGetActiveRange().collapse(true);
      } else {
        if (selSame.rangeCount) selSame.collapseToEnd();
        _zwVaGetActiveRange().collapse(false);
      }
      _zwDelRestoreStatesAndValues(overrides);
      return;
    }

    // deleteData(startOffset, getNodeLength(startNode) - startOffset) 等价：
    // 起点 text 尾段（startOffset 至末尾）整体摘除。
    if (_zwJusIsEditable(startNode) && startNode.nodeType == 3) {
      startNode.nodeValue = String(startNode.nodeValue || '').slice(0, startOffset);
    }

    // Chrome 留壳预扫：原 start 链（编辑宿主为止）上的 inline 包裹，幸存壳 =
    // 最外层 <a>（无 a 时链上最外层壳）。g16-multitest D1 实证：
    //   foo<b>[bar</b>baz] → foo<b>a</b>（链上最外层 b 留）；
    //   foo<u><a>[bar</a></u>baz] → foo<a>a</a>（最外层 a 留、其上 u 摘）；
    //   foo<a><font>[bar</font></a>baz] → foo<a>a</a>（a 留、其下 font 摘）。
    var delKeepShell = null;
    (function () {
      var inlineWrappers = [];
      var m = startNode;
      while (m && m.parentNode && !_zwJusIsEditingHost(m) && _zwJusIsInlineNode(m)
          && _zwListInSameEditingHost(m, m.parentNode)) {
        if (m.nodeType == 1) inlineWrappers.push(m);
        m = m.parentNode;
      }
      for (var wi = 0; wi < inlineWrappers.length; wi++) {
        if (_zwJusIsHtmlElement(inlineWrappers[wi], 'a')) delKeepShell = inlineWrappers[wi];
      }
      if (!delKeepShell && inlineWrappers.length) {
        delKeepShell = inlineWrappers[inlineWrappers.length - 1];
      }
    })();

    var nodeList = _zwJusGetContainedNodes(_zwVaGetActiveRange(),
      function (n) {
        return _zwJusIsEditable(n)
          && !_zwJusIsHtmlElement(n, ['thead', 'tbody', 'tfoot', 'tr', 'th', 'td']);
      }
    );

    for (var i = 0; i < nodeList.length; i++) {
      var node = nodeList[i];
      var parent_ = node.parentNode;
      if (!parent_) continue;
      parent_.removeChild(node);
      var blockOfParent = _zwDelGetBlockNodeOf(parent_);
      if (blockOfParent
          && ![].some.call(blockOfParent.childNodes, _zwJusIsVisible)
          && (_zwJusIsEditable(parent_) || _zwJusIsEditingHost(parent_))
          && parent_ !== delKeepShell) {
        // Chrome delta：幸存壳本身就是块内占位（blockquote>font 选区删净后壳
        // 保留、不再补 br——g17 实证补 br 产出 <font><br></font> 双占位）。
        parent_.appendChild(document.createElement('br'));
      }
      if (stripWrappers
          || (!_zwJusIsAncestorOf(parent_, startNode) && parent_ != startNode)) {
        while (_zwJusIsEditable(parent_)
            && _zwJusIsInlineNode(parent_)
            && _zwJusNodeLength(parent_) == 0) {
          var grandparent = parent_.parentNode;
          // Chrome delta：链上幸存壳不摘（打字续壳）；幸存壳置空计数使其上方
          // 仅含它的包裹仍按空摘除（u>a 链：a 留、u 摘）。非链壳照 spec 摘。
          if (parent_ !== delKeepShell && grandparent) {
            grandparent.removeChild(parent_);
          }
          if (!grandparent) break;
          parent_ = grandparent;
        }
      }
    }

    // deleteData(0, endOffset) 等价：终点 text 头段（0 至 endOffset）整体摘除。
    if (_zwJusIsEditable(endNode) && endNode.nodeType == 3) {
      endNode.nodeValue = String(endNode.nodeValue || '').slice(endOffset);
    }

    _zwDelCanonicalizeWhitespace(_zwVaGetActiveRange().startContainer,
      _zwVaGetActiveRange().startOffset, false);
    _zwDelCanonicalizeWhitespace(_zwVaGetActiveRange().endContainer,
      _zwVaGetActiveRange().endOffset, false);

    var values; // 块合并分支的 recordValues 结果（restoreValues 用）

    if (!blockMerging || !startBlock || !endBlock
        || !_zwListInSameEditingHost(startBlock, endBlock)
        || startBlock == endBlock) {
      var selExit = globalThis.getSelection();
      if (direction == 'forward') {
        if (selExit.rangeCount) selExit.collapseToStart();
        _zwVaGetActiveRange().collapse(true);
      } else {
        if (selExit.rangeCount) selExit.collapseToEnd();
        _zwVaGetActiveRange().collapse(false);
      }
      _zwDelRestoreStatesAndValues(overrides);
      return;
    }

    if (startBlock.childNodes.length == 1
        && _zwDelIsCollapsedBlockProp(startBlock.firstChild)) {
      startBlock.removeChild(startBlock.firstChild);
    }

    if (_zwJusIsAncestorOf(startBlock, endBlock)) {
      var referenceNode = endBlock;
      while (referenceNode.parentNode != startBlock) {
        referenceNode = referenceNode.parentNode;
      }
      var selAnc = globalThis.getSelection();
      selAnc.collapse(startBlock, _zwJusNodeIndex(referenceNode));
      _zwVaGetActiveRange().collapse(true);

      if (!endBlock.hasChildNodes()) {
        while (_zwJusIsEditable(endBlock)
            && endBlock.parentNode.childNodes.length == 1
            && endBlock.parentNode != startBlock) {
          var parEmpty = endBlock;
          parEmpty.removeChild(endBlock);
          endBlock = parEmpty;
        }
        if (_zwJusIsEditable(endBlock)
            && !_zwJusIsInlineNode(endBlock)
            && _zwJusIsInlineNode(endBlock.previousSibling)
            && _zwJusIsInlineNode(endBlock.nextSibling)) {
          endBlock.parentNode.insertBefore(document.createElement('br'), endBlock.nextSibling);
        }
        if (_zwJusIsEditable(endBlock)) {
          endBlock.parentNode.removeChild(endBlock);
        }
        _zwDelRestoreStatesAndValues(overrides);
        return;
      }

      if (!_zwJusIsInlineNode(endBlock.firstChild)) {
        _zwDelRestoreStatesAndValues(overrides);
        return;
      }

      var children = [endBlock.firstChild];
      while (!_zwJusIsHtmlElement(children[children.length - 1], 'br')
          && _zwJusIsInlineNode(children[children.length - 1].nextSibling)) {
        children.push(children[children.length - 1].nextSibling);
      }

      values = _zwDelRecordValues(children);

      while (children[0].parentNode != startBlock) {
        _zwListSplitParent(children);
      }

      if (_zwJusIsEditable(children[0].previousSibling)
          && _zwJusIsHtmlElement(children[0].previousSibling, 'br')) {
        children[0].parentNode.removeChild(children[0].previousSibling);
      }

    } else if (_zwJusIsAncestorOf(endBlock, startBlock)) {
      var selDesc = globalThis.getSelection();
      selDesc.collapse(startBlock, _zwJusNodeLength(startBlock));
      _zwVaGetActiveRange().collapse(true);

      var referenceNode2 = startBlock;
      while (referenceNode2.parentNode != endBlock) {
        referenceNode2 = referenceNode2.parentNode;
      }

      if (_zwJusIsInlineNode(referenceNode2.nextSibling)
          && _zwJusIsHtmlElement(startBlock.lastChild, 'br')) {
        startBlock.removeChild(startBlock.lastChild);
      }

      var nodesToMove = [];
      if (referenceNode2.nextSibling && !_zwJusIsBlockNode(referenceNode2.nextSibling)) {
        nodesToMove.push(referenceNode2.nextSibling);
      }
      if (nodesToMove.length
          && !_zwJusIsHtmlElement(nodesToMove[nodesToMove.length - 1], 'br')
          && nodesToMove[nodesToMove.length - 1].nextSibling
          && !_zwJusIsBlockNode(nodesToMove[nodesToMove.length - 1].nextSibling)) {
        nodesToMove.push(nodesToMove[nodesToMove.length - 1].nextSibling);
      }

      values = _zwDelRecordValues(nodesToMove);

      for (var mi = 0; mi < nodesToMove.length; mi++) {
        _zwJusMovePreservingRanges(nodesToMove[mi], startBlock, -1);
      }

    } else {
      var selElse = globalThis.getSelection();
      selElse.collapse(startBlock, _zwJusNodeLength(startBlock));
      _zwVaGetActiveRange().collapse(true);

      if (_zwJusIsInlineNode(endBlock.firstChild)
          && _zwJusIsHtmlElement(startBlock.lastChild, 'br')) {
        startBlock.removeChild(startBlock.lastChild);
      }

      values = _zwDelRecordValues([].slice.call(endBlock.childNodes));

      while (endBlock.hasChildNodes()) {
        _zwJusMovePreservingRanges(endBlock.firstChild, startBlock, -1);
      }

      while (!endBlock.hasChildNodes()) {
        var parMerge = endBlock.parentNode;
        parMerge.removeChild(endBlock);
        endBlock = parMerge;
      }
    }

    // ol/ul 兄弟合并（vendored :4992-5017；inclusive ancestor 链扫描）。
    var ancestor = startBlock;
    var guardMerge = 0;
    while (guardMerge++ < 256 && (function () {
      var incl = _zwDelGetInclusiveAncestors(ancestor);
      for (var ii = 0; ii < incl.length; ii++) {
        var nd = incl[ii];
        if (_zwListInSameEditingHost(ancestor, nd)
            && ((_zwJusIsHtmlElement(nd, 'ol') && _zwJusIsHtmlElement(nd.nextSibling, 'ol'))
              || (_zwJusIsHtmlElement(nd, 'ul') && _zwJusIsHtmlElement(nd.nextSibling, 'ul')))
            && _zwListInSameEditingHost(ancestor, nd.nextSibling)) {
          return true;
        }
      }
      return false;
    })()) {
      while (!(
          _zwJusIsHtmlElement(ancestor, 'ol')
          && _zwJusIsHtmlElement(ancestor.nextSibling, 'ol')
          && _zwListInSameEditingHost(ancestor, ancestor.nextSibling)
        ) && !(
          _zwJusIsHtmlElement(ancestor, 'ul')
          && _zwJusIsHtmlElement(ancestor.nextSibling, 'ul')
          && _zwListInSameEditingHost(ancestor, ancestor.nextSibling)
        )) {
        if (!ancestor.parentNode) break;
        ancestor = ancestor.parentNode;
      }
      while (ancestor.nextSibling.hasChildNodes()) {
        _zwJusMovePreservingRanges(ancestor.nextSibling.firstChild, ancestor, -1);
      }
      ancestor.parentNode.removeChild(ancestor.nextSibling);
    }

    _zwDelRestoreValues(values);

    if (!startBlock.hasChildNodes()) {
      startBlock.appendChild(document.createElement('br'));
    }

    _zwJusRemoveExtraneousLineBreaksAtTheEndOf(startBlock);

    _zwDelRestoreStatesAndValues(overrides);
  }

  // —— delete action（collapsed 臂；vendored :6193-6815）。FIXME: li/dt/dd 首子
  // 臂（需 normalizeSublists，未移植）与 indentation 元素臂（需 outdentNode，未
  // 移植）按 spec 原序跳过——命中时继续走后续通用臂（spec 语义差，控制面挂账）。
  function _zwDelDeleteAction() {
    var ar = _zwVaGetActiveRange();
    if (!ar) return true;
    if (!ar.collapsed) {
      _zwDelDeleteSelection();
      return true;
    }

    _zwDelCanonicalizeWhitespace(ar.startContainer, ar.startOffset);

    var node = ar.startContainer;
    var offset = ar.startOffset;

    while (true) {
      if (!node) return true;
      if (offset == 0 && _zwJusIsEditable(node.previousSibling)
          && _zwJusIsInvisible(node.previousSibling)) {
        node.parentNode.removeChild(node.previousSibling);
      } else if (0 <= offset - 1 && offset - 1 < node.childNodes.length
          && _zwJusIsEditable(node.childNodes[offset - 1])
          && _zwJusIsInvisible(node.childNodes[offset - 1])) {
        node.removeChild(node.childNodes[offset - 1]);
        offset--;
      } else if ((offset == 0 && _zwJusIsInlineNode(node) && node.parentNode)
          || _zwJusIsInvisible(node)) {
        offset = _zwJusNodeIndex(node);
        node = node.parentNode;
      } else if (0 <= offset - 1 && offset - 1 < node.childNodes.length
          && _zwJusIsEditable(node.childNodes[offset - 1])
          && _zwJusIsHtmlElement(node.childNodes[offset - 1], 'a')) {
        _zwJusRemovePreservingDescendants(node.childNodes[offset - 1]);
        return true;
      } else if (0 <= offset - 1 && offset - 1 < node.childNodes.length
          && !_zwJusIsBlockNode(node.childNodes[offset - 1])
          && !_zwJusIsHtmlElement(node.childNodes[offset - 1], ['br', 'img'])) {
        node = node.childNodes[offset - 1];
        offset = _zwJusNodeLength(node);
      } else {
        break;
      }
    }

    if ((node.nodeType == 3 && offset != 0)
        || (_zwJusIsBlockNode(node) && 0 <= offset - 1
          && offset - 1 < node.childNodes.length
          && _zwJusIsHtmlElement(node.childNodes[offset - 1], ['br', 'hr', 'img']))) {
      var selTxt = globalThis.getSelection();
      selTxt.collapse(node, offset);
      selTxt.extend(node, offset - 1);
      _zwDelDeleteSelection();
      return true;
    }

    if (_zwJusIsInlineNode(node)) return true;

    // FIXME: 「node 是 li/dt/dd 且为父首子且 offset 0」臂——normalizeSublists/
    // setTagName/fixDisallowedAncestors 组合未移植，跳过（控制面 list FIXME 挂账）。
    // FIXME: 「offset 0 且有 indentation 元素祖先容器」臂——outdentNode 未移植，
    // 跳过（blockquote/indent 撤销面，控制面挂账）。

    if (_zwJusIsHtmlElement(_zwDelChildAt(node, offset), 'table')) return true;

    var startNode = node;
    var startOffset = offset;

    while (true) {
      if (startOffset == 0) {
        if (!startNode.parentNode) break;
        startOffset = _zwJusNodeIndex(startNode);
        startNode = startNode.parentNode;
      } else if (0 <= startOffset - 1 && startOffset - 1 < startNode.childNodes.length
          && _zwJusIsEditable(startNode.childNodes[startOffset - 1])
          && _zwJusIsInvisible(startNode.childNodes[startOffset - 1])) {
        startNode.removeChild(startNode.childNodes[startOffset - 1]);
        startOffset--;
      } else {
        break;
      }
    }

    if (0 <= startOffset - 1 && startOffset - 1 < startNode.childNodes.length
        && _zwJusIsHtmlElement(startNode.childNodes[startOffset - 1], 'table')) {
      var selTbl = globalThis.getSelection();
      selTbl.collapse(startNode, startOffset - 1);
      selTbl.extend(startNode, startOffset);
      return true;
    }

    if (offset == 0
        && (_zwJusIsHtmlElement(_zwDelChildAt(startNode, startOffset - 1), 'hr')
          || (_zwJusIsHtmlElement(_zwDelChildAt(startNode, startOffset - 1), 'br')
            && (_zwJusIsHtmlElement(
                  (_zwDelChildAt(startNode, startOffset - 1) || {}).previousSibling, 'br')
              || !_zwJusIsInlineNode(
                  (_zwDelChildAt(startNode, startOffset - 1) || {}).previousSibling))))) {
      var selBr = globalThis.getSelection();
      selBr.collapse(startNode, startOffset - 1);
      selBr.extend(startNode, startOffset);
      _zwDelDeleteSelection();
      var selBr2 = globalThis.getSelection();
      selBr2.collapse(node, offset);
      _zwVaGetActiveRange().collapse(true);
      return true;
    }

    var nextItem = _zwDelChildAt(startNode, startOffset);
    if (_zwJusIsHtmlElement(nextItem, ['li', 'dt', 'dd'])
        && _zwJusIsInlineNode(nextItem.firstChild)
        && startOffset != 0) {
      var previousItem = _zwDelChildAt(startNode, startOffset - 1);
      if (previousItem) {
        if (_zwJusIsInlineNode(previousItem.lastChild)
            && !_zwJusIsHtmlElement(previousItem.lastChild, 'br')) {
          previousItem.appendChild(document.createElement('br'));
        }
        if (_zwJusIsInlineNode(previousItem.lastChild)) {
          previousItem.appendChild(document.createElement('br'));
        }
      }
    }

    if (_zwJusIsHtmlElement(nextItem, ['li', 'dt', 'dd'])
        && _zwJusIsHtmlElement(nextItem.previousSibling, ['li', 'dt', 'dd'])) {
      var originalRange = _zwVaGetActiveRange().cloneRange();
      startNode = nextItem.previousSibling;
      startOffset = _zwJusNodeLength(startNode);
      node = startNode.nextSibling;
      var selLst = globalThis.getSelection();
      selLst.collapse(startNode, startOffset);
      if (node) selLst.extend(node, 0);
      _zwDelDeleteSelection();
      var selLst2 = globalThis.getSelection();
      selLst2.removeAllRanges();
      selLst2.addRange(originalRange);
      _zwVaGetActiveRange().setStart(originalRange.startContainer, originalRange.startOffset);
      _zwVaGetActiveRange().setEnd(originalRange.endContainer, originalRange.endOffset);
      return true;
    }

    while (0 <= startOffset - 1 && startOffset - 1 < startNode.childNodes.length) {
      if (_zwJusIsEditable(startNode.childNodes[startOffset - 1])
          && _zwJusIsInvisible(startNode.childNodes[startOffset - 1])) {
        startNode.removeChild(startNode.childNodes[startOffset - 1]);
        startOffset--;
      } else {
        startNode = startNode.childNodes[startOffset - 1];
        startOffset = _zwJusNodeLength(startNode);
      }
    }

    var selFin = globalThis.getSelection();
    selFin.collapse(startNode, startOffset);
    selFin.extend(node, offset);
    _zwDelDeleteSelection({ direction: 'backward' });
    return true;
  }

  // —— forwarddelete action（collapsed 臂；vendored :6815-7052，全量移植无
  // FIXME 臂）——组合符号（combining mark）按 vendored 基础覆盖（U+0300-036F +
  // 希伯来几点，TODO 同源：完整 M 类目未做）。
  function _zwDelForwardDeleteAction() {
    var ar = _zwVaGetActiveRange();
    if (!ar) return true;
    if (!ar.collapsed) {
      _zwDelDeleteSelection();
      return true;
    }

    _zwDelCanonicalizeWhitespace(ar.startContainer, ar.startOffset);

    var node = ar.startContainer;
    var offset = ar.startOffset;

    while (true) {
      if (!node) return true;
      if (offset == _zwJusNodeLength(node) && _zwJusIsEditable(node.nextSibling)
          && _zwJusIsInvisible(node.nextSibling)) {
        node.parentNode.removeChild(node.nextSibling);
      } else if (offset < node.childNodes.length
          && _zwJusIsEditable(node.childNodes[offset])
          && _zwJusIsInvisible(node.childNodes[offset])) {
        node.removeChild(node.childNodes[offset]);
      } else if ((offset == _zwJusNodeLength(node) && _zwJusIsInlineNode(node)
            && node.parentNode) || _zwJusIsInvisible(node)) {
        offset = 1 + _zwJusNodeIndex(node);
        node = node.parentNode;
      } else if (offset < node.childNodes.length
          && !_zwJusIsBlockNode(node.childNodes[offset])
          && !_zwJusIsHtmlElement(node.childNodes[offset], ['br', 'img'])
          && !_zwDelIsCollapsedBlockProp(node.childNodes[offset])) {
        node = node.childNodes[offset];
        offset = 0;
      } else {
        break;
      }
    }

    if (node.nodeType == 3 && offset != _zwJusNodeLength(node)) {
      var endOffset = offset + 1;
      var data = String(node.nodeValue || '');
      while (endOffset != data.length
          && /^[\u0300-\u036f\u0591-\u05bd\u05c1\u05c2]$/.test(data.charAt(endOffset))) {
        endOffset++;
      }
      var selFw = globalThis.getSelection();
      selFw.collapse(node, offset);
      selFw.extend(node, endOffset);
      _zwDelDeleteSelection();
      return true;
    }

    if (_zwJusIsInlineNode(node)) return true;

    if (offset < node.childNodes.length
        && _zwJusIsHtmlElement(node.childNodes[offset], ['br', 'hr', 'img'])
        && !_zwDelIsCollapsedBlockProp(node.childNodes[offset])) {
      var selBlk = globalThis.getSelection();
      selBlk.collapse(node, offset);
      selBlk.extend(node, offset + 1);
      _zwDelDeleteSelection();
      return true;
    }

    var endNode = node;
    var endOffset = offset;

    if (endOffset < endNode.childNodes.length
        && _zwDelIsCollapsedBlockProp(endNode.childNodes[endOffset])) {
      endOffset++;
    }

    while (true) {
      if (endOffset == _zwJusNodeLength(endNode)) {
        if (!endNode.parentNode) break;
        endOffset = 1 + _zwJusNodeIndex(endNode);
        endNode = endNode.parentNode;
      } else if (endOffset < endNode.childNodes.length
          && _zwJusIsEditable(endNode.childNodes[endOffset])
          && _zwJusIsInvisible(endNode.childNodes[endOffset])) {
        endNode.removeChild(endNode.childNodes[endOffset]);
      } else {
        break;
      }
    }

    if (_zwJusIsHtmlElement(_zwDelChildAt(endNode, endOffset - 1), 'table')) return true;

    if (_zwJusIsHtmlElement(_zwDelChildAt(endNode, endOffset), 'table')) {
      var selTbl2 = globalThis.getSelection();
      selTbl2.collapse(endNode, endOffset);
      selTbl2.extend(endNode, endOffset + 1);
      return true;
    }

    if (offset == _zwJusNodeLength(node)
        && _zwJusIsHtmlElement(_zwDelChildAt(endNode, endOffset), ['br', 'hr'])) {
      var selHr = globalThis.getSelection();
      selHr.collapse(endNode, endOffset);
      selHr.extend(endNode, endOffset + 1);
      _zwDelDeleteSelection();
      var selHr2 = globalThis.getSelection();
      selHr2.collapse(node, offset);
      _zwVaGetActiveRange().collapse(true);
      return true;
    }

    while (endOffset < endNode.childNodes.length) {
      if (_zwJusIsEditable(endNode.childNodes[endOffset])
          && _zwJusIsInvisible(endNode.childNodes[endOffset])) {
        endNode.removeChild(endNode.childNodes[endOffset]);
      } else {
        endNode = endNode.childNodes[endOffset];
        endOffset = 0;
      }
    }

    var selFde = globalThis.getSelection();
    selFde.collapse(node, offset);
    selFde.extend(endNode, endOffset);
    _zwDelDeleteSelection();
    return true;
  }

  // childNodes 越界安全读（vendored childNodes[i] 越界返 undefined 同义）。
  function _zwDelChildAt(node, i) {
    if (!node || !node.childNodes || i < 0 || i >= node.childNodes.length) return null;
    return node.childNodes[i];
  }

  // —— execCommand delete/forwarddelete 入口（part06 单点调用）。活动选区非
  // collapsed → deleteSelection；collapsed → 对应 action。preservesOverrides
  // 等价面（vendored myExecCommand :8495 包裹）：动作前抓 override 原始快照
  // （彼时 stored==current，读面 reset 无效），动作后重锚 stored range 到删除
  // 后 caret 并整体回写——删除动作内部的值查询/状态读会在选区位移后触发
  // staleness reset 误清存储，qS/qV-after-delete 语料期望 override 存活。
  // typing 包裹态（__zwTypingTags）同面快照回写：动作内 caret 落位走 Selection
  // 公共面（collapse/extend/collapseToStart），_zwSync 对显式 selection 变更
  // 清 typing 态（part06 :10355，防跨子测泄漏语义）——Chrome delete 不清
  // typing style（g18 multitest 实证：值命令族 Fix A 已复、状态命令族仍红）。
  globalThis.__zw_ce_delete_apply = function (forward) {
    var ar = _zwVaGetActiveRange();
    if (!ar) return true;
    var snap = _zwVaOverridesTakeRaw();
    var tt = globalThis.__zwTypingTags;
    if (!ar.collapsed) {
      _zwDelDeleteSelection();
    } else if (forward) {
      _zwDelForwardDeleteAction();
    } else {
      _zwDelDeleteAction();
    }
    if (typeof _zwVaReanchorOverrides === 'function') _zwVaReanchorOverrides();
    _zwVaOverridesPutRaw(snap);
    globalThis.__zwTypingTags = tt;
    return true;
  };
