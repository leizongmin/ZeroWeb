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

    // Chrome delta（双壳终点边界捕获面，2026-11-03 gate28 轮）：等价点归一化
    // 会把链外 start 边界下钻进 inline 壳（(abc,3)→(div,1)→(b,0)→(i,0)→
    // (def,0)），keep-shell 与 caret 落位随之按壳内起算——Chrome 语义：原边界
    // 直下宿主（上行链无 inline 元素）时删除后空包裹全摘、caret 落原边界
    //（multitest :2521 族 `abc{<b><i>def</i></b>}ghi` recording 实证）。此处留
    // 原边界快照与链形判据，供 keep-shell 臂与 caret 落位臂使用；链内边界
    //（原链已有 inline 元素，T5 幸存壳面）行为不变。
    var origStartContainer = _zwVaGetActiveRange().startContainer;
    var origStartOffset = _zwVaGetActiveRange().startOffset;
    var origChainHasInline = false;
    var ocM = origStartContainer;
    while (ocM && ocM.parentNode && !_zwJusIsEditingHost(ocM)
        && _zwListInSameEditingHost(ocM, ocM.parentNode)) {
      if (ocM.nodeType == 1 && _zwJusIsInlineNode(ocM)) {
        origChainHasInline = true;
        break;
      }
      ocM = ocM.parentNode;
    }

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
      // Chrome delta（双壳终点边界捕获面）：链外边界整节点删空时，空文本节点
      // 连同空 inline 包裹一并摘除（multitest :2521 族 M1/M2/C02/C05 形——
      // Chrome 空包裹不存活，typed 段由 typing 态重建 `<b><i>def</i></b>`；
      // 链内边界 T5 面空壳由幸存壳承接，维持原路径不摘）。
      if (!origChainHasInline && dSame
          && !String(startNode.nodeValue || '') && startNode.parentNode) {
        var sameStrip = startNode.parentNode;
        sameStrip.removeChild(startNode);
        var sameGuard = 0;
        while (sameStrip && _zwJusIsEditable(sameStrip)
            && _zwJusIsInlineNode(sameStrip)
            && _zwJusNodeLength(sameStrip) == 0
            && sameStrip !== delKeepShell && sameGuard++ < 256) {
          var sameGp = sameStrip.parentNode;
          if (!sameGp) break;
          sameGp.removeChild(sameStrip);
          sameStrip = sameGp;
        }
      }
      var selSame = globalThis.getSelection();
      if (!origChainHasInline && selSame.rangeCount
          && origStartContainer.parentNode) {
        // caret 落原边界（链外语义）。存活门（gate28 multitest 回归修正，
        // 2026-11-03）：[foo<b>bar]</b>baz 形 start=(foo,0) 文本节点整含被删，
        // 前提「原边界节点在删除链外必存活」不成立——collapse 进 detached
        // 节点令 insertText 落空树（?8001-9000 22 断言 expected `abaz` got
        // `baz`；探针 g28-reg-probe.log post0 caret=3@0c DETACHED 实证）。
        // 不存活回落 direction 臂既有语义（pre-delta 全绿行为）。
        selSame.collapse(origStartContainer, Math.min(origStartOffset,
          _zwJusNodeLength(origStartContainer)));
        _zwVaGetActiveRange().collapse(true);
      } else if (direction == 'forward') {
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

    // Chrome 留壳预扫（helper 共享面见 _zwDelKeepShellOf 及其 g16-multitest
    // D1 实证注释）：幸存壳 = 最外层 <a>（无 a 时链上最外层壳）。链外边界
    //（原链无 inline 元素）无幸存壳——归一化 startNode 已下钻入壳，按壳内
    // 起算会错留空壳（:2521 族 M9 形 `abc{<b><i>def</i></b>g]hi` 期望壳全摘）。
    var delKeepShell = origChainHasInline
      ? _zwDelKeepShellOf(startNode) : null;

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
            && (_zwJusNodeLength(parent_) == 0
                // F2b 外壳剥离补真（2026-11-08 link-absorption 刀）：幸存壳
                // 为**空**时，仅含它的外层 inline 包裹按空摘——此前注释声称
                // 「置空计数」但 _zwJusNodeLength 无该特例，wrapper 含空 a 时
                // length=1 提前退出（zz-probe-f2 D2/D3 实证 got
                // `foo<u><a>a</a></u>baz` / `foo<font…><a…></font>`，语料
                // :2143/:2153/:2168 外壳剥离面 4 键红源）。Shape C（a 整含被
                // 删）keepShell 已离树、原条件自然成立，本支不触（现绿不动）；
                // keepShell 非空（部分删）不摘——Chrome 未录形，保守。
                || (delKeepShell && parent_.firstChild === delKeepShell
                    && parent_.childNodes.length === 1
                    && _zwJusNodeLength(delKeepShell) == 0))) {
          var grandparent = parent_.parentNode;
          // Chrome delta：链上幸存壳不摘（打字续壳）；幸存壳置空计数使其上方
          // 仅含它的包裹仍按空摘除（u>a 链：a 留、u 摘）。非链壳照 spec 摘。
          if (parent_ !== delKeepShell && grandparent) {
            if (delKeepShell && parent_.firstChild === delKeepShell
                && parent_.childNodes.length === 1) {
              // F2b：wrapper 唯一子即幸存壳 → 换位摘壳（keepShell 升至
              // wrapper 原位）——removeChild(parent_) 会连带摘走壳内空 link，
              // typed 段将无处落（probe D2 形）。
              grandparent.insertBefore(delKeepShell, parent_);
              grandparent.removeChild(parent_);
              // R262 换位吸附回填：壳移动触发 live-range pre-remove 调整
              //（边界容器 = 被摘壳子树根 → (oldParent, oldIndex) 吸附出壳，
              // 再经空壳 removeChild 二次吸附落宿主位）——caret 漂出幸存
              // link，typed 段落壳外（probe D2 got `</a>abaz` 形）。Chrome
              // 删除落定 caret 仍在幸存 link 内（:2143/:2153/:2168 期望
              // `foo<a…>a[]</a>baz`），活动 range 折回壳内（collapsed 落定由
              // 下方 exit 路径统一完成；空壳 length=0 → 折回 (壳,0)）。
              try {
                var srUw = _zwVaGetActiveRange();
                if (srUw) {
                  srUw.setStart(delKeepShell, 0);
                  srUw.setEnd(delKeepShell, _zwJusNodeLength(delKeepShell));
                }
              } catch (_eUw) {}
            } else {
              grandparent.removeChild(parent_);
            }
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
      if (!origChainHasInline && selExit.rangeCount
          && origStartContainer.parentNode) {
        // Chrome delta（双壳终点边界捕获面）：链外边界 caret 落原边界而非壳内
        // 重锚点——collapseToStart 经已摘壳 remap 产出 (b,0)/(i,0)（multitest
        // :2521 族 M4/M5/M9 形实证），Chrome 落 (abc,3)/(div,1) 原边界。
        // 存活门（gate28 multitest 回归修正，2026-11-03）：原边界节点整含被删
        // 时（[foo<X>bar]</X>baz 族 (foo,0) 形）不落 detached 节点，回落
        // direction 臂既有语义（pre-delta 全绿行为，最小风险恢复面）。
        selExit.collapse(origStartContainer, Math.min(origStartOffset,
          _zwJusNodeLength(origStartContainer)));
        _zwVaGetActiveRange().collapse(true);
      } else if (direction == 'forward') {
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

      // gate28 hang 修复（2026-11-03）：_zwListSplitParent 对 detached/碎片父
      // 静默早退（part01e.js:47-48，_zwJusIsEditable 对 detached/DocumentFragment
      // 均 false）+ _zwJusMovePreservingRanges 吞异常脱锚（part01c.js:653-663
      // 三重 try/catch）时本环零进展自旋——诊断笔实证首迭代即无进展、无任何
      // DOM 变更（g28-hang-diag.log 首变体 no-op）。进展守卫破环：合法迭代
      // parentNode 单调上移必变，守卫只在恒不进展态触发，回落继续走 children[0]
      // 现状（detached 时 :610-:612 条件不命中、尾段 restoreValues 按原对象
      // 恢复，安全域；与上方存活门同哲学：不进展不自旋）。
      var spinGuard = 0;
      while (children[0].parentNode != startBlock) {
        var spinPrev = children[0].parentNode;
        _zwListSplitParent(children);
        if (children[0].parentNode === spinPrev || ++spinGuard > 256) break;
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

  // Chrome 留壳预扫（deleteSelection 与 typing 链捕获共用）：原 start 链（编辑
  // 宿主为止）上的 inline 包裹，幸存壳 = 最外层 <a>（无 a 时链上最外层壳）。
  // g16-multitest D1 实证：
  //   foo<b>[bar</b>baz] → foo<b>a</b>（链上最外层 b 留）；
  //   foo<u><a>[bar</a></u>baz] → foo<a>a</a>（最外层 a 留、其上 u 摘）；
  //   foo<a><font>[bar</font></a>baz] → foo<a>a</a>（a 留、其下 font 摘）。
  function _zwDelKeepShellOf(startNode) {
    var inlineWrappers = [];
    var m = startNode;
    while (m && m.parentNode && !_zwJusIsEditingHost(m) && _zwJusIsInlineNode(m)
        && _zwListInSameEditingHost(m, m.parentNode)) {
      if (m.nodeType == 1) inlineWrappers.push(m);
      m = m.parentNode;
    }
    var keep = null;
    for (var wi = 0; wi < inlineWrappers.length; wi++) {
      if (_zwJusIsHtmlElement(inlineWrappers[wi], 'a')) keep = inlineWrappers[wi];
    }
    if (!keep && inlineWrappers.length) {
      keep = inlineWrappers[inlineWrappers.length - 1];
    }
    return keep;
  }

  // Chrome delta ⑤——delete 幸存壳 typing 链捕获（2026-11-02 editing-compat
  // TypingStyle 轮）：非 collapsed 删除后 caret 落幸存壳内或 Paragraph 拆出的
  // 新块时，Chrome typing style 携带原 start 链的 format 壳层。语料锚点：
  //   :2521 `<div>abc{<b><i>def</i></b>}ghi</div>`+delete+inserttext 期望
  //        `<b><i>def</i></b>`（typed 段重裹 i——幸存壳 b 由 caret 自然承接）；
  //   :2429 `<div><b>[abc]</b></div>`+delete+insertparagraph+inserttext 期望
  //        `<br><div><b>a</b></div>`（caret 离壳入新块，typing 态仍裹 b）；
  //   :2163 `foo<a><u>[bar]</u></a>baz` / :2168 `foo<u><a>[bar]</a></u>baz`
  //        均期望 `foo<a>a[]</a>`（链上有 <a> → typed 段裸奔，捕获置空）。
  // 捕获规则：自 startContainer 上行至编辑宿主收整链；链上任一 <a> → 空；
  // 否则内→外收集 format 族壳层（part06 _zwExecCmdFormatTag 同表 b/i/u/s/
  // sub/sup），行至幸存壳（含）止——幸存壳之上的层删除时已脱离 caret 继承
  // 链（:2168 u 在 a 上不涉、无 a 形 u>b 的 u 不捕）；记录序 = 外层在前
  //（_zwCeTypingInsert 包裹序）；显式 off 表（__zwTypingTagsOff，D4）中的
  // tag 不捕。消费侧由 part01 _zwCeTypingTagsAt 滤除 caret 祖先链既有层
  //（幸存壳自然承接不双裹）。显式 typing 态（__zwTypingTags 非空）优先，
  // 捕获仅在态空时补位——接线见 __zw_ce_delete_apply 尾部。
  var _zwDelFmtTags = { b: 1, i: 1, u: 1, s: 1, sub: 1, sup: 1 };

  // 选区内首个可见内容（可见文本或 img）节点级查找（链外终点捕获专用）：
  // 自 start 位下钻/右移行进，候选须被选区整含（GetPosition(start, head)
  // !== 'after' 即 head ≥ start；GetPosition(end, tail) !== 'before' 即
  // tail ≤ end——_zwDelGetPosition 返回首参相对次参方位），
  // 部分含节点跳过并继续行进（multitest :2660 族 C11 `abc[<b><i>def</i></b>g]hi`
  // 头部 abc 部分含 → 越过收 def 整含链）。inline 包裹（b/i 等）只作下钻
  // 通道不作候选。行进自带宿主边界与 guard 防护（正常 DOM 无环）。
  function _zwDelFirstVisibleInRange(ar) {
    var guard = 0;
    var n = ar.startContainer;
    var no = ar.startOffset;
    if (n.nodeType == 1 && no < n.childNodes.length) {
      n = n.childNodes[no];
      while (n.nodeType == 1 && n.firstChild && guard++ < 256) {
        n = n.firstChild;
      }
    } else if (n.nodeType == 3) {
      // 文本起点：整节点候选头界检查自会拒绝部分含（:2660 C38 形）。
    } else {
      // 元素位超界（no == childNodes.length）：上溯至有右兄弟的层。
      while (n && !n.nextSibling && n.parentNode
          && !_zwJusIsEditingHost(n.parentNode) && guard++ < 256) {
        n = n.parentNode;
      }
      n = n && n.nextSibling;
    }
    while (n && guard++ < 1024) {
      var candOk = (n.nodeType == 3 && _zwJusIsVisible(n))
        || _zwJusIsHtmlElement(n, 'img');
      if (candOk
          && _zwDelGetPosition(ar.startContainer, ar.startOffset, n, 0)
            !== 'after'
          && _zwDelGetPosition(ar.endContainer, ar.endOffset, n,
            _zwJusNodeLength(n)) !== 'before') {
        return n;
      }
      if (n.nodeType == 1 && n.firstChild) {
        n = n.firstChild;
        continue;
      }
      var nx = n.nextSibling;
      while (!nx && n.parentNode && !_zwJusIsEditingHost(n.parentNode)) {
        n = n.parentNode;
        nx = n.nextSibling;
      }
      n = nx;
    }
    return null;
  }

  // newBlock 头 inline 壳链 caret 入壳下行（knife11 v2 处方 b，2026-11-07）：
  // ip 拆分后新行首可见内容带 inline 壳时 Chrome 把 caret 落进壳链深处——空壳
  // `<b></b>` typed 段入壳填充（multitest 515/517/518），幸存壳
  // `<b><i>f</i></b>` typed 段入壳与幸存内容合流（529/539/545/547/553/555，
  // 语料注 "caret should be in the inline elements since the styled text is
  // the first visible content in the line"）。命中返回 [容器, 0]，否则
  // null（调用方保持 (newBlock, 0) 原位）。壳表复用 _zwDelFmtTags。
  globalThis.__zw_ce_caret_in_leading_shell = function (block) {
    var c = block && block.firstChild;
    if (!c || c.nodeType !== 1) return null;
    var tn = String(c.tagName || '').toLowerCase();
    if (!_zwDelFmtTags[tn]) return null;
    var n = c;
    var guard = 0;
    while (guard++ < 32) {
      var cc = n.firstChild;
      if (!cc || cc.nodeType !== 1) break;
      var t2 = String(cc.tagName || '').toLowerCase();
      if (!_zwDelFmtTags[t2]) break;
      n = cc;
    }
    return [n, 0];
  };

  globalThis.__zw_ce_capture_typing_chain = function (ar) {
    var sc = ar && ar.startContainer;
    if (!sc) return null;
    var keep = _zwDelKeepShellOf(sc);
    var chain = [];
    var hasA = false;
    var n = sc;
    var guard = 0;
    while (n && n.parentNode && !_zwJusIsEditingHost(n) && guard++ < 256) {
      if (n.nodeType == 1) {
        if (String(n.tagName || '').toLowerCase() === 'a') { hasA = true; break; }
        chain.push(n);
      }
      n = n.parentNode;
    }
    if (hasA) return [];
    var tags = [];
    for (var ci = 0; ci < chain.length; ci++) {
      var tn = String(chain[ci].tagName || '').toLowerCase();
      if (_zwDelFmtTags[tn]) tags.push(tn);
      if (chain[ci] === keep) break;
    }
    tags.reverse(); // 记录序 = 外层在前
    var toff = globalThis.__zwTypingTagsOff;
    if (Array.isArray(toff) && toff.length) {
      tags = tags.filter(function (t) { return toff.indexOf(t) < 0; });
    }
    if (tags.length || ar.collapsed) return tags;
    // Chrome delta（双壳终点边界捕获面，2026-11-03 gate28 轮）：链外边界
    //（上行链收空）时 typing 态改携选区内**首个可见内容**的 format 壳层——
    // 仅当其被选区整含。语料锚点（multitest :2521/:2660 族 Chrome recording）：
    //   :2521 `abc{<b><i>def</i></b>}ghi`+delete+inserttext → `<b><i>def</i></b>`
    //        （def 整含 → 捕获其 b+i 链，typed 段重裹）；
    //   :2660 `abc[<b><i>de]f</i></b>ghi`+delete+inserttext →
    //        `abcde<b><i>f</i></b>ghi`（def 部分含 → 不保留，typed 段裸奔）；
    //   img 首可见（:2660 C28 形）→ 不保留（期望 `abcdefghi`）。
    var cand = _zwDelFirstVisibleInRange(ar);
    if (!cand || cand.nodeType != 3) return tags;
    // 复用上行链捕获（collapsed 标记防再入扩展）；候选链有 <a> 时该调用自返
    // 空（:2163/:2168 双向实证语义不变）。
    return globalThis.__zw_ce_capture_typing_chain({
      startContainer: cand, startOffset: _zwJusNodeLength(cand),
      endContainer: cand, endOffset: _zwJusNodeLength(cand),
      collapsed: true
    });
  };

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
  // F2 link-absorption delta carry 捕获（2026-11-08 刀，预研档集成点 6/7）：
  // delete 动作前读 link 上下文相对参照的样式 delta，落 `__zwTypingLinkCarry`
  // 供 typing insert 消费（part01 _zwCeTypingInsert link 内层 font）。
  // carry = 删除内容相对 link 参照（Chrome UA blue/underline）的 delta——
  // 通道仅 color/face/size，永不含 decoration（:2163/:2168 u 双位形零 delta
  // 自然成立）。参照钉死 #0000ff 不读 computed（本仓 UA 样式表无 link 色规则
  // ——computed 读黑则 :1170 黑 carry 误判零 delta，待判别②闭合）。
  // 两形：① 物理链形（F2b :2138-:2172）——startContainer 上行至 linkEl 收
  // 内层壳 color/face/size 声明，色经 parseSimpleColor 归一（≡#0000ff 零
  // delta 不捕，:2138/:2139 blue 现绿保持）；② armed pending collapsed 形
  //（F2a :1170 `createlink>delete>inserttext`）——无物理链，读 caret 上下文
  // computed 色（_zwValEffectiveValue 'forecolor'）vs 参照。外链弃捕语义
  //（链上行遇 <a> 前无内层壳则 carry 自然为空）与既有
  // __zw_ce_capture_typing_chain hasA 面（tags 捕获置空）互不相干。
  // 保守边界：物理链 collapsed 形不捕（Chrome 未录形，避免既有绿行 P2F）；
  // armed 非 collapsed 形按 startContainer 同 ② 读（语料无行，语义同源）。
  globalThis.__zw_ce_capture_link_carry = function (ar) {
    var sc = ar && ar.startContainer;
    if (!sc) return null;
    var armed = globalThis.__zwTypingLink != null && globalThis.__zwTypingLink !== ''
      ? String(globalThis.__zwTypingLink) : null;
    var linkEl = null;
    var n = sc;
    var guard = 0;
    while (n && n.parentNode && !_zwJusIsEditingHost(n) && guard++ < 256) {
      if (n.nodeType == 1 && String(n.tagName || '').toLowerCase() === 'a') {
        linkEl = n;
        break;
      }
      n = n.parentNode;
    }
    var carry = null;
    if (linkEl) {
      // ① 内层壳声明面：sc → linkEl 之间 color/face/size，色归一 hex。
      var m = sc;
      guard = 0;
      while (m && m !== linkEl && guard++ < 512) {
        if (m.nodeType == 1 && m.getAttribute) {
          var ca = m.getAttribute('color');
          if (ca != null && typeof _zwVaParseSimpleColor === 'function') {
            var hexc = _zwVaParseSimpleColor(ca);
            if (hexc && hexc !== '#0000ff') {
              carry = carry || {};
              carry.color = hexc;
            }
          }
          var fa = m.getAttribute('face');
          if (fa != null) {
            carry = carry || {};
            carry.face = String(fa);
          }
          var sa = m.getAttribute('size');
          if (sa != null) {
            carry = carry || {};
            carry.size = String(sa);
          }
        }
        m = m.parentNode;
      }
      return carry;
    }
    if (!armed) return null;
    // ② armed pending 形：caret 上下文 computed 色 vs 参照 #0000ff。
    if (typeof _zwValEffectiveValue === 'function') {
      var eff = _zwValEffectiveValue(sc, 'forecolor');
      var hexc2 = (typeof _zwVaParseSimpleColor === 'function')
        ? _zwVaParseSimpleColor(eff) : null;
      if (hexc2 && hexc2 !== '#0000ff') {
        carry = { color: hexc2 };
      }
    }
    return carry;
  };

  globalThis.__zw_ce_delete_apply = function (forward) {
    var ar = _zwVaGetActiveRange();
    if (!ar) return true;
    var snap = _zwVaOverridesTakeRaw();
    var tt = globalThis.__zwTypingTags;
    var ttoff = globalThis.__zwTypingTagsOff; // D4 显式关账表同快照
    // delta ⑤ 幸存壳 typing 链捕获：动作前读 start 链（删后链形已变）；
    // 仅非 collapsed 面（deleteSelection 留壳路径；collapsed 动作无留壳语义）。
    var tcap = (!ar.collapsed && typeof globalThis.__zw_ce_capture_typing_chain === 'function')
      ? globalThis.__zw_ce_capture_typing_chain(ar) : null;
    // F2 link carry 捕获 + pending link 快照：link 幸存 delete（:1170 面），
    // armed href 槽不随动作清（capture 内部动作前读，删后链形已变）；carry
    // 新捕获优先，无捕获保持原值（同 delta ④ 回写式）。
    var snapLink = globalThis.__zwTypingLink;
    var snapCarry = globalThis.__zwTypingLinkCarry;
    var lcap = (typeof globalThis.__zw_ce_capture_link_carry === 'function')
      ? globalThis.__zw_ce_capture_link_carry(ar) : null;
    if (!ar.collapsed) {
      _zwDelDeleteSelection();
    } else if (forward) {
      _zwDelForwardDeleteAction();
    } else {
      _zwDelDeleteAction();
    }
    if (typeof _zwVaReanchorOverrides === 'function') _zwVaReanchorOverrides();
    _zwVaOverridesPutRaw(snap);
    // delta ④ 回写 + delta ⑤ 捕获补位：显式 typing 态存活优先（g18/g19 面），
    // 态空且捕获非空时落捕获链（Chrome typing style 跨 delete 存续——:2521
    // 双壳 / :2429 ipara 拆分块两形）。态与捕获俱空保持原值（null）。
    globalThis.__zwTypingTags = (tt && tt.length) ? tt
      : (tcap && tcap.length ? tcap : tt);
    globalThis.__zwTypingTagsOff = ttoff;
    globalThis.__zwTypingLink = snapLink;
    globalThis.__zwTypingLinkCarry = (lcap && lcap.color != null)
      || (lcap && lcap.face != null) || (lcap && lcap.size != null)
      ? lcap : snapCarry;
    return true;
  };

  // —— insertParagraph action 忠实移植（2026-11-02 editing-compat：ipara
  // other-cmp 153 嵌套面；参考 implementation.js :7486-7826）——
  // deleteSelection 前导（collapsed 即中止）→ 文本归一三段（splitText / 尾偏移
  // 升父 / 文本注释升父）→ 单行容器爬升（p/div 再外爬 dd/dt/li）→ 三支路
  //（非 SLC blockExtend+wrap / pre 族 br / 空 li dt dd splitParent）→ 主拆分支
  //（newLineRange 归一提边、endOfLine、newContainer 命名 H1-6→sep / dt⇄dd、
  // 属性拷贝除 id、insertAfter、extract、frag 非相含后代摘 id、appendChild、
  // 尾段禁段落子下钻双面 br 垫底、collapse 新块 0）。事件序由 part06 execCommand
  // 臂派发，此处仅变更（同 __zw_ce_delete_apply 口径）。
  // newLineRange 的 extract 用本仓 spec 递归实现（_zwIparaExtractRange）：shim
  // Range.extractContents 跨元素端界缺口（zz-probe-d4x X1-X3 实证——start 点
  // 后随兄弟不迁移）本轮不修、挂账独立轮；moveIn 以 R241 同款 append 后残留
  // 强制摘离兜底 shim fragment appendChild 移动语义缺环（X4）。
  // https://w3c.github.io/editing/execCommand.html#the-insertparagraph-command

  // void 元素表（part01.js `_zwCeIsVoidElementTag` 同表——该文件 IIFE 私有作用
  // 域不可达，本地复制；insert-paragraph/text-in-void 语料 voidElements 全集）。
  function _zwIparaIsVoidTag(el) {
    var t = String((el && el.tagName) || '').toLowerCase();
    return ['br', 'img', 'hr', 'input', 'embed', 'wbr'].indexOf(t) >= 0;
  }

  // display 读数（part01.js `_zwCeDisplayOf` 同构——IIFE 私有不可达，本地复制：
  // style 面板优先（wss 语料经 style IDL 设置），attr 扫描兜底）。
  function _zwIparaDisplayOf(el) {
    try {
      if (el.style && typeof el.style.display === 'string'
          && el.style.display !== '') {
        return String(el.style.display);
      }
    } catch (_eIpD1) {}
    try {
      var stDisp = el.getAttribute && el.getAttribute('style');
      if (stDisp) {
        var mDisp = String(stDisp).match(/display\s*:\s*([a-z-]+)/i);
        if (mDisp) return mDisp[1];
      }
    } catch (_eIpD2) {}
    return '';
  }

  function _zwIparaIsSingleLineContainer(node) {
    // "A single-line container is either a non-list single-line container, or
    // an HTML element with local name li, dt, or dd."
    // https://w3c.github.io/editing/execCommand.html#single-line-container
    return _zwJusIsHtmlElement(node, ['address', 'div', 'h1', 'h2', 'h3', 'h4',
        'h5', 'h6', 'listing', 'p', 'pre', 'xmp'])
      || _zwJusIsHtmlElement(node, ['li', 'dt', 'dd']);
  }

  function _zwIparaDeleteData(node, off, count) {
    var d = String(node.nodeValue == null ? '' : node.nodeValue);
    try { node.nodeValue = d.slice(0, off) + d.slice(off + count); } catch (_eDd) {}
  }

  function _zwIparaHasVisibleChild(el) {
    var ks = (el && el.childNodes) || [];
    for (var i = 0; i < ks.length; i++) {
      if (_zwJusIsVisible(ks[i])) return true;
    }
    return false;
  }

  function _zwIparaGetDescendants(root) {
    var out = [];
    (function walk(n) {
      var ks = (n && n.childNodes) || [];
      for (var i = 0; i < ks.length; i++) { out.push(ks[i]); walk(ks[i]); }
    })(root);
    return out;
  }

  // spec dom-range-extractcontents 递归版：frag 结构 [firstPartial 克隆组树,
  // contained 子 move, lastPartial 克隆组树]；partially contained 元素子经
  // 子区间递归承接任意深壳层。移动走 moveIn（append 后源父 childNodes 仍残留
  // 则强制 removeChild——shim fragment appendChild 只改 wire 不摘源列表，X4）。
  // https://dom.spec.whatwg.org/#dom-range-extractcontents
  function _zwIparaExtractRange(rng) {
    var od = null;
    try { od = rng.startContainer.ownerDocument
      || (rng.startContainer.nodeType === 9 ? rng.startContainer : null); } catch (_eOd) {}
    var f = (od && typeof od.createDocumentFragment === 'function')
      ? od.createDocumentFragment() : document.createDocumentFragment();
    if (!rng || !rng.startContainer || !rng.endContainer) return f;
    var sc = rng.startContainer, ec = rng.endContainer;
    var so = rng.startOffset | 0, eo = rng.endOffset | 0;
    var isCd = function (n) {
      var t = n ? n.nodeType : 0;
      return t === 3 || t === 4 || t === 7 || t === 8;
    };
    var moveIn = function (kid, into) {
      if (!kid) return;
      var op = kid.parentNode;
      var ok = (op && op.childNodes) ? op.childNodes : null;
      try { into.appendChild(kid); } catch (_eMv) {}
      try {
        if (ok && ok.indexOf(kid) >= 0 && op !== into
            && typeof op.removeChild === 'function') op.removeChild(kid);
      } catch (_eMr) {}
    };
    var cloneSlice = function (n, a, b) {
      var c = n.cloneNode(false);
      var d = String(n.nodeValue == null ? '' : n.nodeValue);
      try { c.nodeValue = d.slice(a, b); } catch (_eCs) {}
      return c;
    };
    if (sc === ec) {
      var a = Math.min(so, eo), b = Math.max(so, eo);
      if (isCd(sc)) {
        if (b > a) {
          f.appendChild(cloneSlice(sc, a, b));
          _zwIparaDeleteData(sc, a, b - a);
        }
      } else {
        var snap = [].slice.call(sc.childNodes || []).slice(a, b);
        for (var i = 0; i < snap.length; i++) {
          if (snap[i].nodeType === 10) {
            throw new (globalThis.DOMException || Error)(
              'The range includes a DocumentType node.', 'HierarchyRequestError');
          }
          if (isCd(snap[i])) {
            var d = String(snap[i].nodeValue == null ? '' : snap[i].nodeValue);
            f.appendChild(cloneSlice(snap[i], 0, d.length));
            _zwIparaDeleteData(snap[i], 0, d.length);
          } else {
            // 先摘后挂：shim fragment appendChild 只改 wire 父不摘源 childNodes
            //（zz-probe-d4x X4），先 removeChild 消除双亲窗口再入 frag。
            try {
              if (snap[i].parentNode === sc
                  && typeof sc.removeChild === 'function') sc.removeChild(snap[i]);
            } catch (_eRm0) {}
            moveIn(snap[i], f);
          }
        }
      }
      try { rng.setStart(sc, a); rng.setEnd(sc, a); } catch (_eCl1) {}
      return f;
    }
    // cac：sc 包含链上首个同时包含 ec 的容器。
    var cac = null, cur = sc, hops = 0;
    while (cur && hops++ < 256 && !cac) {
      var probe = ec, h = 0;
      while (probe && h++ < 256) {
        if (probe === cur) { cac = cur; break; }
        probe = probe.parentNode;
      }
      cur = cur.parentNode;
    }
    if (!cac) return f;
    var childOnPath = function (node) {
      var n = node;
      while (n && n.parentNode !== cac) n = n.parentNode;
      return n;
    };
    var firstPartial = (sc === cac) ? null : childOnPath(sc);
    var lastPartial = (ec === cac) ? null : childOnPath(ec);
    var contained = [];
    var cks = [].slice.call(cac.childNodes || []);
    for (var ci = 0; ci < cks.length; ci++) {
      if (_zwJusIsContained(cks[ci], rng)) contained.push(cks[ci]);
    }
    if (firstPartial && isCd(firstPartial)) {
      var dsc = String(sc.nodeValue == null ? '' : sc.nodeValue);
      f.appendChild(cloneSlice(sc, so, dsc.length));
      _zwIparaDeleteData(sc, so, dsc.length - so);
    } else if (firstPartial) {
      var fc = firstPartial.cloneNode(false);
      f.appendChild(fc);
      var sub1 = document.createRange();
      sub1.setStart(sc, so);
      sub1.setEnd(firstPartial, _zwJusNodeLength(firstPartial));
      fc.appendChild(_zwIparaExtractRange(sub1));
    }
    for (var k = 0; k < contained.length; k++) {
      if (contained[k].nodeType === 10) {
        throw new (globalThis.DOMException || Error)(
          'The range includes a DocumentType node.', 'HierarchyRequestError');
      }
      if (isCd(contained[k])) {
        var d2 = String(contained[k].nodeValue == null ? '' : contained[k].nodeValue);
        f.appendChild(cloneSlice(contained[k], 0, d2.length));
        _zwIparaDeleteData(contained[k], 0, d2.length);
      } else {
        try {
          if (contained[k].parentNode === cac
              && typeof cac.removeChild === 'function') cac.removeChild(contained[k]);
        } catch (_eRm3) {}
        moveIn(contained[k], f);
      }
    }
    if (lastPartial && isCd(lastPartial)) {
      f.appendChild(cloneSlice(ec, 0, eo));
      _zwIparaDeleteData(ec, 0, eo);
    } else if (lastPartial) {
      var lc = lastPartial.cloneNode(false);
      var sub2 = document.createRange();
      sub2.setStart(lastPartial, 0);
      sub2.setEnd(ec, eo);
      lc.appendChild(_zwIparaExtractRange(sub2));
      f.appendChild(lc);
    }
    var co = firstPartial ? _zwJusNodeIndex(firstPartial) : so;
    try { rng.setStart(cac, co); rng.setEnd(cac, co); } catch (_eCl2) {}
    return f;
  }

  // spec dom-range-insertnode（insertparagraph 非 SLC/pre 支路用）：端点在
  // CharData 内先边界化（0 前插 / len 后插 / 中点 splitText），插入后 start/end
  // 置 (parent, node index + 1)。
  // https://dom.spec.whatwg.org/#dom-range-insertnode
  function _zwIparaInsertNode(rng, node) {
    if (!rng || !rng.startContainer || !node) return;
    var sc = rng.startContainer, so = rng.startOffset | 0;
    var parent, ref;
    if (sc.nodeType === 3 || sc.nodeType === 4 || sc.nodeType === 8) {
      parent = sc.parentNode;
      var d = String(sc.nodeValue == null ? '' : sc.nodeValue);
      if (so === 0) {
        ref = sc;
      } else if (so >= d.length) {
        ref = sc.nextSibling;
      } else {
        var right = null;
        try { right = sc.splitText(so); } catch (_eSp) {}
        ref = (right && right.parentNode === parent) ? right : sc.nextSibling;
      }
    } else {
      parent = sc;
      ref = (sc.childNodes && sc.childNodes[so]) || null;
    }
    if (!parent) return;
    try { parent.insertBefore(node, ref); } catch (_eIn) {}
    var idx = _zwJusNodeIndex(node);
    try { rng.setStart(parent, idx + 1); rng.setEnd(parent, idx + 1); } catch (_eRs) {}
  }

  // 混合派发预检（2026-11-02 editing-compat）：本仓约定 `_zwJusIsEditable`
  // 对编辑宿主本体返 false，上方忠实移植的 SLC 爬升到宿主边界即停——caret
  // 直挂宿主（裸文本/inline 链/元素偏移）时 container 停在非 SLC 节点，落
  // spec wrap 臂产出 `<div>整段</div><div><br></div>` 形，而 Chrome/F2 语义
  // 为宿主内双块拆分（`foo[]bar` → `<div>foo</div><div>bar</div>`）。故
  // 宿主边界面回落旧 `__zw_ce_paragraph_split_at`（gate21 已知绿：echd/
  // wss/void/edob/restoration 各面），嵌套单行容器面（p/div/h1-h6/li/dt/
  // dd/pre 宿主内嵌套，移植 +244 F→P 来源）保留忠实移植。只读预检不
  // mutate；异常一律返 false 走 legacy。
  globalThis.__zw_ce_ipara_wants = function () {
    try {
      var ar = _zwVaGetActiveRange();
      if (!ar) return false;
      // 镜像 spec 文本归一（只读：中点 splitText 后爬升起点不变，跳过）
      var node = ar.startContainer;
      var offset = ar.startOffset;
      if (node.nodeType === 3 && offset != 0 && offset != _zwJusNodeLength(node)) {
        // 中点：保持文本偏移（与 apply 弃 splitText 后的归一对齐）
      } else if (node.nodeType === 3 && offset == _zwJusNodeLength(node)) {
        offset = 1 + _zwJusNodeIndex(node);
        node = node.parentNode;
      }
      if (node.nodeType === 8
          || (node.nodeType === 3 && offset === 0)) {
        offset = _zwJusNodeIndex(node);
        node = node.parentNode;
      }
      if (!node || !node.nodeType) return false;
      var container = node;
      while (!_zwIparaIsSingleLineContainer(container)
          && container.parentNode
          && _zwJusIsEditable(container.parentNode)
          && _zwListInSameEditingHost(node, container.parentNode)) {
        container = container.parentNode;
      }
      // 接管条件：container 是宿主内**元素**。本仓约定下
      // `_zwJusIsEditable(container)` 即「严格位于编辑宿主内」——宿主本体
      // 返 false 自然落 legacy。裸文本面（caret 直挂宿主文本）时 walk 被
      // 同一约定卡住停在 TEXT 节点（nodeType 3 → legacy，Chrome/F2 宿主内
      // 双块拆分）；元素 container 两态皆归移植：嵌套单行容器走 SLC 主
      // 分支，宿主直子 inline 链（`<b>foo[]bar</b>`）走非 SLC wrap 臂产
      // Chrome `<div><b>foo</b></div>` 形（探针 v4 P3 定谳，SLC 判定对
      // 路由非必要）。p/div→dd/dt/li 外爬不改变元素/可编辑判定。
      if (!(container && container.nodeType === 1
          && _zwJusIsEditable(container)
          && _zwListInSameEditingHost(container, node))) {
        return false;
      }
      // 三类移植不辖区（gate24 other 对账残差面）回落 legacy——旧
      // `__zw_ce_paragraph_split_at` 的退化/wss/void 臂已实证 Chrome 绿：
      // ① caret 挂 void 元素（ipiv 语料 hr/embed 形，移植 wrap 臂产 div 壳）；
      // ② 宿主本体是不可容块的行内容器（echd 语料 p/h1-6 宿主，Chrome 退化
      //    `<br>` 插入而非块拆；div 宿主可容块不在此列）；
      // ③ 宿主带 white-space 声明或 caret 祖先链 display:inline（wss 语料
      //    host 样式继承/display:inline 块两族，移植 wrap 臂丢样式继承）。
      if (_zwIparaIsVoidTag(container) || _zwIparaIsVoidTag(node)) {
        return false;
      }
      var ipHost = _zwListEditingHostOf(node);
      if (ipHost) {
        if (_zwIparaIsSingleLineContainer(ipHost)
            && _zwJusTag(ipHost) !== 'div') {
          return false;
        }
        // 宿主 white-space 声明（style 面板优先——wss 语料经 style IDL 设置，
        // attr 为空；attr 扫描兜底同 part01 wss 簇口径）。
        var ipWs = null;
        try {
          if (ipHost.style && typeof ipHost.style.whiteSpace === 'string'
              && ipHost.style.whiteSpace !== '') {
            ipWs = ipHost.style.whiteSpace;
          }
        } catch (_eWs) {}
        if (!ipWs) {
          var ipHostStyle = String(
            (ipHost.getAttribute && ipHost.getAttribute('style')) || '');
          var mWs = ipHostStyle.match(/white-space\s*:\s*([a-z-]+)/i);
          if (mWs) ipWs = mWs[1];
        }
        if (ipWs) {
          return false;
        }
        // caret 祖先链 display:inline / inline-block（wss 语料两族；
        // _zwCeDisplayOf 面板优先 + attr 兜底）。
        for (var ipAnc = node; ipAnc && ipAnc !== ipHost; ipAnc = ipAnc.parentNode) {
          if (ipAnc.nodeType !== 1) continue;
          var ipDisp = _zwIparaDisplayOf(ipAnc);
          if (ipDisp === 'inline' || ipDisp === 'inline-block') {
            return false;
          }
        }
      }
      return true;
    } catch (_eWants) {
      return false;
    }
  };

  globalThis.__zw_ce_ipara_apply = function () {
    // separator 缺省 'p'（第十一刀，见 part01 ① sepTag 刀注：产线块面双面制，
    // 值面 qV 保持 div）。
    var sep = (globalThis.__zwDefaultParagraphSeparator === 'div') ? 'div' : 'p';
    // preservesOverrides: true——override 快照/重锚/回填同 __zw_ce_delete_apply。
    var snap = _zwVaOverridesTakeRaw();
    // delta ④ typing 快照（__zw_ce_delete_apply 同构）：动作内 Selection
    // collapse 经公共面清 typing 态（part06 :10355 防跨子测泄漏语义），动作后
    // 回写——Chrome insertParagraph 不清 typing style。
    var tt = globalThis.__zwTypingTags;
    var ttoff = globalThis.__zwTypingTagsOff;
    try {
      // "Delete the selection."（collapsed 时 spec 中止——不 canonicalize、不删）
      var ar0 = _zwVaGetActiveRange();
      if (!ar0) return true;
      var _zwIpCol = (ar0.startContainer === ar0.endContainer
        && ar0.startOffset === ar0.endOffset) || !!ar0.collapsed;
      // delta ⑤ 幸存壳 typing 链捕获（同 __zw_ce_delete_apply）：仅非 collapsed
      // 面（deleteSelection 留壳路径）动作前读 start 链（删后链形已变）；
      // collapsed 动作无留壳语义。
      var tcap = (!_zwIpCol && typeof globalThis.__zw_ce_capture_typing_chain === 'function')
        ? globalThis.__zw_ce_capture_typing_chain(ar0) : null;
      // 处方 c 容器面（Chrome collapsed→separator / range→container-copy 规则，
      // multitest 495/497/498 语料双源录音面）：非 collapsed（delete 前导）拆分
      // 的 wrap 臂块标签复制编辑宿主标签，collapsed 保持 separator 缺省。
      // 容器复制仅适用于 separator 未设置态（双面制 unset 面）——显式 setter
      // 一票胜出不翻置：insertparagraph 语料 td 跨格族（explicit sep p/div，
      // ip 腿 v2 门禁 P2F×2 定谳）录音期望 `td><p><br></p><p>{}<br></p></td>`
      // 分块随显式 separator，宿主标签复制会翻成 div 形违录音。
      var _zwIpWrapTag = null;
      if (!_zwIpCol
          && globalThis.__zwDefaultParagraphSeparator !== 'p'
          && globalThis.__zwDefaultParagraphSeparator !== 'div') {
        var _zwIpHost = _zwListEditingHostOf(ar0.startContainer);
        _zwIpWrapTag = _zwIpHost ? _zwJusTag(_zwIpHost) : 'div';
      }
      if (!_zwIpCol) _zwDelDeleteSelection();
      // "If the active range's start node is neither editable nor an editing
      // host, return true."
      ar0 = _zwVaGetActiveRange();
      if (!ar0) return true;
      if (!_zwJusIsEditable(ar0.startContainer)
          && !_zwJusIsEditingHost(ar0.startContainer)) {
        return true;
      }
      // "Let node and offset be the active range's start node and offset." +
      // 文本归一。中点 splitText 臂不移植：shim 文本包装层 splitText 后
      // selection 重锚错位（trace 实证 (textL,2)→(div,0)）且 .data 双态
      // 陈旧（clone 读旧串、定位读真树，语料实证 `abcc` 翻倍）。mid-text
      // 起点由 _zwIparaExtractRange first-partial CharData 路径（cloneSlice+
      // deleteData）与 _zwIparaInsertNode 原生消化，语义等价 spec 归一。
      var node = ar0.startContainer;
      var offset = ar0.startOffset;
      if (node.nodeType === 3 && offset == _zwJusNodeLength(node)) {
        offset = 1 + _zwJusNodeIndex(node);
        node = node.parentNode;
      }
      if (node.nodeType === 8
          || (node.nodeType === 3 && offset === 0)) {
        // spec 原臂（text/comment→parent@index）以中点 splitText 消解为前提；
        // 本仓不 split，mid-text 须保持文本偏移，仅 comment/0 位文本提升。
        offset = _zwJusNodeIndex(node);
        node = node.parentNode;
      }
      try { globalThis.getSelection().collapse(node, offset); } catch (_eCl) {}
      // "Let container equal node." + 爬升至单行容器。
      var container = node;
      while (!_zwIparaIsSingleLineContainer(container)
          && container.parentNode
          && _zwJusIsEditable(container.parentNode)
          && _zwListInSameEditingHost(node, container.parentNode)) {
        container = container.parentNode;
      }
      // p/div 单行容器再外爬 dd/dt/li。
      if (_zwJusIsEditable(container)
          && _zwIparaIsSingleLineContainer(container)
          && container.parentNode
          && _zwListInSameEditingHost(node, container.parentNode)
          && (_zwJusTag(container) === 'p' || _zwJusTag(container) === 'div')) {
        var outerContainer = container;
        while (!_zwJusIsHtmlElement(outerContainer, ['dd', 'dt', 'li'])
            && outerContainer.parentNode
            && _zwJusIsEditable(outerContainer.parentNode)) {
          outerContainer = outerContainer.parentNode;
        }
        if (_zwJusIsHtmlElement(outerContainer, ['dd', 'dt', 'li'])) {
          container = outerContainer;
        }
      }
      // 非 SLC 支路：blockExtend + 首个 p 允许子 wrap。
      if (!_zwJusIsEditable(container)
          || !_zwListInSameEditingHost(container, node)
          || !_zwIparaIsSingleLineContainer(container)) {
        var newRange = _zwJusBlockExtend(_zwVaGetActiveRange());
        var nodeList = _zwJusGetContainedNodes(newRange, function (n) {
          return _zwJusIsAllowedChild(n, 'p');
        }).slice(0, 1);
        if (!nodeList.length) {
          if (!_zwJusIsAllowedChild(sep,
              _zwJusTag(_zwVaGetActiveRange().startContainer))) {
            return true;
          }
          container = document.createElement(_zwIpWrapTag || sep);
          _zwIparaInsertNode(_zwVaGetActiveRange(), container);
          container.appendChild(document.createElement('br'));
          try { globalThis.getSelection().collapse(container, 0); } catch (_eCl2) {}
          return true;
        }
        while (nodeList[nodeList.length - 1].nextSibling
            && _zwJusIsAllowedChild(nodeList[nodeList.length - 1].nextSibling, 'p')) {
          nodeList.push(nodeList[nodeList.length - 1].nextSibling);
        }
        container = _zwJusWrap(nodeList, function () { return false; },
          function () { return document.createElement(_zwIpWrapTag || sep); });
        if (!container) return true;
      }
      // pre/address/listing 支路：br 插入 + 非"br 后随即块尾"双 br 垫底。
      if (_zwJusTag(container) === 'address' || _zwJusTag(container) === 'listing'
          || _zwJusTag(container) === 'pre') {
        var br = document.createElement('br');
        _zwIparaInsertNode(_zwVaGetActiveRange(), br);
        try { globalThis.getSelection().collapse(node, offset + 1); } catch (_eCl3) {}
        var nxt = _zwJusNextNode(br);
        if (!nxt || !_zwJusIsAncestorOf(container, nxt)) {
          _zwIparaInsertNode(_zwVaGetActiveRange(), document.createElement('br'));
          try { globalThis.getSelection().collapse(node, offset + 1); } catch (_eCl4) {}
        }
        return true;
      }
      // 空 li/dt/dd 支路：splitParent + br 垫底 + dd/dt 越界改名 + 祖先修复。
      if (_zwJusIsHtmlElement(container, ['li', 'dt', 'dd'])
          && (!_zwJusHasKids(container)
            || (container.childNodes.length === 1
              && _zwJusIsHtmlElement(container.firstChild, 'br')))) {
        _zwListSplitParent([container]);
        if (!_zwJusHasKids(container)) {
          container.appendChild(document.createElement('br'));
        }
        if (_zwJusIsHtmlElement(container, ['li', 'dt', 'dd'])) {
          // li/dt/dd 同宿主祖先无 allowed 容身位（li 仅 ol/ul、dd/dt 仅 dl）→
          // 改名默认单行容器（splitParent 弃列后宿主直挂形；vendored
          // fixDisallowedAncestors 的 while 臂在本仓 isEditable 宿主排除约定
          // 下不触发，此处按浏览器语义同宿主走查补位）。
          var anc = container, allBad = true;
          while ((anc = anc.parentNode)) {
            if (_zwListInSameEditingHost(container, anc)
                && _zwJusIsAllowedChild(container, _zwJusTag(anc))) {
              allBad = false;
              break;
            }
          }
          if (allBad) container = _zwJusSetTagName(container, sep);
        }
        _zwListFixDisallowedAncestors(container);
        return true;
      }
      // 主拆分支：newLineRange 归一提边（0 位升父 / 尾位升父，遇禁段落子止）。
      var arMain = _zwVaGetActiveRange();
      var newLineRange = document.createRange();
      newLineRange.setStart(arMain.startContainer, arMain.startOffset);
      newLineRange.setEnd(container, _zwJusNodeLength(container));
      while (newLineRange.startOffset == 0
          && newLineRange.startContainer.parentNode
          && !_zwListIsProhibitedParagraphChild(newLineRange.startContainer)) {
        newLineRange.setStart(newLineRange.startContainer.parentNode,
          _zwJusNodeIndex(newLineRange.startContainer));
      }
      while (newLineRange.startOffset == _zwJusNodeLength(newLineRange.startContainer)
          && newLineRange.startContainer.parentNode
          && !_zwListIsProhibitedParagraphChild(newLineRange.startContainer)) {
        newLineRange.setStart(newLineRange.startContainer.parentNode,
          1 + _zwJusNodeIndex(newLineRange.startContainer));
      }
      var containedInNewLineRange = _zwJusCollectContainedNodes(newLineRange, null, false);
      // spec 的 endOfLine 以 splitText 后尾段成整 contained 节点为前提；本仓
      // 不 split，起点在文本中段即有尾随内容（spec 下尾段必为 contained，
      // whitespace 与否同判），不得误判行尾触发 h1-6/dt/dd 改名。
      var startInTail = newLineRange.startContainer.nodeType === 3
        && newLineRange.startOffset < _zwJusNodeLength(newLineRange.startContainer);
      var endOfLine = !startInTail
        && (!containedInNewLineRange.length
          || (containedInNewLineRange.length === 1
            && _zwJusIsHtmlElement(containedInNewLineRange[0], 'br')));
      var newContainerName;
      if (/^h[1-6]$/.test(_zwJusTag(container)) && endOfLine) {
        newContainerName = sep;
      } else if (_zwJusTag(container) === 'dt' && endOfLine) {
        newContainerName = 'dd';
      } else if (_zwJusTag(container) === 'dd' && endOfLine) {
        newContainerName = 'dt';
      } else {
        newContainerName = _zwJusTag(container);
      }
      var newContainer = document.createElement(newContainerName);
      var attrs = container.attributes || [];
      for (var ai = 0; ai < attrs.length; ai++) {
        try { newContainer.setAttribute(attrs[ai].name, attrs[ai].value); } catch (_eAt) {}
      }
      try { newContainer.removeAttribute('id'); } catch (_eRm1) {}
      container.parentNode.insertBefore(newContainer, container.nextSibling);
      // 相含节点集先于 extract 记账（extract 后 frag 克隆与原树身份分叉）。
      var containedNodes = _zwJusCollectContainedNodes(newLineRange, null, false);
      var frag = _zwIparaExtractRange(newLineRange);
      // frag 中非相含后代（克隆壳）摘 id。
      var descendants = _zwIparaGetDescendants(frag);
      for (var di = 0; di < descendants.length; di++) {
        if (descendants[di].nodeType === 1
            && containedNodes.indexOf(descendants[di]) === -1) {
          try { descendants[di].removeAttribute('id'); } catch (_eRm2) {}
        }
      }
      newContainer.appendChild(frag);
      // 双面禁段落子下钻 + 不可见面 br 垫底。
      while (_zwListIsProhibitedParagraphChild(container.lastChild)) {
        container = container.lastChild;
      }
      while (_zwListIsProhibitedParagraphChild(newContainer.lastChild)) {
        newContainer = newContainer.lastChild;
      }
      if (!_zwIparaHasVisibleChild(container)) {
        container.appendChild(document.createElement('br'));
      }
      if (!_zwIparaHasVisibleChild(newContainer)) {
        newContainer.appendChild(document.createElement('br'));
      }
      // 处方 b：caret 入壳下行（新块头 inline 壳链，见
      // __zw_ce_caret_in_leading_shell 注）。
      var _zwIpShell = (typeof globalThis.__zw_ce_caret_in_leading_shell === 'function')
        ? globalThis.__zw_ce_caret_in_leading_shell(newContainer) : null;
      try {
        if (_zwIpShell) globalThis.getSelection().collapse(_zwIpShell[0], _zwIpShell[1]);
        else globalThis.getSelection().collapse(newContainer, 0);
      } catch (_eCl5) {}
    } finally {
      if (typeof _zwVaReanchorOverrides === 'function') _zwVaReanchorOverrides();
      _zwVaOverridesPutRaw(snap);
      // delta ④ 回写 + delta ⑤ 捕获补位（同 __zw_ce_delete_apply）：显式 typing
      // 态存活优先（g18/g19 面），态空且捕获非空时落捕获链（Chrome typing
      // style 跨 insertParagraph 存续），俱空保持原值。
      globalThis.__zwTypingTags = (tt && tt.length) ? tt
        : (tcap && tcap.length ? tcap : tt);
      globalThis.__zwTypingTagsOff = ttoff;
    }
    return true;
  };
