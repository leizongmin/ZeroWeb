// part14（2026-11-23 wptfyi-dom DC-2 切片 5 刀③a）：Trusted Types sink
// enforcement 基座 + 属性 sink 判定 + script sink 包装。
//
// 目标面：trusted-types 语料 enforcement 域（DC-2 切片 5 刀② 后余 1,012F
// 的主域）——require-trusted-types-for 'script' 激活后，trusted type sink
// 的字符串赋值走 default policy 管道、无政策时抛 TypeError。刀③a 覆盖：
//   - enforcement 状态（页内 meta CSP 惰性扫描——wpt 语料 144/225 页
//     meta 形态；.headers sidecar 形态的 17 页 runner 无 CSP 头通道，
//     挂账 runner 预处理微刀）
//   - 属性 sink 判定表（setAttribute/setAttributeNS 挂钩在 part04）：
//     事件处理器内容属性 → TrustedScript、iframe srcdoc → TrustedHTML、
//     HTML script src → TrustedScriptURL、SVG script href/xlink:href →
//     TrustedScriptURL（spec
//     https://w3c.github.io/webappsec-trusted-types/dist/spec/
//     #abstract-opdef-get-trusted-type-data-for-attribute ）
//
// 划界（3b）：script sink 包装（eval/Function/setTimeout/setInterval）——
// 安装面已回退（lib 归因/可见性回归，见 §4 FIXME），需 native 伪装 +
// 归因保持重设计。
//
// 划界（3b 后续）：report-only violation 事件面
//（SecurityPolicyViolationEvent）。③b① 已落：setAttributeNode/setNamedItem/
// Attr.value/Node.nodeValue 通道；③b② 已落（钩点在消费面 part02/03/04/05/06）：
// HTML sink 族 innerHTML/ShadowRoot.innerHTML/outerHTML/insertAdjacentHTML/
// setHTMLUnsafe/Document.parseHTMLUnsafe/DOMParser.parseFromString/
// Range.createContextualFragment/document.write|writeln 逐参/iframe srcdoc 与
// script src 属性反射/script.text|innerText|textContent（TrustedScript，HTML
// script ns 门）。
//
// 形制：链尾 IIFE + globalThis 契约（part12/13 同制——外层 shim IIFE 锁
// 顶层函数，链尾件只走 globalThis）。事件处理器名单复用 part13
// _zwSanEventHandlers（globalThis.__zwSanEventHandlers 导出）。
(function () {
  'use strict';
  if (globalThis.__zwTtSinkValue) return; // 幂等（shim 重装配边界）

  var HTML_NS = 'http://www.w3.org/1999/xhtml';
  var SVG_NS = 'http://www.w3.org/2000/svg';
  var MATHML_NS = 'http://www.w3.org/1998/Math/MathML';
  var XLINK_NS = 'http://www.w3.org/1999/xlink';

  // ===== 1. enforcement 状态（惰性 + 缓存）=====
  // meta CSP 只能来自文档头部（spec meta http-equiv 处理约束）；shim 装配
  // 早于页面 script，meta 节点在装配时刻不保证入 DOM——首次 sink 调用时
  // 扫描并缓存（sink 调用必发生在 script 执行中，meta 已在 DOM）。
  var _enforcedCache = null;
  var _zwTtEnforced = function () {
    if (_enforcedCache !== null) return _enforcedCache;
    var enforced = false;
    try {
      var metas = globalThis.document.getElementsByTagName('meta');
      for (var i = 0; i < metas.length; i++) {
        var he = '';
        try { he = String(metas[i].getAttribute('http-equiv') || '').toLowerCase(); } catch (_e1) {}
        if (he !== 'content-security-policy') continue;
        var content = '';
        try { content = String(metas[i].getAttribute('content') || ''); } catch (_e2) {}
        // Report-Only 不 enforce；多指令按子串判（require-trusted-types-for
        // 与 'script' 同现——语料形态 `require-trusted-types-for 'script';`）。
        // FIXME(3b): 按指令/值分词解析（分号敏感形态与引号变体）。
        if (content.indexOf('require-trusted-types-for') >= 0
            && content.indexOf("'script'") >= 0) {
          enforced = true;
          break;
        }
      }
    } catch (_e0) { enforced = false; }
    _enforcedCache = enforced;
    return enforced;
  };

  // ===== 2. sink 管道核心 =====
  // spec sink processing（#create-a-trusted-type-data-value-adjacent 簇）：
  // TT 实例 → 数据串直接用；其余 String 化 → default policy 存在则走其
  // createX（回调抛错或返 undefined → TypeError；返 null → 空串）；无
  // default policy → TypeError。
  //
  // 刀③b：单次透传旗标——Attr.value/nodeValue/textContent setter 已在本 setter
  // 内过管道，写回传播经 oe.setAttribute 时会二次入管道（default policy 二次
  // 调用——语料 default-policy 页 policy 回调 assert seenSinkName 二现崩）。
  // setter 侧管道后置位、传播臂消费一次；窗口为同一同步流，消费前异常路径
  // （管道抛 TypeError）不置位，无跨调用残留。
  var _bypassNext = false;
  var _zwTtBypassNextSink = function () { _bypassNext = true; };
  var _zwTtSinkValue = function (type, value, sinkName) {
    if (_bypassNext) {
      _bypassNext = false;
      return typeof value === 'string' ? value : String(value);
    }
    if (!_zwTtEnforced()) {
      return typeof value === 'string' ? value : String(value);
    }
    var tt = globalThis.trustedTypes;
    if (tt && typeof tt.isHTML === 'function') {
      var isTT = (type === 'TrustedHTML' && tt.isHTML(value))
        || (type === 'TrustedScript' && tt.isScript(value))
        || (type === 'TrustedScriptURL' && tt.isScriptURL(value));
      if (isTT) return String(value);
    }
    var s = typeof value === 'string' ? value : String(value);
    var def = (tt && tt.defaultPolicy) || null;
    if (!def) {
      throw new TypeError(
        "Failed to execute '" + sinkName + "' on 'Element': This document requires '" + type + "' assignment.");
    }
    var fn = type === 'TrustedHTML' ? def.createHTML
      : type === 'TrustedScript' ? def.createScript
        : def.createScriptURL;
    if (typeof fn !== 'function') {
      // default policy 缺对应 create 回调：spec 按未提供处理——与 createPolicy
      // 直调同款（返 undefined → TypeError）。
      throw new TypeError(
        "Failed to execute '" + sinkName + "' on 'Element': This document requires '" + type + "' assignment.");
    }
    var out;
    try {
      // 刀③b：default policy 回调参数序按 spec process-value-with-default-policy
      //（input, type, sink）——语料 default-policy 页 createTrustedType 按
      // (input, trustedTypeName, sinkName) 断言。
      out = fn.call(def, s, type, sinkName);
    } catch (_eP) {
      throw new TypeError(
        "Failed to execute '" + sinkName + "' on 'Element': This document requires '" + type + "' assignment.");
    }
    if (out === undefined) {
      throw new TypeError(
        "Failed to execute '" + sinkName + "' on 'Element': This document requires '" + type + "' assignment.");
    }
    if (out === null) return '';
    return String(out);
  };

  // ===== 3. 属性 sink 判定表 =====
  // 语料 ground truth（support/attributes.js trustedTypeDataForAttribute）：
  //   on<known-handler> @ HTML/SVG/MathML 元素, attrNS=null → TrustedScript
  //   （未知事件名 ondoesnotexist 非 sink；attrNS 非 null 非 sink）
  //   iframe@HTML srcdoc, attrNS=null → TrustedHTML
  //   script@HTML src, attrNS=null → TrustedScriptURL
  //   script@SVG href, attrNS∈{null,xlink} → TrustedScriptURL
  //   其余 → null（无 enforcement 影响）
  var _zwTtAttrSinkData = function (elem, attrNS, name) {
    if (!_zwTtEnforced()) return null;
    var n = String(name);
    var ns = attrNS == null ? null : String(attrNS);
    var elNs = null;
    var elTag = '';
    try {
      elNs = elem.namespaceURI == null ? null : String(elem.namespaceURI);
      elTag = String(elem.tagName || '').toLowerCase();
    } catch (_e3) { return null; }
    var local = n;
    var ci = n.indexOf(':');
    if (ci > 0) local = n.slice(ci + 1);
    // a. 事件处理器内容属性（名单精确匹配——part13 表；前缀 on* 不判，
    //    语料负例 ondoesnotexist 实证）。sink 名恒 'Element ' + local
    //    （语料 default-policy 页实测：svg g ondblclick → "Element ondblclick"）。
    if (ns === null && local.charAt(0) === 'o' && local.charAt(1) === 'n') {
      var list = globalThis.__zwSanEventHandlers;
      if (list && list[local] === true
          && (elNs === HTML_NS || elNs === SVG_NS || elNs === MATHML_NS)) {
        return { type: 'TrustedScript', sink: 'Element ' + local };
      }
      return null;
    }
    // b. iframe srcdoc → TrustedHTML（sink 'HTMLIFrameElement srcdoc'）
    if (elNs === HTML_NS && elTag === 'iframe' && ns === null && local === 'srcdoc') {
      return { type: 'TrustedHTML', sink: 'HTMLIFrameElement srcdoc' };
    }
    // c. HTML script src → TrustedScriptURL（sink 'HTMLScriptElement src'）
    if (elNs === HTML_NS && elTag === 'script' && ns === null && local === 'src') {
      return { type: 'TrustedScriptURL', sink: 'HTMLScriptElement src' };
    }
    // d. SVG script href / xlink:href → TrustedScriptURL（sink 'SVGScriptElement href'）
    if (elNs === SVG_NS && elTag === 'script' && local === 'href'
        && (ns === null || ns === XLINK_NS)) {
      return { type: 'TrustedScriptURL', sink: 'SVGScriptElement href' };
    }
    return null;
  };

  var _zwTtAttrSinkType = function (elem, attrNS, name) {
    var d = _zwTtAttrSinkData(elem, attrNS, name);
    return d ? d.type : null;
  };

  // ===== 4. script sink 包装（eval / Function / setTimeout / setInterval）=====
  // FIXME(3b)：安装面回退（2026-11-23）——恒装 wrap 致 zero-engine lib 11 个
  // 新失败（test_worker_importscripts ×5 / iframe_docelement r214/r216 /
  // iframe_testnodes r209 / p7_import_helper_runtime_call_attribution /
  // surround r210/r212 / worker_toplevel_function_reachable_from_string_timer），
  // 全部落 wrap 语义链：① 内建 eval/Function 被包装后**错误归因位移**
  // （p7 断言 filename=helper.js lineno=2 → wrap 后 helper-main.js lineno=0
  // ——间接 eval 的编译归因取调用点，wrap 函数体非页面脚本语境）；②
  // setup 字符串定义的全局可见性（r216 "setup:paras is not defined"）；③
  // worker string timer 编译单元。重设计需 native 伪装（toString）+
  // 归因保持（QuickJS eval 内建不可换名的约束下可能需引擎宿主桥臂），
  // 不在本刀强推。
  var _wrapScriptSink = function (orig, argIdx) {
    return function () {
      var args = Array.prototype.slice.call(arguments);
      if (args.length > argIdx) {
        var a = args[argIdx];
        if (typeof a === 'string' || (a && typeof a === 'object')) {
          // 非 TT 对象（函数/节点）不进管道；字符串与 TT 实例进。
          var isTT = false;
          var tt = globalThis.trustedTypes;
          if (tt && typeof tt.isScript === 'function' && a && typeof a === 'object') {
            isTT = tt.isScript(a);
          }
          if (typeof a === 'string' || isTT) {
            args[argIdx] = String(_zwTtSinkValue('TrustedScript', a, 'TrustedScript sink'));
          }
        }
      }
      return orig.apply(globalThis, args);
    };
  };

  var _installScriptSinks = function () {
    try {
      if (typeof globalThis.eval === 'function') {
        var origEval = globalThis.eval;
        var wrapped = function (x) {
          if (typeof x === 'string') {
            x = String(_zwTtSinkValue('TrustedScript', x, 'eval'));
          } else if (x && typeof x === 'object' && globalThis.trustedTypes
              && typeof globalThis.trustedTypes.isScript === 'function'
              && globalThis.trustedTypes.isScript(x)) {
            x = String(x);
          }
          return origEval(x);
        };
        try { Object.defineProperty(globalThis, 'eval', { value: wrapped, writable: true, configurable: true }); } catch (_e4) {}
      }
      if (typeof globalThis.Function === 'function') {
        var OrigFunction = globalThis.Function;
        var WrappedFunction = function () {
          var args = Array.prototype.slice.call(arguments);
          if (args.length > 0) {
            var body = args[args.length - 1];
            if (typeof body === 'string' || (body && typeof body === 'object'
                && globalThis.trustedTypes && typeof globalThis.trustedTypes.isScript === 'function'
                && globalThis.trustedTypes.isScript(body))) {
              args[args.length - 1] = String(_zwTtSinkValue('TrustedScript', body, 'Function'));
            }
          }
          return OrigFunction.apply(globalThis, args);
        };
        WrappedFunction.prototype = OrigFunction.prototype;
        try { Object.defineProperty(WrappedFunction, 'name', { value: 'Function', configurable: true }); } catch (_e5) {}
        try { Object.defineProperty(globalThis, 'Function', { value: WrappedFunction, writable: true, configurable: true }); } catch (_e6) {}
      }
      var timers = ['setTimeout', 'setInterval'];
      for (var t = 0; t < timers.length; t++) {
        var nm = timers[t];
        if (typeof globalThis[nm] === 'function') {
          var origT = globalThis[nm];
          try {
            Object.defineProperty(globalThis, nm, {
              value: _wrapScriptSink(origT, 0), writable: true, configurable: true
            });
          } catch (_e7) {}
        }
      }
    } catch (_eS) { /* sink 包装失败保持原行为 */ }
  };
  // FIXME(3b)：`_installScriptSinks()` 暂不安装（lib 回归见上注）——保留
  // 函数定义供 3b 重设计复用。


  // ===== 5. 导出（globalThis 契约——part04 trap / part05 core / part03 Attr setter 消费）=====
  Object.defineProperty(globalThis, '__zwTtEnforced', { value: _zwTtEnforced, writable: true, configurable: true });
  Object.defineProperty(globalThis, '__zwTtSinkValue', { value: _zwTtSinkValue, writable: true, configurable: true });
  Object.defineProperty(globalThis, '__zwTtAttrSinkData', { value: _zwTtAttrSinkData, writable: true, configurable: true });
  Object.defineProperty(globalThis, '__zwTtAttrSinkType', { value: _zwTtAttrSinkType, writable: true, configurable: true });
  Object.defineProperty(globalThis, '__zwTtBypassNextSink', { value: _zwTtBypassNextSink, writable: true, configurable: true });
})();
