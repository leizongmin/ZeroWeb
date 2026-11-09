  // —— F1' shell carry（2026-11-09 ④ 刀 typing 壳继承轮）：collapsed
  // delete/forwarddelete 跨块合并形的 typing 态壳链 carry。根因：spec 忠实
  // 移植的 deleteSelection 把 caret collapse 到两块之间元素位（backward 方向
  // 交换语义），删除边界一侧的 inline 壳层信息丢失——后续 insertText 元素位
  // caret 裸插（残 23F：16 键蓝红双 span 族单一裸字 got + fontSize 中置 7，
  // [普查档](../../../../docs/goal/editing-compat/evidence/
  // 2026-11-09-f6b-post-census-52f-regroup.md) ④；D4 拆壳臂丢 armed tags
  // 同域根因先例 = F1'-B evidence）。Chrome/Blink 以 TypingStyle 自删除边界
  // 推导并存续到后续打字（execCommand spec 无此概念，引擎侧扩展）。
  // 存**节点引用链**非 tag 串：语料期望 typed 段并入既有壳尾（`fooA`
  // extend-in-place，adjacent same-shell 不合并——`<b>a</b><b>b</b>` 字节
  // 不等 `<b>ab</b>`），建新壳必失配。生命周期同 F6b script carry：
  // delete 臂替换式记账（part01f __zw_ce_delete_apply）→ 消费面 part01
  // inserttext 元素位 caret 两臂（consume 即清，单发）→ _zwSync 选区变更
  // 面清除（part06，同 script carry 槽位）。
  // 捕获门（四处皆过才记账）：
  //   ① collapsed 且 caret 处所在块文档序边界位——delete 块内 caret 前方
  //      / forwarddelete 后方内容全为占位 br/块级元素（跨块合并前置形，
  //      块尾 br 是合并消费面非词中内容；词中 inline 内容即非边界）；
  //   ② 左边界文本存在（文档序前方最近文本，Chrome TypingStyle 推导基准）；
  //   ③ 壳链非空（左邻裸文本 → typed 段本就裸落，现状保持）；
  //   ④ 链含 <a> 置空（__zw_ce_capture_typing_chain :2163/:2168 双向实证
  //      语义不变）；off 表壳不捕（delta ⑤ 同门）。
  // 消费门：caret 左边界链与 carry 链**逐节点同一**（合并 move 保节点身份；
  // 跨块/异形链自然失配走原裸插，零回退面）。

  // 块级止步表（链上行到块/宿主即停——壳链只收 inline 层；td/th/tr 等
  // 表中间层一并止步）。
  var _zwShellCarryBlockTags = {
    address: 1, article: 1, aside: 1, blockquote: 1, center: 1, dd: 1,
    details: 1, dialog: 1, dir: 1, div: 1, dl: 1, dt: 1, fieldset: 1,
    figcaption: 1, figure: 1, footer: 1, form: 1, h1: 1, h2: 1, h3: 1,
    h4: 1, h5: 1, h6: 1, header: 1, hgroup: 1, hr: 1, li: 1, listing: 1,
    main: 1, menu: 1, nav: 1, ol: 1, p: 1, plaintext: 1, pre: 1,
    section: 1, summary: 1, table: 1, tbody: 1, td: 1, tfoot: 1, th: 1,
    thead: 1, tr: 1, ul: 1, xmp: 1
  };

  // 边界伴随物判定：占位 br（跨块合并消费面——语料 16 键 expected 全无 br）
  // 与块级元素（跨界合并对象）不构成词中内容；其余（文本/inline 元素）即
  // 非边界。fd 带块尾 br 变体（C10/C11/C13/C14/C16 probe 实证）靠此放行。
  function _zwShellCarrySepOk(kid) {
    if (!kid || kid.nodeType !== 1) return false;
    var tn = String(kid.tagName || '').toLowerCase();
    return tn === 'br' || !!_zwShellCarryBlockTags[tn];
  }

  // (sc, so) 是否处于所在块的文档序边界位——delete（forward=false）要求
  // 块内 caret 前方内容全为占位 br/块级元素，forwarddelete 要求后方同款
  // （跨块合并前置形；词中 inline 内容即非边界，G2/G7 probe 护栏实证）。
  // 逐层检查：文本容器 offset 全出/全进；元素容器 off 两侧子与上行各层
  // 兄弟位按伴随物判定；到所在块（块/宿主父层）检查完自身兄弟位即止。
  function _zwShellCarryAtBoundary(sc, so, forward) {
    var n = sc;
    var off = so;
    var guard = 0;
    for (; n && guard++ < 256;) {
      if (n.nodeType === 3 || n.__zwIsText) {
        if (forward ? off !== _zwJusNodeLength(n) : off !== 0) return false;
      } else if (n.nodeType === 1) {
        var nk = n.childNodes || [];
        var i;
        if (forward) {
          for (i = off; i < nk.length; i++) {
            if (!_zwShellCarrySepOk(nk[i])) return false;
          }
        } else {
          for (i = 0; i < off; i++) {
            if (!_zwShellCarrySepOk(nk[i])) return false;
          }
        }
      } else return false;
      var p = n.parentNode;
      if (!p) return false;
      var idx = Array.prototype.indexOf.call(p.childNodes, n);
      var j;
      if (forward) {
        for (j = idx + 1; j < p.childNodes.length; j++) {
          if (!_zwShellCarrySepOk(p.childNodes[j])) return false;
        }
      } else {
        for (j = 0; j < idx; j++) {
          if (!_zwShellCarrySepOk(p.childNodes[j])) return false;
        }
      }
      if (_zwJusIsEditingHost(p)
          || _zwShellCarryBlockTags[String(p.tagName || '').toLowerCase()]) {
        return true;
      }
      off = idx + (forward ? 1 : 0);
      n = p;
    }
    return false;
  }

  // 自 (sc, so) 向文档序前方找最近文本节点（TypingStyle 边界推导基准）：
  // 文本位 offset>0 即该节点自身；元素位先降 offset 前子最深末后代，耗尽
  // （void/空壳尾）再爬升换 prev 分支续走。到宿主顶仍无 → null。
  function _zwShellCarryPrevText(sc, so) {
    var n = sc;
    var off = so;
    var guard = 0;
    for (; n && guard++ < 256;) {
      if (n.nodeType === 3) {
        if (off > 0) return n;
      } else if (off > 0 && n.childNodes && n.childNodes.length) {
        var d = n.childNodes[off - 1];
        var g2 = 0;
        while (d && d.lastChild && g2++ < 256) d = d.lastChild;
        if (d && d.nodeType === 3) return d;
        n = d;
        off = 0;
        continue;
      }
      if (!n.parentNode || _zwJusIsEditingHost(n)) return null;
      off = Array.prototype.indexOf.call(n.parentNode.childNodes, n);
      n = n.parentNode;
    }
    return null;
  }

  // 边界文本的 inline 壳链（innermost-first 节点引用）：父层上行收非块
  // 元素，块/宿主止步；链含 <a> → null（:2163/:2168 语义）。
  function _zwShellCarryChainOf(txt) {
    var chain = [];
    var n = txt.parentNode;
    var guard = 0;
    while (n && !_zwJusIsEditingHost(n) && guard++ < 256) {
      if (n.nodeType !== 1) return null;
      var tn = String(n.tagName || '').toLowerCase();
      if (tn === 'a') return null;
      if (_zwShellCarryBlockTags[tn]) break;
      chain.push(n);
      n = n.parentNode;
    }
    return chain;
  }

  // 捕获入口（part01f __zw_ce_delete_apply 动作分派前调用）：门 ①—④
  // 全过返回 { chain }（innermost-first 节点引用），否则 null（替换式清账
  // 由调用方回写完成）。
  globalThis.__zw_ce_capture_shell_carry = function (ar, forward) {
    if (!ar || !ar.collapsed) return null;
    var sc = ar.startContainer;
    var so = ar.startOffset;
    if (!sc) return null;
    if (!_zwShellCarryAtBoundary(sc, so, !!forward)) return null;
    var txt = _zwShellCarryPrevText(sc, so);
    if (!txt) return null;
    var chain = _zwShellCarryChainOf(txt);
    if (!chain || !chain.length) return null;
    var toff = globalThis.__zwTypingTagsOff;
    if (Array.isArray(toff) && toff.length) {
      for (var i = 0; i < chain.length; i++) {
        var tn = String(chain[i].tagName || '').toLowerCase();
        if (toff.indexOf(tn) >= 0) return null;
      }
    }
    return { chain: chain };
  };

  // 消费入口（part01 inserttext 元素位 caret 两臂裸插前调用）：caret 左
  // 边界链与 carry 链逐节点同一 → typed 段建文本节点并入 innermost 壳尾
  // （appendChild——壳内既有文本之后，`fooA` 形），consume 即清，返回新
  // 节点；任一门不过 → null（调用方走原裸插）。carry 壳已脱离文档（中间
  // 变更移除）→ 弃账即清防误配。
  globalThis.__zw_ce_shell_carry_consume = function (sc, so, ins) {
    var carry = globalThis.__zwTypingShellCarry;
    if (!carry || !carry.chain || !carry.chain.length) return null;
    var inner = carry.chain[0];
    if (!inner || !inner.parentNode) {
      globalThis.__zwTypingShellCarry = null;
      return null;
    }
    var txt = _zwShellCarryPrevText(sc, so);
    if (!txt) return null;
    var chain = _zwShellCarryChainOf(txt);
    if (!chain || chain.length !== carry.chain.length) return null;
    for (var i = 0; i < chain.length; i++) {
      if (chain[i] !== carry.chain[i]) return null;
    }
    var node = document.createTextNode(ins);
    inner.appendChild(node);
    globalThis.__zwTypingShellCarry = null;
    return node;
  };

  // —— A4 行尾 ws 前向不可见 adjunct（2026-11-09 ⑦ 刀 trailing-empty-p，mt
  // 626）：FD walk 的 invisible 兄弟子判定扩展。根因：spec visible 对 ws 文本
  // 走文档序**回看**（_zwJusIsCollapsedWhitespaceNode，前一路径遇非空白文本
  // 即判行中可见），而 walk 先吃掉块尾 br 后 ws 的回看命中块内正文——行尾
  // 折叠 ws 被误判可见，walk 经 cond4 下降进 ws、text 分支把它当字符单删后
  // 返回，跨块合并径不达（got `<p>abcd</p> <p><br></p>`）。Chrome/Blink 按
  // **前向**行界折叠：块尾 ws 后方到块界/行断（br/块）之间无可见内容即折叠
  // 不可见，FD 一并吸收后继续合并（exp `<p>abcd</p>`）。谓词自身带上下文
  // （_zwJusIsWhitespaceNode 按父链 white-space 判定，pre/break-spaces 面 ws
  // 非折叠空白 → adjunct 恒假，627 wss 护栏不动）。
  // https://w3c.github.io/editing/execCommand.html#delete-the-contents
  function _zwA4SubtreeVisible(el) {
    var k = el.childNodes || [];
    for (var i = 0; i < k.length; i++) {
      var c = k[i];
      if (c.nodeType === 3) {
        if (!_zwJusIsWhitespaceNode(c)) return true;
      } else if (c.nodeType === 1) {
        if (_zwJusIsHtmlElement(c, 'img')) return true;
        if (!_zwJusIsHtmlElement(c, 'br') && _zwA4SubtreeVisible(c)) return true;
      }
    }
    return false;
  }

  // ws 文本节点前向行界直判：文档序前向从下一兄弟起扫描，遇非折叠空白文本
  // 或含可见内容的 inline 子树即行中可见（假），遇 br/块/块界爬升耗尽即行尾
  // 折叠（真）。img 按 spec visible 恒可见（ws 前 img 即行中）。
  function _zwA4TrailWsInvisible(nd) {
    if (!nd || nd.nodeType !== 3 || !_zwJusIsWhitespaceNode(nd)) return false;
    var n = nd;
    var guard = 0;
    while (n && guard++ < 256) {
      var s = n.nextSibling;
      if (!s) {
        var p = n.parentNode;
        if (!p) return true;
        if (_zwJusIsBlockNode(p) || _zwJusIsEditingHost(p)) return true;
        n = p;
        continue;
      }
      if (s.nodeType === 3) {
        if (!_zwJusIsWhitespaceNode(s)) return false;
        n = s;
        continue;
      }
      if (s.nodeType === 1) {
        if (_zwJusIsBlockNode(s)) return true;
        if (_zwJusIsHtmlElement(s, 'br')) { n = s; continue; }
        if (_zwA4SubtreeVisible(s)) return false;
        n = s;
        continue;
      }
      return false;
    }
    return true;
  }

  // —— A1 空 sole div 消解（2026-11-09 ⑦ 刀 host-div-wrap / ip-placeholder
  // ① 路由，multitest 483/485/487）：deleteSelection 全删空 caret 块时 Chrome
  // 消解壳提宿主——exp `<div><b>[abc]</b></div>` delete 后 typing = `<b>a</b>`
  // （got `<div><b>a</b></div>` = 壳未消解）。落 part01h 卫星（part01f 恰
  // 2000 行锚定，④ 刀先例；同闭包复用 _zwDel*/_zwJus*/_zwList* 机具）。
  // 消解条件（482 blockquote 全删空**不消解**反例收窄）：
  //   ① caret 块为宿主直子 div（tag 收窄 div；blockquote/h1-6/p 不消解）；
  //   ② 子全 inline 且无可见内容（空壳链；空 `<p></p>` 块子不提）；
  //   ③ 宿主级无其他元素子/可见内容（消解后宿主只剩提壳链）。
  // 动作 = 子提宿主原位（保序）+ 块删除 + caret 塌缩提壳链首（处方 b 同款
  // `__zw_ce_caret_in_leading_shell`，typed 段入壳——483 typing `<b>a</b>`）。
  // 调用点自限非 collapsed 面（part01f delete_apply 尾 + ipara_apply 前导
  // delete 后）；collapsed delete/fd 内部 deleteSelection 各支路不经此。
  // 跨域摘挂纪律同 DC-4（plain removeChild 后挂载；子为 inline plain 节点，
  // host.insertBefore 同域安全）。
  // https://w3c.github.io/editing/execCommand.html#the-delete-command
  globalThis.__zw_ce_dissolve_emptied_div_tail = function (ar0) {
    try {
      var ar = _zwVaGetActiveRange();
      if (!ar || !ar.startContainer) return false;
      var blk = _zwDelGetBlockNodeOf(ar.startContainer);
      if (!blk || blk.nodeType !== 1 || _zwJusTag(blk) !== 'div') return false;
      var host = _zwListEditingHostOf(blk);
      if (!host || blk.parentNode !== host) return false;
      // ⑥ delmerge 边界祖先语义（zz-probe-f7 D1-D3/V1/V3/K1-K3 实证）：消解
      // 仅当删除前选区完全含于块内某内联壳子树（483 `[abc]` 文本位全在内
      // → 消解提壳）；选区触及块边界（K3 `{abc}` div 子边界位）或尾越块界
      // 入宿主级（V3/D1/K1 `{abc</span><span>def</span>}` 形）→ 块为选区
      // 边界祖先，Chrome 保留块（exp `<div>gh</div>` 族）。
      if (!ar0 || !ar0.startContainer || !ar0.endContainer) return false;
      var a1Shell = ar0.startContainer;
      while (a1Shell && a1Shell.parentNode !== blk) a1Shell = a1Shell.parentNode;
      if (!a1Shell) return false;
      var a1EndIn = false;
      var a1E = ar0.endContainer;
      while (a1E) {
        if (a1E === a1Shell) { a1EndIn = true; break; }
        a1E = a1E.parentNode;
      }
      if (!a1EndIn) return false;
      // display 守卫（D1/D2 flex/grid 保留形同源）：非默认 display 块不消解
      // ——与 ① 臂 dispHost∈{'',block} 门同语义。
      var a1Disp = '';
      try { a1Disp = String(_zwJusCs(blk, 'display') || ''); } catch (_eA1d) {}
      if (a1Disp !== '' && a1Disp !== 'block') return false;
      var kids = blk.childNodes || [];
      if (!kids.length) return false;
      for (var i = 0; i < kids.length; i++) {
        if (!_zwJusIsInlineNode(kids[i])) return false;
        if (_zwJusIsVisible(kids[i])) return false;
      }
      var sibs = host.childNodes || [];
      for (var j = 0; j < sibs.length; j++) {
        var s = sibs[j];
        if (s === blk) continue;
        if (s.nodeType === 1 && !_zwJusIsInlineNode(s)) return false;
        if (_zwJusIsVisible(s)) return false;
      }
      while (blk.firstChild) {
        var kid = blk.firstChild;
        if (kid.parentNode) {
          try { blk.removeChild(kid); } catch (_eA1rm) {}
        }
        host.insertBefore(kid, blk);
      }
      host.removeChild(blk);
      var first = kids[0];
      var nrA1 = document.createRange();
      var shA1 = (typeof globalThis.__zw_ce_caret_in_leading_shell === 'function')
        ? globalThis.__zw_ce_caret_in_leading_shell(host) : null;
      if (shA1) {
        nrA1.setStart(shA1[0], shA1[1]);
      } else {
        nrA1.setStart(first, 0);
      }
      nrA1.collapse(true);
      if (typeof _getSelection === 'function') { _getSelection()._ranges = [nrA1]; }
      try { globalThis.getSelection().collapse(nrA1.startContainer, nrA1.startOffset); } catch (_eA1cl) {}
      return true;
    } catch (_eA1) {
      return false;
    }
  };
