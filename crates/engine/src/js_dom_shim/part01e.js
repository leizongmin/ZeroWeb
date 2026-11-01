// part01e（2026-11-01 editing-compat F1 list 命令簇）：execCommand
// insertorderedlist/insertunorderedlist 实应用引擎 + qState/qIndeterm 读面
//（toggleLists/getSelectionListState 参考实现移植）。挂 part01d 后同闭包复用
// _zwJus* 机具（blockExtend/containedNodes/wrap/movePreservingRanges/
// setTagName/followsLineBreak 系）与 part01d _zwVaIsAllowedChild（spec 44 名
// 版）；part06 经 globalThis.__zw_ce_list_toggle / __zw_ce_list_state 单点调
// 用，事件口径同 justify 臂（beforeinput 已由事件序派发、input 走既有尾部）。
// 规范：https://w3c.github.io/editing/execCommand.html#the-insertorderedlist-
// command（toggleLists :5690 / getSelectionListState :3949）。
// FIXME（切片范围，语料面挂后续切片）：① outdentNode 仅 ol/ul 臂（simple/
// indentation 元素臂 + ancestorList 缩进臂挂后续）；② normalizeSublists 未
// 移植（嵌套列表归并，items 恒按空处理）；③ recordValues/restoreValues 未移
// 植（跨父移动的内联样式保全）；④ isSimpleIndentationElement 缩进合并新父臂
// 简化为恒 createElement(tagName)。

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

  // splitParent（vendored :5058 忠实移植）：nodeList 整体提到原父之前/之后。
  // 尾段独在臂（!firstIn && lastIn）逆序后移 + 尾 br 保全后 abort；首段缺位臂
  // 克隆壳承接头部；中段通用路径：成员前移 + 首尾 br 保全 + 空壳删除。
  // FIXME：recordValues/restoreValues（跨父移动样式保全）未移植。
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
        _zwJusMovePreservingRanges(nodeList[ri], originalParent.parentNode, 1 + _zwJusNodeIndex(originalParent));
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
        _zwJusMovePreservingRanges(originalParent.firstChild, clonedParent, clonedParent.childNodes.length);
      }
    }
    for (var mi = 0; mi < nodeList.length; mi++) {
      _zwJusMovePreservingRanges(nodeList[mi], originalParent.parentNode, _zwJusNodeIndex(originalParent));
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

  // fixDisallowedAncestors（vendored :）：dd/dt → dl wrap；禁排父 → 缺省单行
  // 容器 setTagName 递归。
  function _zwListFixDisallowedAncestors(node) {
    if (!_zwJusIsEditable(node)) return;
    var anc = node.parentNode;
    var ok = false;
    var g = 0;
    while (anc && g++ < 256) {
      if (_zwListInSameEditingHost(node, anc) && _zwVaIsAllowedChild(node, anc)) { ok = true; break; }
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
      if (!_zwVaIsAllowedChild('p', _zwListTagOf(host))) return;
      var sep = globalThis.__zwDefaultParagraphSeparator === 'p' ? 'p' : 'div';
      if (!_zwListIsProhibitedParagraphChild(node)) return;
      node = _zwJusSetTagName(node, sep);
      _zwListFixDisallowedAncestors(node);
    }
  }

  function _zwListTagOf(node) {
    return node && node.nodeType === 1 ? String(node.tagName || '').toLowerCase() : '';
  }

  function _zwListIsProhibitedParagraphChild(node) {
    var list = (typeof _zwVaProhibitedParagraphChildren !== 'undefined' && _zwVaProhibitedParagraphChildren) || null;
    if (!list) return false;
    return list.indexOf(_zwListTagOf(node)) !== -1;
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
        || _zwVaIsAllowedChild(node, 'li'));
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
  // 子 fixDisallowedAncestors。
  // FIXME：simple/indentation 元素臂、ancestorList 缩进臂、recordValues/
  // restoreValues 样式保全未移植。
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
      _zwJusSetTagName(node, 'div');
    } else {
      // removePreservingDescendants（recordValues/restoreValues FIXME 跳过）。
      var pp = node.parentNode;
      var idx = _zwJusNodeIndex(node);
      for (var i = 0; i < children.length; i++) {
        _zwJusMovePreservingRanges(children[i], pp, idx + i);
      }
      pp.removeChild(node);
    }
    for (var ci = 0; ci < children.length; ci++) {
      try { _zwListFixDisallowedAncestors(children[ci]); } catch (_eFd2) {}
    }
  }

  // toggleLists（vendored :5690）：enable 全流程 + disable outdent 臂（ol/ul
  // 切片）移植。选区保持依赖 _zwJusWrap/_zwJusMovePreservingRanges。
  function _zwListToggle(tagName) {
    var range = _zwJusActiveRange();
    if (!range) return;
    var mode = _zwListSelectionState() === tagName ? 'disable' : 'enable';
    // tag 常量小写（_zwJusIsHtmlElement 经 _zwJusTag 小写比较，大写常量恒 miss
    // ——ol→ul 换型恒等实证）。
    var utag = tagName;
    var otherTagName = tagName === 'ol' ? 'ul' : 'ol';

    // normalizeSublists FIXME：items 归并跳过（嵌套面少数派）。

    var newRange = _zwJusBlockExtend(range, false);

    // enable：对 other tag 名可编辑元素换型或并入邻接同名列表。
    if (mode === 'enable') {
      var toConvert = _zwJusCollectContainedNodes(newRange, function (node) {
        return _zwJusIsEditable(node) && _zwJusIsHtmlElement(node, otherTagName);
      }, false);
      for (var ci = 0; ci < toConvert.length; ci++) {
        var list = toConvert[ci];
        var prevL = list.previousSibling, nextL = list.nextSibling;
        if ((_zwJusIsEditable(prevL) && _zwJusIsHtmlElement(prevL, utag))
        || (_zwJusIsEditable(nextL) && _zwJusIsHtmlElement(nextL, utag))) {
          var children = [];
          var ck = list.firstChild;
          while (ck) { children.push(ck); ck = ck.nextSibling; }
          _zwListSplitParent(children);
          _zwJusWrap(children, function (node) { return _zwJusIsHtmlElement(node, utag); }, function () { return document.createElement(utag); });
        } else {
          _zwJusSetTagName(list, utag.toLowerCase());
        }
      }
    }

    // node list：blockExtend 后 contained 可编辑（ol/ul 或其子或 li 允许子）。
    var nodeList = _zwJusGetContainedNodes(newRange, function (node) {
      return _zwJusIsEditable(node)
        && !_zwListIsIndentationElement(node)
        && (_zwJusIsHtmlElement(node, ['ol', 'ul'])
        || _zwJusIsHtmlElement(node.parentNode, ['ol', 'ul'])
        || _zwVaIsAllowedChild(node, 'li'));
    });
    if (mode === 'enable') {
      nodeList = nodeList.filter(function (node) {
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
        _zwListSplitParent(dsub);
        for (var di = 0; di < dsub.length; di++) {
          try { _zwListFixDisallowedAncestors(dsub[di]); } catch (_eFd) {}
        }
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
          sublist.push(_zwJusSetTagName(nodeList[0], 'li'));
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
        _zwListSplitParent(sublist);
        _zwJusWrap(sublist, function (node) { return _zwJusIsHtmlElement(node, utag); }, function () { return document.createElement(utag); });
        continue;
      }

      // 缩进合并新父臂 FIXME：恒 createElement(utag)（isSimpleIndentationElement
      // 未移植，indent+list 组合面挂后续切片）。
      _zwJusWrap(sublist, function (node) { return _zwJusIsHtmlElement(node, utag); }, function () { return document.createElement(utag); });
    }
  }

  globalThis.__zw_ce_list_toggle = function (tagName) {
    try { _zwListToggle(tagName === 'ol' ? 'ol' : 'ul'); } catch (_eLt) {}
  };

  // qState/qIndeterm 读面（vendored commands.insertorderedlist/unorderedlist
  // state/indeterm）：state = list state 恰为本命令 tag；indeterm =
  // /^mixed( ol)?$/（ol）/ /^mixed( ul)?$/（ul）。
  globalThis.__zw_ce_list_state = function (cmd) {
    var isOl = cmd === 'insertorderedlist';
    var want = isOl ? 'ol' : 'ul';
    var st = null;
    try { st = _zwListSelectionState(); } catch (_eLs) { st = null; }
    return {
      state: st === want,
      indeterm: st === 'mixed' || st === (isOl ? 'mixed ol' : 'mixed ul')
    };
  };
