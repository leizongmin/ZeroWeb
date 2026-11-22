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
  // sanitizer-removeUnsafe.html SAFE_BASELINE verbatim）。
  var _zwSanBaselineRemoveEls = [
    { name: 'base', ns: _zwSanNsXhtml },
    { name: 'embed', ns: _zwSanNsXhtml },
    { name: 'frame', ns: _zwSanNsXhtml },
    { name: 'iframe', ns: _zwSanNsXhtml },
    { name: 'object', ns: _zwSanNsXhtml },
    { name: 'script', ns: _zwSanNsXhtml },
    { name: 'script', ns: _zwSanNsSvg },
    { name: 'use', ns: _zwSanNsSvg }
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
  // object → name 强转 + namespace 缺省 XHTML（""→null 化，null 保留）；带
  // attributes member → {name, namespace, attributes}，否则 removeAttributes:[]。
  // 用户提供 removeAttributes member 时保留（modifiers 双键形态输入面）。
  function _zwSanCanonElAllow(v) {
    var name, ns, attrs = null, rmAttrs = null;
    if (v !== null && typeof v === 'object') {
      name = String(v.name);
      ns = v.namespace;
      if (ns === '' || ns === undefined) ns = _zwSanNsXhtml;
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
  // 移除方向/rWC 元素条目：{name, namespace}（namespace 缺省 XHTML、""→null）。
  function _zwSanCanonElPlain(v) {
    if (v !== null && typeof v === 'object') {
      var ns = v.namespace;
      if (ns === '' || ns === undefined) ns = _zwSanNsXhtml;
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
  function _zwSanSamePi(a, b) { return a.target === b.target; }

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
    // dataAttributes：attributes 方向可显式（缺省 true）；removeAttributes 方向
    // 存在即 TypeError（语料 removeAttributes+dataAttributes 互斥案）。
    if (hasAttrs) {
      cfg.dataAttributes = raw.dataAttributes !== undefined ? !!raw.dataAttributes : true;
    } else if (raw.dataAttributes !== undefined) {
      throw new TypeError('Sanitizer: dataAttributes is not allowed with removeAttributes.');
    }
    _zwSanValidateConfig(cfg);
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
        }
      }
      if (cfg.dataAttributes) {
        for (var ig = 0; ig < cfg.attributes.length; ig++) {
          if (_zwSanAsciiLower(cfg.attributes[ig].name).lastIndexOf('data-', 0) === 0) {
            throw new TypeError('Sanitizer: data attribute with dataAttributes enabled.');
          }
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
          for (var irm3 = 0; irm3 < entR.removeAttributes.length; irm3++) {
            if (!_zwSanListHas(cfg.removeAttributes, entR.removeAttributes[irm3], _zwSanSameAttr)) {
              throw new TypeError('Sanitizer: element entry removeAttributes must be within removeAttributes.');
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
    if (cfg.elements) {
      var existing = _zwSanGetMatchingElEntry(cfg, entry.name, entry.namespace);
      if (existing) {
        // 已存在：合并 attributes/removeAttributes member（方向冗余剥除同新条目）。
        if (entry.attributes !== undefined || entry.removeAttributes !== undefined) {
          return _zwSanMergeElEntry(cfg, existing, entry);
        }
        return false;
      }
      if (entry.attributes !== undefined) {
        entry.attributes = _zwSanStripMatches(entry.attributes, cfg.attributes, _zwSanSameAttr);
        if (!entry.attributes.length) delete entry.attributes;
      }
      if (entry.removeAttributes !== undefined) {
        entry.removeAttributes = _zwSanStripMatches(entry.removeAttributes, cfg.removeAttributes, _zwSanSameAttr);
        if (!entry.removeAttributes.length) delete entry.removeAttributes;
      }
      cfg.elements.push(entry);
      return true;
    }
    // removeElements 方向（含 [] marker）：present 即无变化。
    if (_zwSanListHas(cfg.removeElements, entry, _zwSanSameEl)) return false;
    cfg.removeElements.push({ name: entry.name, namespace: entry.namespace });
    return true;
  };
  // 已存条目合并（allow 方向 modifiers 二次调用面）：
  // attributes ∪= 新条目 attributes −(全局 ∪ 条目已允许)；removeAttributes 同向。
  function _zwSanMergeElEntry(cfg, existing, entry) {
    var changed = false;
    if (entry.attributes !== undefined) {
      var cur = existing.attributes ? existing.attributes.slice() : [];
      var pool = cfg.attributes.slice();
      for (var i = 0; i < cur.length; i++) pool.push(cur[i]);
      var add = _zwSanStripMatches(entry.attributes, pool, _zwSanSameAttr);
      for (var j = 0; j < add.length; j++) cur.push(add[j]);
      if (add.length) changed = true;
      if (cur.length) existing.attributes = cur;
      else if (existing.attributes !== undefined) delete existing.attributes;
    }
    if (entry.removeAttributes !== undefined) {
      var curR = existing.removeAttributes ? existing.removeAttributes.slice() : [];
      var poolR = cfg.removeAttributes.slice();
      for (var k = 0; k < curR.length; k++) poolR.push(curR[k]);
      var addR = _zwSanStripMatches(entry.removeAttributes, poolR, _zwSanSameAttr);
      for (var l = 0; l < addR.length; l++) curR.push(addR[l]);
      if (addR.length) changed = true;
      if (curR.length) existing.removeAttributes = curR;
      else if (existing.removeAttributes !== undefined) delete existing.removeAttributes;
    }
    return changed;
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
    } else {
      if (_zwSanListHas(cfg.removeAttributes, attr, _zwSanSameAttr)) return false;
      cfg.removeAttributes.push({ name: attr.name, namespace: attr.namespace });
    }
    _zwSanStripAttrFromElements(cfg, attr);
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
    if (_zwSanStripAttrFromElements(cfg, attr)) changed = true;
    if (cfg.elements) {
      for (var k = 0; k < cfg.elements.length; k++) {
        var ent = cfg.elements[k];
        if (ent.removeAttributes) {
          var after = _zwSanStripMatches(ent.removeAttributes, [attr], _zwSanSameAttr);
          if (after.length !== ent.removeAttributes.length) changed = true;
          ent.removeAttributes = after;
          if (!after.length) delete ent.removeAttributes;
        }
      }
    }
    return changed;
  };
  function _zwSanStripAttrFromElements(cfg, attr) {
    if (!cfg.elements) return false;
    var changed = false;
    for (var i = 0; i < cfg.elements.length; i++) {
      var ent = cfg.elements[i];
      if (ent.attributes) {
        var after = _zwSanStripMatches(ent.attributes, [attr], _zwSanSameAttr);
        if (after.length !== ent.attributes.length) changed = true;
        ent.attributes = after;
        if (!after.length) delete ent.attributes;
      }
    }
    return changed;
  }
  // PI modifiers：allow 方向追加/剔除；remove 方向对称。
  Sanitizer.prototype.allowProcessingInstruction = function (pi) {
    var cfg = this.__zwSanCfg;
    var item = _zwSanCanonPi(pi);
    if (cfg.processingInstructions) {
      if (_zwSanListHas(cfg.processingInstructions, item, _zwSanSamePi)) return false;
      cfg.processingInstructions.push({ target: item.target });
      return true;
    }
    if (_zwSanListHas(cfg.removeProcessingInstructions, item, _zwSanSamePi)) return false;
    cfg.removeProcessingInstructions.push({ target: item.target });
    return true;
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
          cfg.removeElements.push({ name: b.name, namespace: b.ns });
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
    // 5. 内建 animating 面：SVG animate + attributeName href/xlink:href → 整删
    //（#sanitizer-api——语料 sanitizer-javascript-url.html animating 组）。
    if (ns === _zwSanNsSvg && name === 'animate' && _zwSanAnimHrefTarget(n)) return { v: 'remove', entry: null };
    return { v: 'keep', entry: entry };
  }
  // animate 的 attributeName 读面：HTML 解析属性名小写化 + SVG 保留原 case 双查。
  function _zwSanAnimHrefTarget(n) {
    var v = null;
    try { v = n.getAttribute('attributeName'); } catch (e) { v = null; }
    if (v === null || v === undefined) {
      try { v = n.getAttribute('attributename'); } catch (e2) { v = null; }
    }
    if (v === null || v === undefined) return false;
    var s = _zwSanAsciiLower(String(v)).replace(/^[\s]+|[\s]+$/g, '');
    return s === 'href' || s === 'xlink:href';
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
      // 允许列表含 data-* 在构造期已 TypeError，不存在命中面）。
      if (local.lastIndexOf('data-', 0) === 0 && (!ans || ans === _zwSanNsXhtml)) {
        if (cfg.dataAttributes !== true) _zwSanDropAttr(n, qname);
        continue;
      }
      // 2. safe 基线：事件处理器（ns null、真实清单——on* 非前缀匹配）。
      if (safe && ans === null && _zwSanEventHandlers[local]) { _zwSanDropAttr(n, qname); continue; }
      // 3. 内建 javascript: 值剥离（navigating 面：href/action/formaction 任意元素）。
      if ((local === 'href' || local === 'action' || local === 'formaction') &&
          _zwSanIsJsUrl(attr.value)) { _zwSanDropAttr(n, qname); continue; }
      // 4. animating 面：animateMotion/animateTransform 的 attributeName 剥除
      //（animate 整删在元素 verdict 层）。
      if (local === 'attributename' && _zwSanElNs(n) === _zwSanNsSvg) {
        var tg = _zwSanAsciiLower(_zwSanElName(n));
        var v = _zwSanAsciiLower(String(attr.value == null ? '' : attr.value)).replace(/^[\s]+|[\s]+$/g, '');
        if ((tg === 'animatemotion' || tg === 'animatetransform') &&
            (v === 'href' || v === 'xlink:href')) { _zwSanDropAttr(n, qname); continue; }
      }
      // 5. 用户配置方向。
      if (cfg.attributes !== undefined) {
        var ok = _zwSanListHas(cfg.attributes, { name: local, namespace: ans }, _zwSanSameAttr);
        if (!ok && elEntry && elEntry.attributes) {
          ok = _zwSanListHas(elEntry.attributes, { name: local, namespace: ans }, _zwSanSameAttr);
        }
        if (!ok) { _zwSanDropAttr(n, qname); }
      } else {
        var bad = cfg.removeAttributes !== undefined && cfg.removeAttributes.length &&
          _zwSanListHas(cfg.removeAttributes, { name: local, namespace: ans }, _zwSanSameAttr);
        if (bad) _zwSanDropAttr(n, qname);
      }
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
    if (s === undefined || s === null) {
      return safe ? _zwSanGetDefaultInstance() : null;
    }
    if (s instanceof Sanitizer) return s;
    return new Sanitizer(s);
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
    var legacy = globalThis.Document && globalThis.Document.parseHTMLUnsafe;
    if (typeof legacy !== 'function' || !san) {
      // 无消毒面（unsafe 无 options）或机器缺失：直通既有实现。
      return typeof legacy === 'function' ? legacy(html) : null;
    }
    var added = (_zwSanBridge && _zwSanBridge.buildBodyTree)
      ? _zwSanBridge.buildBodyTree(String(html), '', undefined, true).childNodes : [];
    if (added.length) _zwSanFilterList(added, san, safe);
    var clean = '';
    for (var i = 0; i < added.length; i++) {
      if (added[i]) clean += _zwSanBridge.serialize(added[i]);
    }
    return legacy(clean);
  }

  // ===== 全局注册（window===globalThis 单点——part06 同款双访问形态覆盖）=====
  Object.defineProperty(globalThis, 'Sanitizer', {
    value: Sanitizer, writable: true, enumerable: false, configurable: true
  });
  // late-bound 接线（part04 trap / part03 ShadowRoot 原型消费）。
  globalThis.__zwSanSetHtml = _zwSanSetHtmlImpl;
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
          var s = (options !== null && typeof options === 'object') ? options.sanitizer : undefined;
          if (s === undefined || s === null) return _zwSanLegacyUnsafe(html);
          return _zwSanParseHtmlImpl(html, options, false);
        };
        try { globalThis.Document.__zwSanUnsafeWrapped = true; } catch (_eWrap) {}
      }
    }
  } catch (_eSanDoc) {}
})();
