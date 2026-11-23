// part14（2026-11-23 wptfyi-dom DC-2 切片 5 刀③a）：Trusted Types sink
// enforcement 基座 + 属性 sink 判定 + script sink 包装。
//
// 目标面：trusted-types 语料 enforcement 域（DC-2 切片 5 刀② 后余 1,012F
// 的主域）——require-trusted-types-for 'script' 激活后，trusted type sink
// 的字符串赋值走 default policy 管道、无政策时抛 TypeError。刀③a 覆盖：
//   - enforcement 状态（页内 meta CSP 惰性扫描——wpt 语料 144/225 页
//     meta 形态；.headers sidecar 形态 17 页由刀③④ runner 预处理注入
//     等效 meta，同一扫描面）
//   - 属性 sink 判定表（setAttribute/setAttributeNS 挂钩在 part04）：
//     事件处理器内容属性 → TrustedScript、iframe srcdoc → TrustedHTML、
//     HTML script src → TrustedScriptURL、SVG script href/xlink:href →
//     TrustedScriptURL（spec
//     https://w3c.github.io/webappsec-trusted-types/dist/spec/
//     #abstract-opdef-get-trusted-type-data-for-attribute ）
// 刀③⑤ 覆盖：trusted-types 名单指令门（§1 指令分词解析 + part12
// createPolicy 名单/重名阻断）——'*' 优先、'none' 大小写不敏感、无效
// 表达式全阻断、'allow-duplicates' 语义；iframe/本地 scheme 跨 realm
// CSP 继承域（srcdoc require 臂、data/blob/about:blank 组）仍挂账
// （影子 realm 面，随 worker 通道刀位）。
//
// 划界（③③b 已落）：Function/派生构造器编译门经宿主跳板安装（§4，宿主
// 函数不推 JS 帧——③③a 回退的 lib 归因/可见性回归不复现）；③③a 已落
// setTimeout/setInterval 入口管道（part01 shim 内部门）。eval 域整体挂账：
// 直接 eval 是编译器特判拦不到，替换 globalThis.eval 会破坏所有页面直接
// eval 的局部作用域语义（dom 语料 Range 家族等 18 文件实证），间接 eval
// 单独拦截不可行——随引擎宿主桥臂独立评估。
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

  // ===== 1. enforcement 状态 + trusted-types 名单指令（惰性 + 缓存）=====
  // meta CSP 只能来自文档头部（spec meta http-equiv 处理约束）；shim 装配
  // 早于页面 script，meta 节点在装配时刻不保证入 DOM——首次 sink/创建政策
  // 调用时扫描并缓存（调用必发生在 script 执行中，meta 已在 DOM）。刀③④ 起
  // sidecar 头形态经 runner 注入等效 meta，同一扫描面。
  // 刀③⑤：子串判据升级指令分词（part12 createPolicy 名单/重名门消费）——
  //   enforced 位（require-trusted-types-for 'script'，任一政策命中）+
  //   trusted-types 名单（spec
  //   https://w3c.github.io/webappsec-trusted-types/dist/spec/
  //   #should-block-create-policy ）：多政策任一 block 即 block（无名单
  //   政策不 block）；政策内 '*' 优先（'none'/名单项被忽略——
  //   cspTests-none-skip ground truth）；'none' 大小写不敏感＝空名单
  //   （cspTests-case2 'nONe'）；`*X` 类无效表达式＝无有效 tt-expression，
  //   通配不启用、全阻断（wildcard-with-trailing-characters ground
  //   truth）；'allow-duplicates' 缺席时名单域重名阻断
  //  （duplicate-names-list ground truth）。
  var _cspCache = null;
  var _zwTtParseCsp = function () {
    if (_cspCache !== null) return _cspCache;
    var state = { enforced: false, hasList: false, lists: [], allowDuplicates: false };
    try {
      var metas = globalThis.document.getElementsByTagName('meta');
      for (var i = 0; i < metas.length; i++) {
        var he = '';
        try { he = String(metas[i].getAttribute('http-equiv') || '').toLowerCase(); } catch (_e1) {}
        if (he !== 'content-security-policy') continue;
        var content = '';
        try { content = String(metas[i].getAttribute('content') || ''); } catch (_e2) {}
        // 单 meta ＝ 单政策：分号分指令，政策内收集名单语义。
        var curList = null, curStar = false, curNone = false, curBad = false;
        var directives = content.split(';');
        for (var d = 0; d < directives.length; d++) {
          var directive = directives[d].trim();
          if (directive === '') continue;
          var sp = directive.indexOf(' ');
          var dname = (sp < 0 ? directive : directive.slice(0, sp)).toLowerCase();
          var dvalue = sp < 0 ? '' : directive.slice(sp + 1).trim();
          if (dname === 'require-trusted-types-for') {
            if (dvalue.toLowerCase().indexOf("'script'") >= 0) state.enforced = true;
          } else if (dname === 'trusted-types') {
            state.hasList = true;
            if (curList === null) curList = [];
            var tokens = dvalue.split(/\s+/);
            for (var t = 0; t < tokens.length; t++) {
              var tok = tokens[t];
              if (tok === '') continue;
              var low = tok.toLowerCase();
              if (low === "'allow-duplicates'") { state.allowDuplicates = true; continue; }
              if (tok === '*') { curStar = true; continue; }
              if (low === "'none'") { curNone = true; continue; }
              if (tok.charAt(0) === '*' && tok.length > 1) { curBad = true; continue; }
              if (curStar) continue; // '*' 优先：名单项被忽略
              curList.push(tok);
            }
          }
        }
        if (curList !== null) {
          if (curStar) {
            state.lists.push('*'); // 政策语义：不 block 任何名字
          } else if (curBad) {
            state.lists.push(''); // 无效表达式（'*X'）：无有效 tt-expression，block 一切
          } else if (curNone && curList.length === 0) {
            // 'none' 独占（大小写不敏感）＝空名单政策：block 一切
            state.lists.push('');
          } else {
            // 'none' 与其他表达式并存：'none' 被忽略（spec 名单语义，
            // cspTests-none-none-name ground truth：'none' 'none' SomeName
            // → 名单 [SomeName]）
            state.lists.push(curList.join(' '));
          }
        }
      }
    } catch (_e0) { /* meta 读取失败按无政策处理 */ }
    _cspCache = state;
    return state;
  };
  var _zwTtEnforced = function () {
    return _zwTtParseCsp().enforced;
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

  // ===== 4. Function 构造器编译门（刀③③b）=====
  // spec：EnsureCSPDoesNotBlockStringCompilation（#ensure-csp-does-not-block-string-
  // compilation）+ GetTrustedTypeCompliantString——isTrusted（全部实参为
  // TrustedScript；空参恒真）→ 原生编译；否则装配文本过 default policy：
  // 无政策 / 回调抛错 / 返 undefined → EvalError；输出 ≠ 装配文本 → EvalError
  // （default policy 不可改写编译串——语料 mutate / Window-block / applying 页
  // ground truth：X→Y 形输出 = 输入则照常编译，恒变形输出 ≠ 输入则 EvalError）；
  // 输出 === 装配文本 → 以原参数向量照常编译。编译域错误恒 EvalError（属性/
  // HTML sink 域的 TypeError 管道不适用本域）；四构造器 sink 名恒 'Function'
  //（support/block-eval-function-constructor.js 断言 ground truth）。
  //
  // 机制（③③a 回退教训吸收）：全局 wrap 必须是宿主函数（rquickjs RustFunction
  // 不推 JS 帧）保编译归因——JS 包装帧使 new Function 产物归因位移（p7
  // helper-main.js:0 型 lib 11F 不复现）。可构造位经 script-sandbox 宿主工厂
  // `__zwTtMakeCompileTrampoline`（with_constructor）；orig/gate/sink 存 JS
  // 注册表 `__zwTtTrampReg[idx]`（rquickjs 0.7 无持久句柄，宿主闭包不持 JS
  // 值，跳板逐调用经 ctx 取）。shim 内部运行期编译站点走 part01 顶部
  // `__zwNativeFunction` 原生引用快照（脚本本体照常执行；s90 定时器串已在
  // setTimeout/setInterval 入口过 ③③a 管道，不再二次进门）。
  //
  // 划界：直接 eval（`eval(x)` 标识符直调）是编译器特判拦不到；替换
  // globalThis.eval 会破坏所有页面直接 eval 的局部作用域语义——eval 域整体
  // 挂账（见头部注记）。
  var _zwTtCompileGate = function (orig, args, sinkName) {
    var enforced = _zwTtEnforced();
    var n = args.length;
    // 非 enforced：原参透传（TT 实例走原生 ToString 链、空参恒真，零变化）。
    if (!enforced) return orig.apply(globalThis, args);
    var tt = globalThis.trustedTypes;
    var strs = new Array(n);
    var anyPlain = false;
    for (var gi = 0; gi < n; gi++) {
      var a = args[gi];
      var isTT = false;
      try {
        isTT = a !== null && a !== undefined && typeof a === 'object'
          && !!tt && typeof tt.isScript === 'function' && tt.isScript(a);
      } catch (_eG1) { isTT = false; }
      if (isTT) {
        // 内部数据直读（不经 toString——forged-toString 断言要求零调用；
        // __zwTtData 为 part12 工厂挂的非枚举 own 属性，forge 不可及）。
        var d = null;
        try { d = a.__zwTtData; } catch (_eG2) { d = null; }
        strs[gi] = d == null ? String(a) : String(d);
      } else {
        anyPlain = true;
        strs[gi] = typeof a === 'string' ? a : String(a);
      }
    }
    if (!anyPlain) return orig.apply(globalThis, strs);
    // 装配文本恒全形（V8 形制）：`function anonymous(<params>\n) {\n<body>\n}`——
    // 语料 default-policy-callback-arguments 页 ground truth：1 参（仅 body）时
    // policy 收到的也是全形 `function anonymous(\n) {\nreturn 2+2\n}`。
    var text;
    if (n === 0) {
      // new Function()：无 body 臂，空形装配（无 corpus 断言，同形补全）。
      text = 'function anonymous(\n) {\n\n}';
    } else {
      text = 'function anonymous(' + strs.slice(0, n - 1).join(',') + '\n) {\n' + strs[n - 1] + '\n}';
    }
    var def = (tt && tt.defaultPolicy) || null;
    var fn = def && def.createScript;
    if (typeof fn !== 'function') {
      throw new EvalError('Failed to compile a string: This document requires a TrustedScript for compilation.');
    }
    var out;
    try {
      out = fn.call(def, text, 'TrustedScript', sinkName);
    } catch (_eP) {
      throw new EvalError('Failed to compile a string: default policy callback threw.');
    }
    if (out === undefined) {
      throw new EvalError('Failed to compile a string: default policy returned undefined.');
    }
    var s = out === null ? '' : String(out);
    if (s !== text) {
      // default policy 不可改写编译串（输出 ≠ 装配文本 → EvalError）。
      throw new EvalError('Failed to compile a string: default policy modified the value.');
    }
    return orig.apply(globalThis, strs);
  };

  // 跳板注册（每构造器一项；idx 键控——四构造器共享 sink 名 'Function' 不能
  // 以 sink 为键）。返回 null = 宿主工厂缺位（V8 侧），安装臂整刀跳过。
  var _zwTtTrampN = 0;
  var _zwTtWrapCtor = function (orig, gate, sinkName) {
    var mk = globalThis.__zwTtMakeCompileTrampoline;
    if (typeof mk !== 'function') return null;
    var idx = String(_zwTtTrampN++);
    var reg = globalThis.__zwTtTrampReg;
    if (!reg) {
      reg = {};
      Object.defineProperty(globalThis, '__zwTtTrampReg',
        { value: reg, writable: true, configurable: true });
    }
    reg[idx] = { orig: orig, gate: gate, sink: sinkName };
    return mk(idx);
  };

  var _installCompileSinks = function () {
    if (typeof globalThis.__zwTtMakeCompileTrampoline !== 'function') return;
    try {
      var wrap = function (orig) {
        var g = _zwTtWrapCtor(orig, _zwTtCompileGate, 'Function');
        if (!g) return null;
        try { Object.defineProperty(g, 'name', { value: orig.name, configurable: true }); } catch (_eN) {}
        try { Object.defineProperty(g, 'length', { value: orig.length, configurable: true }); } catch (_eL) {}
        try {
          // f instanceof Function 读 wrapper.prototype；实例 __proto__ 恒
          // orig.prototype——置同值保持 instanceof/静态面一致（guidelines #32）。
          Object.defineProperty(g, 'prototype', {
            value: orig.prototype, writable: false, enumerable: false, configurable: false
          });
        } catch (_ePr) {}
        return g;
      };
      var origF = globalThis.Function;
      var gatedF = wrap(origF);
      if (gatedF) {
        // 属性特性保持既有 descriptor（writable/configurable 真、enumerable 假）。
        Object.defineProperty(globalThis, 'Function', { value: gatedF });
        try {
          Object.defineProperty(origF.prototype, 'constructor',
            { value: gatedF, writable: true, configurable: true });
        } catch (_eC0) {}
      }
      // 派生构造器：%X.prototype%.constructor 重定义（可写可配真、enumerable 假，
      // 与内建 descriptor 一致）；identity 经同属性读闭合（page 取 .constructor
      // 得 wrapper，wrapper.prototype === %X.prototype%）。
      var protos = [
        Object.getPrototypeOf(async function () {}),
        Object.getPrototypeOf(function* () {}),
        Object.getPrototypeOf(async function* () {})
      ];
      for (var pi = 0; pi < protos.length; pi++) {
        var origD = protos[pi] && protos[pi].constructor;
        if (typeof origD !== 'function' || origD === gatedF) continue;
        var gD = wrap(origD);
        if (!gD) continue;
        try {
          Object.defineProperty(protos[pi], 'constructor',
            { value: gD, writable: true, configurable: true });
        } catch (_eCD) {}
      }
    } catch (_eIS) { /* 安装失败保持原行为 */ }
  };
  _installCompileSinks();


  // ===== 5. 导出（globalThis 契约——part04 trap / part05 core / part03 Attr setter 消费）=====
  Object.defineProperty(globalThis, '__zwTtEnforced', { value: _zwTtEnforced, writable: true, configurable: true });
  // 刀③⑤：part12 createPolicy 名单/重名门消费面——解析态与逐名判定。
  // spec #should-block-create-policy：任一名单政策 block 即 block；
  // '*' 政策不 block；''（'none'/'*X' 无效表达式）政策 block 一切；
  // 其余政策 block 名单外名字（'default' 须显式入名单）。
  Object.defineProperty(globalThis, '__zwTtCspState', {
    value: function () { return _zwTtParseCsp(); }, writable: true, configurable: true
  });
  Object.defineProperty(globalThis, '__zwTtPolicyAllowed', {
    value: function (name) {
      var st = _zwTtParseCsp();
      if (!st.hasList) return true;
      for (var i = 0; i < st.lists.length; i++) {
        var lst = st.lists[i];
        if (lst === '*') continue;
        var names = lst.split(' ');
        if (names.length === 1 && names[0] === '') return false;
        if (names.indexOf(String(name)) < 0) return false;
      }
      return true;
    }, writable: true, configurable: true
  });
  Object.defineProperty(globalThis, '__zwTtSinkValue', { value: _zwTtSinkValue, writable: true, configurable: true });
  Object.defineProperty(globalThis, '__zwTtAttrSinkData', { value: _zwTtAttrSinkData, writable: true, configurable: true });
  Object.defineProperty(globalThis, '__zwTtAttrSinkType', { value: _zwTtAttrSinkType, writable: true, configurable: true });
  Object.defineProperty(globalThis, '__zwTtBypassNextSink', { value: _zwTtBypassNextSink, writable: true, configurable: true });
})();
