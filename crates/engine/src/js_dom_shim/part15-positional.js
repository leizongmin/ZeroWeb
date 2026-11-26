// part15（2026-11-23 wptfyi-dom DC-2 切片 7）：WICG declarative-partial-updates
// 位置方法族——appendHTML/prependHTML（ParentNode 面）+ beforeHTML/afterHTML/
// replaceWithHTML（ChildNode 面）± Unsafe，及 stream{Append,Prepend,Before,After,
// ReplaceWith}HTML{Unsafe} 次序壳。形制：链尾 IIFE + globalThis 契约
//（part12/13/14 同制——外层 shim IIFE 锁顶层函数，链尾件只走 globalThis）。
// spec：https://github.com/WICG/declarative-partial-updates（patching explainer，
// 正式算法文本未发布）——语义 ground truth = domparsing/tentative 语料三页
//（positional-methods / -with-parser-options / -with-trusted-types）断言面。
//
// 求值次序（语料实证矩阵）：
//   1. parser options：default policy createParserOptions 包装 caller options
//     （仅 enforcement 页；TrustedParserOptions 标记对象跳过——part12 政策方法
//     盖章）。先于层级/父判定（parser-options 页「checks createParserOptions
//     before hierarchy checks」，stream 壳同序）。
//   2. TT createHTML——**仅 Unsafe 变体**（trusted-types 页「does not go / goes
//     via createHTML」：safe 走内建 sanitizer，不经信任管道——反 innerHTML 惯例）。
//     直调 default policy.createHTML（__zwTtSinkValue 会把回调异常包成 TypeError
//     ——语料 assert_throws_exactly 需原样传播；TrustedHTML 实例经 isHTML 通道
//     直通）。回调里对 DOM 的变更（移动节点/prepend 新子）在其后生效——父与参考
//     子判定严格在其后（trusted-types 页三案 mutation 断言）。
//   3. 容器判定：append/prepend → receiver（template 元素路由 .content）；before/
//     after/replaceWith → parentNode——null 静默返回；Document（nodeType 9）与
//     普通 DocumentFragment（nodeType 11 且非 ShadowRoot（host 在席）、非模板
//     contents（__zwTplContents 印））→ HierarchyRequestError（methods 页两案）。
//   4. 解析：__zwSanBridge.fragmentAddedUnsafeInert 统一 inert 物化（part05 新
//     导出——scripting=false 强制，script 物化 inert 不执行，handle 域分流与
//     标记全保留；语料「script kept」断言面），顶层视图列表。
//   5. 消毒：__zwSanResolveOptions({sanitizer: processed.sanitizer}, safe) +
//     __zwSanFilterList——safe 无配置 → 默认实例（script 移除）；processed.
//     sanitizer 在席 → Sanitizer(config)（Unsafe 也应用——parser-options 页
//     「Policy can inject sanitizer」为 Unsafe 面）。
//   6. 插入：container 既有公开插入原语逐个落位（appendChild/insertBefore——
//     registry 记账/R5002 就地建册/t7 锚链/MO/CE 全链继承；IAH handle-only 分支
//     同制）。prepend 参考子在 TT 之后读（trusted-types 页 prepend mutation 案）。
//     replaceWith = 前插全批后 remove 自身（spec dom-childnode-replacewith 两步）。
//   7. 脚本执行：!safe && options.runScripts === true && processed.runScripts ===
//     true && 容器非模板 contents（三门与「Policies can disable scripts but not
//     enable them」一致——caller 请求是前置、policy 可否决不可启用；TrustedParser
//     Options 路径两门同值）。内联（无 src、无 type）script 按文档序 indirect
//     eval（part06 write 面同原语 (0, eval).call(globalThis, …)）；src 外链与
//     type 模块面挂账 FIXME。执行通道不设 TT script 门——enforcement 页语料
//     期望 did_run = true（HTML 串信任已由 createHTML 管道裁决）。
// 消毒/解析底座消费 part13 导出（__zwSanResolveOptions/__zwSanFilterList）与
// part05 导出（__zwSanBridge）；方法分派经 part04 get trap 新分支 late-bound
//（part04 trap 消费 __zwSanSetHtml 同制——getPrototypeOf trap 不影响属性读，
// part05 R1835 实证，proto 链定义对 proxy 不可达）。
(function () {
  'use strict';
  // 原型分派面先行（不受下方重入卫门）：shim 可能同 realm 多次求值，part03
  // 每次都**无条件重建** CharacterData/Text/Comment 原型链（2520/2533/2590——
  // `globalThis.X.prototype = Object.create(...)` 无守卫），本件卫门 return 会把
  // 原型面 stranded 在被替换的旧原型对象上（切片 8 探针 p13c 定谳：foreign
  // doc Comment 经新链找 stream*HTML 全 undefined，而 __zwPosHtml 在席）。分派
  // 为幂等 defineProperty，逐次求值重挂零副作用。
  var _POS_METHODS = ['appendHTML', 'appendHTMLUnsafe', 'prependHTML', 'prependHTMLUnsafe',
    'beforeHTML', 'beforeHTMLUnsafe', 'afterHTML', 'afterHTMLUnsafe',
    'replaceWithHTML', 'replaceWithHTMLUnsafe'];
  var _POS_STREAM_METHODS = ['streamHTML', 'streamHTMLUnsafe',
    'streamAppendHTML', 'streamAppendHTMLUnsafe',
    'streamPrependHTML', 'streamPrependHTMLUnsafe',
    'streamBeforeHTML', 'streamBeforeHTMLUnsafe',
    'streamAfterHTML', 'streamAfterHTMLUnsafe',
    'streamReplaceWithHTML', 'streamReplaceWithHTMLUnsafe'];
  var _posProtoDispatch = function (prop) {
    return function (htmlText, options) {
      var fn = globalThis.__zwPosHtml;
      if (typeof fn !== 'function') return;
      var core = prop.slice(-6) === 'Unsafe' ? prop.slice(0, -6) : prop;
      var pos = core.slice(0, -4); // 去 'HTML' 后缀 → append/prepend/before/after/replaceWith
      return fn(this, pos, htmlText, options, prop.slice(-6) !== 'Unsafe');
    };
  };
  // stream 族分派（切片 8 补——原型面旧只装配非 stream 十方法，foreign doc
  // Comment 的 stream*HTML 直读 undefined：探针 p2a/p13c 定谳）。
  var _posStreamDispatch = function (prop) {
    return function (options) {
      var fn = globalThis.__zwPosStream;
      if (typeof fn !== 'function') return;
      // 位置推导与 part04 get trap 分派同制：先剥 Unsafe 后缀，再去 'HTML'
      // 后缀；streamHTML 无位置后缀 → pos='html'。
      var coreS = prop.slice(-6) === 'Unsafe' ? prop.slice(6, -6) : prop.slice(6);
      var pos = coreS === 'HTML' ? 'html' : coreS.slice(0, -4);
      pos = pos.charAt(0).toLowerCase() + pos.slice(1);
      return fn(this, pos, options, prop.slice(-6) !== 'Unsafe');
    };
  };
  try {
    var _posProtos = [globalThis.Element && globalThis.Element.prototype,
      globalThis.CharacterData && globalThis.CharacterData.prototype];
    for (var _pi2 = 0; _pi2 < _posProtos.length; _pi2++) {
      var _pp = _posProtos[_pi2];
      if (!_pp) continue;
      for (var _pj2 = 0; _pj2 < _POS_METHODS.length; _pj2++) {
        try {
          Object.defineProperty(_pp, _POS_METHODS[_pj2], {
            value: _posProtoDispatch(_POS_METHODS[_pj2]),
            writable: true, configurable: true, enumerable: false
          });
        } catch (_ePosDef) {} // 单方法隔离——一个 proto 装配失败不拖垮其余面
      }
      for (var _pk2 = 0; _pk2 < _POS_STREAM_METHODS.length; _pk2++) {
        try {
          Object.defineProperty(_pp, _POS_STREAM_METHODS[_pk2], {
            value: _posStreamDispatch(_POS_STREAM_METHODS[_pk2]),
            writable: true, configurable: true, enumerable: false
          });
        } catch (_ePosDefS) {}
      }
    }
  } catch (_ePosProto) {}

  if (globalThis.__zwPosHtml) return; // 幂等（shim 重装配边界）——实现体域

  var _hierEx = function (method) {
    return new (globalThis.DOMException || Error)(
      "Failed to execute '" + method + "' on 'ChildNode': The node to be inserted is not valid for insertion into a Document.",
      'HierarchyRequestError');
  };

  // 次序 1：parser options——default policy createParserOptions 包装（仅
  // enforcement；TrustedParserOptions 标记对象直通，异常原样传播）。
  var _processOptions = function (options) {
    var enforced = typeof globalThis.__zwTtEnforced === 'function' && globalThis.__zwTtEnforced();
    if (!enforced) return options;
    var tt = globalThis.trustedTypes;
    var def = (tt && tt.defaultPolicy) || null;
    if (!def || typeof def.createParserOptions !== 'function') return options;
    if (options && options.__zwTtParserOptions === true) return options;
    return def.createParserOptions(options === undefined || options === null ? {} : options);
  };

  // WebIDL DOMString 转换（spec primitive conversion → ToString：Symbol 抛
  // TypeError——本宿主 String() 对 Symbol 非抛（弱化面），sink write 的 chunk
  // 强转需独立门：语料「writing a Symbol to streamHTML()」write 拒绝案）。
  var _p15DomString = function (v) {
    if (typeof v === 'symbol') {
      throw new TypeError('Cannot convert a Symbol value to a string');
    }
    return String(v);
  };

  // 次序 2：TT createHTML 管道（仅 Unsafe）——直调政策方法保回调异常原样传播
  //（__zwTtSinkValue 的 catch 会把异常包成 TypeError，语料 assert_throws_exactly
  // 不兼容）。
  var _ttHtml = function (htmlText, sinkName) {
    var s = String(htmlText);
    var enforced = typeof globalThis.__zwTtEnforced === 'function' && globalThis.__zwTtEnforced();
    if (!enforced) return s;
    var tt = globalThis.trustedTypes;
    if (tt && typeof tt.isHTML === 'function' && tt.isHTML(htmlText)) return s;
    var def = (tt && tt.defaultPolicy) || null;
    if (!def || typeof def.createHTML !== 'function') {
      throw new TypeError("Failed to execute '" + sinkName + "' on 'Element': This document requires 'TrustedHTML' assignment.");
    }
    return String(def.createHTML(s, 'TrustedHTML', sinkName));
  };

  var _isTemplate = function (node) {
    if (!node || node.nodeType !== 1) return false;
    var t = '';
    try { t = String(node.localName || node.tagName || '').toLowerCase(); } catch (_eT) { t = ''; }
    return t === 'template';
  };

  // 次序 3 的层级门：Document / 普通 DocumentFragment 拒绝；ShadowRoot（host
  // 在席）与模板 contents（__zwTplContents 印）豁免（methods 页两案 + 语料矩阵
  // ref.parentNode = template 元素/ShadowRoot 成功臂）。
  var _hierOk = function (parent) {
    if (!parent) return false;
    if (parent.nodeType === 9) return false;
    if (parent.nodeType === 11) {
      var isSR = false;
      try { isSR = parent.host != null; } catch (_eH) { isSR = false; }
      var isTpl = false;
      try { isTpl = parent.__zwTplContents === true; } catch (_eTc) { isTpl = false; }
      return isSR || isTpl;
    }
    return true;
  };

  // script 收集（文档序；子树 walk——**根节点自身入列**：语料输入 script 恒为
  // 顶层 kid，只查子节点的 walk 会整批漏收；nodeBudget 防病态输入二次放大）。
  var _collectScripts = function (roots) {
    var out = [];
    var budget = 4096;
    var walk = function (node) {
      if (budget-- <= 0 || !node) return;
      if (node.nodeType === 1) {
        var tag = '';
        try { tag = String(node.tagName || '').toUpperCase(); } catch (_eTg) { tag = ''; }
        // script 元素自身入列即返回——其内部文本是脚本源，不再下钻（嵌套
        // script 非法面）。
        if (tag === 'SCRIPT') { out.push(node); return; }
      }
      var kids = null;
      try { kids = node.childNodes; } catch (_eK) { return; }
      if (!kids) return;
      for (var i = 0; i < kids.length; i++) walk(kids[i]);
    };
    for (var r = 0; r < roots.length; r++) walk(roots[r]);
    return out;
  };

  // script 内联文本（child text 节点 data 串接——解析视图 text 载体双形态容错）。
  var _scriptText = function (el) {
    var out = '';
    try {
      var kids = el.childNodes || [];
      for (var i = 0; i < kids.length; i++) {
        var c = kids[i];
        if (c && c.nodeType === 3) {
          out += (c.nodeValue != null ? c.nodeValue : (c.data != null ? c.data : ''));
        }
      }
    } catch (_eTx) { out = ''; }
    return out;
  };

  // inert 解析统一底座（位置方法族与 stream 写入体共用）：fragmentAddedUnsafeInert
  // 物化 + 活数组快照 + s118 already-started 盖章。scripting=1 实解析两臂（context/
  // 全文档）都按活文档执行语义剥 script（probe 实证：div context 臂与 null-handle
  // 全文档臂输入 script 均不物化），而语料「script kept」断言要求 script 物化为
  // inert 元素、执行只走调用方 options 门（模板 contents 永不执行——spec contents
  // owner document 无 browsing context → scripting disabled，同 TEMPLATE handle
  // 分支 _uScr=false 先例与 template.innerHTML R5000 片 a 同制）。
  var _p15ParseInert = function (text, hostHandle, isTplContents, isShadowFace) {
    var bridge = globalThis.__zwSanBridge || null;
    var kids;
    if (bridge && typeof bridge.fragmentAddedUnsafeInert === 'function') {
      kids = bridge.fragmentAddedUnsafeInert(text, hostHandle);
    } else if ((isTplContents || isShadowFace) && bridge && typeof bridge.buildBodyTree === 'function') {
      // 降级臂：无 inert 导出的旧桥——模板 contents/ShadowRoot 走 buildBodyTree
      // scripting=false（script 物化），元素 handle 面回落旧导出。
      var _inertBody = bridge.buildBodyTree(text, '', false, true, '');
      kids = (_inertBody && _inertBody.childNodes) ? _inertBody.childNodes : [];
    } else {
      kids = (bridge && typeof bridge.fragmentAddedUnsafe === 'function')
        ? bridge.fragmentAddedUnsafe(text, hostHandle) : [];
    }
    if (!kids || !kids.length) {
      return [];
    }
    // 快照——parse body 内部活数组引用。mock 容器（模板 contents 等）的
    // appendChild/insertBefore 走「预摘旧父」臂（part04 R145 视图 mutation 面），
    // 源数组 splice 移位后逐索引消费把后续 kid 静默漏收（探针 zz-knife-s7-p4/p5
    // 实证：SPAN 先落位后 kids[1] 已 undefined → SCRIPT 整体丢失）。
    kids = kids.slice();
    // s118 同章（part05 _zwSetHtmlUnsafeImpl _uS118 打标点）：解析产物 SCRIPT 盖
    // already-started 章（__zwFragNoExec）——spec prepare the script element：fragment
    // parsing 产物置 already started，物化进活文档不得由引擎插入臂自动执行；
    // 执行只走 options 门。R368 盖 handle 章点转移 run-once 表（part04:8291）。
    // https://html.spec.whatwg.org/multipage/scripting.html#prepare-the-script-element
    var _p15Stamp = function (nd) {
      if (!nd || nd.nodeType !== 1) return;
      var tg = '';
      try { tg = String(nd.tagName || '').toUpperCase(); } catch (_eP15tg) { tg = ''; }
      if (tg === 'SCRIPT') {
        try { nd.__zwFragNoExec = true; } catch (_eP15st) {}
      }
      var kc = null;
      try { kc = nd.childNodes; } catch (_eP15kc) { return; }
      if (kc) for (var ki = 0; ki < kc.length; ki++) _p15Stamp(kc[ki]);
    };
    for (var _p15si = 0; _p15si < kids.length; _p15si++) _p15Stamp(kids[_p15si]);
    return kids;
  };

  // parse 产物 text/comment kid 落引擎代管父（proxy）静默丢弃（probe12 实证：
  // appendChild 零抛、childNodes=0、css 文本不生效；元素 kid 自带 wire 不受影响
  // ——div control children=1）。经 document 原语重建落位（createTextNode 带
  // engine 代管身份——probe14 t2 实证）。引擎代管父不可从 JS 面探测（__zwHandle
  // get trap 域外隐藏——probe14 t1 实证 style/div 双 null），故反转判据：已知
  // mock 族容器（模板 contents __zwTplContents 章 / ShadowRoot face）保持原 kid
  // 直插，其余容器（引擎代管 proxy、普通 mock 元素）一律重建——重建产物在 mock
  // 父下 splice 落位文本读面等价。
  var _p15Materialize = function (kid, container, safe) {
    var isTextComment = false;
    try { isTextComment = kid != null && (kid.nodeType === 3 || kid.nodeType === 8); } catch (_eMc) { isTextComment = false; }
    if (!isTextComment) return kid;
    // safe 变体对 SCRIPT 容器的 text kid 不物化：文本即脚本源，engine 对已连接
    // script 的文本子追加动态启动执行（probe20 t1 直 appendChild 实证，与流无关
    // 的 engine 面语料 stream-html-script-safe「should not allow script execution」
    // 3F 根因）——safe 语义 = inert，源不落位即不执行（sanitize 剥 script 内容
    // 同向）。Unsafe 变体照常落位（执行语义随 engine，等于直 appendChild）。
    if (safe !== false) {
      var cTag = '';
      try { cTag = String(container && container.tagName || '').toUpperCase(); } catch (_eSct) { cTag = ''; }
      if (cTag === 'SCRIPT' && kid.nodeType === 3) return null;
    }
    var mockSurface = false;
    try {
      mockSurface = container != null
        && (container.__zwTplContents === true
          || (container.nodeType === 11 && container.host != null));
    } catch (_eMs) { mockSurface = false; }
    if (mockSurface) return kid;
    try {
      var docM = null;
      try { docM = container.ownerDocument || null; } catch (_eMd) { docM = null; }
      if (!docM && typeof globalThis.document !== 'undefined') docM = globalThis.document;
      if (!docM) return kid;
      var dataM = null;
      try { dataM = kid.data; } catch (_eMdt) { dataM = null; }
      if (dataM == null) { try { dataM = kid.textContent; } catch (_eMtx) { dataM = null; } }
      dataM = dataM == null ? '' : dataM;
      if (kid.nodeType === 3) return docM.createTextNode(dataM);
      return docM.createComment(dataM);
    } catch (_eMm) {
      return kid;
    }
  };

  // 消毒统一底座（safe 无配置 → 默认实例；processed.sanitizer 在席 → 配置实例——
  // Unsafe 也应用，parser-options 页「Policy can inject sanitizer」为 Unsafe 面）。
  // 底座经 __zwSanBridge 契约域（part13 桥扩展导出——globalThis 直名不在契约）。
  var _p15Sanitize = function (kids, processed, safe, sanPre) {
    if (!kids || !kids.length) return;
    var bridge = globalThis.__zwSanBridge || null;
    var resolve = bridge ? bridge.resolveOptions : null;
    var filter = bridge ? bridge.filterList : null;
    if (typeof filter !== 'function') return;
    // sanPre 在席 = 流创建时快照实例（config 快照语义——流期间调用方变异不
    // 生效，且省逐冲刷 re-resolve）；缺席 = 单发管线即时解析。
    var san = (sanPre !== undefined) ? sanPre : null;
    if (san == null && typeof resolve === 'function') {
      var sanConfig = (processed && typeof processed === 'object') ? processed.sanitizer : undefined;
      san = resolve({ sanitizer: sanConfig }, safe);
    }
    if (san) filter(kids, san, safe);
  };

  // runScripts 有效性门（语料语义锚：stream-html-unsafe「should execute scripts
  // when connected」Unsafe 变体默认允许；runScripts-off 页 options.runScripts:
  // false 显式关；stream-html-run-scripts-option 页 safe 变体 options.runScripts:
  // true 显式开；runScripts-cannot-be-turned-on 页 TT parser-options 不能把 safe
  // 变体开成执行；pass-trusted 页 TT 覆盖可把 Unsafe 关掉）。
  var _p15RunScriptsOn = function (options, processed, safe) {
    if (safe === false) {
      if (options && typeof options === 'object' && options.runScripts === false) return false;
    } else {
      if (!(options && typeof options === 'object' && options.runScripts === true)) return false;
    }
    if (processed && typeof processed === 'object' && processed.runScripts === false) return false;
    return true;
  };

  // close 时连接性门（spec 连接 document 才 ready-to-be-executed；stream-html-unsafe
  // 「should not execute scripts when disconnected」案）——engine proxy 走 isConnected
  //（part04:3324 get trap 服务面）；ShadowRoot face 按 shadow-including 连接性判
  //（宿主连入 document 即连接——variants 页 ShadowRoot target 执行面持位）；mock 面
  //（模板 contents 等）无 isConnected 恒 false（spec contents owner 无 browsing context）。
  // https://dom.spec.whatwg.org/#concept-shadow-including-root
  var _p15Connected = function (node) {
    try {
      var ic = (node && typeof node.isConnected === 'boolean') ? node.isConnected : undefined;
      if (ic === true) return true;
      try {
        if (node && node.nodeType === 11 && node.host != null && node.host.isConnected === true) return true;
      } catch (_eShc) {}
      return false;
    } catch (_eCn) {}
    return false;
  };

  // 内联 script 求值（src/type 面挂账 FIXME；调用方持三门）。
  var _p15EvalScripts = function (scripts) {
    for (var s = 0; s < scripts.length; s++) {
      var sc = scripts[s];
      var src = '';
      try { src = String((sc.getAttribute && sc.getAttribute('src')) || ''); } catch (_eSrc) { src = ''; }
      var ty = '';
      try { ty = String((sc.getAttribute && sc.getAttribute('type')) || ''); } catch (_eTy) { ty = ''; }
      // FIXME(spec-gap): src 外链 fetch 链与 type 分类（module/classic 求值域）随
      // 脚本通道深项刀位——语料位置方法族输入恒内联经典 script，本刀只走内联臂。
      if (src || ty) continue;
      var code = _scriptText(sc);
      if (code) {
        // 刀㉙返修三g（k29 配置 F dp 电池 20F 判读）：位置式内联体免门遮蔽——
        // `(0, eval)` 过 globalThis.eval，eval 门在位时（strict 车道体前检查点
        // 装置，返修三f）体是 String 实参 → TT face → default policy 无
        // createScript → EvalError 被 catch 吞 → 插入脚本静默跳过
        //（domparsing positional-methods-with-parser-options did_run 20F 实证；
        // 测试自移除 script 后延后执行取消）。「run a classic script」不是
        // eval 编译（刀㉘ 契约，part03 s109 臂/part06 写面同形遮蔽）——
        // 页面显式 eval 面（SHIMW 门管辖域）与脚本执行体分流。
        // https://html.spec.whatwg.org/multipage/webappapis.html#run-a-classic-script
        try { (0, (typeof globalThis.__zwNativeEval === 'function' ? globalThis.__zwNativeEval : eval)).call(globalThis, code); } catch (_eEv) {}
      }
    }
  };

  // 核心管线（次序 1-7）。
  var _posHtml = function (self, pos, htmlText, options, safe) {
    var cap = pos.charAt(0).toUpperCase() + pos.slice(1);
    var sinkBase = cap + 'HTML' + (safe ? '' : 'Unsafe');
    // 1. parser options（先于一切层级判定；异常原样传播）
    var processed = _processOptions(options);
    // 2. TT createHTML（仅 Unsafe；DOMString 强转与 innerHTML/setHTML 同制）
    var text = safe ? String(htmlText) : _ttHtml(htmlText, 'Element ' + sinkBase);
    // 3. 容器判定
    var container = self;
    var refNode = null;
    var isReplace = false;
    if (pos === 'before' || pos === 'after' || pos === 'replaceWith') {
      var parent = null;
      try { parent = self.parentNode; } catch (_eP) { parent = null; }
      if (parent == null) return; // 无父静默返回（trusted-types 页 null-parent 臂——TT 已先评）
      if (!_hierOk(parent)) throw _hierEx(sinkBase);
      // template 元素作父 → 插入落 contents（spec the-template-element 语义;
      // util 矩阵 ref.parentNode 即模板元素/内容视图两形态）。
      container = (_isTemplate(parent) && parent.content) ? parent.content : parent;
      refNode = self;
      isReplace = pos === 'replaceWith';
    } else if (_isTemplate(self)) {
      // ParentNode 面 receiver 为 template 元素 → 路由 contents（HTMLTemplateElement
      // 继承 Element.prototype，方法体内路由；spec 面矩阵 object=template）。
      try {
        var tplC = self.content;
        if (tplC) container = tplC;
      } catch (_eC) {}
    }
    var isTplContents = false;
    try { isTplContents = container != null && container.__zwTplContents === true; } catch (_eTc2) {}
    // 4. 解析（统一 inert 底座——handle 的 ctxNs/ctxTag/shadow 分流与
    // __zwFragHostHandle/parentNode 标记在 _p15ParseInert 消费面保留）。
    var hostHandle = null;
    try { hostHandle = (container && container.__zwHandle != null) ? container.__zwHandle : null; } catch (_eHd) { hostHandle = null; }
    var _isShadowFace = false;
    try {
      _isShadowFace = container != null && container.nodeType === 11 && container.host != null;
    } catch (_eSh) { _isShadowFace = false; }
    var kids = _p15ParseInert(text, hostHandle, isTplContents, _isShadowFace);
    // 5. 消毒
    _p15Sanitize(kids, processed, safe);
    // 6. 插入（prepend 参考子在 TT 后读——trusted-types 页 prepend mutation 案；
    // after 参考子 = self 后继快照，null = 尾插；replaceWith 前插全批后移除自身）
    var insertRef = refNode;
    if (pos === 'after' && refNode) {
      try { insertRef = refNode.nextSibling || null; } catch (_eNs) { insertRef = null; }
    }
    for (var i = 0; i < kids.length; i++) {
      var kid = kids[i];
      if (!kid) continue;
      kid = _p15Materialize(kid, container, safe);
      if (!kid) continue;
      if (pos === 'append') container.appendChild(kid);
      else if (pos === 'prepend') container.insertBefore(kid, container.firstChild || null);
      else container.insertBefore(kid, insertRef);
    }
    if (isReplace && refNode) {
      // 摘除走容器原语（spec dom-childnode-replacewith 移除步 = 父 removeChild）——
      // refNode.remove() 的元素臂经引擎 registry 寻父（sel/handle wire），mock 容器
      //（模板 contents 视图）父形态引擎侧无父 → lenient no-op，ref 滞留（语料
      // 「Position check: replaceWithHTML (template.content)」18F 根因）。mock
      // removeChild 按身份 splice（part04 R145 视图），引擎父形态语义等价。
      try {
        container.removeChild(refNode);
      } catch (_eRm) {
        try { if (typeof refNode.remove === 'function') refNode.remove(); } catch (_eRm2) {}
      }
    }
    // 7. 脚本执行（runScripts 有效性门 + 模板 contents 永不执行——语料
    // target_type !== "template.content" 门；src/type 面挂账见 _p15EvalScripts）
    if (!isTplContents && _p15RunScriptsOn(options, processed, safe) && _p15Connected(container)) {
      _p15EvalScripts(_collectScripts(kids));
    }
  };

  // stream 面（语料可见门 + 写入体）：
  //   1. parser options（同 _posHtml 次序 1——先于一切层级/TT 判定）。
  //   2. TT 门（仅 Unsafe）：无 default policy → TypeError；policy 缺
  //     createParserOptions → TypeError（no-policy/no-createParserOptions 页
  //     assert_throws_js(TypeError) 矩阵——Unsafe 流要求 parser-options 能力策略）。
  //   3. sanitizer 配置校验：resolveOptions 构造 Sanitizer(config)（无效配置抛，
  //     流不创建——sanitizer 页 10 案，safe/Unsafe 双变体）。
  //   4. 层级门（before/after/replaceWith 父 null 静默、Document/fragment HRE——
  //     与 _posHtml 次序 3 同制）。
  //   5. 流构造（写入体，语料语义锚 stream-html.html / stream-positional.html /
  //     stream-html-run-scripts-option.html / stream-html-sanitizer.html）：
  //     - write(chunk)：String(chunk) 强转（Symbol → TypeError → write promise
  //       拒绝——「writing a Symbol」案）；buffer 追加后增量冲刷（未完标签 +
  //       RAWTEXT 未闭元素尾持留——「partial script 三写」案）。
  //     - 逐冲刷：inert 解析（统一底座）→ 消毒（options.sanitizer）→ 插入位 →
  //       脚本执行（批次本地；runScripts 有效性 + 连接性门——语料
  //       stream-html-unsafe 三写案 `</script>` 补全冲刷即执行；safe 门关 =
  //       「should not execute scripts when connected」案）。
  //     - abort：回滚本流已插节点（「Aborting」案——内容清空）。
  //     - ReplaceWith：写体落 receiver 本体，close 时 receiver 被自身子替换
  //      （语料 create_stream 的 placeholder 形态；stream-positional 页终态断言）。
  // FIXME(spec-gap): RAWTEXT 尾持已覆盖开标签粒度（script/style/textarea/title
  // 未闭开标签 → 整元素滞留，语料三写 script 案实证）；残面 = 同标签多次开关的
  // lastIndexOf 归属（<script>a</script><script 未闭 → 正确持第二段）与 RAWTEXT
  // 文本内字面 '</script' 转义域，随 stream 深面（defer-async-script/
  // with-declarative-shadow-root 尾簇）补章。
  var _posStream = function (self, pos, options, safe) {
    // 1. parser options
    var processed = _processOptions(options);
    // 2. TT 门（仅 Unsafe；stream 无 html 实参——只验 default policy 存在性与
    // createParserOptions 能力）
    if (safe === false) {
      var mNameS = pos === 'html' ? 'streamHTMLUnsafe'
        : 'stream' + pos.charAt(0).toUpperCase() + pos.slice(1) + 'HTMLUnsafe';
      var enforcedS = typeof globalThis.__zwTtEnforced === 'function' && globalThis.__zwTtEnforced();
      if (enforcedS) {
        var ttS = globalThis.trustedTypes;
        var defS = (ttS && ttS.defaultPolicy) || null;
        if (!defS) {
          throw new TypeError("Failed to execute '" + mNameS + "' on 'Element': This document requires 'TrustedHTML' assignment.");
        }
        if (typeof defS.createParserOptions !== 'function') {
          throw new TypeError("Failed to execute '" + mNameS + "' on 'Element': The default policy does not define createParserOptions.");
        }
      }
    }
    // 3. sanitizer 配置校验（无效配置抛 → 流不创建）；实例创建时快照。
    var bridgeS = globalThis.__zwSanBridge || null;
    var resolveS = bridgeS ? bridgeS.resolveOptions : null;
    var sanSnapS = null;
    if (typeof resolveS === 'function') {
      var sanConfigS = (processed && typeof processed === 'object') ? processed.sanitizer : undefined;
      sanSnapS = resolveS({ sanitizer: sanConfigS }, safe !== false);
      // config 快照语义：the sanitizer is snapshotted at stream creation——
      // resolve 对 Sanitizer 实例是透传（part13 resolveOptions instanceof 臂），
      // 流期间调用方 removeElement/allowElement 变异不得影响已建流
      //（stream-html-sanitizer「mutating the sanitizer while streaming」12 案）。
      // 实例形态 get() 平面配置重建（canon 构造期深拷贝零泄漏）；dict 形态
      // 构造期已 canon 拷贝，不属调用方可变异面。
      // https://wicg.github.io/declarative-partial-updates/
      if (sanSnapS != null && sanConfigS != null && sanSnapS === sanConfigS &&
          typeof sanSnapS.get === 'function' && typeof globalThis.Sanitizer === 'function') {
        try { sanSnapS = new globalThis.Sanitizer(sanSnapS.get()); } catch (_eSanSnap) {}
      }
    }
    // 4. 层级门 + 插入位路由（语料 create_stream：Before 接收者即 ref——插 ref 前；
    // After 接收者 = ref.previousSibling——插其后＝ref 前；ReplaceWith 接收者为
    // 占位元素——写体落占位、close 占位被子替换；Append/Prepend/html 容器 =
    // receiver 本体，模板元素路由 contents 同 _posHtml）。
    var containerS = self;
    var refS = null;           // before 位锚（逐冲刷插其前）
    var afterModeS = false;    // after 位开关
    var afterRefS = null;      // after 位锚（创建时快照 self.nextSibling——逐冲刷插
                               // 其前保写序；null = 尾插）
    var replaceWrapS = false;  // replaceWith：写体落 self，close 拆壳
    var isTplContentsS = false;
    if (pos === 'before' || pos === 'after' || pos === 'replaceWith') {
      var parentS = null;
      try { parentS = self.parentNode; } catch (_eP2) { parentS = null; }
      if (parentS == null) return;
      if (!_hierOk(parentS)) {
        throw _hierEx('stream' + pos.charAt(0).toUpperCase() + pos.slice(1) + 'HTMLUnsafe');
      }
      containerS = (_isTemplate(parentS) && parentS.content) ? parentS.content : parentS;
      if (pos === 'before') refS = self;
      else if (pos === 'after') {
        afterModeS = true;
        try { afterRefS = self.nextSibling || null; } catch (_eNsS) { afterRefS = null; }
      } else {
        // replaceWith：写体落**接收者自身**（占位壳）——语料 create_stream 形为
        // 占位元素（insertBefore 占位, ref 后流挂占位），写体逐冲刷入占位、close
        // 拆壳（占位子提升到父位 + 占位摘除）。旧版容器误设 parentS——写体直落
        // 父尾（位置/时序双错：stream-positional「with a ref_node」族 got
        // `<p>X</p><span></span>A`；探针 p10 midPh 空 = 占位零内容实证）。
        // 切片 8 补（sanitizer-safe 全量跑红 4 案实证）：receiver 为 Comment/
        // Text（`Comment/Text.streamReplaceWithHTML` 直挂形态）不能承载子——
        // 铸真占位元素插 receiver 前位（spec placeholder 语义），写体落占位、
        // close 拆壳换占位并移除 receiver（ReplaceWith 语义=内容替换目标）。
        replaceWrapS = true;
        containerS = self;
        var rwWrapS = null;
        if (self.nodeType !== 1) {
          try {
            rwWrapS = self.ownerDocument.createElement('div');
            parentS.insertBefore(rwWrapS, self);
            containerS = rwWrapS;
          } catch (_eRwPh) { rwWrapS = null; containerS = self; }
        }
      }
    } else if (_isTemplate(self)) {
      try {
        var tplCS = self.content;
        if (tplCS) containerS = tplCS;
      } catch (_eCS) {}
    }
    try { isTplContentsS = containerS != null && containerS.__zwTplContents === true; } catch (_eTcS) { isTplContentsS = false; }
    // 锚定校验（切片 8：stream-insertion-logic reject 族——「ref node is removed /
    // no longer child of the same parent」12 案：ref 摘除或换父后 write 须拒
    // HierarchyRequestError，插入点失效即拒绝、内容不落位）。锚 = 创建时快照：
    // before 锚接收者自身（receiver 即 ref）；replaceWith 锚创建时 self.next
    // Sibling（语料占位后邻 = ref 形——占位自身未动而 ref 离位，须按后邻锚定）；
    // prepend 锚创建时 container.firstChild；after 锚接收者自身离位（语料无
    // after reject 断言面，随 spec 插入点随接收者）。逐写校验在 sink write，
    // 失效返 rejected promise（流 error）。
    var _p15AnchorDead = null;
    if (pos === 'before') {
      _p15AnchorDead = function () {
        try { return refS.parentNode !== containerS; } catch (_eAbB) { return true; }
      };
    } else if (pos === 'replaceWith') {
      var rwRefS = null;
      try { rwRefS = self.nextSibling || null; } catch (_eAbR) { rwRefS = null; }
      // 锚父 = 创建时 parentS（非 containerS——replaceWith 容器已改占位自身，
      // ref/self 的 parentNode 判定必须对创建时父）。
      var rwParentS = parentS;
      _p15AnchorDead = function () {
        try { if (self.parentNode !== rwParentS) return true; } catch (_eAbR2) { return true; }
        if (!rwRefS) return false;
        try { return rwRefS.parentNode !== rwParentS; } catch (_eAbR3) { return true; }
      };
    } else if (pos === 'prepend') {
      var ppFirstS = null;
      try { ppFirstS = containerS.firstChild || null; } catch (_eAbP) { ppFirstS = null; }
      _p15AnchorDead = function () {
        if (!ppFirstS) return false;
        try { return ppFirstS.parentNode !== containerS; } catch (_eAbP2) { return true; }
      };
    } else if (pos === 'after') {
      _p15AnchorDead = function () {
        try { return self.parentNode !== containerS; } catch (_eAbA) { return true; }
      };
    }
    // streamHTML 清容（创建即清；同容器活流计数豁免——WeakMap 键容 proxy/mock
    // 双形态；无 WeakMap 退化为每次清，语料 Interleaved 案降级）。
    if (pos === 'html') {
      var doClearS = true;
      if (typeof globalThis.WeakMap === 'function') {
        try {
          if (!globalThis.__zwPosStreamCount) globalThis.__zwPosStreamCount = new globalThis.WeakMap();
          var cntS = globalThis.__zwPosStreamCount.get(containerS) || 0;
          doClearS = cntS === 0;
          globalThis.__zwPosStreamCount.set(containerS, cntS + 1);
        } catch (_eCnt) { doClearS = true; }
      }
      if (doClearS) {
        try { while (containerS.firstChild) containerS.removeChild(containerS.firstChild); } catch (_eClr) {}
      }
    }
    // 5. 流构造（写入体）
    if (typeof globalThis.WritableStream !== 'function') return undefined;
    var stateS = {
      buffer: '',
      inserted: [],      // 本流已落位顶层节点（abort 回滚 + close 脚本收集域）
      lastIns: null,     // prepend 位末写锚（保写序——插上一批末节点之后）
      closed: false,
      self: self, container: containerS, safe: safe,
      options: options, processed: processed, isTplContents: isTplContentsS,
      replaceWrap: replaceWrapS,
      wrap: rwWrapS,     // replaceWith 铸占位（Comment/Text 接收者；null=元
                         // 素接收者自壳）
      san: sanSnapS,     // 创建时快照实例（config 快照语义；null=无桥域）
    };
    // 逐冲刷：未完标签尾持留 → 解析 → 消毒 → 插入位。
    var _p15Flush = function (st, isClose) {
      var buf = st.buffer;
      if (!buf) return;
      var cut = buf.length;
      if (!isClose) {
        var lt = buf.lastIndexOf('<');
        if (lt >= 0 && buf.indexOf('>', lt) < 0) cut = lt;
        // RAWTEXT 未闭元素尾持（spec 流解析缓冲 script data——语料
        // stream-html-unsafe「should execute scripts when connected」三写案：
        // <script> 开标签后数据到 </script> 前不得入冲刷批次，否则引擎 inert
        // 解析对未闭合 RAWTEXT 产空文本 script、执行域拿不到源）。textarea/title
        // 同章（可显转义文本面）。开标签自身未完（无 '>'）更早持留。
        // https://html.spec.whatwg.org/multipage/parsing.html#script-data-state
        var bufLow = buf.toLowerCase();
        var rtNames = ['script', 'style', 'textarea', 'title'];
        for (var ri = 0; ri < rtNames.length; ri++) {
          var openRt = bufLow.lastIndexOf('<' + rtNames[ri]);
          if (openRt < 0) continue;
          var afterRt = bufLow.charAt(openRt + 1 + rtNames[ri].length);
          if (afterRt && /[a-z0-9]/.test(afterRt)) continue;
          var gtRt = bufLow.indexOf('>', openRt);
          if (gtRt < 0) { if (openRt < cut) cut = openRt; continue; }
          if (bufLow.indexOf('</' + rtNames[ri], gtRt) < 0 && openRt < cut) cut = openRt;
        }
      }
      if (cut <= 0 && !isClose) return;
      var head = buf.slice(0, cut);
      st.buffer = buf.slice(cut);
      if (!head) return;
      var hostHandleF = null;
      try { hostHandleF = (st.container && st.container.__zwHandle != null) ? st.container.__zwHandle : null; } catch (_eHdF) { hostHandleF = null; }
      var shadowFaceF = false;
      try { shadowFaceF = st.container != null && st.container.nodeType === 11 && st.container.host != null; } catch (_eShF) { shadowFaceF = false; }
      var kidsF = _p15ParseInert(head, hostHandleF, st.isTplContents, shadowFaceF);
      _p15Sanitize(kidsF, st.processed, st.safe, st.san);
      var batchF = [];
      for (var iF = 0; iF < kidsF.length; iF++) {
        var kidF = kidsF[iF];
        if (!kidF) continue;
        kidF = _p15Materialize(kidF, st.container, st.safe);
        if (!kidF) continue;
        try {
          if (st.replaceWrap) st.container.appendChild(kidF);
          else if (refS) st.container.insertBefore(kidF, refS);
          else if (afterModeS) st.container.insertBefore(kidF, afterRefS);
          else if (pos === 'prepend') {
            // prepend 写序保持（spec 插入点驻留形）：首批锚容器当前首子（旧
            // firstIns 形的回退臂——insertion-logic ref_node 8 案实证 anchor=null
            // 尾插回退丢位），后续批插上一批末节点之后保写序（双写案 got "21"
            // 反转形实证——旧 firstIns 前插形把后续写放到首写之前）。
            var anchorF = null;
            try { anchorF = st.container.firstChild || null; } catch (_ePpF) { anchorF = null; }
            if (st.lastIns) {
              try { anchorF = st.lastIns.nextSibling || null; } catch (_eLi) { anchorF = null; }
            }
            st.container.insertBefore(kidF, anchorF);
            st.lastIns = kidF;
          } else st.container.appendChild(kidF);
        } catch (_eInsF) { continue; }
        st.inserted.push(kidF);
        batchF.push(kidF);
      }
      // 脚本执行挂逐冲刷（语料 stream-html-unsafe「should execute scripts when
      // connected」三写案：`</script>` 补全的冲刷批次立即执行；尾持保证跨写
      // 拼装的 script 不早执行——不完整标签滞留 buffer 不入批次）。批次本地
      // 收集防跨冲刷重执行。门同 _posHtml step 7：runScripts 有效性 + 连接性 +
      // 模板 contents 永不执行。
      if (!st.isTplContents && _p15RunScriptsOn(st.options, st.processed, st.safe)
          && _p15Connected(st.container)) {
        _p15EvalScripts(_collectScripts(batchF));
      }
    };
    return new globalThis.WritableStream({
      write: function (chunk) {
        if (stateS.closed) return;
        // 锚定校验先于内容吸收（失效写不进 buffer——close 尾冲刷亦不见失效块；
        // 拒绝经 sink write 返 rejected promise，流 error，语料
        // promise_rejects_dom 断言面）。
        if (_p15AnchorDead && _p15AnchorDead()) {
          var deadNameS = 'stream' + pos.charAt(0).toUpperCase() + pos.slice(1) + 'HTML'
            + (safe === false ? 'Unsafe' : '');
          return globalThis.Promise.reject(_hierEx(deadNameS));
        }
        // String 强转即 TT 域外的 WebIDL DOMString 语义：Symbol 抛 TypeError →
        // write promise 拒绝（语料「writing a Symbol」案）；null/undefined/数字
        // 按 String() 文本化（「null undefined」/「345」案）。
        stateS.buffer += _p15DomString(chunk);
        _p15Flush(stateS, false);
      },
      close: function () {
        if (stateS.closed) return;
        stateS.closed = true;
        _p15Flush(stateS, true);
        // 脚本执行已挂逐冲刷（close 只冲刷尾批）；模板 contents 永不执行由
        // _p15Flush 门保持。
        // ReplaceWith 拆壳：占位（wrap 或元素接收者自身）子提升到接收者位、
        // 占位与接收者摘除（写体落 placeholder 形态收尾——stream-positional
        // 页终态断言面；Comment/Text 接收者双摘除=ReplaceWith 内容替换目标）。
        if (stateS.replaceWrap) {
          var wrapW = stateS.wrap || stateS.self;
          var pW = null;
          try { pW = wrapW.parentNode; } catch (_ePw) { pW = null; }
          if (pW != null) {
            var kidsW = [];
            try {
              var ckW = wrapW.childNodes || [];
              for (var iw = 0; iw < ckW.length; iw++) if (ckW[iw]) kidsW.push(ckW[iw]);
            } catch (_eCkW) { kidsW = []; }
            for (var jw = 0; jw < kidsW.length; jw++) {
              try { pW.insertBefore(kidsW[jw], stateS.self); } catch (_eMvW) {}
            }
            try { pW.removeChild(wrapW); } catch (_eRmW) {
              try { if (typeof wrapW.remove === 'function') wrapW.remove(); } catch (_eRmW2) {}
            }
            if (stateS.wrap) {
              try { pW.removeChild(stateS.self); } catch (_eRmW3) {
                try { if (typeof stateS.self.remove === 'function') stateS.self.remove(); } catch (_eRmW4) {}
              }
            }
          }
        }
        try {
          if (pos === 'html' && typeof globalThis.WeakMap === 'function' && globalThis.__zwPosStreamCount) {
            var cntC = globalThis.__zwPosStreamCount.get(containerS) || 1;
            globalThis.__zwPosStreamCount.set(containerS, Math.max(0, cntC - 1));
          }
        } catch (_eCntC) {}
      },
      abort: function (reason) {
        if (stateS.closed) return;
        stateS.closed = true;
        // 回滚本流已落位节点（语料「Aborting」案——内容清空）；template contents
        // 面已插的也回滚（流生命周期一致）。
        for (var iAb = 0; iAb < stateS.inserted.length; iAb++) {
          var nAb = stateS.inserted[iAb];
          if (!nAb) continue;
          try {
            if (nAb.parentNode && typeof nAb.parentNode.removeChild === 'function') nAb.parentNode.removeChild(nAb);
            else if (typeof nAb.remove === 'function') nAb.remove();
          } catch (_eAb) {}
        }
        // 铸占位同步回滚（Comment/Text 接收者 replaceWith——占位空壳不残留）。
        if (stateS.wrap) {
          try {
            if (stateS.wrap.parentNode && typeof stateS.wrap.parentNode.removeChild === 'function') {
              stateS.wrap.parentNode.removeChild(stateS.wrap);
            } else if (typeof stateS.wrap.remove === 'function') stateS.wrap.remove();
          } catch (_eAbW) {}
        }
        try {
          if (pos === 'html' && typeof globalThis.WeakMap === 'function' && globalThis.__zwPosStreamCount) {
            var cntA = globalThis.__zwPosStreamCount.get(containerS) || 1;
            globalThis.__zwPosStreamCount.set(containerS, Math.max(0, cntA - 1));
          }
        } catch (_eCntA) {}
      },
    });
  };

  globalThis.__zwPosHtml = _posHtml;
  globalThis.__zwPosStream = _posStream;
})();
