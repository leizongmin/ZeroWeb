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
// script ns 门；刀⑲ R521 起后三者改由 HTMLScriptElement.prototype accessor
// 承载，见 §6——trap 内旧 TT 分支撤除，管道与 sink 名不变）。
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
  var _cspCacheFp = null;
  // 刀⑳b：CSP 政策闩锁——meta http-equiv 交付的政策在处理时收集进文档政策
  // 清单，移除 meta **元素**不回收已交付政策（script-enforcement-014/015
  // 「meta tag removed 后 script 仍 untrusted」）。k20 指纹重扫只对动态注入
  // 方向成立（012/013），移除方向会把 enforcement 误关为 P2F——k19 首调缓存
  // 恰被 lib 装载期 createPolicy（__zwTtCspState 扫描）质 poll 为 ON 才偶然
  // 绿。置位点在 _zwTtParseCsp 内：任一扫描路径（sink 名单门/执行门）见到
  // enforced 即单调置位；指纹重扫只为捕捉 OFF→ON 动态注入，ON 永不回退。
  // 近似边界：meta 在任何扫描前已被移除的页无法追溯收集（shim 装配早于
  // DOM 组装、解析期 meta 无扫描钩子），语料无此形态——按 lazy 扫描近似。
  // https://www.w3.org/TR/CSP3/#meta-element
  var _zwCspEnforcedLatch = false;
  var _zwTtParseCsp = function () {
    // 刀⑳：动态 meta CSP 指纹失效——script-enforcement-013「source was set
    // before TrustedTypes got enabled」face 在页面脚本执行中 insertAdjacentHTML
    // 注入 meta（真实浏览器 meta http-equiv 插入即处理）。原「首调即缓存」
    // 假设 meta 静态，动态注入的 enforcement 永不生效（k19 借 detached 抢置
    // 空转假绿——抢置时点 enforcement 尚 false 但 stale 文本恰无副作用；刀⑳
    // 抢置修复后 append 时点门真正跑通，谎报裸露为 P2F）。指纹＝全部 meta 的
    // http-equiv/content 拼接（属性读廉价，分词解析才是缓存重点）；指纹不变
    // 走缓存，静态页零行为变化。
    // https://html.spec.whatwg.org/multipage/semantics.html#meta-http-equiv-content-security-policy
    var fp = '';
    try {
      var metasFp = globalThis.document.getElementsByTagName('meta');
      var partsFp = [];
      for (var iFp = 0; iFp < metasFp.length; iFp++) {
        var heFp = '', coFp = '';
        try { heFp = String(metasFp[iFp].getAttribute('http-equiv') || ''); } catch (_eFp1) {}
        try { coFp = String(metasFp[iFp].getAttribute('content') || ''); } catch (_eFp2) {}
        partsFp.push(heFp + '\u0001' + coFp);
      }
      fp = partsFp.join('\u0002');
    } catch (_eFp3) { fp = ''; }
    if (_cspCache !== null && fp === _cspCacheFp) return _cspCache;
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
    _cspCacheFp = fp;
    // 刀⑳b：enforced 闩锁置位（见上方 _zwCspEnforcedLatch 注记）——
    // __zwTtCspState 直调路径（createPolicy 名单门）同样质 poll。
    if (state.enforced) _zwCspEnforcedLatch = true;
    return state;
  };
  var _zwTtEnforced = function () {
    if (!_zwCspEnforcedLatch) _zwCspEnforcedLatch = _zwTtParseCsp().enforced;
    return _zwCspEnforcedLatch;
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
    // 刀⑩b（dc2-s10k10）：回调异常原样传播（spec process-value-with-default-policy
    // rethrow 语义——语料 sanitizer-api sethtml-with-trustedtypes「exception in
    // createHTML prevents createParserOptions from running」12 sink 案
    // assert_throws_js(Error)——旧 TypeError 包装翻脸）。TT 主语料 throw 脸仅
    // createXYZTests 直调（part12 直调路径已原样）与 eval 门（自有 EvalError
    // 包装，不走本管道）——本改动零 TT 回归面。
    // https://w3c.github.io/webappsec-trusted-types/dist/spec/#process-value-with-default-policy
    var out;
    try {
      // 刀③b：default policy 回调参数序按 spec process-value-with-default-policy
      //（input, type, sink）——语料 default-policy 页 createTrustedType 按
      // (input, trustedTypeName, sinkName) 断言。
      out = fn.call(def, s, type, sinkName);
    } catch (_eP) {
      throw _eP;
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


  // ===== 6. script 身份门（刀⑲ R521：provenance 快照 + prepare 时救援 + 原型分层）=====
  // spec https://w3c.github.io/webappsec-trusted-types/dist/spec/#enforcement-in-scripts
  // ——require-trusted-types-for 'script' 页，「prepare the script element」时读
  // script text：可信 → 照常执行；不可信 → default policy createScript 救援
  // （sink 恒 "<Interface> text"——HTML/SVG 接口名分型，语料 002/004 双页实证；
  // 返回值**替换执行体**，DOM 树文本不变）→ 无政策/回调抛错/返 undefined → 阻断
  //（不执行、不置 run-once）。
  //
  // 可信判定＝**快照等值**（corpus 全矩阵反推的最小充分模型——trust 是身份属性
  // 不是值属性：deleteData 后与原文等值的 fused text 仍判不可信，002 deleteData
  // 案实证；splitText/normalize 后 fused 不变仍判可信，002/004 splitText+normalize
  // 案实证）。快照只在两处盖章：
  //   a) 声明 sink 写入成功后（HTMLScriptElement.prototype text/textContent/
  //      innerText accessor setter 尾——下方 §6.3 安装）；
  //   b) 解析投递时（part06 __zw_stamp_parser_scripts 对全部解析产物 script 枚举
  //      盖章——parser 创建的 text 可信；static type=unknown SVG script 摘 type 后
  //      经动态臂重 prepare 的 003/004 「initially trusted」判据面）。
  // 其余一切写面（nodeValue/data/appendData/insertData/replaceData/deleteData/
  // before/after/replaceWith/appendChild/insertBefore/replaceChild/removeChild/
  // prepend/append/replaceChildren/moveBefore/insertAdjacentText/Range 系/
  // cloneNode/cloneContents/innerHTML/setHTMLUnsafe 系）不盖章 → fused text 与
  // 快照失配即不可信；clone 产物无快照键（新元素实例）恒不可信。等值模型对
  // normalize（合并文本子、fused 不变）自动判可信——与语料一致。
  //
  // 键 = `__zwSelector`（解析期宿主身份，跨代理实例稳定）/`__zwHandle`（动态
  // 元素身份）双表——createElement 产物无 sel 只有 handle（001 声明 sink 全案），
  // 解析产物有 sel（003/004 static 面）。表项随元素数线性、生命周期页面级。
  //
  // gate 契约（part03 _zwScriptMaybeStartDynamic 内联臂消费，空体检查后、module
  // 臂前——prepare 时点）：返回 undefined＝可信/未强制（照常执行）；返回 string＝
  // 救援成功（执行返回串）；返回 null＝阻断（不执行不置记账）。
  if (!globalThis._zwTtScriptTrustSel) {
    Object.defineProperty(globalThis, '_zwTtScriptTrustSel',
      { value: {}, writable: true, configurable: true });
  }
  if (!globalThis._zwTtScriptTrustHdl) {
    Object.defineProperty(globalThis, '_zwTtScriptTrustHdl',
      { value: {}, writable: true, configurable: true });
  }
  // 6.0b plain 弱表——CCF 等工厂产物构建期无 sel/handle（R368 身份盖章在插入时
  // 点才发生），快照只能按**对象身份**寄存（WeakMap：插入后同一对象获得 sel/
  // handle，gate 先查 sel/hdl 表未中再落本表；唯一写入方＝part06 CCF 构建期
  // parser 投递盖章）。https://w3c.github.io/webappsec-trusted-types/dist/spec/
  // #abstract-opdef-should-sink-type-mismatch-create-policy 派生的脚本投递信任
  // 只覆盖 parser 投递产物——DOM 变异 API 新建脚本不进本表。
  if (!globalThis._zwTtScriptTrustPlain) {
    Object.defineProperty(globalThis, '_zwTtScriptTrustPlain',
      { value: new WeakMap(), writable: true, configurable: true });
  }
  // 6.1 fused text 读——与 part03 内联臂源读**同形**（el.text → 空白回落
  // el.textContent；spec script text IDL 与 textContent 同源融合）。盖章与 gate
  // 双侧共用本函数 → 等值判定逐字节同源。
  var _zwTtScriptFusedText = function (el) {
    try {
      var t = el.text;
      if (t != null && String(t).trim() !== '') return String(t);
    } catch (_eFt1) {}
    try { return String(el.textContent || ''); } catch (_eFt2) { return ''; }
  };
  // 6.2 盖章（声明 sink 尾 + 解析投递）。fused 读数即快照——任何后续文本变更
  //（无论经哪条写面）自动失配。
  var _zwTtScriptTrustStamp = function (el) {
    if (!el) return;
    var sel = null, hdl = null;
    try { sel = el.__zwSelector; } catch (_eSt1) {}
    try { hdl = el.__zwHandle; } catch (_eSt2) {}
    var t = _zwTtScriptFusedText(el);
    if ((sel == null || sel === '') && (hdl == null || hdl === '')) {
      try { globalThis._zwTtScriptTrustPlain.set(el, t); } catch (_eSt5) {}
      return;
    }
    try { if (sel != null && sel !== '') globalThis._zwTtScriptTrustSel[String(sel)] = t; } catch (_eSt3) {}
    try { if (hdl != null && hdl !== '') globalThis._zwTtScriptTrustHdl[String(hdl)] = t; } catch (_eSt4) {}
  };
  // 6.2b accessor getter 专用自足融合（childNodes 直走，零 trap 属性读——R93
  // 原型回落会把 document 等无臂节点的 textContent 读派发到 6.4 通用 getter，
  // getter 若再经 trap 读同名属性即无限递归；spec descendant text content：
  // Text/CDATA data 连接、element 递归）。
  // https://dom.spec.whatwg.org/#dom-node-textcontent
  var _zwTtChildTextFused = function (node) {
    var out = '';
    try {
      var kids = node.childNodes;
      if (kids) {
        for (var i = 0; i < kids.length; i++) {
          var k = kids[i];
          if (!k) continue;
          var nt = 0;
          try { nt = k.nodeType; } catch (_eCt0) { continue; }
          if (nt === 3 || nt === 4 || nt === 8) {
            try { out += String(k.data != null ? k.data : ''); } catch (_eCt1) {}
          } else if (nt === 1) {
            out += _zwTtChildTextFused(k);
          }
        }
      }
    } catch (_eCt2) {}
    return out;
  };
  // 6.3 prepare 时身份门（契约见 §6 头注）。管道异常（无政策 TypeError/回调抛
  // 错/返 undefined TypeError）一律归阻断——spec blocked script 不向页面抛错
  //（violation report 面挂账，见头部划界）。
  var _zwTtScriptExecGate = function (el, text) {
    if (!_zwTtEnforced()) return undefined;
    var sel = null, hdl = null;
    try { sel = el.__zwSelector; } catch (_eGt1) {}
    try { hdl = el.__zwHandle; } catch (_eGt2) {}
    var snap = null;
    try {
      if (sel != null && sel !== '') snap = globalThis._zwTtScriptTrustSel[String(sel)] || null;
      if (snap == null && hdl != null && hdl !== '') snap = globalThis._zwTtScriptTrustHdl[String(hdl)] || null;
      // 6.0b plain 弱表兜底（sel/hdl 双表未中时查 CCF 工厂产物按对象身份寄存
      // 的 parser 投递快照——插入后对象已获 sel/handle，键在而表空，不能以
      // sel/hdl 缺席为前置条件）
      if (snap == null) {
        try {
          if (globalThis._zwTtScriptTrustPlain && globalThis._zwTtScriptTrustPlain.has(el)) {
            snap = globalThis._zwTtScriptTrustPlain.get(el) || null;
          }
        } catch (_eGt6) {}
      }
    } catch (_eGt3) { snap = null; }
    if (snap != null && snap === text) return undefined;
    var iface = 'HTMLScriptElement';
    try {
      if (String(el.namespaceURI || '') === SVG_NS) iface = 'SVGScriptElement';
    } catch (_eGt4) {}
    try {
      return String(_zwTtSinkValue('TrustedScript', text, iface + ' text'));
    } catch (_eGt5) {
      return null;
    }
  };
  // 6.4 原型分层安装。真实 DOM 的 TT 声明 sink 是接口 accessor
  //（HTMLScriptElement.prototype own text/textContent/innerText）——语料 001/002
  // `Object.getOwnPropertyDescriptor(HTMLElement.prototype, …)` 直调面同时要求
  // Node.prototype.textContent / HTMLElement.prototype.innerText 存在**通用**
  // accessor（bypass 写不进声明 sink → prepare 时救援）。
  //
  // 委托机制：accessor setter 写树走 `__zwTtAccBypass` 计数旗 + Reflect.set 回
  // trap（part04 set trap 的 R521 派发臂见旗让位 → 落通用 textContent 写臂）；
  // 旗是计数器不是布尔——委托窗内嵌套写（同步 CEReactions 面等）不丢旗。
  // accessor getter＝fused 读（直调面；普通属性读走 get trap 既有臂——R93 原型
  // 回落只在全部臂未命中时可达，读路径零变化）。
  var _zwTtAccBypass = function (fn) {
    globalThis.__zwTtAccBypass = (globalThis.__zwTtAccBypass || 0) + 1;
    try { return fn(); } finally { globalThis.__zwTtAccBypass--; }
  };
  var _zwTtInstallScriptProtos = function () {
    // HTMLScriptElement.prototype 接 HTMLElement 链（s93 占位构造器的函数自动
    // prototype 原挂 Object.prototype——__zwHtmlTagIface getPrototypeOf trap 返
    // 本对象，instanceof HTMLElement 断链；幂等补链 R4004 同款）。
    try {
      if (globalThis.HTMLScriptElement && globalThis.HTMLScriptElement.prototype
          && globalThis.HTMLElement && globalThis.HTMLElement.prototype) {
        var _k21p = globalThis.HTMLScriptElement.prototype;
        var _k21c = _k21p, _k21g = 0, _k21linked = false;
        while (_k21c && _k21g++ < 32) {
          if (_k21c === globalThis.HTMLElement.prototype) { _k21linked = true; break; }
          _k21c = Object.getPrototypeOf(_k21c);
        }
        if (!_k21linked) Object.setPrototypeOf(_k21p, globalThis.HTMLElement.prototype);
      }
    } catch (_eK21l) {}
    // 声明 sink accessor（幂等：已有 own 同名不覆盖——重装配/宿主预定义界）。
    var _k21Sink = ['text', 'textContent', 'innerText'];
    for (var _k21i = 0; _k21i < _k21Sink.length; _k21i++) {
      (function (name) {
        var proto = globalThis.HTMLScriptElement && globalThis.HTMLScriptElement.prototype;
        if (!proto || Object.prototype.hasOwnProperty.call(proto, name)) return;
        try {
          Object.defineProperty(proto, name, {
            configurable: true, enumerable: true,
            get: function () { return _zwTtChildTextFused(this); },
            set: function (v) {
              // text 是普通 DOMString（非 enforced 保 s95 null→"null" 语义）；
              // textContent/innerText 是 LegacyNullToEmptyString（null→''）。
              // enforced 双臂都归一 null→'' 进管道（与撤除的 trap 旧分支同口径）。
              var enforced = _zwTtEnforced();
              var sv;
              if (enforced) {
                sv = String(_zwTtSinkValue('TrustedScript',
                  v == null ? '' : v, 'HTMLScriptElement ' + name));
              } else if (name === 'text') {
                sv = (v === null) ? 'null' : String(v);
              } else {
                sv = (v == null) ? '' : String(v);
              }
              _zwTtAccBypass(function () {
                Reflect.set(this, 'textContent', sv);
              }.bind(this));
              // 管道成功 + 落树成功 → 快照＝落树后 fused（gate 等值基准）。
              _zwTtScriptTrustStamp(this);
            },
          });
        } catch (_eK21d) {}
      })(_k21Sink[_k21i]);
    }
    // Node.prototype.textContent 通用 accessor（descriptor 直调面 + SVG script
    // 原型链可达面——SVGScriptElement 链 SVGElement→Element→Node；写经 bypass
    // 委托回 trap 通用写臂，语义零变化；不盖章 → prepare 时救援）。
    try {
      var _k21Node = globalThis.Node && globalThis.Node.prototype;
      if (_k21Node && !Object.prototype.hasOwnProperty.call(_k21Node, 'textContent')) {
        Object.defineProperty(_k21Node, 'textContent', {
          configurable: true, enumerable: true,
          get: function () { return _zwTtChildTextFused(this); },
          set: function (v) {
            _zwTtAccBypass(function () { Reflect.set(this, 'textContent', v); }.bind(this));
          },
        });
      }
    } catch (_eK21n) {}
    // HTMLElement.prototype.innerText 通用 accessor（001/002 descriptor 直调面；
    // getter 简单融合——渲染可见性语义与既有 trap 读臂同域简化，FIXME 记接口
    // innerText 完整 rendered-text 面）。
    try {
      var _k21He = globalThis.HTMLElement && globalThis.HTMLElement.prototype;
      if (_k21He && !Object.prototype.hasOwnProperty.call(_k21He, 'innerText')) {
        Object.defineProperty(_k21He, 'innerText', {
          configurable: true, enumerable: true,
          get: function () { return _zwTtChildTextFused(this); }, // FIXME: rendered-text 面
          set: function (v) {
            _zwTtAccBypass(function () { Reflect.set(this, 'innerText', v); }.bind(this));
          },
        });
      }
    } catch (_eK21h) {}
  };
  _zwTtInstallScriptProtos();

  // ===== 6b. iframe srcdoc / HTMLElement setHTMLUnsafe plain 世界（刀㉒ dc2-s10k22，
  // 2026-11-25）=====
  // MEl plain 世界（createHTMLDocument 产物等）经 R125 继承按 tag 接口原型——
  // HTMLIFrameElement.prototype/HTMLElement.prototype 为其可达面。旧缺 accessor：
  // srcdoc 赋值落 own expando（secondary-document 语料 srcdoc throws 面无管道可抛）、
  // setHTMLUnsafe 缺方法（throws 面缺方法 TypeError 意外绿）。门序镜像 SR 版
  //（part03:2400：sink 先于物化）；门后单次透传旗委托既有单咽喉（srcdoc →
  // setAttribute 属性门 part03:26204 / innerHTML → MEl 工厂 setter part03:26xxx），
  // 已门值经旗透传免双门。trap 世界 proxy 走 part04:1939/2435 特判先于原型，
  // 本二臂零可达；ShadowRoot 另有 own setHTMLUnsafe（part03:2400）不受影响。
  // https://w3c.github.io/webappsec-trusted-types/dist/spec/#enforcement-in-attributes
  var _zwTtInstallIframeSrcdoc = function () {
    var proto = globalThis.HTMLIFrameElement && globalThis.HTMLIFrameElement.prototype;
    if (!proto || Object.prototype.hasOwnProperty.call(proto, 'srcdoc')) return;
    try {
      Object.defineProperty(proto, 'srcdoc', {
        configurable: true, enumerable: true,
        get: function () {
          var v = (this.getAttribute && this.getAttribute('srcdoc'));
          return v == null ? '' : String(v);
        },
        set: function (v) {
          var sv;
          if (_zwTtEnforced()) {
            sv = String(_zwTtSinkValue('TrustedHTML', v, 'HTMLIFrameElement srcdoc'));
            if (typeof globalThis.__zwTtBypassNextSink === 'function') globalThis.__zwTtBypassNextSink();
          } else {
            sv = String(v); // 普通 DOMString（plain union）：null → 'null'
          }
          if (this.setAttribute) this.setAttribute('srcdoc', sv);
        },
      });
    } catch (_eK22sd) {}
  };
  _zwTtInstallIframeSrcdoc();

  var _zwTtInstallElemSetHtmlUnsafe = function () {
    var proto = globalThis.HTMLElement && globalThis.HTMLElement.prototype;
    if (!proto || Object.prototype.hasOwnProperty.call(proto, 'setHTMLUnsafe')) return;
    try {
      Object.defineProperty(proto, 'setHTMLUnsafe', {
        writable: true, enumerable: true, configurable: true,
        value: function (html) {
          var sv;
          if (_zwTtEnforced()) {
            sv = String(_zwTtSinkValue('TrustedHTML', html, 'Element setHTMLUnsafe'));
            if (typeof globalThis.__zwTtBypassNextSink === 'function') globalThis.__zwTtBypassNextSink();
          } else {
            sv = (html == null) ? '' : String(html);
          }
          this.innerHTML = sv;
        },
      });
    } catch (_eK22sh) {}
  };
  _zwTtInstallElemSetHtmlUnsafe();

  // ===== 7. 导出（globalThis 契约——part04 trap / part05 core / part03 Attr setter 消费）=====
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
  // 刀⑲ R521：script 身份门三件套（part03 prepare 门 / part06 解析投递盖章消费）。
  Object.defineProperty(globalThis, '__zwTtScriptExecGate', { value: _zwTtScriptExecGate, writable: true, configurable: true });
  Object.defineProperty(globalThis, '__zwTtScriptTrustStamp', { value: _zwTtScriptTrustStamp, writable: true, configurable: true });
  Object.defineProperty(globalThis, '__zwTtScriptFusedText', { value: _zwTtScriptFusedText, writable: true, configurable: true });
})();
