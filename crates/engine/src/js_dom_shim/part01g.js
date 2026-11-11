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

  // Chrome delta（2026-11-10 fb apply 侧包裹刀）：xmp/listing 为 wrap-only 块
  // ——Chrome formatBlock 不作 single-line container 解包，元素整体保留并入
  // 新块：`<xmp>[foobar]</xmp>` + <p> 录音 `<p><xmp>[foobar]</xmp></p>`、
  // + <div> 录音 `<div><xmp>[foo]</xmp></div>`（formatblock.js :633/:1764/
  // :1774，xmp/listing 十行 compare 全形）；spec unwrap 臂产出 `<p>foobar</p>`
  // 与录音相悖。unwrap 门豁免、nodeList 收集门单独放行（xmp/listing 非 p
  // 允许子，spec 判据收不进）。
  function _zwFbqIsWrapOnlyBlock(node) {
    return _zwJusIsHtmlElement(node, ['xmp', 'listing']);
  }

  // SLC 溶解声明收割（2026-11-10 数据丢失刀，见 apply unwrap 臂 carry 注）：
  // 只收四值命令——forecolor/hilitecolor/fontsize/fontname 的 specified 读面
  // 直读对应 style/font 声明；boolean 命令（bold/italic/underline/strikethrough/
  // subscript）不入收割——Chrome carry 面只转移值声明，boolean 载体按 spec
  // record/restore 行为走（读面 else 臂错读已于 boolean 读面刀在 part01d
  // 修为族内正确读，此处维持不收）。
  function _zwFbqHarvestBlockValues(block) {
    var out = [];
    var carryCmds = ['forecolor', 'hilitecolor', 'fontsize', 'fontname'];
    for (var i = 0; i < carryCmds.length; i++) {
      var val = null;
      try { val = _zwVaSpecifiedValue(block, carryCmds[i]); } catch (_eFbqH) {}
      // hilitecolor 的 spec 读面对块级返 null（display 门，见 part01d :209），
      // 但 Chrome fb carry 面转移被替换块的内联声明不分 display——背景块形
      // 直读 plain 声明补收割（fbl2 面谱 background 全丢 + boolface 探针
      // bf1 双空 span 实证；语料无背景 fb 行，探针面断言）。
      if (val === null && carryCmds[i] === 'hilitecolor') {
        try {
          val = _zwPlainDecl(block, 'background-color') || null;
        } catch (_eFbqH2) {}
      }
      if (val !== null) out.push([carryCmds[i], val]);
    }
    return out;
  }

  // 收割值落块本体的 style 属性名（forceValue property 映射同口径）。
  function _zwFbqCarryProperty(command) {
    return command === 'fontsize' ? 'font-size'
      : (command === 'fontname' ? 'font-family'
        : (command === 'forecolor' ? 'color' : 'background-color'));
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
    // Chrome delta（可编辑链守卫）：起点叶子到编辑宿主链上遇不可编辑元素 →
    // 拦（不应用，零树变更）但 **返 true**——该形 Chrome ret 录音 [true]（语料
    // 条目 362 `{ <span cE=false>A/B/C }` 真意 wrap 不可编辑节点、本引擎不
    // 具备该 wrap 能力只拦不报 false）；part06 :5187 返回值贯通（2026-11-09
    // apply 内容模型边界刀）后返 false 会误翻 entry 362 ret 行（P2F）。
    // `contenteditable=false` 宿主 + 内层 contenteditable 元素形（:1864/:1871/
    // :1876）不走此守卫——链在内层 editing host 即截停，由下方宿主内容模型
    // 守卫拒（ret=false 对齐 Chrome）。`<div><div contenteditable=false><div
    // contenteditable>[foo]` 形链内层 div 可编辑不拦照 wrap（ret=true）。
    // display:block span 岛形（条目 360）链全程可编辑不拦。
    var sl = _zwFbqEdgeLeaf(range.startContainer, range.startOffset, false);
    if (!sl) return false;
    var cur = sl;
    var fbqHost = null;
    var g0 = 0;
    while (cur && g0++ < 256) {
      if (_zwJusIsEditingHost(cur)) { fbqHost = cur; break; }
      if (!_zwJusIsEditable(cur)) return true;
      cur = cur.parentNode;
    }
    // Chrome delta（宿主内容模型守卫）：新块物化在选区 editing host 内，host
    // 不能容块级子时 Chrome 整体放弃——树不变 ret=false，qV before/after 恒 ""
    // （VisBlock 爬升遇 host 先返 null，守卫在此前拦）。语料 ret 录音四面对账：
    // "<p> cannot have <div>, so, do nothing"（:1876）/ "<span> cannot have
    // <p>"（:1866/:1871）→ [false]；"<div> can have <p> / <div>, so, wrap"
    // （:1881/:1886）→ [true] 照 wrap。**ret=false 经 part06 :5187 返回值贯通
    // execCommand 尾返回**（2026-11-09 apply 内容模型边界刀——此前返回值被
    // 丢弃恒 true，语料 ret 断言行 ×3 红）。严格 HTML 内容模型表不适用：
    // `<div>[foobar]</div>` + dd（:757）录音真 wrap 后 fixDisallowedAncestors
    // 抬 dl（dd∉div 内容模型仍放行）——判据只看「host 可否含块」：p 宿主拒；
    // 其余按 computed display inline 族拒（display:block span 岛形条目 361
    // 可容块照 wrap）。b/i/font 样式壳不触发（壳非 host，宿主为外层块容器照
    // wrap——mt `[["bold",""],["formatblock","<div>"]]` compare 全绿面）。
    if (fbqHost && fbqHost.nodeType === 1) {
      if (_zwJusIsHtmlElement(fbqHost, 'p')) return false;
      var fbqHostDisp = 'block';
      try {
        fbqHostDisp = globalThis.getComputedStyle
          ? globalThis.getComputedStyle(fbqHost).display : 'block';
      } catch (_eFbqDisp) {}
      if (String(fbqHostDisp).indexOf('inline') === 0) return false;
    }

    // "Block-extend the active range, and let new range be the result."
    var newRange = _zwJusBlockExtend(range, false);

    // "For each node node contained in new range, append node to node list
    // if it is editable, ... node is either a non-list single-line container
    // or an allowed child of "p" or a dd or dt, and node is not the ancestor
    // of a prohibited paragraph child."
    var nodeList = _zwJusGetContainedNodes(newRange, function (node) {
      return _zwJusIsEditable(node)
        && (_zwFbqIsNonListSingleLineContainer(node)
            || _zwFbqIsWrapOnlyBlock(node)
            || _zwJusIsAllowedChild(node, 'p')
            || _zwJusIsHtmlElement(node, ['dd', 'dt']))
        && !_zwFbqHasProhibitedParagraphChildDescendant(node);
    });

    // Chrome delta（同 tag no-op）：node 祖先链上已有目标 tag 的可编辑同宿主
    // formattable 块 → 从 nodeList 剔除（spec splitParent/wrap 两环都不再处理）。
    // 爬升对 ol/ul/li 列表透明（穿越不判、命中即剔）——prohibited-paragraph
    // veto 在此环不适用：`<div><ol><li>[foo]</ol></div>` + div 的 div 含 li
    // （∈ prohibited 名单）但录音要求树不变（:1784），veto 会使外层 div 漏剔
    // → 误 wrap 出 `<li><div>foo</div></li>`。语料树录音 ×3：`<div>[foobar]
    // </div>` + div 树不变（:134）；`<p>[foo<p>bar]` + p 两 text 均链上命中 p
    // 树不变（:1410）；:1784 穿 li/ol 命中 div。反向锚定：`<ol><li>[foobar]</ol>`
    // + div 链上无 div（ol 直上宿主）不剔除照 wrap（`<ol><li><div>[foobar]
    // </div></li></ol>`，:478 p+p 同理 host 位截停）；`<dl><dt>[foo]` dt 链 dl
    // 非 formattable 不剔除照换块。
    for (i = nodeList.length - 1; i >= 0; i--) {
      var anc2 = nodeList[i].parentNode;
      var g2 = 0;
      while (anc2 && g2++ < 256) {
        if (_zwJusIsEditingHost(anc2)) break;
        if (_zwJusIsHtmlElement(anc2, ['ol', 'ul', 'li'])) {
          anc2 = anc2.parentNode;
          continue;
        }
        // Chrome delta（per-paragraph 不合并刀，2026-11-10）：成员为非
        // wrap-only SLC（p/h1-6/div/pre/address 等）时不剔除——Chrome 对块
        // 成员作原位换块（`<blockquote><p>[foo]<p>bar</blockquote><p>extra`
        // + <blockquote> 录音 `<blockquote><blockquote>[foo]</blockquote>
        // <p>bar</p></blockquote>...`，formatblock.js :698），剔除会照树
        // no-op。该臂原意 = 裸 text/内联成员防双包（:1784 列表爬升形成员为
        // text，剔除保持）；SLC 成员放行走尾环溶解臂原位重 wrap（:134
        // div+div 幂等同树不变）。wrap-only（xmp/listing）维持剔除（防双包
        // div>div>xmp，无语料行，探针 zz-probe-fbpara K8 锚）。
        if (_zwJusIsHtmlElement(anc2, v)
            && _zwJusIsEditable(anc2)
            && _zwListInSameEditingHost(anc2, nodeList[i])
            && !(_zwFbqIsNonListSingleLineContainer(nodeList[i])
                 && !_zwFbqIsWrapOnlyBlock(nodeList[i]))) {
          nodeList.splice(i, 1);
        }
        break;
      }
    }
    if (!nodeList.length) return true;

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
          // Chrome delta（列表屏障）：祖先爬升途中遇 ol/ul 即停——`<div><ol>
          // <li>[foo]</ol></div>` + formatblock div 录音树不变（div 不拆），spec
          // 判据 div 同宿主 formattable 命中会拆。dl 不屏障（`<dl><dt>[foo]`
          // 录音 dt 真拆换块）。
          if (_zwJusIsHtmlElement(anc, ['ol', 'ul'])) break;
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
          // Chrome delta（div>p 摆位刀，2026-11-10）：vetoed formattable 直亲
          //（node 直属该块、块含 prohibited 后代——spec splitParent 爬升在此
          // 刹车）且新块 tag 为其允许子时，Chrome 段落摆位把部分覆盖段落的
          // 替换块落块侧旁而非块内——**内联成员** splitParent 外提后 wrap 落
          // 宿主层（`<div>[foo<p>bar]</p></div>` + <p> 录音 `<p>[foo</p><div>
          // <p>bar]</p></div>`，formatblock.js :1754/:1759 双 dps 变体）。三道
          // 防线（TRACE 实证首轮三缺陷后定谳）：
          // **直亲门**——node 与 anc 之间隔 barrier（td 等，如 `<div><table>
          // <tr><td>[foo]</table></div>` 录音 div 落 td 内 :1794）时 splitParent
          // 只拆直亲链，跨 barrier 外提=表格撕裂（首轮 F2P=0 + 表格 P2F ×2
          // 实证），非直亲不外提维持 spec 停；**块成员门**——node 为块级成员
          //（blockExtend 后选区含整个 SLC 块时该块本身入 nodeList，探针
          // TRACE：afterFilter=[foo,<p>]）不外提——外提会连锁拆壳（splitParent
          // move-out 臂移出成员 + 空壳删除→被替换块整体消失，首证 got
          // `<p>foo</p><p>bar</p>` 丢 `<div><p>bar</p></div>` 录音形）；块成员
          // 留位走尾环 SLC 溶解臂，溶解在原块内重 wrap 产出 `<div><p>bar</p>
          // </div>` 录音形；**值门**——该成员 recordValues 有非 null 值
          //（styled 块 forecolor 等）时 restore 会落内联载体，外提后载体×
          // wrap×fixDisallowed 链丢内容（`<div style=color:blue>[foo<p>bar]
          // </div>` 探针 foo 失踪实证），有值成员维持 spec 停。
          // 新块非允许子形（h1→p 外壳）不外提维持 spec 停——fixDisallowed
          // 允许子臂拆父产出合并录音形（`<p>[foo<p>bar]` + h1 →
          // `<h1>[foo<br>bar]</h1>`，:1744 现行绿面；h1∉p 允许子为判据分野）。
          if (!_zwJusIsEditingHost(anc)
              && node.parentNode === anc
              && _zwJusIsInlineNode(node)
              && _zwJusIsEditable(anc)
              && _zwListInSameEditingHost(anc, node)
              && _zwJusIsHtmlElement(anc, _zwFbqFormattableNames)
              && _zwFbqHasProhibitedParagraphChildDescendant(anc)
              && _zwJusIsAllowedChild(v, _zwJusTag(anc))) {
            var fbqHasVals = false;
            for (var vi = 0; vi < values.length; vi++) {
              if (values[vi][0] === node && values[vi][2] !== null) { fbqHasVals = true; break; }
            }
            if (!fbqHasVals) {
              var prevP2 = node.parentNode;
              _zwListSplitParent([node]);
              if (node.parentNode !== prevP2) keep = true;
            }
            break;
          }
          anc = anc.parentNode;
        }
      }
    }

    // "Restore the values from values."
    _zwDelRestoreValues(values);

    // "While node list is not empty: ..."（single-line container 解包臂 ∨
    // 连续 sibling 聚 sublist 后 wrap）。
    // per-paragraph 不合并刀（2026-11-10）：本次 apply 已产出新块登记表——
    // sibCrit 合并只对自产邻居放行（v2，见下方 sibCrit 注）。
    var fbqCreated = [];
    while (nodeList.length) {
      var sublist;
      // "If the first member of node list is a single-line container: ..."
      // Chrome delta（wrap-only 块）：xmp/listing 虽名在 single-line container
      // 表，Chrome 不解包（语料十行录音元素整体入新块，见 _zwFbqIsWrapOnlyBlock
      // 注）——unwrap 门豁免，走 else 臂 wrap 单元整体。聚合停止条件仍用全表
      // （xmp/listing 阻断连续 sibling 并入同一 sublist，单元素独立成块）。
      var fbqCarry = [];
      if (_zwIparaIsSingleLineContainer(nodeList[0])
          && !_zwFbqIsWrapOnlyBlock(nodeList[0])) {
        sublist = Array.prototype.slice.call(nodeList[0].childNodes);
        // Chrome delta（SLC 溶解声明 carry，2026-11-10 数据丢失刀）：Chrome
        // formatBlock 把被替换块的内联声明转移到替换块本体——语料
        // `<div style=color:blue>[foo]</div>` + <p> 录音 `<p style=
        // "color:rgb(0, 0, 255)">[foo]</p>`，stylewithcss true/false 双变体
        // 同形（:1834-1857 八行），无 font/span 内联载体。spec 的 vals2
        // record→dissolve→restore 路径在本引擎把裸文本 forceValue 成载体
        //（font/span），共享尾 wrap 的新块落进载体（disallowed 父）→
        // fixDisallowed 允许子臂拆父 + 对块二次 forceValue 再裹载体——链条
        // 终态宿主清空、内容序列化丢失（TRACE 级实锤：unwrap 后 foo 入
        // font 载体，wrap done hostkids=0，fixdone tree=null，RET 仍
        // [true]——part06 异常壳掩盖）。数据修复优先于兼容面：溶解前收割
        // 块级声明（forecolor/hilitecolor/fontsize/fontname 四值命令，
        // boolean 命令 specified 读面缺省错读他族声明——hilite 形探针
        // bold=yellow 实证——不入收割），等值匹配的 vals2 三元组抑制
        // restore（更深自有值的子树照常 forceValue），wrap 成块后声明落
        // 块本体（setStyleProp CSSOM 归一 color:rgb 形，fixDisallowed 前
        // 落——块 specified 即值，爬升面自然短路不再 forceValue）。
        fbqCarry = _zwFbqHarvestBlockValues(nodeList[0]);
        var vals2 = _zwDelRecordValues(sublist);
        if (fbqCarry.length) {
          vals2 = vals2.filter(function (t2) {
            for (var ci = 0; ci < fbqCarry.length; ci++) {
              if (fbqCarry[ci][0] === t2[1]
                  && _zwVaEquivalentValues(t2[1], t2[2], fbqCarry[ci][1])) {
                return false;
              }
            }
            return true;
          });
        }
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
      // Chrome delta（per-paragraph 不合并刀 v2，2026-11-10）：sibCrit 只对
      // 本次 apply 产出的邻居块合并——v1 恒 false 被门禁批打回（fb 主门
      // P2F=7）：跨段部分选择行（`<p>[foo</p>bar]` + <h1> 录音单块合并形
      // `<h1>[foo<br>bar]</h1>`，formatblock.js :1734/:1739 等 ×6）与全覆盖
      // 双段行（`<p>[foo<p>bar]<p>baz` + <blockquote> 的 qV-after 面绑合并
      // 形，:708，SLC 分组条件使 p1/p2 各自成 sublist、聚合全靠 sibCrit）
      // 的绿面依赖分段 wrap 合并。而 `[foo]<blockquote>bar</blockquote>` +
      // <blockquote>（:703）的邻居块是命令前已存在，Chrome 录音分开两块
      // `<blockquote>[foo]</blockquote><blockquote>bar</blockquote>`——
      // Chrome delta 实为「不并入命令前已存在的邻块」，自产分段块照 Chrome
      // 录音合并。div/p 维持恒 false（:1754/:1759 摆位录音分段不合并）。
      var sibCrit = (['div', 'p'].indexOf(v) !== -1)
        ? function () { return false; }
        : function (n) {
            return fbqCreated.indexOf(n) !== -1
              && _zwJusIsHtmlElement(n, v)
              && n.attributes && n.attributes.length === 0;
          };
      var fbqBlock = _zwJusWrap(sublist, sibCrit,
        function () { return document.createElement(v); }, true);
      if (fbqBlock && fbqBlock.nodeType === 1) {
        fbqCreated.push(fbqBlock);
      }
      if (fbqCarry.length && fbqBlock && fbqBlock.nodeType === 1) {
        for (var ci2 = 0; ci2 < fbqCarry.length; ci2++) {
          _zwVaSetStyleProp(fbqBlock, _zwFbqCarryProperty(fbqCarry[ci2][0]),
            fbqCarry[ci2][1]);
        }
      }
      // Chrome delta（per-paragraph 不合并刀，2026-11-10）：新块直父为
      // hgroup 时跳过 fixDisallowed——hgroup 内容模型只容 h1-h6（part01e
      // allowed-child 表），blockquote 落 hgroup 内会被允许子臂 splitParent
      // 外提到 section（qV 读外提后 blockquote 故绿、compare 红），Chrome
      // 保留非法嵌套不修（`<section><hgroup><h1>[foo]</h1><h2>bar</h2>
      // </hgroup><p>baz</section>` + <blockquote> 录音 `<section><hgroup>
      // <blockquote>[foo]</blockquote><h2>bar</h2></hgroup><p>baz</p>
      // </section>`，formatblock.js :726）。其余 hgroup 行（:1042/:1360）
      // wrap 落宿主层直父非 hgroup，fix 照跑 no-op。
      if (!(fbqBlock && fbqBlock.nodeType === 1
            && _zwJusIsHtmlElement(fbqBlock.parentNode, 'hgroup'))) {
        _zwListFixDisallowedAncestors(fbqBlock);
      }
    }
    return true;
  };

  // formatBlock value 读面（vendored implementation.js :6773-6805 为 spec 算法）。
  // Chrome 口径（语料 362 case qV-before/after 录音归纳）双端口径：起、终可见位
  // 置各自归一到内侧可见叶子后沿父链爬升（跳过非块名祖先——`<div><ol><li>[foo]
  // </ol></div>` 录音 "div" 证实穿越 li/ol；`<dl>[foo]</dl>` 录音 "dl" 证实读面
  // 名册比 apply 白名单宽、另收 dl），命中读面块名册且可编辑且无 prohibited 后
  // 代 → 返 tag；爬至 editing host → ""（宿主豁免——裸宿主 caret 形 16/16 录音
  // ""）；两端块不同 → ""（`<p>[foo</p>bar]` 等跨块五形录音 ""，apply 后选区
  // 重锚单块内不触发）。apply 后 caret 落 wrap 内录音非空（`<div>foo[]bar</div>`
  // qV-after="div"），故 caret 与 range 同算法、无 collapsed 守卫。
  // https://w3c.github.io/editing/execCommand.html#the-formatblock-command
  var _zwFbqValueBlockNames = _zwFbqFormattableNames.concat(['dl']);

  // 端点 → 可见叶子（元素锚归一）。文本锚直接用。元素锚按边界点折叠语义取
  // 行走域（2026-11-10 table partial-span qV 刀）：START 端 0<o<len 向前折
  // child[o] 子树首可见叶（行走域 [o,len)——`<tr data-start=1>` 起端折入
  // td2 内新块首叶，formatblock.js :83/:88/:418/:423 qV-after 录音
  // "div"/"p"）；END 端 0<o<len 向后折 child[o-1] 子树末可见叶（行走域
  // [0,o) 逆序早返——终点 (tr,2) 折回 td2 内新块与起端同块）；折叠域内无
  // 可见叶时回落全域旧走法（`{<p><p> <p>foo</p>}` apply 后新块含
  // [" ","foo"] 两文本节点、终点 (div,1) 折域 [0,1) 仅空白不可见，不回落
  // 会返 null 使 qV-after 丢 "div"——语料 4 条目实证）；其余形行走全域
  // 且恒读树序首可见叶（旧侧行为全保：`{<p>foo</p>}` 终点 (host,1) 全域
  // 读首叶爬回 p 录音 "p"、o=0 首叶锚全保——逆序不早返时 found 依次覆
  // 盖，终值仍为域内树序首叶）。
  function _zwFbqEdgeLeaf(node, offset, isEnd) {
    // 单域行走：[lo,hi) 前向早返 / 逆序按 earlyBack 早返（全域逆序不早返
    // 时 found 依次覆盖，终值=域内树序首可见叶，旧走法原样）。
    function walk(lo, hi, forward, earlyBack) {
      var stack = [];
      var found = null;
      var i;
      if (len) {
        if (forward) {
          for (i = hi - 1; i >= lo; i--) stack.push(kids[i]);
        } else {
          for (i = lo; i < hi; i++) stack.push(kids[i]);
        }
      } else {
        stack.push(node);
      }
      while (stack.length) {
        var n = stack.pop();
        if (!n) continue;
        if (n.nodeType === 3 || !n.childNodes || n.childNodes.length === 0) {
          if (_zwJusIsVisible(n)) {
            found = n;
            if (forward || earlyBack) return n;
          }
        } else {
          var nk = n.childNodes;
          if (forward) {
            for (i = nk.length - 1; i >= 0; i--) stack.push(nk[i]);
          } else {
            for (i = 0; i < nk.length; i++) stack.push(nk[i]);
          }
        }
      }
      return found;
    }
    if (node.nodeType === 3) return node;
    var kids = node.childNodes;
    var len = kids ? kids.length : 0;
    var interior = offset > 0 && offset < len;
    // 折叠域分派：内区锚（0<o<len）按端侧折入邻子树（END 折域需早返取末
    // 叶）；边界锚保持全域旧走法（o=0 前向早返 / 其余逆序不早返）。
    var lo = 0;
    var hi = len;
    var forward = offset === 0;
    var earlyBack = false;
    if (interior) {
      if (isEnd) {
        hi = offset;
        forward = false;
        earlyBack = true;
      } else {
        lo = offset;
        forward = true;
      }
    }
    var leaf = walk(lo, hi, forward, earlyBack);
    if (leaf === null && interior) {
      leaf = walk(0, len, false, false);
    }
    return leaf;
  }

  // 端点叶子 → FB 块：命中 _zwFbqValueBlockNames 且可编辑 → 返元素；爬至
  // editing host 或断链 → null。无 prohibited-paragraph veto：`<div><ol><li>
  // [foo]</ol></div>` qV-before 录音 "div"（:1784，div 含 li 仍返名），veto
  // 会使该形误返 ""。
  function _zwFbqVisBlock(node, offset, isEnd) {
    var leaf = _zwFbqEdgeLeaf(node, offset, isEnd);
    if (!leaf) return null;
    var cur = leaf;
    var g = 0;
    while (cur && g++ < 256) {
      if (_zwJusIsEditingHost(cur)) return null;
      // Chrome delta（table 结构 qV 隐形）：td/th/tr/table 族为块边界但非 FB
      // 块名，遇之即返 null——`<div><table><tr><td>[foo]</table></div>`
      // qV-before 录音 ""（:1794/:1799），穿 table 落外层 div 会误返 "div"。
      if (_zwJusIsHtmlElement(cur,
          ['td', 'th', 'tr', 'table', 'tbody', 'thead', 'tfoot'])) return null;
      if (_zwJusIsHtmlElement(cur, _zwFbqValueBlockNames)) {
        if (_zwJusIsEditable(cur)) return cur;
        return null;
      }
      cur = cur.parentNode;
    }
    return null;
  }

  // 范围面后缀扫描：末端 FB 块之后、编辑宿主内是否尚有可见 FB 块（子树前序
  // 扫，table 族 veto 同 _zwFbqVisBlock 读面隐形域）。
  function _zwFbqSubtreeHasVisBlock(root) {
    var stack = [root];
    var g = 0;
    while (stack.length && g++ < 256) {
      var n = stack.pop();
      if (!n) continue;
      if (_zwJusIsEditingHost(n)) continue;
      if (_zwJusIsHtmlElement(n,
          ['td', 'th', 'tr', 'table', 'tbody', 'thead', 'tfoot'])) continue;
      if (_zwJusIsHtmlElement(n, _zwFbqValueBlockNames) && _zwJusIsEditable(n)) {
        return true;
      }
      var kids = n.childNodes;
      for (var i = 0; i < kids.length; i++) stack.push(kids[i]);
    }
    return false;
  }

  function _zwFbqHasVisBlockAfter(node) {
    var cur = node;
    var g = 0;
    while (cur && g++ < 256) {
      var sib = cur.nextSibling;
      while (sib) {
        if (_zwFbqSubtreeHasVisBlock(sib)) return true;
        sib = sib.nextSibling;
      }
      cur = cur.parentNode;
      if (!cur || _zwJusIsEditingHost(cur)) return false;
    }
    return false;
  }

  // 异型多块公共祖先：自 b1 爬升（宿主边界止），首个 FB 块名且可编辑且含 b2
  // 的祖先即返；无则 null。qV（dl 跨块形）与 qI 否决共用。
  function _zwFbqCommonVisBlock(b1, b2) {
    var anc = b1.parentNode;
    var g = 0;
    while (anc && g++ < 256) {
      if (_zwJusIsEditingHost(anc)) return null;
      if (_zwJusIsEditable(anc) && _zwJusIsHtmlElement(anc, _zwFbqValueBlockNames)
      && _zwJusIsAncestorOf(anc, b2)) return anc;
      anc = anc.parentNode;
    }
    return null;
  }

  globalThis.__zw_ce_fb_value = function () {
    var range = _zwVaGetActiveRange();
    if (!range) return '';
    var b1 = _zwFbqVisBlock(range.startContainer, range.startOffset, false);
    if (!b1) return '';
    var b2 = _zwFbqVisBlock(range.endContainer, range.endOffset, true);
    if (!b2) return '';
    if (b1 === b2) return _zwJusTag(b1);
    // Chrome delta（范围多块面，2026-11-10 fb 读面刀）：同型多块按 tag 相等读
    // 块名——`<p>[foo<p>bar]` qV-before 录音 "p"（52 行实证，此前 b1!==b2 元素
    // 恒返 ""）；宿主尾块之后尚有可见 FB 块时 Chrome 读 ""——`<p>[foo<p>bar]
    // <p>baz` 录音 "" + qI=false（:708，范围面上下文敏感非端点语义）。
    if (_zwJusTag(b1) === _zwJusTag(b2)) {
      return _zwFbqHasVisBlockAfter(b2) ? '' : _zwJusTag(b1);
    }
    // Chrome delta（异型多块面）：读含两端的最近 FB 块公共祖先——`<dl><dt>
    // [foo<dd>bar]</dl>` qV-before 录音 "dl"（dt/dd 均为 FB 块名停于端点，
    // Chrome 读共同块 dl，fbVals=div/p 四条目 qV 全 "dl"）；宿主内无公共 FB 块
    // 祖先形（`<div>[foo<p>bar]</p></div>`、`<h1>[foo</h1><h2>bar]</h2>` 等）
    // 录音 ""。
    var anc = _zwFbqCommonVisBlock(b1, b2);
    return anc ? _zwJusTag(anc) : '';
  };

  // Chrome delta（formatBlock qI 读面，2026-11-10 fb 读面刀）：原始端点 FB 块
  // 恰单侧有块、或异型且宿主内无公共 FB 块祖先 → true（语料 qI true 14 行全由
  // 本判据覆盖：`<p>[foo</p>bar]` 单侧、`<h1>[foo</h1><h2>bar]</h2>` 异型无
  // 公共祖先等）；同型多块（含 `<p>[foo<p>bar]<p>baz` 尾块形 :708 录音 false）、
  // 单块、异型爬达共同祖先形（`<dl><dt>[foo<dd>bar]</dl>` fbVals=div 条目
  // qI 录音 false）→ false。同形 fbVals=p 条目 qI 录音 true 与 fbVals=div
  // false 互斥＝录音会话顺序泄漏签名，残留 2 行维持红挂账。端点折叠语义随
  // table partial-span qV 刀同步贯通（起点向前折/终点向后折，qV 同口径）。
  globalThis.__zw_ce_fb_indeterm = function () {
    var range = _zwVaGetActiveRange();
    if (!range) return false;
    var b1 = _zwFbqVisBlock(range.startContainer, range.startOffset, false);
    var b2 = _zwFbqVisBlock(range.endContainer, range.endOffset, true);
    if (!b1 || !b2) return !!(b1 || b2);
    if (_zwJusTag(b1) === _zwJusTag(b2)) return false;
    return !_zwFbqCommonVisBlock(b1, b2);
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

  // —— outdent 刀（2026-11-11）：outdent 语料 census v1 全 171 compare 行红，
  // got == 输入树（命令整体 no-op）——根因 = outdent 无实应用臂（part06 旧注
  // 「零变更无重锚不需要」）。刀体 = vendored outdentNode（:5520-5692）+
  // outdent action（:8209-8311）忠实移植，依赖 helper 全已存在（indentNodes/
  // splitParent/fixDisallowedAncestors/normalizeSublists/indentation 元素判据/
  // removePreservingDescendants/setTagName/recordValues/restoreValues）。
  // spec：https://w3c.github.io/editing/execCommand.html#outdent。
  // spec styleWithCSS 对 outdent 无分支（语料 swcss=true 行期望与 plain 同形，
  // 均为 unwrap/换壳面）。
  function _zwFbqHasEditableDescendant(node) {
    var stack = Array.prototype.slice.call(node.childNodes || []);
    var g = 0;
    while (stack.length && g++ < 4096) {
      var c = stack.pop();
      if (_zwJusIsEditable(c)) return true;
      var kids = c.childNodes;
      for (var i = 0; kids && i < kids.length; i++) stack.push(kids[i]);
    }
    return false;
  }

  // outdentNode（vendored implementation.js :5520-5692 忠实移植）。
  function _zwFbqOutdentNode(node) {
    // "If node is not editable, abort these steps."
    if (!_zwJusIsEditable(node)) return;
    // "If node is a simple indentation element, remove node, preserving its
    // descendants. Then abort these steps."
    if (_zwListIsSimpleIndentationElement(node)) {
      _zwJusRemovePreservingDescendants(node);
      return;
    }
    // "If node is an indentation element:"
    if (_zwListIsIndentationElement(node)) {
      // "Unset the dir attribute of node, if any."
      node.removeAttribute('dir');
      // "Unset the margin, padding, and border CSS properties of node."
      node.style.margin = '';
      node.style.padding = '';
      node.style.border = '';
      if (node.getAttribute('style') == ''
      || node.getAttribute('style') == 'border-width: initial; border-color: initial; ') {
        node.removeAttribute('style');
      }
      // "Set the tag name of node to "div"."
      _zwJusSetTagName(node, 'div');
      // "Abort these steps."
      return;
    }
    // "Let current ancestor be node's parent."
    var currentAncestor = node.parentNode;
    // "Let ancestor list be a list of nodes, initially empty."
    var ancestorList = [];
    // "While current ancestor is an editable Element that is neither a simple
    // indentation element nor an ol nor a ul, append current ancestor to
    // ancestor list and then set current ancestor to its parent."
    while (_zwJusIsEditable(currentAncestor)
    && currentAncestor.nodeType == 1
    && !_zwListIsSimpleIndentationElement(currentAncestor)
    && !_zwJusIsHtmlElement(currentAncestor, ['ol', 'ul'])) {
      ancestorList.push(currentAncestor);
      currentAncestor = currentAncestor.parentNode;
    }
    // "If current ancestor is not an editable simple indentation element:"
    if (!_zwJusIsEditable(currentAncestor)
    || !_zwListIsSimpleIndentationElement(currentAncestor)) {
      // "Let current ancestor be node's parent."
      currentAncestor = node.parentNode;
      // "Let ancestor list be the empty list."
      ancestorList = [];
      // "While current ancestor is an editable Element that is neither an
      // indentation element nor an ol nor a ul, append current ancestor to
      // ancestor list and then set current ancestor to its parent."
      while (_zwJusIsEditable(currentAncestor)
      && currentAncestor.nodeType == 1
      && !_zwListIsIndentationElement(currentAncestor)
      && !_zwJusIsHtmlElement(currentAncestor, ['ol', 'ul'])) {
        ancestorList.push(currentAncestor);
        currentAncestor = currentAncestor.parentNode;
      }
    }
    // "If node is an ol or ul and current ancestor is not an editable
    // indentation element:"
    if (_zwJusIsHtmlElement(node, ['ol', 'ul'])
    && (!_zwJusIsEditable(currentAncestor)
    || !_zwListIsIndentationElement(currentAncestor))) {
      // "Unset the reversed, start, and type attributes of node."
      node.removeAttribute('reversed');
      node.removeAttribute('start');
      node.removeAttribute('type');
      // "Let children be the children of node."
      var children = Array.prototype.slice.call(node.childNodes);
      // "If node has attributes, and its parent is not an ol or ul, set the
      // tag name of node to "div"."
      if (node.attributes.length
      && !_zwJusIsHtmlElement(node.parentNode, ['ol', 'ul'])) {
        _zwJusSetTagName(node, 'div');
      } else {
        // "Record the values of node's children, remove node preserving its
        // descendants, restore the values."
        var values = _zwDelRecordValues(Array.prototype.slice.call(node.childNodes));
        _zwJusRemovePreservingDescendants(node);
        _zwDelRestoreValues(values);
      }
      // "Fix disallowed ancestors of each member of children."
      for (var ci = 0; ci < children.length; ci++) {
        _zwListFixDisallowedAncestors(children[ci]);
      }
      // "Abort these steps."
      return;
    }
    // "If current ancestor is not an editable indentation element, abort."
    if (!_zwJusIsEditable(currentAncestor)
    || !_zwListIsIndentationElement(currentAncestor)) {
      return;
    }
    // "Append current ancestor to ancestor list."
    ancestorList.push(currentAncestor);
    // "Let original ancestor be current ancestor."
    var originalAncestor = currentAncestor;
    // "While ancestor list is not empty:"
    while (ancestorList.length) {
      // "Let current ancestor be the last member of ancestor list, and remove
      // the last member."
      currentAncestor = ancestorList.pop();
      // "Let target be the child of current ancestor that is equal to either
      // node or the last member of ancestor list."
      var target = node.parentNode == currentAncestor
        ? node
        : ancestorList[ancestorList.length - 1];
      // "If target is an inline node that is not a br, and its nextSibling is
      // a br, remove target's nextSibling from its parent."
      if (_zwJusIsInlineNode(target)
      && !_zwJusIsHtmlElement(target, 'br')
      && _zwJusIsHtmlElement(target.nextSibling, 'br')) {
        target.parentNode.removeChild(target.nextSibling);
      }
      // "Let preceding siblings be the preceding siblings of target, and let
      // following siblings be the following siblings of target."
      var tIdx = _zwJusNodeIndex(target);
      var kids = currentAncestor.childNodes;
      var precedingSiblings = Array.prototype.slice.call(kids, 0, tIdx);
      var followingSiblings = Array.prototype.slice.call(kids, 1 + tIdx);
      // "Indent preceding siblings."
      _zwFbqIndentNodes(precedingSiblings);
      // "Indent following siblings."
      _zwFbqIndentNodes(followingSiblings);
    }
    // "Outdent original ancestor."
    _zwFbqOutdentNode(originalAncestor);
  }

  // outdent action（vendored implementation.js :8209-8311 忠实移植）。
  globalThis.__zw_ce_outdent_apply = function () {
    var range = _zwVaGetActiveRange();
    if (!range) return true;
    // "Let items be a list of all lis that are ancestor containers of the
    // range's start and/or end node."（树序；vendored 端臂停 cca、start 臂
    // 爬至根——两臂守卫不对称是 vendored 原样，非移植笔误。）
    var cca = _zwFbqCommonAncestor(range.startContainer, range.endContainer);
    var items = [];
    var n = range.endContainer;
    while (n && n !== cca) {
      if (_zwJusIsHtmlElement(n, 'li')) items.unshift(n);
      n = n.parentNode;
    }
    n = range.startContainer;
    while (n) {
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
    // "Let node list be the contained nodes whose last member is not an
    // ancestor; node is editable; and node has no editable descendants, or is
    // an ol or ul, or is an li whose parent is an ol or ul."
    var nodeList = _zwJusGetContainedNodes(newRange, function (node) {
      return _zwJusIsEditable(node)
        && (!_zwFbqHasEditableDescendant(node)
        || _zwJusIsHtmlElement(node, ['ol', 'ul'])
        || (_zwJusIsHtmlElement(node, 'li')
            && _zwJusIsHtmlElement(node.parentNode, ['ol', 'ul'])));
    });
    // "While node list is not empty: ..."
    while (nodeList.length) {
      // "While the first member of node list is an ol or ul or is not the
      // child of an ol or ul, outdent it and remove it from node list."
      while (nodeList.length
      && (_zwJusIsHtmlElement(nodeList[0], ['ol', 'ul'])
      || !_zwJusIsHtmlElement(nodeList[0].parentNode, ['ol', 'ul']))) {
        _zwFbqOutdentNode(nodeList.shift());
      }
      // "If node list is empty, break."
      if (!nodeList.length) break;
      // "Let sublist be [first], extended over consecutive nextSiblings that
      // are not ol or ul."
      var sublist = [nodeList.shift()];
      while (nodeList.length
      && nodeList[0] === sublist[sublist.length - 1].nextSibling
      && !_zwJusIsHtmlElement(nodeList[0], ['ol', 'ul'])) {
        sublist.push(nodeList.shift());
      }
      // "Record the values of sublist; split the parent of sublist with new
      // parent null; fix disallowed ancestors; restore the values."
      var values = _zwDelRecordValues(sublist);
      _zwListSplitParent(sublist);
      for (var j = 0; j < sublist.length; j++) {
        _zwListFixDisallowedAncestors(sublist[j]);
      }
      _zwDelRestoreValues(values);
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
