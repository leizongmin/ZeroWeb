  // ============================================================
  // editing-compat style-with-value 簇（2026-11-01）：execCommand 五标准行内值
  // 命令（fontsize/fontname/forecolor/backcolor/hilitecolor）实应用引擎。
  //
  // 归因（gate7 语料普查 + zz-probe-fs6 探针实证）：run 语料 compare innerHTML
  // 567F 全部走 document.execCommand（tests.js:5613——compare 列读 testDiv，
  // vendored myExecCommand 只服务 spec 表格列），而引擎 execCommand 此前只有
  // _zwExecCmdInputType 事件映射（beforeinput/input 照派）+ 值查询面
  // （part01c __zw_ce_value_query，2026-11-01 接通），零变更臂 → 一切期待
  // 包裹的 apply 行 no-op 失败（探针实证 size 2/4/6 全返原始 HTML）。
  //
  // 本文件按 editing spec 参考实现（tests/wpt-runner/wpt-data/editing/include/
  // implementation.js）移植 setSelectionValue 全流程：
  //   ① 无 contained formattable（collapsed）→ state/value override 记账（值
  //     override 经 part01c __zw_ce_value_query 回读——Chrome qV-after 期望源）；
  //   ② 边界 splitText（引擎安全拆分：data 写 + createTextNode + insertBefore
  //     ——引擎 splitText 仅 handle-based wrapper 可用，plain 文本节点缺法，
  //     zz-probe-fs4 实证）；
  //   ③ contained element clearValue pass + node pushDownValues/forceValue
  //     pass（forceValue：CSS flag false 时 font color/face/size attr 臂 +
  //     span style 写入尾；wrap/reorder/isSimpleModifiableElement 全链）；
  //   ④ areEquivalentValues（色三命令 normalizeColor 等值）+
  //     areLooselyEquivalentValues（fontsize sizeMap——font[size] 1-7 级联
  //     px 读数静态表）。
  // 共享机具复用 part01c 闭包：_zwJus*（move/range/tree/visible/editable/cs）
  // + _zwVal*（containment/formattable/effective）+ _zwPlainDecl/_zwPlainColorSpec
  // （specified style 读数与颜色解析）。选区单 range 面经 sel._ranges[0]
  // （_zwJusMovePreservingRanges 边界补偿回写——对象身份不变）。
  // 挂载位：js_dom_bridge.rs 在 part01c 后、part07（闭合外层 IIFE）前拼接，
  // 同闭包可见全部 _zwJus*/_zwVal* 机具；part06 经 globalThis.__zw_ce_value_apply
  // 单点调用。
  // https://w3c.github.io/editing/execCommand.html#set-the-selection's-value
  // https://w3c.github.io/editing/execCommand.html#forcing-the-value-of-a-node
  // FIXME: 值等值/解析面窄口——rgb() 空格语法与 calc()/系统色未覆盖（语料
  //   范围外）；isExtraneousLineBreak 沿用 justify 簇近似（非布局探测）。
  // FIXME: bold/italic/underline/strikethrough 等其余 format 命令仍走
  //   part06 既有 flat 臂（本文件不接管）。
  // ============================================================

  // —— 值命令表（五标准行内值命令）——
  var _zwVaFive = { fontsize: 1, fontname: 1, forecolor: 1, backcolor: 1, hilitecolor: 1 };

  function _zwVaCssFlag() {
    return !!globalThis.__zwStyleWithCSS;
  }

  function _zwVaGetActiveRange() {
    var sel = globalThis.getSelection ? globalThis.getSelection() : null;
    if (!sel || !sel.rangeCount) return null;
    var r = sel._ranges && sel._ranges.length ? sel._ranges[0] : null;
    if (!r || !r.startContainer || !r.endContainer) return null;
    if (r.startContainer.nodeType !== 1 && r.startContainer.nodeType !== 3) return null;
    if (r.endContainer.nodeType !== 1 && r.endContainer.nodeType !== 3) return null;
    return r;
  }

  // —— CSSOM 口径颜色归一（参考实现 :415 normalizeColor 的本仓等价面）——
  // vendored 借真浏览器 gCS 通道归一（钳位/命名/hex/hsl/rgba α=1 合并全在此
  // 发生）；本仓复用 part01c _zwPlainColorSpec（named 表 + rgb/hsl 序列化）
  // 后补 rgb 分量 clamp——plain 解析未钳而 gCS 原生钳位（语料 rgb(15, -10,
  // 375) → rgb(15, 0, 255) 实证）。不可解析返 ''（调用方按无效色分支）。
  function _zwVaNormalizeCssColor(v) {
    var out;
    try {
      out = _zwPlainColorSpec(String(v == null ? '' : v), null);
    } catch (_eNcc) {
      return '';
    }
    if (!out) return '';
    var m = /^(rgba?)\(([^)]*)\)$/.exec(out);
    if (!m) return out;
    var nums = m[2].split(', ');
    for (var i = 0; i < 3; i++) {
      var n = Math.round(Number(nums[i]));
      nums[i] = String(n < 0 ? 0 : (n > 255 ? 255 : n));
    }
    return m[1] + '(' + nums.join(', ') + ')';
  }

  // parseSimpleColor（参考实现 :471）：normalizeColor → 'rgb(N, N, N)' 形 →
  // '#xxxxxx'（HTML 简单色序列化形态，语料期望 <font color="#0000ff"> 实证）。
  // rgba(α<1)/transparent/currentcolor 归一后非 rgb 形 → null（同参考实现——
  // CSS false 旗由此落 span+style 臂而非 font 臂）。
  function _zwVaParseSimpleColor(v) {
    var c = _zwVaNormalizeCssColor(v);
    if (!c) return null;
    if (!/^rgb\([0-9]+, [0-9]+, [0-9]+\)$/.test(c)) return null;
    var nums = c.slice(4, -1).split(', ');
    var hex = function (n) {
      var h = Number(n).toString(16);
      return h.length < 2 ? '0' + h : h;
    };
    return '#' + hex(nums[0]) + hex(nums[1]) + hex(nums[2]);
  }

  // normalizeColor 等值键：可解析 → 规整 rgb/rgba 串（含钳位）；不可解析 →
  // ''（两侧皆不可解析 = 等值，参考实现「neither string is a valid CSS color」臂）。
  function _zwVaNormalizeColorKey(v) {
    return _zwVaNormalizeCssColor(v);
  }

  // areEquivalentValues（参考实现 :2008）——五命令子集：fontsize/fontname 无
  // equivalentValues 定义（纯串等值），色三命令 normalizeColor 等值。
  function _zwVaEquivalentValues(cmd, v1, v2) {
    if (v1 === null && v2 === null) return true;
    if (typeof v1 !== 'string' || typeof v2 !== 'string') return false;
    if (v1 === v2) return true;
    if (cmd === 'forecolor' || cmd === 'backcolor' || cmd === 'hilitecolor') {
      return _zwVaNormalizeColorKey(v1) === _zwVaNormalizeColorKey(v2);
    }
    return false;
  }

  // areLooselyEquivalentValues 的 sizeMap（参考实现 :2049-2071）——font[size]
  // 1-7 的级联 font-size 读数静态表。**host 引擎 UA 计算口径**（10/13/16/18/
  // 24/32/48px 整数表，与 part01c _zwValLegacyBounds 中点标度同构）——不复刻
  // vendored 的动态 gCS 读法：createElement 自建 wrapper 无 host 映射，gCS 走
  // plain 回落表（10.08/13.12/18.08px 小数），与 effectiveValue 的 host 通道
  // （语料元素 handle-proxy → 18px 整数）异链——动态读 sizeMap['large']=
  // '18.08px' ≠ effective '18px'，looselyEquivalent 恒 false 致同值选区误包
  // （fontsize 5 例 P→F 实证）。Chrome 两侧同走原生 gCS 恒同值，我方等价面 =
  // 与 effectiveValue 链一致。
  var _zwVaSizeMapData = {
    'x-small': '10px', small: '13px', medium: '16px', large: '18px',
    'x-large': '24px', 'xx-large': '32px', 'xxx-large': '48px'
  };
  function _zwVaSizeMap() {
    return _zwVaSizeMapData;
  }

  function _zwVaLooselyEquivalent(cmd, v1, v2) {
    if (_zwVaEquivalentValues(cmd, v1, v2)) return true;
    if (cmd !== 'fontsize' || typeof v1 !== 'string' || typeof v2 !== 'string') return false;
    var map = _zwVaSizeMap();
    return v1 === map[v2] || v2 === map[v1];
  }

  // getSpecifiedCommandValue（参考实现 :2201）——五命令子集：色三命令 display
  // 非 inline → null；style attr 相关属性读数（_zwPlainDecl：style IDL 优先、
  // attr 扫描兜底）；font color/face/size 表现提示臂（size → 关键字表）。
  function _zwVaSpecifiedValue(el, cmd) {
    if (!el || el.nodeType !== 1) return null;
    if ((cmd === 'backcolor' || cmd === 'hilitecolor')
        && _zwJusCs(el, 'display') !== 'inline') {
      return null;
    }
    var prop = cmd === 'fontsize' ? 'font-size'
      : (cmd === 'fontname' ? 'font-family'
        : (cmd === 'forecolor' ? 'color' : 'background-color'));
    var sv = _zwPlainDecl(el, prop);
    if (sv) return String(sv);
    if (_zwJusTag(el) === 'font') {
      if (cmd === 'forecolor' && el.hasAttribute && el.hasAttribute('color')) {
        return el.getAttribute('color');
      }
      if (cmd === 'fontname' && el.hasAttribute && el.hasAttribute('face')) {
        return el.getAttribute('face');
      }
      if (cmd === 'fontsize' && el.hasAttribute && el.hasAttribute('size')) {
        var size = parseInt(el.getAttribute('size'), 10);
        if (isNaN(size)) return null;
        if (size < 1) size = 1;
        if (size > 7) size = 7;
        return { 1: 'x-small', 2: 'small', 3: 'medium', 4: 'large',
          5: 'x-large', 6: 'xx-large', 7: 'xxx-large' }[size];
      }
    }
    return null;
  }

  // cssSizeToLegacy（参考实现 :138）——关键字 → 1-7。
  function _zwVaCssSizeToLegacy(cssVal) {
    return { 'x-small': '1', small: '2', medium: '3', large: '4',
      'x-large': '5', 'xx-large': '6', 'xxx-large': '7' }[cssVal];
  }

  // 绝对长度 → px（CSS 绝对单位 96dpi 口径；'2em' 等相对单位 → null）。
  function _zwVaPxOf(s) {
    var m = /^([0-9]+(\.[0-9]+)?)(cm|mm|in|pt|pc|px)$/.exec(s);
    if (!m) return null;
    var n = parseFloat(m[1]);
    switch (m[3]) {
      case 'cm': return n * 96 / 2.54;
      case 'mm': return n * 96 / 25.4;
      case 'in': return n * 96;
      case 'pt': return n * 96 / 72;
      case 'pc': return n * 16;
      default: return n;
    }
  }

  // getLegacyFontSize 关键字/数字臂（参考实现 :3320 头段）——override 读数
  // 归一（qV 面需 legacy 数字串；'2em' 等不可映射 → null）。
  function _zwVaLegacyValueOf(size) {
    var s = String(size == null ? '' : size);
    var kw = _zwVaCssSizeToLegacy(s);
    if (kw) return kw;
    if (_zwVaNormalizeFontSize(s) !== null) return _zwVaCssSizeToLegacy(_zwVaNormalizeFontSize(s));
    if (/^[0-9]+(\.[0-9]+)?(cm|mm|in|pt|pc|px)$/.test(s)) {
      var px = _zwVaPxOf(s);
      if (px !== null) {
        // 中点映射（同 _zwValLegacyFontSize 标度）。
        var bounds = _zwValLegacyBounds;
        for (var i = 0; i < 6; i++) {
          var next = i < 5 ? bounds[i + 1] : 48;
          if (px < (bounds[i] + next) / 2) return String(i + 1);
        }
        return '7';
      }
    }
    return null;
  }

  // —— style 写入（CSS 化尾：span style 臂）——engine style IDL setProperty
  // 优先；**读核验以 style attr 为权威面**（innerHTML 序列化与 plain gCS 回落
  // 都读 attr——host setProperty 通道对未注册 handle 的自建 wrapper 只写内部
  // 样式面、attr 未落，序列化即丢声明：fontsize wrap span 丢 style 实证）。
  // 核验不过回落 style attr 手工合并（plain/M 形工厂兼容）；清空值从 attr 面
  // 移除该声明。
  function _zwVaSetStyleProp(el, prop, val) {
    var valS = String(val);
    // CSSOM 口径：color 系写入即归一（浏览器 style.setProperty 原生序列化
    // rgb() 形，语料 color:rgb(0, 0, 255) 实证——裸 attr 合并丢归一即 compare
    // 面 color:#0000FF 错形）。归一失败（非色值）保持原串。
    if (prop === 'color' || prop === 'background-color') {
      var norm = _zwVaNormalizeCssColor(valS);
      if (norm) valS = norm;
    }
    try {
      if (el.style && typeof el.style.setProperty === 'function') {
        el.style.setProperty(prop, valS);
      }
    } catch (_eSp) {}
    try {
      var decls = _zwPlainStyleDecls(el);
      var cur = String(decls[prop] == null ? '' : decls[prop]);
      if (cur !== valS) {
        if (valS === '') delete decls[prop];
        else decls[prop] = valS;
        var parts = [];
        for (var k in decls) {
          if (Object.prototype.hasOwnProperty.call(decls, k)) parts.push(k + ': ' + decls[k]);
        }
        var serial = parts.join('; ');
        if (serial) el.setAttribute('style', serial);
        else el.removeAttribute('style');
      }
    } catch (_eSp2) {}
  }

  // —— isAllowedChild（参考实现 :1572）——inline contents 白名单 +
  // prohibited paragraph child 名单（static 表照录）。
  var _zwVaInlineContents = ['a', 'abbr', 'b', 'bdi', 'bdo', 'cite', 'code', 'dfn',
    'em', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'i', 'kbd', 'mark', 'p', 'pre', 'q',
    'rp', 'rt', 'ruby', 's', 'samp', 'small', 'span', 'strong', 'sub', 'sup', 'u',
    'var', 'acronym', 'listing', 'strike', 'xmp', 'big', 'blink', 'font', 'marquee',
    'nobr', 'tt'];
  var _zwVaProhibitedParagraphChildren = ['address', 'article', 'aside', 'blockquote',
    'canvas', 'center', 'dd', 'details', 'dialog', 'dir', 'div', 'dl', 'fieldset',
    'figcaption', 'figure', 'footer', 'form', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
    'header', 'hgroup', 'hr', 'li', 'main', 'menu', 'nav', 'ol', 'p', 'pre',
    'section', 'summary', 'table', 'ul'];

  function _zwVaIsElementWithInlineContents(el) {
    return el && el.nodeType === 1
      && _zwVaInlineContents.indexOf(_zwJusTag(el)) !== -1;
  }

  function _zwVaIsAllowedChild(child, parent_) {
    var blockTab = ['colgroup', 'table', 'tbody', 'tfoot', 'thead', 'tr'];
    if ((blockTab.indexOf(parent_) !== -1
         || _zwJusIsHtmlElement(parent_, blockTab))
        && child && child.nodeType === 3
        && !/^[ \t\n\f\r]*$/.test(String(child.data))) {
      return false;
    }
    var noNonText = ['script', 'style', 'plaintext', 'xmp'];
    if ((noNonText.indexOf(parent_) !== -1
         || _zwJusIsHtmlElement(parent_, noNonText))
        && (!child || child.nodeType !== 3)) {
      return false;
    }
    if (child && (child.nodeType === 9 || child.nodeType === 11 || child.nodeType === 10)) {
      return false;
    }
    var childName = null;
    if (child && child.nodeType === 1 && _zwJusIsHtmlElement(child)) {
      childName = _zwJusTag(child);
    }
    if (childName === null) return true;
    // 字符串 parent（forceValue 'span' 字面调用）无 DOM 祖先链——vendored
    // 尾段 table 的字符串参与面照录（'span'×prohibited paragraph child →
    // false；此前直接进 DOM 爬升遇字符串 nodeType!==1 即 break 恒 true，p 元
    // 素被误 wrap——fontsize 跨 p 案实证）。
    if (typeof parent_ === 'string') {
      if (childName === 'a' && parent_ === 'a') return false;
      if (_zwVaInlineContents.indexOf(parent_) !== -1
          && _zwVaProhibitedParagraphChildren.indexOf(childName) !== -1) {
        return false;
      }
      if (/^h[1-6]$/.test(childName) && /^h[1-6]$/.test(parent_)) return false;
      return true;
    }
    var ancestor = parent_;
    var guard = 0;
    while (ancestor && guard++ < 256) {
      if (ancestor.nodeType !== 1) break;
      if (childName === 'a' && _zwJusIsHtmlElement(ancestor, 'a')) return false;
      if (_zwVaProhibitedParagraphChildren.indexOf(childName) !== -1
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
    return true;
  }

  // —— isModifiableElement / isSimpleModifiableElement（参考实现 :1833/:1883）
  // ——style 单属性形态读数经 _zwPlainDecl（style attr 朴素扫描键数）。
  function _zwVaIsModifiable(node) {
    if (!node || node.nodeType !== 1 || !_zwJusIsHtmlElement(node)) return false;
    var tag = _zwJusTag(node);
    var nAttrs = node.attributes && node.attributes.length ? node.attributes.length : 0;
    var hasStyle = node.hasAttribute && node.hasAttribute('style');
    if (['b', 'em', 'i', 's', 'span', 'strike', 'strong', 'sub', 'sup', 'u']
        .indexOf(tag) !== -1) {
      if (nAttrs === 0) return true;
      if (nAttrs === 1 && hasStyle) return true;
    }
    if (tag === 'font' || tag === 'a') {
      var num = nAttrs;
      if (hasStyle) num--;
      if (tag === 'font') {
        if (node.hasAttribute && node.hasAttribute('color')) num--;
        if (node.hasAttribute && node.hasAttribute('face')) num--;
        if (node.hasAttribute && node.hasAttribute('size')) num--;
      }
      if (tag === 'a' && node.hasAttribute && node.hasAttribute('href')) num--;
      if (num === 0) return true;
    }
    return false;
  }

  function _zwVaIsSimpleModifiable(node) {
    if (!node || node.nodeType !== 1 || !_zwJusIsHtmlElement(node)) return false;
    var tag = _zwJusTag(node);
    if (['a', 'b', 'em', 'font', 'i', 's', 'span', 'strike', 'strong', 'sub',
         'sup', 'u'].indexOf(tag) === -1) return false;
    var nAttrs = node.attributes && node.attributes.length ? node.attributes.length : 0;
    if (nAttrs === 0) return true;
    if (nAttrs > 1) return false;
    var hasStyle = node.hasAttribute && node.hasAttribute('style');
    // 恰一属性 = style 且零声明（语料形态；单属性 prop 臂逐属性核对）。
    if (hasStyle) {
      var decls = _zwPlainStyleDecls(node);
      var keys = [];
      for (var k in decls) {
        if (Object.prototype.hasOwnProperty.call(decls, k)) keys.push(k);
      }
      if (keys.length === 0) return true;
      if (keys.length === 1) {
        var p0 = keys[0];
        if ((tag === 'b' || tag === 'strong') && p0 === 'font-weight') return true;
        if ((tag === 'i' || tag === 'em') && p0 === 'font-style') return true;
        if ((tag === 'a' || tag === 'font' || tag === 'span') && p0 !== 'text-decoration') {
          return true;
        }
        if (['a', 'font', 's', 'span', 'strike', 'u'].indexOf(tag) !== -1
            && p0 === 'text-decoration'
            && ['line-through', 'underline', 'overline', 'none']
              .indexOf(String(decls[p0])) !== -1) {
          return true;
        }
      }
      return false;
    }
    if (tag === 'a' && node.hasAttribute && node.hasAttribute('href')) return true;
    if (tag === 'font'
        && (node.hasAttribute('color') || node.hasAttribute('face')
            || node.hasAttribute('size'))) {
      return true;
    }
    return false;
  }

  // —— remove extraneous line breaks（参考实现 :1305-1377）——复用 justify 簇
  // 近似 extraneous 判定（非布局探测；FIXME 同 justify 簇挂账）。
  function _zwVaRemoveExtraneousLineBreaksBefore(node) {
    var ref = node.previousSibling;
    if (!ref) return;
    while (_zwJusHasKids(ref)) ref = ref.lastChild;
    var guard = 0;
    while (_zwJusIsInvisible(ref) && !_zwJusIsExtraneousLineBreak(ref)
           && ref !== node.parentNode && guard++ < 4096) {
      ref = _zwJusPreviousNode(ref);
    }
    if (_zwJusIsEditable(ref) && _zwJusIsExtraneousLineBreak(ref)) {
      if (ref.parentNode) ref.parentNode.removeChild(ref);
    }
  }

  function _zwVaRemoveExtraneousLineBreaksAtTheEndOf(node) {
    var ref = node;
    while (_zwJusHasKids(ref)) ref = ref.lastChild;
    var guard = 0;
    while (_zwJusIsInvisible(ref) && !_zwJusIsExtraneousLineBreak(ref)
           && ref !== node && guard++ < 4096) {
      ref = _zwJusPreviousNode(ref);
    }
    if (_zwJusIsEditable(ref) && _zwJusIsExtraneousLineBreak(ref)) {
      while (_zwJusIsEditable(ref.parentNode) && _zwJusIsInvisible(ref.parentNode)) {
        ref = ref.parentNode;
      }
      if (ref.parentNode) ref.parentNode.removeChild(ref);
    }
  }

  function _zwVaRemoveExtraneousLineBreaksFrom(node) {
    _zwVaRemoveExtraneousLineBreaksBefore(node);
    _zwVaRemoveExtraneousLineBreaksAtTheEndOf(node);
  }

  // —— wrap（参考实现 :1377）——globalRange 簿记换 sel._ranges[0] 单 range
  // 补偿（新 parent 直接插入臂的边界 offset 右移；移动臂经
  // _zwJusMovePreservingRanges 自带补偿）。
  function _zwVaWrap(nodeList, siblingCriteria, newParentInstructions) {
    if (typeof siblingCriteria === 'undefined') {
      siblingCriteria = function () { return false; };
    }
    if (typeof newParentInstructions === 'undefined') {
      newParentInstructions = function () { return null; };
    }
    var allInvisible = true;
    var i;
    for (i = 0; i < nodeList.length; i++) {
      if (!_zwJusIsInvisible(nodeList[i])) { allInvisible = false; break; }
    }
    var hasBr = false;
    for (i = 0; i < nodeList.length; i++) {
      if (_zwJusIsHtmlElement(nodeList[i], 'br')) { hasBr = true; break; }
    }
    if (allInvisible && !hasBr) return null;
    if (!nodeList[0].parentNode) return null;

    var last = nodeList[nodeList.length - 1];
    if (!_zwJusIsBlockNode(last) && !_zwJusIsHtmlElement(last, 'br')
        && _zwJusIsHtmlElement(last.nextSibling, 'br')) {
      nodeList.push(last.nextSibling);
    }
    // Chrome delta：拉邻条件放宽——vendored 只拉 invisible 兄弟，空白-only 文本
    // 兄弟同样随携带（`<span>foo</span> <span>bar</span>` 合并形语料期望空白并入
    // 单一外层样式化 wrapper：空白先入列 sibCrit 才能沿 nodeList[0].previousSibling
    // 命中已样式化邻接 wrapper 合并；仅随携带不独获值——forceValue 入口空白门）。
    while (nodeList[0].previousSibling
        && (_zwJusIsInvisible(nodeList[0].previousSibling) || _zwVaIsWsText(nodeList[0].previousSibling))) {
      nodeList.unshift(nodeList[0].previousSibling);
    }
    while (nodeList[nodeList.length - 1].nextSibling
        && (_zwJusIsInvisible(nodeList[nodeList.length - 1].nextSibling) || _zwVaIsWsText(nodeList[nodeList.length - 1].nextSibling))) {
      nodeList.push(nodeList[nodeList.length - 1].nextSibling);
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
      var insParent = nodeList[0].parentNode;
      var insIdx = _zwJusNodeIndex(newParent);
      // 边界点补偿：宿主选区 boundary 落插入点及之后 → offset +1。
      var selW = globalThis.getSelection ? globalThis.getSelection() : null;
      var rngW = selW && selW._ranges && selW._ranges.length ? selW._ranges[0] : null;
      if (rngW) {
        if (rngW.startContainer === insParent && rngW.startOffset >= insIdx) {
          try { rngW.setStart(rngW.startContainer, rngW.startOffset + 1); } catch (_eW1) {}
        }
        if (rngW.endContainer === insParent && rngW.endOffset >= insIdx) {
          try { rngW.setEnd(rngW.endContainer, rngW.endOffset + 1); } catch (_eW2) {}
        }
      }
      try {
        insParent.insertBefore(newParent, nodeList[0]);
      } catch (_eW3) {
        try { insParent.appendChild(newParent); } catch (_eW4) { return null; }
      }
    }

    var originalParent = nodeList[0].parentNode;
    var inlineKids = function (n) {
      return n && !_zwJusIsBlockNode(n);
    };

    if (_zwJusIsBefore(newParent, nodeList[0])) {
      var visKids = [];
      for (i = 0; i < newParent.childNodes.length; i++) {
        if (_zwJusIsVisible(newParent.childNodes[i])) visKids.push(newParent.childNodes[i]);
      }
      var visMembers = [];
      for (i = 0; i < nodeList.length; i++) {
        if (_zwJusIsVisible(nodeList[i])) visMembers.push(nodeList[i]);
      }
      if (!_zwJusIsBlockNode(newParent)
          && inlineKids(visKids[visKids.length - 1])
          && inlineKids(visMembers[0])
          && !_zwJusIsHtmlElement(newParent.lastChild, 'br')
          // Chrome delta：连接点被空白-only 成员隔断时不插 br（`<span>foo</span>
          // <span>bar</span>` 合并形语料期望无 br——空白已防 run 连字，vendored
          // br 反成多余可见断行）。
          && !_zwVaIsWsText(nodeList[0])) {
        newParent.appendChild(document.createElement('br'));
      }
      for (i = 0; i < nodeList.length; i++) {
        _zwJusMovePreservingRanges(nodeList[i], newParent, -1);
      }
    } else {
      var firstVis = null;
      for (i = 0; i < newParent.childNodes.length; i++) {
        if (_zwJusIsVisible(newParent.childNodes[i])) { firstVis = newParent.childNodes[i]; break; }
      }
      var lastVisMember = null;
      for (i = nodeList.length - 1; i >= 0; i--) {
        if (_zwJusIsVisible(nodeList[i])) { lastVisMember = nodeList[i]; break; }
      }
      if (!_zwJusIsBlockNode(newParent)
          && inlineKids(firstVis)
          && inlineKids(lastVisMember)
          && !_zwJusIsHtmlElement(nodeList[nodeList.length - 1], 'br')
          // Chrome delta：同上——nodeList 侧端点是空白-only 成员时不插 br。
          && !_zwVaIsWsText(nodeList[nodeList.length - 1])) {
        newParent.insertBefore(document.createElement('br'), newParent.firstChild);
      }
      for (i = nodeList.length - 1; i >= 0; i--) {
        _zwJusMovePreservingRanges(nodeList[i], newParent, 0);
      }
    }

    if (_zwJusIsEditable(originalParent) && !_zwJusHasKids(originalParent)) {
      if (originalParent.parentNode) {
        originalParent.parentNode.removeChild(originalParent);
      }
    }

    if (_zwJusIsEditable(newParent.nextSibling)
        && siblingCriteria(newParent.nextSibling)) {
      if (!_zwJusIsBlockNode(newParent)
          && inlineKids(newParent.lastChild)
          && inlineKids(newParent.nextSibling.firstChild)
          && !_zwJusIsHtmlElement(newParent.lastChild, 'br')
          // Chrome delta：同上——连接点 newParent 侧端点是空白-only 文本时不插 br。
          && !_zwVaIsWsText(newParent.lastChild)) {
        newParent.appendChild(document.createElement('br'));
      }
      var guardM = 0;
      while (_zwJusHasKids(newParent.nextSibling) && guardM++ < 4096) {
        _zwJusMovePreservingRanges(newParent.nextSibling.firstChild, newParent, -1);
      }
      if (newParent.nextSibling.parentNode) {
        newParent.parentNode.removeChild(newParent.nextSibling);
      }
    }

    _zwVaRemoveExtraneousLineBreaksFrom(newParent);
    return newParent;
  }

  // —— clearValue（参考实现 :2476）——五命令子集。
  function _zwVaClearValue(element, command) {
    if (!_zwJusIsEditable(element)) return [];
    if (_zwVaSpecifiedValue(element, command) === null) return [];
    if (_zwVaIsSimpleModifiable(element)) {
      var kids = [];
      var i;
      for (i = 0; i < element.childNodes.length; i++) kids.push(element.childNodes[i]);
      var idx = _zwJusNodeIndex(element);
      for (i = 0; i < kids.length; i++) {
        _zwJusMovePreservingRanges(kids[i], element.parentNode, idx + i);
      }
      if (element.parentNode) element.parentNode.removeChild(element);
      return kids;
    }
    var prop = command === 'fontsize' ? 'font-size'
      : (command === 'fontname' ? 'font-family'
        : (command === 'forecolor' ? 'color' : 'background-color'));
    if (_zwPlainDecl(element, prop)) {
      _zwVaSetStyleProp(element, prop, '');
      try {
        var attr = element.getAttribute('style');
        if (attr !== null && String(attr).trim() === '') element.removeAttribute('style');
      } catch (_eCv) {}
    }
    if (_zwJusTag(element) === 'font') {
      if (command === 'forecolor' && element.hasAttribute('color')) {
        element.removeAttribute('color');
      }
      if (command === 'fontname' && element.hasAttribute('face')) {
        element.removeAttribute('face');
      }
      if (command === 'fontsize' && element.hasAttribute('size')) {
        element.removeAttribute('size');
      }
    }
    if (_zwVaSpecifiedValue(element, command) === null) return [];
    return [_zwJusSetTagName(element, 'span')];
  }

  // —— pushDownValues（参考实现 :2586）。
  function _zwVaPushDownValues(node, command, newValue) {
    if (!node.parentNode || node.parentNode.nodeType !== 1) return;
    if (_zwVaLooselyEquivalent(command, _zwValEffectiveValue(node, command), newValue)) {
      return;
    }
    var currentAncestor = node.parentNode;
    var ancestorList = [];
    var guard = 0;
    while (_zwJusIsEditable(currentAncestor)
           && currentAncestor.nodeType === 1
           && !_zwVaLooselyEquivalent(command,
                _zwValEffectiveValue(currentAncestor, command), newValue)
           && guard++ < 256) {
      ancestorList.push(currentAncestor);
      currentAncestor = currentAncestor.parentNode;
    }
    if (!ancestorList.length) return;
    var propagatedValue = _zwVaSpecifiedValue(
      ancestorList[ancestorList.length - 1], command);
    if (propagatedValue === null && propagatedValue !== newValue) return;
    if (newValue !== null
        && !_zwVaLooselyEquivalent(command,
             _zwValEffectiveValue(ancestorList[ancestorList.length - 1].parentNode, command),
             newValue)) {
      return;
    }
    while (ancestorList.length) {
      var currentAncestor2 = ancestorList.pop();
      var spec2 = _zwVaSpecifiedValue(currentAncestor2, command);
      if (spec2 !== null) {
        propagatedValue = spec2;
        _zwVaClearValue(currentAncestor2, command);
      }
      var children = [];
      var i;
      for (i = 0; i < currentAncestor2.childNodes.length; i++) {
        children.push(currentAncestor2.childNodes[i]);
      }
      for (i = 0; i < children.length; i++) {
        var child = children[i];
        if (child === node) continue;
        if (child.nodeType === 1
            && _zwVaSpecifiedValue(child, command) !== null
            && !_zwVaEquivalentValues(command, propagatedValue,
                 _zwVaSpecifiedValue(child, command))) {
          continue;
        }
        if (child === ancestorList[ancestorList.length - 1]) continue;
        _zwVaForceValue(child, command, propagatedValue);
      }
    }
  }

  // —— forceValue（参考实现 :2696）——五命令子集：bold/italic/strike/underline/
  // createlink 新 parent 臂不移植（part06 flat 臂域）。CSS flag false →
  // font color/face/size attr；fontsize xxx-large 恒 font attr（spec 特例）。
  function _zwVaForceValue(node, command, newValue) {
    if (!node.parentNode) return;
    if (newValue === null) return;
    // Chrome delta：空白-only 文本节点不强制赋值（不获独立包裹、不进 children
    // 递归臂）——`<p>foo</p> <p>bar</p>` 跨块、`<p>…<span>bar</span>…</p>` 块内
    // 伴形、`<span>foo</span> <span>bar</span>` 行内伴形语料期望空白恒裸或仅随
    // 邻近结构 wrap 携带（五语料 3 形 × 2 变体实证）。vendored 经 wrap 臂独立
    // 包值；本门置 forceValue 入口统一覆盖主循环/pushDownValues/children 递归
    // 各调用点，pushDownValues 本体维持 vendored 全量。_zwJusIsWhitespaceNode
    // 感知 white-space 属性，pre 格式化空白不受此门。
    if (node.nodeType === 3 && _zwJusIsWhitespaceNode(node)) return;
    var i;
    if (_zwVaIsAllowedChild(node, 'span')) {
      var sibCrit = function (n) {
        return _zwVaIsSimpleModifiable(n)
          && _zwVaEquivalentValues(command, _zwVaSpecifiedValue(n, command), newValue)
          && _zwVaLooselyEquivalent(command, _zwValEffectiveValue(n, command), newValue);
      };
      _zwVaWrap([node], sibCrit, function () { return null; });
    }
    if (_zwJusIsInvisible(node)) return;
    if (_zwVaLooselyEquivalent(command, _zwValEffectiveValue(node, command), newValue)) {
      return;
    }
    if (!_zwVaIsAllowedChild(node, 'span')) {
      var children = [];
      for (i = 0; i < node.childNodes.length; i++) {
        var kid = node.childNodes[i];
        if (kid.nodeType === 1) {
          var specKid = _zwVaSpecifiedValue(kid, command);
          if (specKid !== null && !_zwVaEquivalentValues(command, newValue, specKid)) {
            continue;
          }
        }
        children.push(kid);
      }
      for (i = 0; i < children.length; i++) {
        _zwVaForceValue(children[i], command, newValue);
      }
      return;
    }
    if (_zwVaLooselyEquivalent(command, _zwValEffectiveValue(node, command), newValue)) {
      return;
    }
    var newParent = null;
    if (!_zwVaCssFlag()) {
      if (command === 'fontsize') {
        newParent = document.createElement('font');
        newParent.setAttribute('size', _zwVaCssSizeToLegacy(newValue));
      }
      if (command === 'forecolor' && _zwVaParseSimpleColor(newValue)) {
        newParent = document.createElement('font');
        newParent.setAttribute('color', _zwVaParseSimpleColor(newValue));
      }
      if (command === 'fontname') {
        newParent = document.createElement('font');
        newParent.setAttribute('face', newValue);
      }
    }
    if (!newParent) {
      newParent = document.createElement('span');
    }
    // 插入点选区边界补偿（真浏览器 insertBefore 原生调整的等价面，_zwVaWrap
    // 同款）：新 parent 插在 node 前（idx = node 索引），落点 ≥ idx 的宿主选
    // 区端点 +1。缺补则端点漂到 newParent 与 node 之间，随后的 movePreserving
    // 端点随动把它带进 (newParent, 0)（语料 qV-after 读缺省黑实证——Chrome
    // 应落 (newParent, 1) bar 之后）。
    var insIdxFv = _zwJusNodeIndex(node);
    var selFv = globalThis.getSelection ? globalThis.getSelection() : null;
    var rngFv = selFv && selFv._ranges && selFv._ranges.length ? selFv._ranges[0] : null;
    if (rngFv) {
      if (rngFv.startContainer === node.parentNode && rngFv.startOffset >= insIdxFv) {
        try { rngFv.setStart(rngFv.startContainer, rngFv.startOffset + 1); } catch (_eFv1) {}
      }
      if (rngFv.endContainer === node.parentNode && rngFv.endOffset >= insIdxFv) {
        try { rngFv.setEnd(rngFv.endContainer, rngFv.endOffset + 1); } catch (_eFv2) {}
      }
    }
    node.parentNode.insertBefore(newParent, node);
    var property = command === 'fontsize' ? 'font-size'
      : (command === 'fontname' ? 'font-family'
        : (command === 'forecolor' ? 'color' : 'background-color'));
    if (!_zwVaLooselyEquivalent(command, _zwValEffectiveValue(newParent, command), newValue)) {
      _zwVaSetStyleProp(newParent, property, newValue);
    }
    _zwJusMovePreservingRanges(node, newParent, newParent.childNodes.length);
    if (node.nodeType === 1
        && !_zwVaEquivalentValues(command, _zwValEffectiveValue(node, command), newValue)) {
      _zwJusMovePreservingRanges(node, newParent.parentNode, _zwJusNodeIndex(newParent));
      if (newParent.parentNode) newParent.parentNode.removeChild(newParent);
      var children2 = [];
      for (i = 0; i < node.childNodes.length; i++) {
        var kid2 = node.childNodes[i];
        if (kid2.nodeType === 1) {
          var specKid2 = _zwVaSpecifiedValue(kid2, command);
          if (specKid2 !== null && !_zwVaEquivalentValues(command, newValue, specKid2)) {
            continue;
          }
        }
        children2.push(kid2);
      }
      for (i = 0; i < children2.length; i++) {
        _zwVaForceValue(children2[i], command, newValue);
      }
    }
  }

  // —— reorderModifiableDescendants（参考实现 :2345）。
  function _zwVaReorderModifiableDescendants(node, command, newValue) {
    var candidate = node;
    var guard = 0;
    while (_zwVaIsModifiable(candidate)
           && candidate.childNodes && candidate.childNodes.length === 1
           && _zwVaIsModifiable(candidate.firstChild)
           && (!_zwVaIsSimpleModifiable(candidate)
               || !_zwVaEquivalentValues(command,
                    _zwVaSpecifiedValue(candidate, command), newValue))
           && guard++ < 256) {
      candidate = candidate.firstChild;
    }
    if (candidate === node
        || !_zwVaIsSimpleModifiable(candidate)
        || !_zwVaEquivalentValues(command, _zwVaSpecifiedValue(candidate, command), newValue)
        || !_zwVaLooselyEquivalent(command,
             _zwValEffectiveValue(candidate, command), newValue)) {
      return;
    }
    var guardM = 0;
    while (_zwJusHasKids(candidate) && guardM++ < 4096) {
      _zwJusMovePreservingRanges(candidate.firstChild, candidate.parentNode,
        _zwJusNodeIndex(candidate));
    }
    node.parentNode.insertBefore(candidate, node.nextSibling);
    _zwJusMovePreservingRanges(node, candidate, -1);
  }

  // —— 引擎安全 splitText（part04 splitText 仅 handle-based wrapper；plain 文本
  // 节点缺法——zz-probe-fs4 实证）——data 写 + createTextNode + insertBefore。
  // 选区边界不在此处补偿（setSelectionValue 调用点显式回写）。
  function _zwVaSplitText(node, offset) {
    var data = String(node.data);
    var after = data.slice(offset);
    var newText = document.createTextNode(after);
    node.data = data.slice(0, offset);
    var p = node.parentNode;
    var next = node.nextSibling;
    try {
      if (next) p.insertBefore(newText, next);
      else p.appendChild(newText);
    } catch (_eSt) {
      try { p.appendChild(newText); } catch (_eSt2) {}
    }
    return newText;
  }

  // —— state/value override（参考实现 :1102-1171）——存储范围快照不匹配即重置
  // （selection 变化作废旧 override）。backcolor/hilitecolor 双向镜像。
  var _zwVaStateOverrides = {};
  var _zwVaValueOverrides = {};
  var _zwVaStoredRange = null;

  function _zwVaOverridesReset() {
    var r = null;
    try { r = _zwVaGetActiveRange(); } catch (_eOr) {}
    var cur = r
      ? { sc: r.startContainer, so: r.startOffset, ec: r.endContainer, eo: r.endOffset }
      : null;
    var stale = !_zwVaStoredRange || !cur
      || _zwVaStoredRange.sc !== cur.sc || _zwVaStoredRange.so !== cur.so
      || _zwVaStoredRange.ec !== cur.ec || _zwVaStoredRange.eo !== cur.eo;
    if (stale) {
      _zwVaStateOverrides = {};
      _zwVaValueOverrides = {};
      _zwVaStoredRange = cur;
    }
  }

  function _zwVaSetValueOverride(command, newValue) {
    _zwVaOverridesReset();
    _zwVaValueOverrides[command] = newValue;
    if (command === 'backcolor') _zwVaValueOverrides.hilitecolor = newValue;
    else if (command === 'hilitecolor') _zwVaValueOverrides.backcolor = newValue;
  }

  function _zwVaGetValueOverride(command) {
    _zwVaOverridesReset();
    return _zwVaValueOverrides[command];
  }

  function _zwVaUnsetValueOverride(command) {
    _zwVaOverridesReset();
    delete _zwVaValueOverrides[command];
    if (command === 'backcolor') delete _zwVaValueOverrides.hilitecolor;
    else if (command === 'hilitecolor') delete _zwVaValueOverrides.backcolor;
  }

  // normalizeFontSize（参考实现 :3192）——非法 → null；clamp 1-7；±3 相对模式。
  function _zwVaNormalizeFontSize(value) {
    value = String(value == null ? '' : value).trim();
    if (!/^[-+]?[0-9]+(\.[0-9]+)?([eE][-+]?[0-9]+)?$/.test(value)) return null;
    var mode;
    if (value.charAt(0) === '+') { value = value.slice(1); mode = 'relative-plus'; }
    else if (value.charAt(0) === '-') { value = value.slice(1); mode = 'relative-minus'; }
    else { mode = 'absolute'; }
    var num = parseInt(value, 10);
    if (mode === 'relative-plus') num += 3;
    if (mode === 'relative-minus') num = 3 - num;
    if (num < 1) num = 1;
    if (num > 7) num = 7;
    return { 1: 'x-small', 2: 'small', 3: 'medium', 4: 'large',
      5: 'x-large', 6: 'xx-large', 7: 'xxx-large' }[num];
  }

  // 空白-only 文本节点判定（wrap 拉邻放宽 / br 抑制共用；white-space 感知）。
  function _zwVaIsWsText(n) {
    return !!n && n.nodeType === 3 && _zwJusIsWhitespaceNode(n);
  }
  // —— setSelectionValue（参考实现 :2965）——五命令主流程。
  function _zwVaSetSelectionValue(command, newValue) {
    var rng = _zwVaGetActiveRange();
    if (!rng) return;
    // 无 contained formattable → override 记账臂（collapsed 选区 Chrome 口径）。
    var containedAll = _zwValGetAllEffectivelyContained(rng, function () { return true; });
    var anyFormattable = false;
    var i;
    for (i = 0; i < containedAll.length; i++) {
      if (_zwValIsFormattable(containedAll[i])) { anyFormattable = true; break; }
    }
    if (!anyFormattable) {
      _zwVaSetValueOverride(command, newValue);
      return;
    }

    // 边界 splitText（start 侧）——vendored 的 removeAllRanges/addRange 舞蹈在其
    // globalRange 存在时是无效面（getActiveRange 恒返回 globalRange 本体，真机制
    // = getActiveRange().setStart(newNode, 0) 原位写、end 不动）；本仓无
    // globalRange，sel._ranges[0] 即该面。半成品 newActiveRange（不同容器臂 end
    // 未设 → setStart 塌缩）经舞蹈重绑后 rng 丢失真端点 → contained 集空 → 整段
    // apply 零变异（`foo[bar<b>baz]qoz</b>quz` 族五语料各 2 变体实证）——改为对
    // rng 全程原位写，不做选区重绑。
    if (_zwJusIsEditable(rng.startContainer)
        && rng.startContainer.nodeType === 3
        && rng.startOffset !== 0
        && rng.startOffset !== _zwJusNodeLength(rng.startContainer)) {
      if (rng.startContainer === rng.endContainer) {
        var newEndOffset = rng.endOffset - rng.startOffset;
        var newNode = _zwVaSplitText(rng.startContainer, rng.startOffset);
        rng.setEnd(newNode, newEndOffset);
        rng.setStart(newNode, 0);
      } else {
        var newNode2 = _zwVaSplitText(rng.startContainer, rng.startOffset);
        rng.setStart(newNode2, 0);
      }
    }
    // 边界 splitText（end 侧）。
    if (_zwJusIsEditable(rng.endContainer)
        && rng.endContainer.nodeType === 3
        && rng.endOffset !== 0
        && rng.endOffset !== _zwJusNodeLength(rng.endContainer)) {
      var newStart = [rng.startContainer, rng.startOffset];
      var newEnd = [rng.endContainer, rng.endOffset];
      _zwVaSplitText(rng.endContainer, rng.endOffset);
      rng.setStart(newStart[0], newStart[1]);
      rng.setEnd(newEnd[0], newEnd[1]);
      var selE = globalThis.getSelection();
      selE.removeAllRanges();
      selE.addRange(rng);
    }

    // contained element clearValue pass。
    var elements = _zwValGetAllEffectivelyContained(rng, function (node) {
      return _zwJusIsEditable(node) && node.nodeType === 1;
    });
    for (i = 0; i < elements.length; i++) {
      _zwVaClearValue(elements[i], command);
    }

    // node pushDownValues + forceValue pass。
    var nodes = _zwValGetAllEffectivelyContained(rng, _zwJusIsEditable);
    for (i = 0; i < nodes.length; i++) {
      var node = nodes[i];
      _zwVaPushDownValues(node, command, newValue);
      if (_zwVaIsAllowedChild(node, 'span')) {
        _zwVaForceValue(node, command, newValue);
      }
    }
  }

  // —— 纯校验导出（part06 编辑分支 beforeinput 前置）——无效值在事件前拒绝
  //（Chrome 口径：action 校验失败零事件、execCommand 返 false）。fontname 任意
  // 串合法（vendored action 无校验，:3180）。
  globalThis.__zw_ce_value_validate = function (cmd, rawValue) {
    if (!_zwVaFive[String(cmd)]) return null;
    // enabled 面（语料 fontname tt 两形实证 [false]：起点入 tt[ce=false] 与
    // 跨 tt 交叉两形均返 false）——选区任一端点不在可编辑内容（宿主自身或
    // 宿主内可编辑节点）→ 命令未启用，execCommand 返 false 零事件。
    var selEn = globalThis.getSelection ? globalThis.getSelection() : null;
    var rngEn = selEn && selEn._ranges && selEn._ranges.length ? selEn._ranges[0] : null;
    var boundEditable = function (n) {
      return !!n && (_zwJusIsEditingHost(n) || _zwJusIsEditable(n));
    };
    if (rngEn && (!boundEditable(rngEn.startContainer)
        || !boundEditable(rngEn.endContainer))) {
      return false;
    }
    var value = rawValue == null ? '' : String(rawValue);
    if (cmd === 'fontsize') return _zwVaNormalizeFontSize(value) !== null;
    if (cmd === 'forecolor' || cmd === 'backcolor' || cmd === 'hilitecolor') {
      // Chrome 口径：非法色不前置拒绝（语料 [true] 实证 return true 零变更），
      // 仅 currentColor 维持 spec 拒绝（语料 [false]、vendored 同）。
      return value.toLowerCase() !== 'currentcolor';
    }
    return true;
  };

  // —— override 读数导出（part01c __zw_ce_value_query 消费）——未设返
  // undefined（调用方按 undefined 区分「无 override」）；读前走 reset（选区
  // 变动作废旧 override——同 vendored getValueOverride 惯用法）。fontsize 消费
  // 面 = getLegacyFontSize（vendored myQueryCommandValue :655-657）——
  // _zwVaLegacyValueOf 两段式（关键字直转 + normalize 链 + px 中点映射），
  // 不可映射返 ''（queryCommandValue 串面契约）。
  globalThis.__zw_ce_value_override_value = function (cmd) {
    var v = _zwVaGetValueOverride(String(cmd));
    if (v === undefined) return undefined;
    if (String(cmd) === 'fontsize') {
      var legacy = _zwVaLegacyValueOf(v);
      return legacy === null ? '' : String(legacy);
    }
    if (String(cmd) === 'forecolor' || String(cmd) === 'backcolor'
        || String(cmd) === 'hilitecolor') {
      // Chrome override 存色对象、读出即 CSSOM 序列化形（语料 collapsed
      // #0000FF → qV 'rgb(0, 0, 255)' 实证——vendored 存原始串直返是 delta）。
      return _zwVaNormalizeCssColor(v) || v;
    }
    return v;
  };

  // 选区整体是否落在同一 a[href] 内（Chrome foreColor 不改链内样式的 delta
  // 门；语料 3 例 `<a href>foo[bar]baz</a>` 实证：DOM 不变 + [true] + qV 读
  // 新值）。两端点各自最近 a[href] 祖先同且非空即判命中。
  function _zwVaInHrefAnchor() {
    var sel = globalThis.getSelection ? globalThis.getSelection() : null;
    var rng = sel && sel._ranges && sel._ranges.length ? sel._ranges[0] : null;
    if (!rng || rng.collapsed) return false;
    var up = function (n) {
      // 文本节点起点直接判元素会漏（while 条件首查即败）——沿父链逐级查。
      while (n) {
        if (n.nodeType === 1 && _zwJusTag(n) === 'a'
            && n.getAttribute && n.getAttribute('href') != null) return n;
        n = n.parentNode;
      }
      return null;
    };
    var a1 = up(rng.startContainer);
    return !!a1 && a1 === up(rng.endContainer);
  }

  // —— 引擎入口（part06 execCommand 五命令臂单点调用）——返回 false = 无效值
  // （execCommand 返 false——语料 return value 断言面）。fontsize 归一后以
  // 关键字进流程；色三命令 currentColor 拒（spec），非法色 Chrome 口径返
  // true 零变更（语料 [true] + DOM 不变实证，与 vendored return false 异）。
  globalThis.__zw_ce_value_apply = function (cmd, rawValue) {
    if (!_zwVaFive[String(cmd)]) return null;
    var value = rawValue == null ? '' : String(rawValue);
    if (cmd === 'fontsize') {
      var kw = _zwVaNormalizeFontSize(value);
      if (kw === null) return false;
      _zwVaSetSelectionValue('fontsize', kw);
      return true;
    }
    if (cmd === 'forecolor' || cmd === 'backcolor' || cmd === 'hilitecolor') {
      if (/^([0-9a-fA-F]{3}){1,2}$/.test(value)) value = '#' + value;
      // currentColor 独走 spec 拒绝（语料 [false]，vendored 同）。
      if (value.toLowerCase() === 'currentcolor') return false;
      // Chrome 口径（语料实证、与 vendored return false 相异的 delta）：非法
      // 色返 true 且零变更——DOM 不动、override 不记（qV-after 落缺省黑）。
      if (!_zwVaNormalizeCssColor(value)) return true;
      // Chrome delta（语料 forecolor 3 例实证，仅 forecolor 观测面）：选区整
      // 体在 a[href] 内不落 DOM——返 true + 值 override 记账（qV-after 读新
      // 值）。vendored 无此门。
      if (cmd === 'forecolor' && _zwVaInHrefAnchor()) {
        _zwVaSetValueOverride(cmd, _zwVaNormalizeCssColor(value));
        return true;
      }
      _zwVaSetSelectionValue(cmd, value);
      return true;
    }
    _zwVaSetSelectionValue('fontname', value);
    return true;
  };
