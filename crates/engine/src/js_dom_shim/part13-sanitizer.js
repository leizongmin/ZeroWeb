// part13（2026-11-22 wptfyi-dom DC-2 切片 5 刀②）：Sanitizer API 构造面 +
// Element/ShadowRoot.setHTML/setHTMLUnsafe options 挂接 + Document.parseHTML(Unsafe)。
//
// 目标面：sanitizer-api 语料 wholesale "Sanitizer is not defined"（切片 4 census
// 104 处）；F 大头 sanitizer-config 71F（canonicalization + TypeError 矩阵）、
// sanitizer-javascript-url 42F、sanitizer-parseHTML 25F、sanitizer-names 20F、
// sanitizer-modifiers 19F、sethtml-tree-construction/safety/context（setHTML
// 恒消毒面）、sanitizer-default-config 1F（121 元素/58 属性全表）等。
//
// 规范：https://wicg.github.io/sanitizer-api/
//   - 构造 canonicalization：#parse-sanitizer-config（string 简写元素→
//     {name, namespace: XHTML, removeAttributes: []}、属性→{name, namespace:
//     null}；namespace ""→null；空字典补 removeAttributes/
//     removeProcessingInstructions 空表 marker + comments 缺省 true）
//   - TypeError 矩阵：#validate-config（elements XOR removeElements 等三互斥对、
//     逐列表去重、rWC∩(elements|removeElements)=∅、rWC 限可替换元素
//     {html,svg,math}、per-element attributes/removeAttributes 方向规则、
//     dataAttributes 与 removeAttributes 方向互斥）
//   - get()：#dom-sanitizer-get（ns null 先→ns 升序→name 升序；PI 按 target）
//   - modifiers：#dom-sanitizer-allowelement 等十方法（方向内移动 + 冗余剥除
//     ——语料 sanitizer-modifiers.html 合并案实证；返回值 true=有变化）
//   - 默认配置：#built-in-safe-default-configuration（121 元素/58 属性全表，
//     sanitizer-default-config.html verbatim；comments:false、
//     dataAttributes:false、processingInstructions:[]）
//   - 安全基线：#built-in-safe-baseline-configuration（removeElements 8 项
//     XHTML base/embed/frame/iframe/object/script + SVG script/use；
//     removeAttributes = 事件处理器内容属性表）——仅 safe 方法（setHTML/
//     parseHTML）强制附加；unsafe 方法有 options 时仅按用户配置（语料
//     sethtml-safety vs unsafety {} 配置 use 行为分叉实证）
//   - 内建 javascript: 值剥离：#sanitizer-api（navigating：href/action/
//     formaction 任意元素；animating：SVG animate 删元素、animateMotion/
//     animateTransform 删 attributeName）
//   - setHTML 语义：#dom-element-sethtml（恒消毒；script 类容器产出空；
//     DOMString 强转 undefined→"undefined"）
//
// 形制：自包含 IIFE（part12 同款——外层 shim IIFE 在 part06 尾闭合，链尾 part
// 只走 globalThis 契约：消费 part05 `__zwSanBridge` 物化/序列化底座导出 +
// 注册 `__zwSanSetHtml` 供 part04 trap / part03 ShadowRoot 原型 late-bound
// 调用；顶层 function 声明不进 global，直调跨 part 不可达——刀②首轮探针
// "_zwSetHtmlUnsafeImpl is not defined" 实证）。
//
// 管线：materialize-then-filter——`fragmentAddedUnsafe` 物化视图（mock 代理
// 树，完整 attributes/removeAttribute API）就地过滤后经 `serialize`（part03
// `_zwMSerialize` 既有序列化器）重组净串，走 `setHtmlUnsafeImpl` 既有写链
//（引擎写/种子注册/handle 重建零改动，双端同态纪律保持）。零新 Rust 桥。
//
// FIXME（刀③后置域）：Document.parseHTML 全文档保真（DOCTYPE/head 属性归置）；
// Sanitizer.sanitize()/sanitizeToFragment()（语料本集无钉面）；处理指令树域
// 往返序列化（`<?…?>` 经 HTML 解析器成 bogus comment，processing-instructions
// 页 2F/2P 维持）；removeUnsafe 对 allow 方向含基线条目时的剥除形（语料仅钉
// 干净面两案）。
(function () {
  'use strict';
  if (globalThis.Sanitizer && globalThis.__zwSanSetHtml) return; // 幂等（shim 重装配边界）

  // 物化/序列化底座（part05 导出桥；IIFE 执行时 part05 已装配 ✓）。
  var _zwSanBridge = globalThis.__zwSanBridge || null;

  var _zwSanNsXhtml = 'http://www.w3.org/1999/xhtml';
  var _zwSanNsMathml = 'http://www.w3.org/1998/Math/MathML';
  var _zwSanNsSvg = 'http://www.w3.org/2000/svg';

  // ASCII 大小写折叠（'ſ' 非 ASCII 不折叠——part12 同款语义）。
  function _zwSanAsciiLower(s) {
    return String(s).replace(/[A-Z]/g, function (c) { return c.toLowerCase(); });
  }

  // 事件处理器内容属性清单（HTML spec #event-handlers-on-elements——与 part12
  // 同表副本；on* 非前缀匹配，removeUnsafe 基线与 safe 属性强制删面共用）。
  var _zwSanEventHandlers = {};
  ('onabort onauxclick onbeforeinput onbeforematch onbeforetoggle onbeforecopy ' +
   'onbeforecut onbeforepaste onblur oncancel oncanplay oncanplaythrough onchange ' +
   'onclick onclose oncontextlost oncontextmenu oncontextrestored oncopy oncuechange ' +
   'oncut ondblclick ondrag ondragend ondragenter ondragleave ondragover ondragstart ' +
   'ondrop ondurationchange onemptied onended onerror onfocus onformdata oninput ' +
   'oninvalid onkeydown onkeypress onkeyup onload onloadeddata onloadedmetadata ' +
   'onloadstart onmousedown onmouseenter onmouseleave onmousemove onmouseout ' +
   'onmouseover onmouseup onmousewheel onpaste onpause onplay onplaying onprogress ' +
   'onratechange onreset onresize onscroll onscrollend onsecuritypolicyviolation ' +
   'onseeked onseeking onselect onslotchange onstalled onsuspend onsubmit ' +
   'ontimeupdate ontoggle onvolumechange onwaiting onwebkitanimationend ' +
   'onwebkitanimationiteration onwebkitanimationstart onwebkittransitionend onwheel')
    .split(' ').forEach(function (h) { _zwSanEventHandlers[h] = true; });

  // 安全基线 removeElements（#built-in-safe-baseline-configuration——语料
  // sanitizer-removeUnsafe.html SAFE_BASELINE verbatim）。刀㊼（s10k60）：条目
  // 键名 ns→namespace 归一——SameEl 比较器读 namespace，旧键名使三条消费臂
  //（verdict safe 强制基线＋removeUnsafe 剔除/并入）恒失配从未生效（默认实例
  // removeElements 路径兜住无 config 面，显式 config 面暴露＝sethtml-safety
  // #3/#9-15 九 F 根因）。
  var _zwSanBaselineRemoveEls = [
    { name: 'base', namespace: _zwSanNsXhtml },
    { name: 'embed', namespace: _zwSanNsXhtml },
    { name: 'frame', namespace: _zwSanNsXhtml },
    { name: 'iframe', namespace: _zwSanNsXhtml },
    { name: 'object', namespace: _zwSanNsXhtml },
    { name: 'script', namespace: _zwSanNsXhtml },
    { name: 'script', namespace: _zwSanNsSvg },
    { name: 'use', namespace: _zwSanNsSvg }
  ];

  // 可整体替换元素（#non-replaceable-elements 之外全集——非替换三件 html/svg/math）。
  function _zwSanIsNonReplaceable(name, ns) {
    return (name === 'html' && ns === _zwSanNsXhtml) ||
      (name === 'svg' && ns === _zwSanNsSvg) ||
      (name === 'math' && ns === _zwSanNsMathml);
  }

  // ===== 默认配置全表（#built-in-safe-default-configuration；语料
  // sanitizer-default-config.html verbatim 提取。元素条目 [name, nsIdx, attrs?]：
  // nsIdx 0=XHTML 1=MathML 2=SVG；attrs 内 string=null ns、[name, nsIdx] 限定）=====
  var _zwSanDefaultElementsRaw = [
    ["math",1],["merror",1],["mfrac",1],["mi",1],["mmultiscripts",1],["mn",1],
    ["mo",1,["fence","form","largeop","lspace","maxsize","minsize","movablelimits","rspace","separator","stretchy","symmetric"]],
    ["mover",1,["accent"]],
    ["mpadded",1,["depth","height","lspace","voffset","width"]],
    ["mphantom",1],["mprescripts",1],["mroot",1],["mrow",1],["ms",1],
    ["mspace",1,["depth","height","width"]],
    ["msqrt",1],["mstyle",1],["msub",1],["msubsup",1],["msup",1],["mtable",1],
    ["mtd",1,["columnspan","rowspan"]],
    ["mtext",1],["mtr",1],
    ["munder",1,["accentunder"]],
    ["munderover",1,["accent","accentunder"]],
    ["semantics",1],
    ["a",0,["href","hreflang","type"]],
    ["abbr",0],["address",0],["article",0],["aside",0],["b",0],["bdi",0],["bdo",0],
    ["blockquote",0,["cite"]],
    ["body",0],["br",0],["caption",0],["cite",0],["code",0],
    ["col",0,["span"]],
    ["colgroup",0,["span"]],
    ["data",0,["value"]],
    ["dd",0],
    ["del",0,["cite","datetime"]],
    ["dfn",0],["div",0],["dl",0],["dt",0],["em",0],["figcaption",0],["figure",0],
    ["footer",0],["h1",0],["h2",0],["h3",0],["h4",0],["h5",0],["h6",0],["head",0],
    ["header",0],["hgroup",0],["hr",0],["html",0],["i",0],
    ["ins",0,["cite","datetime"]],
    ["kbd",0],
    ["li",0,["value"]],
    ["main",0],["mark",0],["menu",0],["nav",0],
    ["ol",0,["reversed","start","type"]],
    ["p",0],["pre",0],["q",0],["rp",0],["rt",0],["ruby",0],["s",0],["samp",0],
    ["search",0],["section",0],["small",0],["span",0],["strong",0],["sub",0],
    ["sup",0],["table",0],["tbody",0],
    ["td",0,["colspan","headers","rowspan"]],
    ["tfoot",0],
    ["th",0,["abbr","colspan","headers","rowspan","scope"]],
    ["thead",0],
    ["time",0,["datetime"]],
    ["title",0],["tr",0],["u",0],["ul",0],["var",0],["wbr",0],
    ["a",2,["href","hreflang","type"]],
    ["circle",2,["cx","cy","pathLength","r"]],
    ["defs",2],["desc",2],
    ["ellipse",2,["cx","cy","pathLength","rx","ry"]],
    ["foreignObject",2,["height","width","x","y"]],
    ["g",2],
    ["line",2,["pathLength","x1","x2","y1","y2"]],
    ["marker",2,["markerHeight","markerUnits","markerWidth","orient","preserveAspectRatio","refX","refY","viewBox"]],
    ["metadata",2],
    ["path",2,["d","pathLength"]],
    ["polygon",2,["pathLength","points"]],
    ["polyline",2,["pathLength","points"]],
    ["rect",2,["height","pathLength","rx","ry","width","x","y"]],
    ["svg",2,["height","preserveAspectRatio","viewBox","width","x","y"]],
    ["text",2,["dx","dy","lengthAdjust","rotate","textLength","x","y"]],
    ["textPath",2,["lengthAdjust","method","path","side","spacing","startOffset","textLength"]],
    ["title",2],
    ["tspan",2,["dx","dy","lengthAdjust","rotate","textLength","x","y"]]
  ];
  var _zwSanDefaultAttributesRaw = ["alignment-baseline","baseline-shift","clip-path","clip-rule","color","color-interpolation","cursor","dir","direction","display","displaystyle","dominant-baseline","fill","fill-opacity","fill-rule","font-family","font-size","font-size-adjust","font-stretch","font-style","font-variant","font-weight","lang","letter-spacing","marker-end","marker-mid","marker-start","mathbackground","mathcolor","mathsize","opacity","paint-order","pointer-events","scriptlevel","shape-rendering","stop-color","stop-opacity","stroke","stroke-dasharray","stroke-dashoffset","stroke-linecap","stroke-linejoin","stroke-miterlimit","stroke-opacity","stroke-width","text-anchor","text-decoration","text-overflow","text-rendering","title","transform","transform-origin","unicode-bidi","vector-effect","visibility","white-space","word-spacing","writing-mode"];

  var _zwSanNsByIdx = [_zwSanNsXhtml, _zwSanNsMathml, _zwSanNsSvg];

  // ===== canonicalization helpers =====
  // 属性条目：string → {name, namespace:null}（attributes 方向默认 ns null——
  // spec #canonicalize-a-sanitizer-attribute）；object → name 强转 + namespace
  // 缺省/null 保留、""→null。键序 name→namespace。
  function _zwSanCanonAttrEntry(v) {
    if (v !== null && typeof v === 'object') {
      var n = String(v.name);
      var ns = v.namespace;
      if (ns === '' || ns === undefined) ns = null;
      else if (ns !== null) ns = String(ns);
      return { name: n, namespace: ns };
    }
    return { name: String(v), namespace: null };
  }
  function _zwSanCanonAttrList(arr) {
    var out = [];
    for (var i = 0; i < arr.length; i++) out.push(_zwSanCanonAttrEntry(arr[i]));
    return out;
  }
  // 允许方向元素条目：string → {name, namespace:XHTML, removeAttributes:[]}；
  // object → name 强转 + namespace 缺省（member 缺席→XHTML、显式 ""→null 化、
  // null 保留——config 页 normalization 脸 [{"name":"b","namespace":""}] 期望
  // null；字符串形 XHTML 缺省由 assert_elems 字符串分支 XHTML_NS 佐证）；带
  // attributes member → {name, namespace, attributes}，否则 removeAttributes:[]。
  // 用户提供 removeAttributes member 时保留（modifiers 双键形态输入面）。
  function _zwSanCanonElAllow(v) {
    var name, ns, attrs = null, rmAttrs = null;
    if (v !== null && typeof v === 'object') {
      name = String(v.name);
      ns = v.namespace;
      if (ns === undefined) ns = _zwSanNsXhtml;
      else if (ns === '') ns = null;
      else if (ns !== null) ns = String(ns);
      if (v.attributes !== undefined) attrs = _zwSanCanonAttrList(v.attributes);
      if (v.removeAttributes !== undefined) rmAttrs = _zwSanCanonAttrList(v.removeAttributes);
    } else {
      name = String(v);
      ns = _zwSanNsXhtml;
    }
    var e = { name: name, namespace: ns };
    if (attrs !== null) e.attributes = attrs;
    if (rmAttrs !== null) e.removeAttributes = rmAttrs;
    else if (attrs === null) e.removeAttributes = [];
    return e;
  }
  // 移除方向/rWC 元素条目：{name, namespace}（namespace member 缺席→XHTML、
  // 显式 ""→null、null 保留）。
  function _zwSanCanonElPlain(v) {
    if (v !== null && typeof v === 'object') {
      var ns = v.namespace;
      if (ns === undefined) ns = _zwSanNsXhtml;
      else if (ns === '') ns = null;
      else if (ns !== null) ns = String(ns);
      return { name: String(v.name), namespace: ns };
    }
    return { name: String(v), namespace: _zwSanNsXhtml };
  }
  // 处理指令条目：string → {target}；object → {target: String(target)}。
  function _zwSanCanonPi(v) {
    if (v !== null && typeof v === 'object') return { target: String(v.target) };
    return { target: String(v) };
  }

  // 排序：namespace null 先、ns 升序、name 升序（#dom-sanitizer-get）。
  function _zwSanNsNameCmp(a, b) {
    var an = a.namespace, bn = b.namespace;
    if (an === null && bn !== null) return -1;
    if (an !== null && bn === null) return 1;
    if (an !== null && bn !== null && an !== bn) return an < bn ? -1 : 1;
    if (a.name === b.name) return 0;
    return a.name < b.name ? -1 : 1;
  }
  function _zwSanTargetCmp(a, b) {
    if (a.target === b.target) return 0;
    return a.target < b.target ? -1 : 1;
  }
  function _zwSanSortList(list, cmp) {
    return list.slice().sort(cmp);
  }

  function _zwSanSameEl(a, b) { return a.name === b.name && a.namespace === b.namespace; }
  function _zwSanSameAttr(a, b) { return a.name === b.name && a.namespace === b.namespace; }
  // 刀㊻b（s10k59）：allow 方向条目名 case 不敏感比较——config 条目按作者书写
  // case（SVG 保留形态 "attributeName"），解析产物传参是 lower 化 local，exact
  // 恒失配致 attributeName 被 over-strip（语料 sanitizer-svg-animate 15F）。
  function _zwSanSameAttrCi(a, b) {
    return _zwSanAsciiLower(String(a.name)) === _zwSanAsciiLower(String(b.name)) &&
      a.namespace === b.namespace;
  }
  function _zwSanSamePi(a, b) { return a.target === b.target; }
  // 属性允许名单命中（允许方向 arm 共用）：解析产物属性 ns 恒 null（mock 桥
  // 无属性 namespaceURI），canon 字符串形/缺省 ns 条目归一 XHTML——XHTML 条目
  // 对 null-ns 实际属性放行（显式 null 命中走第一查；其余 ns 精确匹配）。
  // 条目名比对走 Ci 刀㊻b（case 不敏感兜底——负钉面普查 sanitizer-api 语料无
  // case 敏感反例；remove 方向仍 exact）。
  function _zwSanAttrAllowHit(list, local, ans) {
    if (!list || !list.length) return false;
    if (_zwSanListHas(list, { name: local, namespace: ans }, _zwSanSameAttrCi)) return true;
    if (ans !== _zwSanNsXhtml &&
        _zwSanListHas(list, { name: local, namespace: _zwSanNsXhtml }, _zwSanSameAttrCi)) return true;
    return false;
  }

  function _zwSanIsJsUrl(v) {
    var s = String(v == null ? '' : v);
    // URL 解析：全串剥 tab/LF/CR；scheme 段剥前导 C0 控制与空格（#sanitizer-api
    // javascript: 判定——语料 sanitizer-javascript-url.html 实证）。
    var rest = s.replace(/[\t\n\r]/g, '');
    var i = 0;
    while (i < rest.length && rest.charCodeAt(i) <= 0x20) i++;
    rest = rest.slice(i).toLowerCase();
    return rest.lastIndexOf('javascript:', 0) === 0;
  }

  // ===== 构造 canonicalization + 校验（#parse-sanitizer-config /
  // #validate-config——TypeError 矩阵）=====
  function _zwSanCanonConfig(raw) {
    var cfg = {};
    var hasEls = raw.elements !== undefined;
    var hasRemoveEls = raw.removeElements !== undefined;
    if (hasEls && hasRemoveEls) {
      throw new TypeError('Sanitizer: elements and removeElements are mutually exclusive.');
    }
    if (hasEls) {
      if (!Array.isArray(raw.elements)) {
        throw new TypeError('Sanitizer: elements must be an array.');
      }
      cfg.elements = [];
      for (var ie = 0; ie < raw.elements.length; ie++) {
        cfg.elements.push(_zwSanCanonElAllow(raw.elements[ie]));
      }
    } else {
      cfg.removeElements = [];
      if (hasRemoveEls) {
        if (!Array.isArray(raw.removeElements)) {
          throw new TypeError('Sanitizer: removeElements must be an array.');
        }
        for (var ire = 0; ire < raw.removeElements.length; ire++) {
          cfg.removeElements.push(_zwSanCanonElPlain(raw.removeElements[ire]));
        }
      }
    }
    var hasAttrs = raw.attributes !== undefined;
    var hasRemoveAttrs = raw.removeAttributes !== undefined;
    if (hasAttrs && hasRemoveAttrs) {
      throw new TypeError('Sanitizer: attributes and removeAttributes are mutually exclusive.');
    }
    if (hasAttrs) {
      if (!Array.isArray(raw.attributes)) {
        throw new TypeError('Sanitizer: attributes must be an array.');
      }
      cfg.attributes = _zwSanCanonAttrList(raw.attributes);
    } else {
      cfg.removeAttributes = [];
      if (hasRemoveAttrs) {
        if (!Array.isArray(raw.removeAttributes)) {
          throw new TypeError('Sanitizer: removeAttributes must be an array.');
        }
        cfg.removeAttributes = _zwSanCanonAttrList(raw.removeAttributes);
      }
    }
    var hasPi = raw.processingInstructions !== undefined;
    var hasRemovePi = raw.removeProcessingInstructions !== undefined;
    if (hasPi && hasRemovePi) {
      throw new TypeError('Sanitizer: processingInstructions and removeProcessingInstructions are mutually exclusive.');
    }
    if (hasPi) {
      if (!Array.isArray(raw.processingInstructions)) {
        throw new TypeError('Sanitizer: processingInstructions must be an array.');
      }
      cfg.processingInstructions = [];
      for (var ip = 0; ip < raw.processingInstructions.length; ip++) {
        cfg.processingInstructions.push(_zwSanCanonPi(raw.processingInstructions[ip]));
      }
    } else {
      cfg.removeProcessingInstructions = [];
      if (hasRemovePi) {
        if (!Array.isArray(raw.removeProcessingInstructions)) {
          throw new TypeError('Sanitizer: removeProcessingInstructions must be an array.');
        }
        for (var irp = 0; irp < raw.removeProcessingInstructions.length; irp++) {
          cfg.removeProcessingInstructions.push(_zwSanCanonPi(raw.removeProcessingInstructions[irp]));
        }
      }
    }
    // comments：提供 dict 缺省 true（#parse-sanitizer-config——语料
    // boolean-defaults `new Sanitizer({}).get().comments===true` 实证）。
    cfg.comments = raw.comments !== undefined ? !!raw.comments : true;
    // 刀⑭（s10k14）：comments 显式真值印记——parseHTML 文档路径注释剥离取
    // raw dict 显式真值（实例缺省 true 是 get() 钉面；剥离面见
    // _zwSanParseHtmlImpl 漏斗），显式 comments:true 才豁免。
    cfg.__zwCommentsExplicitTrue = raw.comments === true;
    // 刀⑭（s10k14）：dataAttributes 显式真值印记——parseHTML safe 漏斗 data-*
    // 缺省剥取 raw 显式真值豁免（缺省 true 是 get()/直通面钉不可动；覆写面见
    // _zwSanParseHtmlImpl）。
    cfg.__zwDataAttrExplicitTrue = raw.dataAttributes === true;
    // 切片 8（wptfyi-dom，2026-11-24）：replaceWithChildrenElements 规范化入
    // canon（条目同 removeElements 形——string/{name,namespace}）。旧版 canon
    // 全缺该键 → verdict rWC 匹配恒落空（stream-html-sanitizer 模板解包族
    // 探针 p14 实证：`<div id=outer>` 原样落位零解包）；validate 的 dup/
    // non-replaceable 检查随在席生效。
    if (raw.replaceWithChildrenElements !== undefined) {
      if (!Array.isArray(raw.replaceWithChildrenElements)) {
        throw new TypeError('Sanitizer: replaceWithChildrenElements must be an array.');
      }
      cfg.replaceWithChildrenElements = [];
      for (var irw = 0; irw < raw.replaceWithChildrenElements.length; irw++) {
        cfg.replaceWithChildrenElements.push(_zwSanCanonElPlain(raw.replaceWithChildrenElements[irw]));
      }
    }
    // dataAttributes：attributes 方向可显式（缺省 true）；removeAttributes 方向
    // 存在即 TypeError（语料 removeAttributes+dataAttributes 互斥案）。
    // 校验在缺省前执行——validate 的 data-* 检查只认 raw 显式 true（语料
    // sethtml-with-custom-elements 三脸 attributes:["is","data-attr"] 无
    // dataAttributes 构造零异常 ground truth；先缺省 true 再校验会让
    // data-* 检查误炸——刀⑪ 电池 P2F 实证）；缺省 true 只服务过滤面
    // （#data-attributes data-* 直通）与 get() 输出（assert_config
    // PROPERTIES 含 dataAttributes，allow 方向 canon 恒在席）。
    if (hasAttrs) {
      cfg.dataAttributes = raw.dataAttributes !== undefined ? !!raw.dataAttributes : undefined;
    } else if (raw.dataAttributes !== undefined) {
      throw new TypeError('Sanitizer: dataAttributes is not allowed with removeAttributes.');
    }
    _zwSanValidateConfig(cfg);
    if (hasAttrs && cfg.dataAttributes === undefined) cfg.dataAttributes = true;
    return cfg;
  }

  function _zwSanDupCheck(list, same, msg) {
    for (var i = 0; i < list.length; i++) {
      for (var j = i + 1; j < list.length; j++) {
        if (same(list[i], list[j])) {
          throw new TypeError('Sanitizer: ' + msg);
        }
      }
    }
  }
  function _zwSanListHas(list, item, same) {
    for (var i = 0; i < list.length; i++) if (same(list[i], item)) return true;
    return false;
  }

  function _zwSanValidateConfig(cfg) {
    var elsAllow = cfg.elements;
    var elsRemove = cfg.removeElements;
    if (elsAllow) {
      _zwSanDupCheck(elsAllow, _zwSanSameEl, 'duplicate element in elements.');
    } else if (elsRemove) {
      _zwSanDupCheck(elsRemove, _zwSanSameEl, 'duplicate element in removeElements.');
    }
    var rwc = cfg.replaceWithChildrenElements;
    if (rwc) {
      _zwSanDupCheck(rwc, _zwSanSameEl, 'duplicate element in replaceWithChildrenElements.');
      for (var ir = 0; ir < rwc.length; ir++) {
        if (_zwSanIsNonReplaceable(rwc[ir].name, rwc[ir].namespace)) {
          throw new TypeError('Sanitizer: replaceWithChildrenElements contains a non-replaceable element.');
        }
        if (elsAllow && _zwSanListHas(elsAllow, rwc[ir], _zwSanSameEl)) {
          throw new TypeError('Sanitizer: replaceWithChildrenElements overlaps elements.');
        }
        if (elsRemove && _zwSanListHas(elsRemove, rwc[ir], _zwSanSameEl)) {
          throw new TypeError('Sanitizer: replaceWithChildrenElements overlaps removeElements.');
        }
      }
    }
    if (cfg.attributes) {
      _zwSanDupCheck(cfg.attributes, _zwSanSameAttr, 'duplicate attribute in attributes.');
    } else if (cfg.removeAttributes) {
      _zwSanDupCheck(cfg.removeAttributes, _zwSanSameAttr, 'duplicate attribute in removeAttributes.');
    }
    if (cfg.processingInstructions) {
      _zwSanDupCheck(cfg.processingInstructions, _zwSanSamePi, 'duplicate target in processingInstructions.');
    } else if (cfg.removeProcessingInstructions) {
      _zwSanDupCheck(cfg.removeProcessingInstructions, _zwSanSamePi, 'duplicate target in removeProcessingInstructions.');
    }
    // per-element 属性规则（#validate-config——attributes 允许方向）：
    // 条目内去重、与全局 attributes 无交集、dataAttributes 真时不允许 data-*。
    if (elsAllow && cfg.attributes) {
      for (var iea = 0; iea < elsAllow.length; iea++) {
        var ent = elsAllow[iea];
        if (ent.attributes) {
          _zwSanDupCheck(ent.attributes, _zwSanSameAttr, 'duplicate attribute in element entry.');
          for (var iea2 = 0; iea2 < ent.attributes.length; iea2++) {
            if (_zwSanListHas(cfg.attributes, ent.attributes[iea2], _zwSanSameAttr)) {
              throw new TypeError('Sanitizer: element entry attribute overlaps global attributes.');
            }
            if (cfg.dataAttributes && _zwSanAsciiLower(ent.attributes[iea2].name).lastIndexOf('data-', 0) === 0) {
              throw new TypeError('Sanitizer: data attribute with dataAttributes enabled.');
            }
          }
        }
        if (ent.removeAttributes) {
          _zwSanDupCheck(ent.removeAttributes, _zwSanSameAttr, 'duplicate attribute in element entry removeAttributes.');
          // 16.2.1.4（util.js assert_config_is_valid 步骤号）：条目
          // removeAttributes（缺省 « »）⊆ config[attributes]——违反 TypeError
          // （config 页「element[removeAttributes] should be a subset of
          // config[attributes]」ground truth；空数组循环零次恒真）。
          for (var irm4 = 0; irm4 < ent.removeAttributes.length; irm4++) {
            if (!_zwSanListHas(cfg.attributes, ent.removeAttributes[irm4], _zwSanSameAttr)) {
              throw new TypeError('Sanitizer: element entry removeAttributes must be a subset of attributes.');
            }
          }
        }
      }
    }
    // 16.3（util.js 步骤号，独立于 elements 在席——config 页
    // 「config[attributes] with a data attribute must not co-exist with
    // config[dataAttributes] set to true」无 elements 构造 ground truth）：
    if (cfg.attributes && cfg.dataAttributes) {
      for (var ig = 0; ig < cfg.attributes.length; ig++) {
        if (_zwSanAsciiLower(cfg.attributes[ig].name).lastIndexOf('data-', 0) === 0) {
          throw new TypeError('Sanitizer: data attribute with dataAttributes enabled.');
        }
      }
    }
    // removeAttributes 方向 per-element 规则：条目双带 attributes+removeAttributes
    // TypeError；条目 attributes 与全局 removeAttributes 交集 TypeError；
    // 条目 removeAttributes ⊆ 全局 removeAttributes 违反 TypeError；条目内部去重。
    if (elsAllow && cfg.removeAttributes) {
      for (var irm = 0; irm < elsAllow.length; irm++) {
        var entR = elsAllow[irm];
        if (entR.attributes && entR.removeAttributes) {
          throw new TypeError('Sanitizer: element entry cannot have both attributes and removeAttributes.');
        }
        if (entR.attributes) {
          _zwSanDupCheck(entR.attributes, _zwSanSameAttr, 'duplicate attribute in element entry.');
          for (var irm2 = 0; irm2 < entR.attributes.length; irm2++) {
            if (_zwSanListHas(cfg.removeAttributes, entR.attributes[irm2], _zwSanSameAttr)) {
              throw new TypeError('Sanitizer: element entry attribute overlaps removeAttributes.');
            }
          }
        }
        if (entR.removeAttributes) {
          _zwSanDupCheck(entR.removeAttributes, _zwSanSameAttr, 'duplicate attribute in element entry removeAttributes.');
          // 17.1.1.5（util.js 步骤号）：config[removeAttributes] ∩
          // element[removeAttributes] = ∅——相交 TypeError（config 页
          // 「config[removeAttributes] and element[removeAttributes] should
          // not intersect」ground truth）。旧版误为 ⊆ 检查（ modifiers 脸
          // 「allowAttribute with global rm + element's rm」构造炸根因——
          // standalone per-element removeAttributes 与全局名单独立合法）。
          for (var irm3 = 0; irm3 < entR.removeAttributes.length; irm3++) {
            if (_zwSanListHas(cfg.removeAttributes, entR.removeAttributes[irm3], _zwSanSameAttr)) {
              throw new TypeError('Sanitizer: element entry removeAttributes overlaps removeAttributes.');
            }
          }
        }
      }
    }
  }

  // ===== 默认配置实例（#built-in-safe-default-configuration）=====
  var _zwSanDefaultInstance = null;
  function _zwSanBuildDefaultCfg() {
    var cfg = { comments: false, dataAttributes: false };
    cfg.processingInstructions = [];
    cfg.elements = [];
    for (var i = 0; i < _zwSanDefaultElementsRaw.length; i++) {
      var raw = _zwSanDefaultElementsRaw[i];
      var e = { name: raw[0], namespace: _zwSanNsByIdx[raw[1]], attributes: [] };
      if (raw[2]) {
        for (var j = 0; j < raw[2].length; j++) {
          var a = raw[2][j];
          e.attributes.push(typeof a === 'string'
            ? { name: a, namespace: null }
            : { name: a[0], namespace: _zwSanNsByIdx[a[1]] });
        }
      }
      cfg.elements.push(e);
    }
    cfg.attributes = [];
    for (var k = 0; k < _zwSanDefaultAttributesRaw.length; k++) {
      cfg.attributes.push({ name: _zwSanDefaultAttributesRaw[k], namespace: null });
    }
    return cfg;
  }
  function _zwSanGetDefaultInstance() {
    if (_zwSanDefaultInstance === null) {
      _zwSanDefaultInstance = new Sanitizer('default');
    }
    return _zwSanDefaultInstance;
  }

  // ===== Sanitizer 接口（#sanitizer-interface；WebIDL interface object 形态）=====
  function _zwSanCloneAttr(a) { return { name: a.name, namespace: a.namespace }; }
  function _zwSanCloneEl(e) {
    var out = { name: e.name, namespace: e.namespace };
    if (e.attributes !== undefined) {
      out.attributes = _zwSanSortList(e.attributes, _zwSanNsNameCmp).map(_zwSanCloneAttr);
    }
    if (e.removeAttributes !== undefined) {
      out.removeAttributes = _zwSanSortList(e.removeAttributes, _zwSanNsNameCmp).map(_zwSanCloneAttr);
    }
    return out;
  }

  function Sanitizer(config) {
    var cfg;
    if (config === 'default' || config === undefined || config === null) {
      cfg = _zwSanBuildDefaultCfg();
    } else if (config !== null && typeof config === 'object') {
      cfg = _zwSanCanonConfig(config);
    } else {
      // 非 'default' 字符串/其他标量：WebIDL dict 语义宽松面——按空 dict。
      cfg = _zwSanCanonConfig({});
    }
    Object.defineProperty(this, '__zwSanCfg', { value: cfg });
  }

  Sanitizer.prototype.get = function () {
    var cfg = this.__zwSanCfg;
    var out = {};
    if (cfg.elements !== undefined) {
      out.elements = _zwSanSortList(cfg.elements, _zwSanNsNameCmp).map(_zwSanCloneEl);
    } else if (cfg.removeElements !== undefined) {
      out.removeElements = _zwSanSortList(cfg.removeElements, _zwSanNsNameCmp)
        .map(function (e) { return { name: e.name, namespace: e.namespace }; });
    }
    if (cfg.replaceWithChildrenElements !== undefined) {
      out.replaceWithChildrenElements = _zwSanSortList(cfg.replaceWithChildrenElements, _zwSanNsNameCmp)
        .map(function (e) { return { name: e.name, namespace: e.namespace }; });
    }
    if (cfg.attributes !== undefined) {
      out.attributes = _zwSanSortList(cfg.attributes, _zwSanNsNameCmp).map(_zwSanCloneAttr);
    } else if (cfg.removeAttributes !== undefined) {
      out.removeAttributes = _zwSanSortList(cfg.removeAttributes, _zwSanNsNameCmp).map(_zwSanCloneAttr);
    }
    if (cfg.processingInstructions !== undefined) {
      out.processingInstructions = _zwSanSortList(cfg.processingInstructions, _zwSanTargetCmp)
        .map(function (p) { return { target: p.target }; });
    } else if (cfg.removeProcessingInstructions !== undefined) {
      out.removeProcessingInstructions = _zwSanSortList(cfg.removeProcessingInstructions, _zwSanTargetCmp)
        .map(function (p) { return { target: p.target }; });
    }
    out.comments = cfg.comments;
    if (cfg.dataAttributes !== undefined) out.dataAttributes = cfg.dataAttributes;
    return out;
  };

  // ===== modifiers（#dom-sanitizer-allowelement 等十方法；方向内移动 + 冗余
  // 剥除——sanitizer-modifiers.html 合并案语义；返回值 true=有变化）=====
  function _zwSanStripMatches(list, pool, same) {
    var out = [];
    for (var i = 0; i < list.length; i++) {
      if (!_zwSanListHas(pool, list[i], same)) out.push(list[i]);
    }
    return out;
  }
  function _zwSanGetMatchingElEntry(cfg, name, ns) {
    if (!cfg.elements) return null;
    for (var i = 0; i < cfg.elements.length; i++) {
      if (cfg.elements[i].name === name && cfg.elements[i].namespace === ns) return cfg.elements[i];
    }
    return null;
  }
  Sanitizer.prototype.allowElement = function (element) {
    var cfg = this.__zwSanCfg;
    var entry = _zwSanCanonElAllow(element);
    var hadAttrs = element !== null && typeof element === 'object' && element.attributes !== undefined;
    var hadRm = element !== null && typeof element === 'object' && element.removeAttributes !== undefined;
    // 原始输入 member（canon 形、未剥）——attrs/rm 交叉剥除的基准（modifiers
    // 语料三臂腿 {p, attributes:[含 lang], removeAttributes:[含 lang]} 双向
    // 佐证：条目双带时 attrs 侧让位，rm 侧成员仅在全局名单方向有效时保留）。
    var rawAttrs = hadAttrs ? entry.attributes : null;
    var rawRm = hadRm ? entry.removeAttributes : null;
    if (cfg.elements) {
      // elements 方向：rWC 命中先剔除（config 页「Test elements
      // replacewithchildren」allowElement({div}) rWC 移动臂）。
      var changed = false;
      if (cfg.replaceWithChildrenElements) {
        for (var irw = 0; irw < cfg.replaceWithChildrenElements.length; irw++) {
          if (_zwSanSameEl(cfg.replaceWithChildrenElements[irw], entry)) {
            cfg.replaceWithChildrenElements.splice(irw, 1); changed = true; break;
          }
        }
      }
      var existing = _zwSanGetMatchingElEntry(cfg, entry.name, entry.namespace);
      // 条目 member 归一（新条目与替换共用）：
      // rm −= 全局 removeAttributes；cfg.attributes 在席（allow 方向，rm 为
      //   合法覆盖机制）→ rm ∩ 全局 attributes；缺席（remove 方向，条目 rm
      //   仅在不在全局任何名单时才具元素级禁用语义）→ rm −= 原始 attrs。
      // attrs −= 全局 attributes −= 全局 removeAttributes −= 原始 rm。
      if (hadRm) {
        var rmN = _zwSanStripMatches(rawRm, cfg.removeAttributes || [], _zwSanSameAttr);
        if (cfg.attributes) {
          var kept = [];
          for (var ikr = 0; ikr < rmN.length; ikr++) {
            if (_zwSanListHas(cfg.attributes, rmN[ikr], _zwSanSameAttr)) kept.push(rmN[ikr]);
          }
          rmN = kept;
        } else if (hadAttrs) {
          rmN = _zwSanStripMatches(rmN, rawAttrs, _zwSanSameAttr);
        }
        entry.removeAttributes = _zwSanAttrListDedup(rmN);
      }
      if (hadAttrs) {
        var atN = _zwSanStripMatches(rawAttrs, cfg.attributes || [], _zwSanSameAttr);
        atN = _zwSanStripMatches(atN, cfg.removeAttributes || [], _zwSanSameAttr);
        if (hadRm) atN = _zwSanStripMatches(atN, rawRm, _zwSanSameAttr);
        entry.attributes = _zwSanAttrListDedup(atN);
      }
      if (existing) {
        // 已存在：member 替换语义（config 页「attribute-per-element sets
        // (i.e. overwrites)」+ modifiers 全序列 ground truth）——输入提供
        // 的 member 归一替换（含去重；输入 member 本身含重复即计变化——
        // ["dir","dir"]→["dir"] 值同仍返 true）；未提供的 member 删键
        // （attributes 与 removeAttributes 对称——三臂腿期望 rm:undefined
        // 与 rm-only 腿期望 attrs:undefined 双向佐证）；纯条目（双无）=
        // 重置缺省形：attrs 删键、rm 置 []（{name:"p"}→{attrs:undefined,
        // removeAttributes:[]} 脸；再调用纯条目零变化→false）。
        var c2 = false;
        if (hadAttrs) {
          if (_zwSanAttrListHasDup(rawAttrs)) c2 = true;
          // 宽松等价（键缺席 ≡ 空数组）——同输入重复应用时归一剥空与既有
          // 缺席状态逐值相同（modifiers 三臂腿重复调用期望 false；严格比较
          // 在此误计 changed）。
          if (!_zwSanAttrListEqLoose(existing.attributes, entry.attributes)) {
            if (!entry.attributes.length) delete existing.attributes;
            else existing.attributes = entry.attributes;
            c2 = true;
          }
        } else if (existing.attributes !== undefined) {
          delete existing.attributes; c2 = true;
        }
        if (hadRm) {
          if (_zwSanAttrListHasDup(rawRm)) c2 = true;
          if (!_zwSanAttrListEqLoose(existing.removeAttributes, entry.removeAttributes)) {
            if (!entry.removeAttributes.length) delete existing.removeAttributes;
            else existing.removeAttributes = entry.removeAttributes;
            c2 = true;
          }
        } else if (!hadAttrs) {
          // 纯条目重置缺省形（rm 置 []——canon 缺省形）。
          if (!_zwSanAttrListEq(existing.removeAttributes, [])) {
            existing.removeAttributes = []; c2 = true;
          }
        } else if (existing.removeAttributes !== undefined) {
          delete existing.removeAttributes; c2 = true;
        }
        return changed || c2;
      }
      // 新条目：attrs 剥空删键；rm 剥空——用户 member→删键（显式空意图），
      // canon 缺省形→保留 []（config 页「Test elements addition」
      // {name:"bla",…,removeAttributes:[]} + modifiers 脸 {name:"p"}→[]）。
      if (hadAttrs && !entry.attributes.length) delete entry.attributes;
      if (hadRm && !entry.removeAttributes.length) delete entry.removeAttributes;
      cfg.elements.push(entry);
      return true;
    }
    // removeElements 方向（统一允许语义——config 页「Test elements
    // removal」{p,XHTML} present 剔除 + modifiers 脸 rm:[] 剔除；带
    // attributes member 或非空 removeAttributes member 拒绝）。
    if (hadAttrs) return false;
    if (hadRm && rawRm.length) return false;
    if (_zwSanListHas(cfg.removeElements, entry, _zwSanSameEl)) {
      for (var ire2 = 0; ire2 < cfg.removeElements.length; ire2++) {
        if (_zwSanSameEl(cfg.removeElements[ire2], entry)) { cfg.removeElements.splice(ire2, 1); break; }
      }
      return true;
    }
    if (cfg.replaceWithChildrenElements && _zwSanListHas(cfg.replaceWithChildrenElements, entry, _zwSanSameEl)) {
      for (irw = 0; irw < cfg.replaceWithChildrenElements.length; irw++) {
        if (_zwSanSameEl(cfg.replaceWithChildrenElements[irw], entry)) {
          cfg.replaceWithChildrenElements.splice(irw, 1); break;
        }
      }
      return true;
    }
    return false;
  };
  function _zwSanAttrListHasDup(list) {
    if (!list) return false;
    for (var i = 0; i < list.length; i++) {
      for (var j = i + 1; j < list.length; j++) {
        if (_zwSanSameAttr(list[i], list[j])) return true;
      }
    }
    return false;
  }
  function _zwSanAttrListDedup(list) {
    var out = [];
    for (var i = 0; i < list.length; i++) {
      if (!_zwSanListHas(out, list[i], _zwSanSameAttr)) out.push(list[i]);
    }
    return out;
  }
  function _zwSanAttrListEq(a, b) {
    var la = a === undefined ? -1 : a.length;
    var lb = b === undefined ? -1 : b.length;
    if (la !== lb) return false;
    if (la < 0) return true;
    for (var i = 0; i < la; i++) {
      if (!_zwSanSameAttr(a[i], b[i])) return false;
    }
    return true;
  }
  // 宽松版：键缺席与空数组等价（归一剥空后的比较基准）——纯条目重置分支
  // 需要区分缺席与 []（rm 置 [] 即变化），不用此版。
  function _zwSanAttrListEqLoose(a, b) {
    var la = a === undefined ? 0 : a.length;
    var lb = b === undefined ? 0 : b.length;
    if (la !== lb) return false;
    for (var i = 0; i < la; i++) {
      if (!_zwSanSameAttr(a[i], b[i])) return false;
    }
    return true;
  }
  // removeElement：elements 方向 → 从 elements/rWC 剔；removeElements 方向 →
  // 从 rWC 剔并落 removeElements（已在则 false）；缺席则入 removeElements。
  Sanitizer.prototype.removeElement = function (element) {
    var cfg = this.__zwSanCfg;
    var entry = _zwSanCanonElPlain(element);
    if (cfg.elements) {
      var changed = false;
      for (var i = 0; i < cfg.elements.length; i++) {
        if (_zwSanSameEl(cfg.elements[i], entry)) { cfg.elements.splice(i, 1); changed = true; break; }
      }
      if (cfg.replaceWithChildrenElements) {
        for (var j = 0; j < cfg.replaceWithChildrenElements.length; j++) {
          if (_zwSanSameEl(cfg.replaceWithChildrenElements[j], entry)) {
            cfg.replaceWithChildrenElements.splice(j, 1); changed = true; break;
          }
        }
      }
      return changed;
    }
    for (var k = 0; k < cfg.removeElements.length; k++) {
      if (_zwSanSameEl(cfg.removeElements[k], entry)) return false;
    }
    if (cfg.replaceWithChildrenElements) {
      for (var m = 0; m < cfg.replaceWithChildrenElements.length; m++) {
        if (_zwSanSameEl(cfg.replaceWithChildrenElements[m], entry)) {
          cfg.replaceWithChildrenElements.splice(m, 1);
          cfg.removeElements.push({ name: entry.name, namespace: entry.namespace });
          return true;
        }
      }
    }
    cfg.removeElements.push({ name: entry.name, namespace: entry.namespace });
    return true;
  };
  // replaceElementWithChildren：缺席 rWC 且非替换件 → 从 elements/removeElements
  // 剔（若有）并入 rWC；已在 rWC 或非替换件 → false（无变化）。
  Sanitizer.prototype.replaceElementWithChildren = function (element) {
    var cfg = this.__zwSanCfg;
    var entry = _zwSanCanonElPlain(element);
    if (_zwSanIsNonReplaceable(entry.name, entry.namespace)) return false;
    if (cfg.replaceWithChildrenElements &&
        _zwSanListHas(cfg.replaceWithChildrenElements, entry, _zwSanSameEl)) return false;
    if (cfg.elements) {
      for (var i = 0; i < cfg.elements.length; i++) {
        if (_zwSanSameEl(cfg.elements[i], entry)) { cfg.elements.splice(i, 1); break; }
      }
    } else if (cfg.removeElements) {
      for (var j = 0; j < cfg.removeElements.length; j++) {
        if (_zwSanSameEl(cfg.removeElements[j], entry)) { cfg.removeElements.splice(j, 1); break; }
      }
    }
    if (!cfg.replaceWithChildrenElements) cfg.replaceWithChildrenElements = [];
    cfg.replaceWithChildrenElements.push({ name: entry.name, namespace: entry.namespace });
    return true;
  };
  // allowAttribute：attributes 方向 → 全局允许 + 各元素条目 attributes 剥冗余；
  // removeAttributes 方向 → 全局禁用追加 + 各元素条目 attributes 剥命中项。
  Sanitizer.prototype.allowAttribute = function (attribute) {
    var cfg = this.__zwSanCfg;
    var attr = _zwSanCanonAttrEntry(attribute);
    if (cfg.attributes) {
      if (_zwSanListHas(cfg.attributes, attr, _zwSanSameAttr)) return false;
      cfg.attributes.push({ name: attr.name, namespace: attr.namespace });
      _zwSanStripAttrFromElements(cfg, attr);
      return true;
    }
    // removeAttributes 方向（corpus sanitizer-modifiers「allowAttribute() with
    // global removeAttributes」ground truth）：在名单＝被禁——剥除恢复允许→
    // true（空名单不删键——assert_config 断言 removeAttributes: [] 在席）；
    // 缺席＝默认已允许——零变化→false。旧版双向反置（在名单返 false、缺席
    // 无条件推入名单把默认允许属性反禁）——immutable 语料 allowAttribute("id")
    // 脸（div#allowed 丢 id → null）＋modifiers 族红脸同根因。刀⑩ 修。
    var _aaIdx = -1;
    if (cfg.removeAttributes) {
      for (var _aai = 0; _aai < cfg.removeAttributes.length; _aai++) {
        if (_zwSanSameAttr(cfg.removeAttributes[_aai], attr)) { _aaIdx = _aai; break; }
      }
    }
    if (_aaIdx < 0) return false;
    cfg.removeAttributes.splice(_aaIdx, 1);
    return true;
  };
  // removeAttribute：attributes 方向 → 全局允许剔除（缺席不 false——元素条目面
  // 仍可能变化）；removeAttributes 方向 → 全局禁用追加；两向均剥元素条目
  // attributes 与 removeAttributes 命中项（modifiers 语料 removeAttribute
  // 双列表剥除案）。
  Sanitizer.prototype.removeAttribute = function (attribute) {
    var cfg = this.__zwSanCfg;
    var attr = _zwSanCanonAttrEntry(attribute);
    var changed = false;
    if (cfg.attributes) {
      for (var i = 0; i < cfg.attributes.length; i++) {
        if (_zwSanSameAttr(cfg.attributes[i], attr)) { cfg.attributes.splice(i, 1); changed = true; break; }
      }
    } else {
      if (_zwSanListHas(cfg.removeAttributes, attr, _zwSanSameAttr)) return false;
      cfg.removeAttributes.push({ name: attr.name, namespace: attr.namespace });
      changed = true;
    }
    if (_zwSanStripAttrFromElements(cfg, attr, true)) changed = true;
    if (cfg.elements) {
      for (var k = 0; k < cfg.elements.length; k++) {
        var ent = cfg.elements[k];
        if (ent.removeAttributes) {
          var after = _zwSanStripMatches(ent.removeAttributes, [attr], _zwSanSameAttr);
          if (after.length !== ent.removeAttributes.length) changed = true;
          // 剥空保留 []（modifiers「removeAttribute with global attributes and
          // elements」期望 {attributes:[], removeAttributes:[]} 双键在席）。
          ent.removeAttributes = after;
        }
      }
    }
    return changed;
  };
  function _zwSanStripAttrFromElements(cfg, attr, keepEmpty) {
    if (!cfg.elements) return false;
    var changed = false;
    for (var i = 0; i < cfg.elements.length; i++) {
      var ent = cfg.elements[i];
      if (ent.attributes) {
        var after = _zwSanStripMatches(ent.attributes, [attr], _zwSanSameAttr);
        if (after.length !== ent.attributes.length) changed = true;
        ent.attributes = after;
        if (!after.length && !keepEmpty) delete ent.attributes;
      }
    }
    return changed;
  }
  // PI modifiers：allow 方向追加/剔除；remove 方向对称。allow 的 remove 方向
  // 修正：在名单＝被禁——剥除恢复处理→true（config 页「Test
  // processingInstructions removal」{t1} 剥除→2 脸）；缺席＝默认处理→false。
  Sanitizer.prototype.allowProcessingInstruction = function (pi) {
    var cfg = this.__zwSanCfg;
    var item = _zwSanCanonPi(pi);
    if (cfg.processingInstructions) {
      if (_zwSanListHas(cfg.processingInstructions, item, _zwSanSamePi)) return false;
      cfg.processingInstructions.push({ target: item.target });
      return true;
    }
    for (var i = 0; i < cfg.removeProcessingInstructions.length; i++) {
      if (_zwSanSamePi(cfg.removeProcessingInstructions[i], item)) {
        cfg.removeProcessingInstructions.splice(i, 1);
        return true;
      }
    }
    return false;
  };
  Sanitizer.prototype.removeProcessingInstruction = function (pi) {
    var cfg = this.__zwSanCfg;
    var item = _zwSanCanonPi(pi);
    if (cfg.processingInstructions) {
      for (var i = 0; i < cfg.processingInstructions.length; i++) {
        if (_zwSanSamePi(cfg.processingInstructions[i], item)) {
          cfg.processingInstructions.splice(i, 1);
          return true;
        }
      }
      return false;
    }
    if (_zwSanListHas(cfg.removeProcessingInstructions, item, _zwSanSamePi)) return false;
    cfg.removeProcessingInstructions.push({ target: item.target });
    return true;
  };
  Sanitizer.prototype.setComments = function (v) {
    this.__zwSanCfg.comments = v ? true : false;
  };
  Sanitizer.prototype.setDataAttributes = function (v) {
    this.__zwSanCfg.dataAttributes = v ? true : false;
  };
  // removeUnsafe（#dom-sanitizer-removeunsafe）：allow 方向干净面零变化（默认
  // 配置两案实证——elements/attributes 存在时仅剥基线命中条目，键保持）；
  // 无 allow 方向时补基线 removeElements/removeAttributes。
  Sanitizer.prototype.removeUnsafe = function () {
    var cfg = this.__zwSanCfg;
    if (cfg.elements !== undefined) {
      var filtered = [];
      for (var i = 0; i < cfg.elements.length; i++) {
        var e = cfg.elements[i];
        var unsafe = false;
        for (var j = 0; j < _zwSanBaselineRemoveEls.length; j++) {
          if (_zwSanSameEl(e, _zwSanBaselineRemoveEls[j])) { unsafe = true; break; }
        }
        if (!unsafe) filtered.push(e);
      }
      cfg.elements = filtered;
    } else {
      if (cfg.removeElements === undefined) cfg.removeElements = [];
      for (var k = 0; k < _zwSanBaselineRemoveEls.length; k++) {
        var b = _zwSanBaselineRemoveEls[k];
        if (!_zwSanListHas(cfg.removeElements, b, _zwSanSameEl)) {
          cfg.removeElements.push({ name: b.name, namespace: b.namespace });
        }
      }
    }
    if (cfg.attributes !== undefined) {
      var fAttrs = [];
      for (var m = 0; m < cfg.attributes.length; m++) {
        if (!_zwSanEventHandlers[_zwSanAsciiLower(cfg.attributes[m].name)]) fAttrs.push(cfg.attributes[m]);
      }
      cfg.attributes = fAttrs;
    } else {
      if (cfg.removeAttributes === undefined) cfg.removeAttributes = [];
      for (var hn in _zwSanEventHandlers) {
        var ha = { name: hn, namespace: null };
        if (!_zwSanListHas(cfg.removeAttributes, ha, _zwSanSameAttr)) cfg.removeAttributes.push(ha);
      }
    }
    if (cfg.replaceWithChildrenElements) {
      var fRwc = [];
      for (var r = 0; r < cfg.replaceWithChildrenElements.length; r++) {
        if (!_zwSanIsNonReplaceable(cfg.replaceWithChildrenElements[r].name,
            cfg.replaceWithChildrenElements[r].namespace)) fRwc.push(cfg.replaceWithChildrenElements[r]);
      }
      cfg.replaceWithChildrenElements = fRwc;
    }
  };

  // ===== 过滤引擎（materialize-then-filter 的 filter 臂）=====
  // 节点数组就地过滤（mock 域）：元素（remove/unwrap/keep + 属性面）、注释
  //（comments）、PI（processingInstructions 面）；文本恒保留。
  function _zwSanFilterList(kids, san, safe) {
    var cfg = san.__zwSanCfg;
    for (var i = kids.length - 1; i >= 0; i--) {
      var n = kids[i];
      if (!n || typeof n.nodeType !== 'number') continue;
      if (n.nodeType === 1) {
        var g = n.childNodes;
        if (g && g.length) _zwSanFilterList(g, san, safe);
        // template contents 子树同域消毒（spec contents 属节点树；mock 面
        // template 子在 content fragment 不在 childNodes——旧版整跳 content，
        // per-element 属性方向在模板内不生效 12 案）。
        // https://html.spec.whatwg.org/multipage/scripting.html#the-template-element
        try {
          var gT = n.content && n.content.childNodes;
          if (gT && gT.length) _zwSanFilterList(gT, san, safe);
        } catch (_eTplF) {}
        var vd = _zwSanElementVerdict(n, cfg, safe);
        if (vd.v === 'remove') {
          kids.splice(i, 1);
        } else if (vd.v === 'unwrap') {
          var inner = n.childNodes || [];
          var args = [i, 1];
          for (var j = 0; j < inner.length; j++) args.push(inner[j]);
          Array.prototype.splice.apply(kids, args);
        } else {
          _zwSanFilterElementAttrs(n, cfg, safe, vd.entry);
        }
      } else if (n.nodeType === 8) {
        if (!cfg.comments) kids.splice(i, 1);
      } else if (n.nodeType === 7) {
        if (!_zwSanPiAllowed(n, cfg)) kids.splice(i, 1);
      }
    }
  }
  function _zwSanElName(n) {
    // SVG/MathML 名大小写敏感（语料 names.html feBlend/textPath）；XHTML ASCII-ci。
    var ln = null;
    try { ln = n.localName != null ? String(n.localName) : null; } catch (e) { ln = null; }
    if (ln === null) {
      try { ln = String(n.tagName || ''); } catch (e2) { ln = ''; }
    }
    return ln;
  }
  function _zwSanElNs(n) {
    try { return n.namespaceURI == null ? null : String(n.namespaceURI); } catch (e) { return null; }
  }
  // 元素 verdict：返回 {v: 'remove'|'unwrap'|'keep', entry}——entry=elements
  // 允许方向的命中条目（per-element attributes 过滤面；返回值传递而非节点写面
  // ——handle 域 proxy 的 set trap 会把未知属性误写引擎）。
  function _zwSanElementVerdict(n, cfg, safe) {
    var ln = _zwSanElName(n);
    var ns = _zwSanElNs(n);
    var isXhtml = ns === _zwSanNsXhtml || ns === null;
    var name = isXhtml ? _zwSanAsciiLower(ln) : ln;
    var probe = { name: name, namespace: ns };
    // 1. safe 强制基线（setHTML/parseHTML——unsafe 方法仅用户配置）。
    if (safe) {
      for (var b = 0; b < _zwSanBaselineRemoveEls.length; b++) {
        if (_zwSanSameEl(probe, _zwSanBaselineRemoveEls[b])) return { v: 'remove', entry: null };
      }
    }
    // 2. rWC（modifiers 动态面 + validate 允许的显式输入）。
    if (cfg.replaceWithChildrenElements &&
        _zwSanListHas(cfg.replaceWithChildrenElements, probe, _zwSanSameEl)) return { v: 'unwrap', entry: null };
    // 3. removeElements 命中。
    if (cfg.removeElements !== undefined && cfg.removeElements.length &&
        _zwSanListHas(cfg.removeElements, probe, _zwSanSameEl)) return { v: 'remove', entry: null };
    // 4. elements 允许方向：缺席即删；removeElements 方向：默认 keep。
    var entry = null;
    if (cfg.elements !== undefined) {
      for (var e = 0; e < cfg.elements.length; e++) {
        if (_zwSanSameEl(cfg.elements[e], probe)) { entry = cfg.elements[e]; break; }
      }
      if (entry === null) return { v: 'remove', entry: null };
    }
    // 5. 内建 animating URL 属性面：SVG animate/animateTransform/set +
    // attributeName 指向 href 系（href 或 *:href 后缀）→ **元素保留**，仅属性
    // 过滤层剥 attributeName（下方第 4 臂 safe 面）；animateMotion 不在清单＝
    // 元素与属性全保留（语料 built-in-animating-url-attributes-list 期望树：
    // animate/animateTransform/set 保元素剥属性、animateMotion 全保留——旧版
    // animate 在此整删方向订正刀㊻b）。
    // https://html.spec.whatwg.org/multipage/dynamic-markup-insertion.html#built-in-animating-url-attributes-list
    return { v: 'keep', entry: entry };
  }
  // PI 允许判定（#processing-instructions 面）。
  function _zwSanPiAllowed(n, cfg) {
    var target = null;
    try { target = String(n.target != null ? n.target : n.nodeName); } catch (e) { target = ''; }
    if (cfg.processingInstructions !== undefined) {
      return _zwSanListHas(cfg.processingInstructions, { target: target }, _zwSanSamePi);
    }
    if (cfg.removeProcessingInstructions !== undefined) {
      return !_zwSanListHas(cfg.removeProcessingInstructions, { target: target }, _zwSanSamePi);
    }
    return true;
  }
  // 元素属性面过滤：快照 NamedNodeMap 后逐 attr 判定（判定序：data-* → safe
  // 基线 on* → javascript: 值面 → animating attributeName → 用户配置方向）。
  // elEntry=verdict 传递的允许条目（双域共用——mock 数组臂与 handle 真实树臂）。
  function _zwSanFilterElementAttrs(n, cfg, safe, elEntry) {
    var list = [];
    try {
      var am = n.attributes;
      if (am) for (var i = 0; i < am.length; i++) { if (am[i]) list.push(am[i]); }
    } catch (e) { return; }
    elEntry = elEntry || null;
    for (var k = 0; k < list.length; k++) {
      var attr = list[k];
      var qname = String(attr.name);
      var lower = _zwSanAsciiLower(qname);
      var local = lower;
      var ci = lower.indexOf(':');
      if (ci >= 0) local = lower.slice(ci + 1);
      var ans = null;
      try { ans = attr.namespaceURI == null ? null : String(attr.namespaceURI); } catch (e2) { ans = null; }
      // 1. data-*（#data-attributes 面——dataAttributes 真即显式允许直通，
      // 允许列表含 data-* 且 dataAttributes:true 在构造期已 TypeError，不存在
      // 命中面；dataAttributes:false 时显式允许名单（全局/per-element）仍放行
      //——dataAttributes 是额外放行门不否定显式 allowlist；语料
      // sanitizer-basic-filtering dataAttributes/2：{attributes:['data-x'],
      // dataAttributes:false} 期望仅 data-x 保留，data-y/data-z 剥除）。
      // https://wicg.github.io/sanitizer-api/#data-attributes
      if (local.lastIndexOf('data-', 0) === 0 && (!ans || ans === _zwSanNsXhtml)) {
        if (cfg.dataAttributes !== true) {
          var _daHit = _zwSanAttrAllowHit(cfg.attributes, local, ans) ||
            (elEntry != null && _zwSanAttrAllowHit(elEntry.attributes, local, ans));
          if (!_daHit) _zwSanDropAttr(n, qname);
        }
        continue;
      }
      // 2. safe 基线：事件处理器（ns null、真实清单——on* 非前缀匹配）。
      if (safe && ans === null && _zwSanEventHandlers[local]) { _zwSanDropAttr(n, qname); continue; }
      // 3. 内建 javascript: 值剥离（navigating 面：href/action/formaction 任意元素）。
      if ((local === 'href' || local === 'action' || local === 'formaction') &&
          _zwSanIsJsUrl(attr.value)) { _zwSanDropAttr(n, qname); continue; }
      // 4. animating URL 属性面（safe 方法）：SVG animate/animateTransform/set
      // 的 attributeName 指向 URL 属性（href 或 *:href 后缀——xlink:href 及
      // 声明/未声明的同形伪前缀 ylink:href/zlink:href）→ 剥 attributeName、
      // 元素保留；animateMotion 不在清单；unsafe 方法全保留（语料
      // sanitizer-svg-animate unsafe 档 retention 断言全保留）。hreflang/
      // xref/双冒号（xlink:href:x）非 URL 目标不剥（inert 档）。刀㊻b（s10k59）：
      // 名单 animateMotion→animate/set 换员 + safe 门 + *:href 后缀泛化。
      // https://html.spec.whatwg.org/multipage/dynamic-markup-insertion.html#built-in-animating-url-attributes-list
      if (safe && local === 'attributename' && _zwSanElNs(n) === _zwSanNsSvg) {
        var tg = _zwSanAsciiLower(_zwSanElName(n));
        // 值比对 exact（ASCII case 不敏感）**不 trim**——语料 javascript-url
        // allowed/2 " href "（带空格）期望保留＝非 URL 目标对照面。
        var v = _zwSanAsciiLower(String(attr.value == null ? '' : attr.value));
        if ((tg === 'animate' || tg === 'animatetransform' || tg === 'set') &&
            (v === 'href' || (v.length > 5 && v.slice(-5) === ':href'))) { _zwSanDropAttr(n, qname); continue; }
      }
      // 5. 用户配置方向（全局与 per-element 条目同向并集——spec SanitizerConfig
      // element entry 的 attributes/removeAttributes 与全局列表并合生效；任一
      // allow 列表在席即 allow 向，remove 命中或全不命中才剥除。旧版仅在全局
      // attributes 在席时查 elEntry——纯 element 条目方向整面落空）。
      // https://wicg.github.io/sanitizer-api/#sanitizerconfig-element
      var hitGRem = cfg.removeAttributes !== undefined && cfg.removeAttributes.length &&
        _zwSanListHas(cfg.removeAttributes, { name: local, namespace: ans }, _zwSanSameAttr);
      var hitERem = elEntry != null && elEntry.removeAttributes !== undefined &&
        elEntry.removeAttributes.length &&
        _zwSanListHas(elEntry.removeAttributes, { name: local, namespace: ans }, _zwSanSameAttr);
      // qname 前缀 ns 派生（mock 属性面 namespaceURI 缺失桥——语料
      // sanitizer-names「xlink:href inside SVG tree」：per-element allow 列表
      // {href, XLINK_NS} 对 qname 前缀形 xlink:href 命中需 ns 补全）。仅外来
      // 内容（SVG/MathML）派生——HTML 内容解析器保留 qname 且 ns 为 null
      //（spec #3 形：{href, XLINK} 对 HTML 内容 xlink:href 必须不命中剥除）。
      // 仅方向块使用——on*/data-* 安全门仍用原始 ans 判定不过桥。
      // https://html.spec.whatwg.org/multipage/parsing.html#adjust-foreign-attributes
      var ansDir = ans;
      var elNsDir = _zwSanElNs(n);
      if (ansDir == null && ci >= 0 &&
          (elNsDir === _zwSanNsSvg || elNsDir === _zwSanNsMathml)) {
        var pfxDir = lower.slice(0, ci);
        if (pfxDir === 'xlink') ansDir = 'http://www.w3.org/1999/xlink';
        else if (pfxDir === 'xml') ansDir = 'http://www.w3.org/XML/1998/namespace';
        else if (pfxDir === 'xmlns') ansDir = 'http://www.w3.org/2000/xmlns/';
      }
      var hasAllow = cfg.attributes !== undefined ||
        (elEntry != null && elEntry.attributes !== undefined);
      var hitAllow = false;
      if (hasAllow) {
        hitAllow = (cfg.attributes !== undefined &&
            _zwSanListHas(cfg.attributes, { name: local, namespace: ansDir }, _zwSanSameAttrCi)) ||
          (elEntry != null && elEntry.attributes !== undefined &&
            _zwSanListHas(elEntry.attributes, { name: local, namespace: ansDir }, _zwSanSameAttrCi));
      }
      if (hitGRem || hitERem || (hasAllow && !hitAllow)) { _zwSanDropAttr(n, qname); }
    }
  }
  function _zwSanDropAttr(n, qname) {
    try { n.removeAttribute(qname); } catch (e) {}
  }

  // ===== options 解析（#dom-element-sethtml options 字典）=====
  // safe 方法：sanitizer 缺席 → 默认配置实例；unsafe 方法：缺席 → null（不消毒）。
  // sanitizer 为实例直用；其余按 dict 构造（TypeError 透传）。
  function _zwSanResolveOptions(options, safe) {
    var s = (options !== null && typeof options === 'object') ? options.sanitizer : undefined;
    // 刀⑩：TrustedParserOptions.sanitizer() 方法形归一（spec branded 产物
    // sanitizer 是方法——每调返回 Sanitizer clone；part12 包装升级后 branded
    // 上为函数值属性。函数形调用取实例再走既有管道；part15 stream 面经
    // 导出同函数消费自动覆盖）。
    if (typeof s === 'function') {
      try { s = s(); } catch (_eK10fn) { s = undefined; }
    }
    if (s === undefined || s === null) {
      return safe ? _zwSanGetDefaultInstance() : null;
    }
    if (s instanceof Sanitizer) return s;
    return new Sanitizer(s);
  }

  // ===== 刀⑩（dc2-s10k10，2026-11-24）：default policy createParserOptions 链 =====
  // spec：TT enforced 时每个 HTML 解析 sink（setHTMLUnsafe/innerHTML/outerHTML/
  // insertAdjacentHTML/createContextualFragment/parseHTMLUnsafe）解析前经 default
  // policy 的 createParserOptions 回调取得 ParserOptions（caller 无 options 传
  // `{}`）；返回值须 object（null/undefined/number/string → TypeError 且零落盘）；
  // 回调异常原样传播；回调收到的 caller options 是副本（immutable 语料两案：
  // 回调变异 options/sanitizer 不回写 caller——dict 形 JSON 深拷贝、Sanitizer
  // 实例 clone）。TrustedParserOptions 品牌对象（part12 `__zwTtParserOptions`
  // 印）直通不二次回调。无 default policy 或缺 createParserOptions 回调 →
  // caller options 原样返回（零变化——TT 主语料 createHTML-only 页安全）。
  // 与 part15 stream 面 `_processOptions` 同源语义（彼处含能力缺失 TypeError
  // 门——stream 语料专有；本面无该语料脸，缺回调不抛）。消费面：
  // 1) `processed.sanitizer` 经 `_zwSanResolveOptions` 走既有消毒管道；
  // 2) `processed.runScripts === false` → 产物 script 盖 already-started 章
  //   （ccf 臂——setHTMLUnsafe 写链恒不执行 script，无需盖章）。
  // https://wicg.github.io/sanitizer-api/#integrate-with-trusted-types
  // https://w3c.github.io/webappsec-trusted-types/dist/spec/#create-parser-options-callback
  function _zwSanProcessParserOptions(options) {
    var enforced = typeof globalThis.__zwTtEnforced === 'function' && globalThis.__zwTtEnforced();
    if (!enforced) return options;
    var tt = globalThis.trustedTypes;
    var def = (tt && tt.defaultPolicy) || null;
    if (!def || typeof def.createParserOptions !== 'function') return options;
    if (options && options.__zwTtParserOptions === true) return options;
    var _copy = (options && typeof options === 'object') ? {} : {};
    if (options && typeof options === 'object') {
      for (var _pk in options) {
        if (Object.prototype.hasOwnProperty.call(options, _pk)) _copy[_pk] = options[_pk];
      }
      if (_copy.sanitizer && typeof _copy.sanitizer === 'object'
          && typeof globalThis.Sanitizer === 'function') {
        if (_copy.sanitizer instanceof globalThis.Sanitizer) {
          try { _copy.sanitizer = new globalThis.Sanitizer(_copy.sanitizer.get()); } catch (_ePcInst) {}
        } else {
          try { _copy.sanitizer = JSON.parse(JSON.stringify(_copy.sanitizer)); } catch (_ePcCfg) {}
        }
      }
    }
    var out = def.createParserOptions(_copy);
    if (out === null || typeof out !== 'object') {
      throw new globalThis.TypeError(
        "The default policy's createParserOptions must return an object.");
    }
    return out;
  }
  // 串级净化（刀⑩ sink 面消费形——materialize → filter → serialize 净串，与
  // `_zwSanSetHtmlImpl` sel/handle 域同一管道）：po 为 processed ParserOptions
  //（`sanitizer` 字段缺席/null → 原串返回零变化；enforced 页 processed 恒 object
  // 由 _zwSanProcessParserOptions 校验保证）。净化只改串——调用方常规解析链照走
  //（章/MO/引擎写零动）。handle=null detached 物化（fragmentAddedUnsafe 既有形态）。
  function _zwSanSanitizeString(str, po, safe) {
    var san = _zwSanResolveOptions(po, safe);
    if (!san) return str;
    var added = (_zwSanBridge && _zwSanBridge.fragmentAddedUnsafe)
      ? _zwSanBridge.fragmentAddedUnsafe(String(str), null) : [];
    if (san && added.length) _zwSanFilterList(added, san, safe);
    var clean = '';
    for (var _sk = 0; _sk < added.length; _sk++) {
      if (added[_sk]) clean += _zwSanBridge.serialize(added[_sk]);
    }
    return clean;
  }
  // script 类容器判定（#dom-element-sethtml context check——script 任意 ns +
  // 前缀限定 x:script 形；语料 sethtml-context 实证产出空）。
  function _zwSanIsScriptLike(sel, handle) {
    var tn = '';
    if (_zwSanBridge && typeof _zwSanBridge.realTag === 'function') {
      try {
        // realTag 双域互斥消费（part05 同款——handle 域传 null 防 sel 支路误走）。
        tn = _zwSanAsciiLower((handle != null)
          ? (_zwSanBridge.realTag(null, handle) || '')
          : (_zwSanBridge.realTag(sel, null) || ''));
      } catch (e) { tn = ''; }
    }
    return tn === 'script' || tn.lastIndexOf(':script') === tn.length - 7;
  }
  // setHTML/setHTMLUnsafe 共实现（part04 trap 双域 + part03 ShadowRoot mock 域
  // 调人，经 `globalThis.__zwSanSetHtml` late-bound）。target 非 null = ShadowRoot
  // 域（childNodes 数组就地过滤替换，无引擎写）；否则 sel/handle 域：物化视图
  // 过滤 → `serialize` 净串 → 既有写链（双端同态保持——引擎侧收到的已是净化后
  // 字符串）。
  function _zwSanSetHtmlImpl(sel, handle, target, value, options, safe) {
    var san = _zwSanResolveOptions(options, safe);
    if (target) {
      var kids = target.childNodes;
      if (!kids || !kids.length) return;
      // DOMString 强转（setHTML 语料：undefined→"undefined"、{}→"[object Object]"）。
      var strS = String(value);
      var addedS = (_zwSanBridge && _zwSanBridge.buildBodyTree)
        ? _zwSanBridge.buildBodyTree(strS, '', undefined, true).childNodes : [];
      if (san && addedS.length) _zwSanFilterList(addedS, san, safe);
      kids.length = 0;
      for (var i = 0; i < addedS.length; i++) if (addedS[i]) kids.push(addedS[i]);
      return;
    }
    // safe context check：script 类容器恒产出空（先于消毒——抛 TypeError 面
    // 目标保持原状，语料断言目标本空）。
    if (safe && _zwSanIsScriptLike(sel, handle)) {
      _zwSanBridge.setHtmlUnsafeImpl(sel, handle, '');
      return;
    }
    var str = String(value);
    // 无消毒面（safe=false 且 options 无 sanitizer → san=null）：过滤恒等，直通
    // 既有 unsafe 写链。serialize+reparse 中转对声明式 shadow root 视图**有损**——
    // mock 影子字段（__zwDeclarativeSR）不进 light 序列化，template 在中转消失
    //（ShadowRoot-init-declarative setHTMLUnsafe 残 1F 根因，s10 刀⑤）。
    // https://wicg.github.io/sanitizer-api/#dom-element-sethtmlunsafe
    if (!san) {
      _zwSanBridge.setHtmlUnsafeImpl(sel, handle, str);
      return;
    }
    var added = (_zwSanBridge && _zwSanBridge.fragmentAddedUnsafe)
      ? _zwSanBridge.fragmentAddedUnsafe(str, handle) : [];
    if (san && added.length) _zwSanFilterList(added, san, safe);
    var clean = '';
    for (var j = 0; j < added.length; j++) {
      if (added[j]) clean += _zwSanBridge.serialize(added[j]);
    }
    _zwSanBridge.setHtmlUnsafeImpl(sel, handle, clean);
  }
  // Document.parseHTML/parseHTMLUnsafe（#dom-document-parsehtml——materialize-
  // then-filter 净串走既有 `Document.parseHTMLUnsafe` 写机器（s180/gethtml，
  // part03：detached doc + `write` 全文档解析）。
  // FIXME（文档级消毒微刀，2026-11-23 勘验记档）：语料期望树含 html/head/body
  // 骨架管控（elements 不含 head → head 删；不含 html → 空文档）——片段过滤+
  // write 重组绕开骨架，26 案 "Additional nodes" 域。write 后真实树过滤臂已试
  //（probe12 实证）死面：detached doc 产物 proxy 的 attributes **读**正常
  //（len/idx/item 三通）而 `removeAttribute`/PI `remove()` **静默失败**（注释
  // `remove()` 同机器却工作——写面按节点类型分机器），骨架过滤需先修 detached
  // doc 写机器或走引擎臂，另立微刀。
  function _zwSanParseHtmlImpl(html, options, safe) {
    var san = _zwSanResolveOptions(options, safe);
    // 刀⑩：直通目标优先取包装前原始实现（`_zwSanLegacyUnsafe`，var 提升同闭包
    // 可见）——公共 `parseHTMLUnsafe` 名已被 TT/消毒包装占据，净串经它回环会在
    // enforced＋default policy inject 面上无限递归（钉测 p8 爆栈实证）。
    var legacy = (typeof _zwSanLegacyUnsafe === 'function')
      ? _zwSanLegacyUnsafe
      : (globalThis.Document && globalThis.Document.parseHTMLUnsafe);
    if (typeof legacy !== 'function' || !san) {
      // 无消毒面（unsafe 无 options）或机器缺失：直通既有实现。
      return typeof legacy === 'function' ? legacy(html) : null;
    }
    var added = (_zwSanBridge && _zwSanBridge.buildBodyTree)
      ? _zwSanBridge.buildBodyTree(String(html), '', undefined, true).childNodes : [];
    // 刀⑭（s10k14）：parseHTML 文档路径注释剥离（safe tc3——{} 配置期望注释
    // 移除且相邻文本并 "ab"：剥后序列化并文自然合并）。判定取 raw dict 显式
    // 真值（实例缺省 comments:true 是 boolean-defaults get() 钉面不可动）；
    // unsafe 路径配置被忽略→零动（unsafe tc3 期望注释保留）。setHTML 共享
    // 过滤臂零扰（_zwSanFilterList 注释臂照旧）。
    if (safe) {
      var _k14KeepComments = false;
      try { _k14KeepComments = !!(san.__zwSanCfg && san.__zwSanCfg.__zwCommentsExplicitTrue); } catch (_eK14c) {}
      if (!_k14KeepComments) _zwK14StripComments(added);
    }
    if (added.length) {
      // 刀⑭（s10k14）：safe 漏斗 data-* 缺省剥（parseHTML tc2 safe 形——
      // {attributes:['id']} 期望 data-xyz 同剥；实例 dict 缺省 dataAttributes:true
      // 是 get()/直通面钉不可动，raw 显式真值豁免后局部覆写过滤调用，调用后
      // 恢复实例态）。unsafe 路径配置忽略语义零动（tc2 unsafe 形 data-xyz
      // 保留）。setHTML 共享过滤臂零扰。
      var _k14RestoreDA = false;
      if (safe && san.__zwSanCfg && san.__zwSanCfg.dataAttributes === true &&
          san.__zwSanCfg.__zwDataAttrExplicitTrue !== true) {
        _k14RestoreDA = true;
        san.__zwSanCfg.dataAttributes = false;
      }
      _zwSanFilterList(added, san, safe);
      if (_k14RestoreDA) san.__zwSanCfg.dataAttributes = true;
    }
    var clean = '';
    for (var i = 0; i < added.length; i++) {
      if (added[i]) clean += _zwSanBridge.serialize(added[i]);
    }
    var doc = legacy(clean);
    // 刀⑭（s10k14）：文档级骨架过滤＋DOCTYPE 注入。骨架走 _zwSanElementVerdict
    // 同源语义（elements 定义但未列→remove＝tc1/3/5/6/7/8 期望的 head-less/
    // html-only/空文档形；html remove→整文档清空）；safe 基线表不含
    // html/head/body（骨架缺省保留）。DOCTYPE 从原始输入提取（body 域净化串
    // 不含）——full-document tc 期望首子。
    // https://wicg.github.io/sanitizer-api/#parse-html
    try {
      var _k14docEl = doc && doc.documentElement;
      if (_k14docEl) {
        var _k14htmlVd = _zwSanElementVerdict(_k14docEl, san.__zwSanCfg, safe);
        if (_k14htmlVd.v === 'remove') {
          doc.childNodes.length = 0;
        } else {
          var _k14dk = _k14docEl.childNodes;
          for (var _k14i = _k14dk.length - 1; _k14i >= 0; _k14i--) {
            var _k14sn = _k14dk[_k14i];
            if (_k14sn && _k14sn.nodeType === 1
                && _zwSanElementVerdict(_k14sn, san.__zwSanCfg, safe).v === 'remove') {
              _k14dk.splice(_k14i, 1);
            }
          }
          var _k14dt = (typeof globalThis.__zwK14ExtractDoctype === 'function')
            ? globalThis.__zwK14ExtractDoctype(String(html)) : null;
          if (_k14dt) {
            _k14dt.parentNode = doc;
            doc.childNodes.unshift(_k14dt);
          }
        }
      }
    } catch (_eK14doc) {}
    return doc;
  }
  // 刀⑭ 辅助：added[] 深度注释剥离（漏斗窄面专用——共享 _zwSanFilterList 注释
  // 臂按 cfg.comments 判，实例缺省恒真，parseHTML 文档路径需 spec 默认配置
  // 基线语义）。
  function _zwK14StripComments(kids) {
    for (var i = kids.length - 1; i >= 0; i--) {
      var n = kids[i];
      if (!n || typeof n.nodeType !== 'number') continue;
      if (n.nodeType === 8) kids.splice(i, 1);
      else if (n.nodeType === 1 && n.childNodes && n.childNodes.length) _zwK14StripComments(n.childNodes);
    }
  }

  // ===== 全局注册（window===globalThis 单点——part06 同款双访问形态覆盖）=====
  Object.defineProperty(globalThis, 'Sanitizer', {
    value: Sanitizer, writable: true, enumerable: false, configurable: true
  });
  // late-bound 接线（part04 trap / part03 ShadowRoot 原型消费）。
  globalThis.__zwSanSetHtml = _zwSanSetHtmlImpl;
  // 刀⑩：default policy createParserOptions 链＋串级净化导出（part04 setter 面 /
  // part03 ShadowRoot 原型 / part06 ccf 消费——跨 part globalThis 契约纪律）。
  globalThis.__zwSanProcessParserOptions = _zwSanProcessParserOptions;
  globalThis.__zwSanSanitizeString = _zwSanSanitizeString;
  // 事件处理器名单导出（part14 TT 属性 sink 判定消费——on* 名单精确匹配，
  // 语料负例 ondoesnotexist 实证前缀匹配不可用）。
  globalThis.__zwSanEventHandlers = _zwSanEventHandlers;
  try {
    if (globalThis.Document) {
      // 静态成员（partial interface Document { static ... }——首轮探针挂
      // prototype 致 "Document.parseHTML is not a function"，改挂接口对象）。
      if (!globalThis.Document.parseHTML) {
        Object.defineProperty(globalThis.Document, 'parseHTML', {
          value: function (html, options) { return _zwSanParseHtmlImpl(html, options, true); },
          writable: true, enumerable: false, configurable: true
        });
      }
      // parseHTMLUnsafe（part03 s180 已有单参实现）：包装两参形——sanitizer
      // options 缺省/null 直通既有实现（s180 语义零变化），带 options 走过滤臂
      //（unsafe 面＝仅用户配置，无安全基线）。
      if (!globalThis.Document.__zwSanUnsafeWrapped && typeof globalThis.Document.parseHTMLUnsafe === 'function') {
        var _zwSanLegacyUnsafe = globalThis.Document.parseHTMLUnsafe;
        globalThis.Document.parseHTMLUnsafe = function (html, options) {
          // 刀⑩：enforced 时 default policy createParserOptions 链（caller 无
          // options 传 {}——parsehtmlunsafe 语料「can inject a sanitizer config」
          // 无第二参脸；返回值校验 TypeError；TrustedParserOptions 印直通）。
          // 非 enforcement 页零变化。
          var _p10Enf = typeof globalThis.__zwTtEnforced === 'function' && globalThis.__zwTtEnforced();
          if (_p10Enf && typeof globalThis.__zwSanProcessParserOptions === 'function') {
            options = globalThis.__zwSanProcessParserOptions(options);
          }
          var s = (options !== null && typeof options === 'object') ? options.sanitizer : undefined;
          if (s === undefined || s === null) return _zwSanLegacyUnsafe(html);
          return _zwSanParseHtmlImpl(html, options, false);
        };
        try { globalThis.Document.__zwSanUnsafeWrapped = true; } catch (_eWrap) {}
      }
    }
  } catch (_eSanDoc) {}
  // DC-2 切片 7（wptfyi-dom，2026-11-23）：桥扩展——part15 位置方法族（链尾件，
  // 只走 globalThis 契约域）消费消毒底座两原语。桥本体在 part05 建（早于本件
  // 装配），此处原地补键；对象在席守卫防装配序倒置。
  if (globalThis.__zwSanBridge) {
    globalThis.__zwSanBridge.resolveOptions = _zwSanResolveOptions;
    globalThis.__zwSanBridge.filterList = _zwSanFilterList;
  }
})();
