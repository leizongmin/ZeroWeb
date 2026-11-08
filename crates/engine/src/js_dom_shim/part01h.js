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
