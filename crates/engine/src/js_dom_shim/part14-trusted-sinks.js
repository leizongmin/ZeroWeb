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
// setTimeout/setInterval 入口管道（part01 shim 内部门）。eval 域刀㉙（s10k29）
// 已落 §5.5 通用 eval 门：替换 globalThis.eval（间接/经引用调用全部收敛），
// 直接 eval 因读取 globalThis.eval 同样进门（dom 语料抽样实证 eval 全为全局
// 名读取，无依赖直接 eval 局部作用域的语料面）；「run a classic script」与
// shim/module 内部求值经 __zwNativeEval 快照（part01 顶）绕门。RO header 面
//（.headers sidecar 的 CSP-Report-Only 只进 runner 子资源通道、主文档不可见）
// 仍挂账，随 runner CSP 管道刀位。
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
  // 返修②（T108/r79 workers 套件 OOM 定谳，见指纹块内注记）：无 document 环境
  // （DedicatedWorker/SharedWorker scope）恒量——首次判定后指纹恒 ''（worker 全局
  // 不会转正出 document）。
  var _cspFpNoDoc = false;
  // 刀㊹b（wptfyi-dom s10k44b）face P：per-document CSP 状态册——网络交付的
  // iframe 子文档政策容器**自有**（spec CSP §policy-containers：about:blank/
  // srcdoc/data:/blob: 本地 scheme 继承父容器，网络交付自取自家 delivery）。
  // 记录由 part01 finish 管线经 __zwTtFrameOwnCsp44b 落册（＝无 CSP 交付近似，
  // 宿主 fetch 桥未 wire 帧响应 CSP 头）；无记录文档（主文档、本地 scheme 帧）
  // 回落主文档 meta 扫描＝继承语义零变化。读窗＝`__zwActiveIframeDoc` 槽位
  //（R206 执行窗／s339 注册窗／刀㊸ 帧消息派发窗／刀㊹b write 执行窗四处同键）。
  var _zwTtPerDocCsp44b = typeof WeakMap === 'function' ? new WeakMap() : null;
  var _zwTtPerDocState44b = function () {
    if (!_zwTtPerDocCsp44b) return null;
    var ad = null;
    try { ad = globalThis.__zwActiveIframeDoc || null; } catch (_e44bAd) { ad = null; }
    if (!ad || typeof ad !== 'object') return null;
    try { return _zwTtPerDocCsp44b.get(ad) || null; } catch (_e44bG) { return null; }
  };
  var _zwTtParseCsp = function () {
    // 刀㊹b face P：帧自有交付记录优先——记录存在即整状态直返（帧 CSP 静态、
    // 生命周期随文档），不走主文档 meta 指纹缓存。
    var _pdState44b = _zwTtPerDocState44b();
    if (_pdState44b) return _pdState44b;
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
    // 返修②（T108/r79 workers 套件 OOM 实证）：无 document 环境下原指纹计算每次
    // 经 document.getElementsByTagName 属性链 throw 一个 TypeError（Error 对象＋
    // 栈捕获＝每次调用的固定分配）——worker 消息洪流场景 __zwTtEnforced 热路径
    // 成为 GC 追不上的分配/积压风暴（mark-compact 后存活 3.8 GB → 90s 执行超时
    // 前堆触顶 OOM），k20b 闩锁只免 ON 页重扫、false 页每调用仍全量重扫。document
    // 缺席是 scope 恒量：O(1) 判空后置 _cspFpNoDoc，后续指纹恒 '' 走缓存。
    // 枚举域同时收窄到 head——CSP meta 的规范位置（body 内 meta 非规范形态，
    // FIXME 挂账）；head 子树近空的页面（workers 套件等）从 O(全文档) gETN 重建
    // 降为 O(head)。有 document 臂语义（指纹比对→缓存/重扫→闩锁置位）不变——
    // 动态注入 face（013）跨事件边界经 __zwTtCspDirty 置脏即生效。
    // https://html.spec.whatwg.org/multipage/semantics.html#meta-http-equiv-content-security-policy
    if (!_cspFpNoDoc) {
      var docFp = globalThis.document;
      if (docFp == null) {
        _cspFpNoDoc = true;
      } else {
        try {
          var _cspMetaRoot = docFp.head || docFp;
          var metasFp = _cspMetaRoot.getElementsByTagName('meta');
          var partsFp = [];
          for (var iFp = 0; iFp < metasFp.length; iFp++) {
            var heFp = '', coFp = '';
            try { heFp = String(metasFp[iFp].getAttribute('http-equiv') || ''); } catch (_eFp1) {}
            try { coFp = String(metasFp[iFp].getAttribute('content') || ''); } catch (_eFp2) {}
            partsFp.push(heFp + '\u0001' + coFp);
          }
          fp = partsFp.join('\u0002');
        } catch (_eFp3) { fp = ''; }
      }
    }
    if (_cspCache !== null && (_cspFpNoDoc || fp === _cspCacheFp)) return _cspCache;
    var state = { enforced: false, hasList: false, lists: [], allowDuplicates: false, reportOnly: false, roPolicy: '', enforcePolicy: '', listPolicy: '', scriptSrcPolicy: '', scriptSrcHasUnsafeEval: false };
    try {
      var metas = (globalThis.document.head || globalThis.document).getElementsByTagName('meta');
      for (var i = 0; i < metas.length; i++) {
        var he = '';
        try { he = String(metas[i].getAttribute('http-equiv') || '').toLowerCase(); } catch (_e1) {}
        // 刀㉖：RO 面 meta（http-equiv content-security-policy-report-only）同
        // 循环解析——只置 reportOnly/roPolicy，不触 enforced 名单域（RO 政策只
        // 报告不执法，其 trusted-types 名单不进 createPolicy 名单门；
        // https://www.w3.org/TR/CSP3/#cspro-header ）。
        var isRo = he === 'content-security-policy-report-only';
        if (he !== 'content-security-policy' && !isRo) continue;
        var content = '';
        try { content = String(metas[i].getAttribute('content') || ''); } catch (_e2) {}
        // 违规事件 originalPolicy 载体：RO 面 = 首见 RO 政策原文（语料
        // trusted-types-report-only originalPolicy.startsWith('trusted-types two;')
        // 按 RO meta content 原序断言）。执法/名单面按**指令归属**逐政策记录
        //（刀㉗：原「首见非 RO 政策」口径在多 meta 页把名单/连接政策错标为
        // 执法政策——语料 clipping-of-sample 三 meta 页 originalPolicy 断言
        // 'require-trusted-types-for' 政策本体，须在指令分支内记录）。
        if (isRo) { if (!state.roPolicy) state.roPolicy = content; }
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
            if (dvalue.toLowerCase().indexOf("'script'") >= 0) {
              if (isRo) { state.reportOnly = true; } else { if (!state.enforcePolicy) state.enforcePolicy = content; state.enforced = true; }
            }
          } else if (dname === 'script-src') {
            // 刀㉙：script-src 政策原文记录（eval 门 CSP face 违规
            // originalPolicy 载体——CSP §create-violation-for-global 同款
            // 「政策本体」口径）。meta 兜底面：'unsafe-eval' token 判定按
            // 指令值（首见 script-src；全政策串搜会撞其他指令的
            // 'wasm-unsafe-eval' 假阳——引号包裹 token 精确匹配）。
            // FIXME: default-src 回退（语料无此形态）；多政策 script-src 并集。
            if (isRo) continue;
            if (!state.scriptSrcPolicy) state.scriptSrcPolicy = content;
            if (!state.scriptSrcHasUnsafeEval) state.scriptSrcHasUnsafeEval = dvalue.indexOf("'unsafe-eval'") >= 0;
          } else if (dname === 'trusted-types') {
            if (isRo) continue; // RO 名单不进执法名单域（见上方 state 注记）
            if (!state.listPolicy) state.listPolicy = content; // 刀㉗：名单政策原文（createPolicy block 违规 originalPolicy 载体）
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
  // 返修②（T108/r79 workers 套件 OOM 实证）：微任务域 memo——同一同步执行块
  // （单事件 handler/单段页面脚本/单 sink 序列）内首查算、后续读缓存，微任务
  // 排空即失效。动态 meta 注入 face（013「注入后下一次 script 执行 enforcement
  // 生效」）跨事件边界必走新查询；HTML sink 写面出口经 __zwTtCspDirty 置脏
  // （part03/04/06 各 sink 守卫后），同块注入亦即时生效。k20b 闩锁只免 ON 页
  // 重扫，false 页（绝大多数页面）逐 sink 全文档 gETN 重建的洪流成本由本 memo
  // 摊平为每事件一次 O(head) 探测＋一个失效排程闭包。
  var _zwTtEnfVal = false;
  var _zwTtEnfDirty = true;
  var _zwTtEnfFlushQueued = false;
  var _zwTtEnforced = function () {
    // 刀㊹b face P：帧自有交付记录先于全局闩锁/memo——enforced 主文档下的
    // 网络交付帧恒 false（记录态读，不触闩锁）。
    var _pdEnf44b = _zwTtPerDocState44b();
    if (_pdEnf44b) return _pdEnf44b.enforced === true;
    if (!_zwTtEnfDirty) return _zwTtEnfVal;
    if (!_zwCspEnforcedLatch) _zwCspEnforcedLatch = _zwTtParseCsp().enforced;
    _zwTtEnfVal = _zwCspEnforcedLatch;
    _zwTtEnfDirty = false;
    if (!_zwTtEnfFlushQueued && typeof Promise === 'function') {
      _zwTtEnfFlushQueued = true;
      Promise.resolve().then(function () {
        _zwTtEnfDirty = true;
        _zwTtEnfFlushQueued = false;
      });
    }
    return _zwTtEnfVal;
  };

  // 刀㉖：RO monitoring 查询——镜像 _zwTtEnforced 的微任务域 memo（__zwTtCspDirty
  // 同置双脏）。enforced 页经 _zwCspEnforcedLatch 短路后 sink 管道不达 RO 查询，
  // 无需独立闩锁；RO 页 enforced=false 走本 memo 摊平每事件一次 O(head) 探测。
  var _zwRoVal = false;
  var _zwRoDirty = true;
  var _zwRoFlushQueued = false;
  var _zwRoActive = function () {
    // 刀㊹b face P：帧自有交付记录先于 memo（同 _zwTtEnforced 形）。
    var _pdRo44b = _zwTtPerDocState44b();
    if (_pdRo44b) return _pdRo44b.reportOnly === true;
    if (!_zwRoDirty) return _zwRoVal;
    _zwRoVal = _zwTtParseCsp().reportOnly;
    _zwRoDirty = false;
    if (!_zwRoFlushQueued && typeof Promise === 'function') {
      _zwRoFlushQueued = true;
      Promise.resolve().then(function () {
        _zwRoDirty = true;
        _zwRoFlushQueued = false;
      });
    }
    return _zwRoVal;
  };

  // 刀㉗：sample 40 单元截断（spec TT §create-a-trusted-type-violation-record——
  // sample 取前 40 UTF-16 码元；尾单元落 lone 高代理（D800–DBFF）时延至 41 保
  // 代理对完整＝Firefox 口径，w3c/webappsec-csp#704；语料 clipping-of-sample
  // tentative 页 surrogate 四案 + 正页 39/40/41 案分型断言）。
  var _zwTtClipSample = function (s) {
    if (s.length <= 40) return s;
    var cut = (s.charCodeAt(39) >= 0xD800 && s.charCodeAt(39) < 0xDC00) ? 41 : 40;
    return s.slice(0, cut);
  };
  // 刀㉖：TT 违规派发（spec CSP3 §report-the-violation 邻域）；documentURI =
  // document.documentURI（= location.href，语料 trusted-types-report-only
  // report.documentURI == location 断言面）。刀㉗：sinkName 面 sample =「sink
  // 名|截断后值」（sink 名表照 spec sink group 列——本引擎 sinkName 实参已同形，
  // 如 'Element innerHTML'；createPolicy block 面传空 sinkName → 无前缀，
  // directive 'trusted-types'）；effectiveDirective/violatedDirective 缺省
  // require-trusted-types-for（语料 assert_equals(report.effectiveDirective,
  // "require-trusted-types-for")）。刀㉘：blockedURI 按 spec TT
  // §create-a-trusted-type-violation-record——sink 拦截面 = 'trusted-types-sink'
  //（语料 reporting-for-* 各页 assert_equals(violation.blockedURI,
  // "trusted-types-sink")），createPolicy block 面传 'trusted-types-policy'
  //（语料 trusted-types-reporting 页同字段断言）；第六参缺省取 sink 面。
  var _zwTtViolation = function (sinkName, sample, originalPolicy, disposition, directive, blockedURI) {
    try {
      if (typeof globalThis.__zw_dispatch_securitypolicyviolation !== 'function') return;
      // 刀㊺c（wptfyi-dom s10k53）：违规位置归因——CSP3 §create-violation-for-global
      // script 执行触发面取**当前脚本**位置（栈捕获，k48b Face C 处方）。V8 栈形：
      // shim 帧/页内联帧 = `<anonymous>:l:c`（无 URL），外链页脚本帧（sourceURL
      // 注释 s336 面）= `at [fn] (https://…:line:col)`。自顶向下走栈，跳过 shim
      // 帧，**首个非 shim 帧定归属**：http 帧 = 外链脚本（sink-003 断言 line=3/
      // column≥5、creation-005 断言 line=4/column≥7 靶形）；行号 < 5000 的匿名帧
      // = 页内联块 → 停在 null（派发侧回落 documentURI = 内联触发面的正确归属，
      // source-file-path same-document 断言页 URL 实证）。shim/页界线取 5000：
      // shim 拼接偏移自 part01 起万行级（part01.js ~17k 行、TT 门整体 ≥ part12
      // 段 174k 行），页内联块通常 < 1000 行；纯行号界对 part01 < 5000 行区域
      // 调用链有理论误判（误停→null 回落，退化为刀前行为，不致错归）。
      // 捕获须在同步窗（deferred setTimeout 任务栈是 timer 回调，非违规点）；
      // 无页帧（timer/宿主触发面）→ 缺省 undefined，派发侧回落 documentURI/0/0
      // 零漂移。
      // https://w3c.github.io/webappsec-csp/#create-violation-for-global
      var _vLoc = null;
      try {
        var _vLines = String(new Error().stack || '').split('\n');
        for (var _vI = 0; _vI < _vLines.length; _vI++) {
          var _vM = _vLines[_vI].match(/(https?:\/\/[^()\s]+):(\d+):(\d+)/);
          if (_vM) {
            _vLoc = { file: _vM[1], line: Number(_vM[2]), col: Number(_vM[3]) };
            break;
          }
          var _vA = _vLines[_vI].match(/<anonymous>:(\d+):(\d+)/);
          if (_vA && Number(_vA[1]) < 5000) break;
        }
      } catch (_eVLoc) {}
      // 刀㉖：违规派发按 CSP spec §report-violation 入 **task** 队列（setTimeout 0
      // 载体，非微任务）——语料 Count/listener 面把监听注册在 promise_test 体
      // （harness promise_tests 微任务链），同步派发先于注册即全数丢失；
      // setTimeout 任务晚于全部微任务 → 监听就绪后按排队序送达（Chrome 同构）。
      // 参数现值闭包捕获（政策原文不随任务窗漂移）。
      var dir = directive == null || directive === '' ? 'require-trusted-types-for' : String(directive);
      var body = sinkName ? String(sinkName) + '|' + _zwTtClipSample(sample == null ? '' : String(sample))
        : _zwTtClipSample(sample == null ? '' : String(sample));
      setTimeout(function () {
        try {
          var du = '';
          try { du = String(globalThis.document && globalThis.document.documentURI || ''); } catch (_eV1) {}
          globalThis.__zw_dispatch_securitypolicyviolation({
            documentURI: du,
            // 刀㊺c：位置字段透传（part06 派发侧 `|| documentURI`/`|| 0` 缺省回落）。
            sourceFile: _vLoc ? _vLoc.file : undefined,
            lineNumber: _vLoc ? _vLoc.line : undefined,
            columnNumber: _vLoc ? _vLoc.col : undefined,
            effectiveDirective: dir,
            violatedDirective: dir,
            originalPolicy: originalPolicy == null ? '' : String(originalPolicy),
            sample: body,
            // 刀㉘：sink 面占位语义（spec TT §violation record）；createPolicy
            // block 面由 part12 调用点传 'trusted-types-policy' 覆盖。
            blockedURI: blockedURI == null || blockedURI === '' ? 'trusted-types-sink' : String(blockedURI),
            disposition: disposition === 'report' ? 'report' : 'enforce'
          });
        } catch (_eV3) {}
      }, 0);
    } catch (_eV2) {}
  };

  // 刀㉖：RO monitoring sink 臂——Report-Only 政策对 sink 赋值不执法：TT 实例
  // 直通；default policy 及对应回调存在 → 调用（回调异常原样传播、无违规——
  // 语料 default-policy-report-only Count 面 reject() 分桶不含 throw/typeerror
  // 样本；返串 → 造后值入 sink、无违规）；返 null/undefined 或无 default
  // policy/对应回调 → 原串入 sink + report 违规（disposition 'report'，语料
  // trusted-types-report-only report.disposition 断言面）。
  // https://w3c.github.io/webappsec-trusted-types/dist/spec/#process-value-with-default-policy
  var _zwTtRoSink = function (type, value, sinkName) {
    var tt = globalThis.trustedTypes;
    if (tt && typeof tt.isHTML === 'function') {
      var isTT = (type === 'TrustedHTML' && tt.isHTML(value))
        || (type === 'TrustedScript' && tt.isScript(value))
        || (type === 'TrustedScriptURL' && tt.isScriptURL(value));
      if (isTT) return String(value);
    }
    var s = typeof value === 'string' ? value : String(value);
    var def = (tt && tt.defaultPolicy) || null;
    // 刀㉖：绑 raw 回调（见 part12 __zwRawCbs 寄存注记——包装方法 undefined/null
    // 语义只归直调面；RO 面 null/undefined → report 违规 + 原样透传）。
    var _roCbs = (def && def.__zwRawCbs) || def;
    var fn = def ? (type === 'TrustedHTML' ? _roCbs.createHTML
      : type === 'TrustedScript' ? _roCbs.createScript
        : _roCbs.createScriptURL) : null;
    if (def && typeof fn === 'function') {
      var out;
      try { out = fn.call(def, s, type, sinkName); } catch (_eRoP) { throw _eRoP; }
      if (out !== undefined && out !== null) return String(out);
    }
    var st = _zwTtParseCsp();
    _zwTtViolation(sinkName, s, st.roPolicy, 'report');
    return s;
  };

  // ===== 2. sink 管道核心 =====
  // spec sink processing（#create-a-trusted-type-data-value-adjacent 簇）：
  // TT 实例 → 数据串直接用；其余 String 化 → default policy 存在则走其
  // createX（回调抛错原样传播；返 undefined/null → TypeError + 违规派发——
  // 语料 default-policy null 面 assert_throws_js(TypeError)，刀㉖ 前返 null 曾
  // 错置空串）；无 default policy/对应回调 → TypeError + 违规派发。
  // RO 面（Report-Only 政策）走 _zwTtRoSink 监测臂（不执法）。
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
      if (_zwRoActive()) return _zwTtRoSink(type, value, sinkName);
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
      // 刀㉖：block 面违规派发（语料 default-policy Count 面 nodefault 样本计数）
      // 后原 TypeError 照抛。
      _zwTtViolation(sinkName, s, _zwTtParseCsp().enforcePolicy, 'enforce');
      throw new TypeError(
        "Failed to execute '" + sinkName + "' on 'Element': This document requires '" + type + "' assignment.");
    }
    // 刀㉖：绑 raw 回调（见 part12 __zwRawCbs 寄存注记）——enforced 面
    // null/undefined → TypeError + 违规派发由本管道持有（包装层语义只归直调）。
    var _enCbs = (def && def.__zwRawCbs) || def;
    var fn = type === 'TrustedHTML' ? _enCbs.createHTML
      : type === 'TrustedScript' ? _enCbs.createScript
        : _enCbs.createScriptURL;
    if (typeof fn !== 'function') {
      // default policy 缺对应 create 回调：spec 按未提供处理——与 createPolicy
      // 直调同款（返 undefined → TypeError）。
      _zwTtViolation(sinkName, s, _zwTtParseCsp().enforcePolicy, 'enforce');
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
    if (out === undefined || out === null) {
      // 刀㉖：null/undefined 返回同质——TypeError + 违规派发（sink 保持原值，
      // 语料 default-policy null 面 assert_throws_js(TypeError) + sink==''；
      // Count 面 null/undefined/done 样本计数）。刀㉖ 前 null 曾错置空串不抛。
      _zwTtViolation(sinkName, s, _zwTtParseCsp().enforcePolicy, 'enforce');
      throw new TypeError(
        "Failed to execute '" + sinkName + "' on 'Element': This document requires '" + type + "' assignment.");
    }
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
    // 刀㉖：RO 面属性 sink 监测——表照常出数据（type/sink），写面走 _zwTtSinkValue
    // 时由 RO 监测臂处置（不执法、report 违规）。enforced 页短路不变。
    if (!_zwTtEnforced() && !_zwRoActive()) return null;
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
  // 划界：本门只管 Function/派生构造器编译域；eval 域（间接/经引用/直接
  // eval）由 §5.5 通用 eval 门接管（刀㉙——globalThis.eval 替换后直接 eval
  // 同样进门，见该节注记）。
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
    // 刀㉙：编译域违规派发（spec CSP §ensure-csp-does-not-block-string-compilation
    // 阻断即 report violation）——旧版只抛 EvalError 不派发，语料
    // trusted-types-reporting-for-Window-function-constructor 四 face
    // 「Violation report for <Ctor> with plain string」恒红。sample＝装配文本
    // 去 `function anonymous` 前缀（语料断言 `Function|` + clip(`(\n) {\n…\n}`)
    // ——_zwTtViolation 的 sinkName|clip 组装补 'Function|' 前缀）。
    var gateSample = text.slice('function anonymous'.length);
    var _gateBlock = function (msg) {
      _zwTtViolation(sinkName, gateSample, _zwTtParseCsp().enforcePolicy, 'enforce');
      throw new EvalError(msg);
    };
    if (typeof fn !== 'function') {
      _gateBlock('Failed to compile a string: This document requires a TrustedScript for compilation.');
    }
    var out;
    try {
      out = fn.call(def, text, 'TrustedScript', sinkName);
    } catch (_eP) {
      _gateBlock('Failed to compile a string: default policy callback threw.');
    }
    if (out === undefined) {
      _gateBlock('Failed to compile a string: default policy returned undefined.');
    }
    var s = out === null ? '' : String(out);
    if (s !== text) {
      // default policy 不可改写编译串（输出 ≠ 装配文本 → EvalError）。
      _gateBlock('Failed to compile a string: default policy modified the value.');
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

  // ===== 5.5 通用 eval 门（刀㉙ s10k29 建；刀㊾ s10k50 重形 arg-transform）=====
  // spec https://w3c.github.io/webappsec-trusted-types/dist/spec/#eval-dom ＋
  // CSP §ensure-csp-does-not-block-string-compilation（"script-src restrictions
  // apply after the default policy runs"——TT face 先、CSP face 后；TT-block
  // 短路 CSP face，单违规——语料 trusted-types-eval-reporting-no-unsafe-eval
  // helper `violations.length === 1` 面）。
  //
  // 刀㊾ 重形：装配期**不再替换 globalThis.eval**（直接性＝ECMA-262 语法
  // callee 判定——裸 `eval` 解析到原生绑定且语法 callee 为 eval 标识符才走
  // PerformEval 调用方作用域；任何绑定替换都把直接调用退化为间接——刀㉙
  // 全包装版 enforced 页直接 eval 声明落点丢失＝reporting-for-Window-eval
  // 「No violation reported for eval with TrustedScript.」唯一红面实证：TS 面
  // 透传后 native 尾调落全局作用域，函数局部 beacon 不被赋值）。门改
  // **实参变换器**：part03 prepare 对 armed 页（三臂条件同旧全装条件）脚本
  // 文本裸 `eval(` 调用点改写为 `eval(__zwTtEvalArg(expr))`——语法 callee
  // 保持 eval 原生绑定，作用域链零损伤；门体三 face 迁移至 __zwTtEvalArg
  //（调用时点 fresh parse 分派，k29 返修三口径同）：
  //   ① 非 String 非 TrustedScript 实参 → 原样返回（ECMA-262 PerformEval
  //      非串不转换：eval({}) 保全等、eval(42)===42、eval(null/undefined) 同值）；
  //   ② TrustedScript（isScript 判定）→ ToString 转换（WebIDL wrapper 边界
  //      转换——native eval 对对象原样返回不转换，probe1 P3 实证）→ 免 TT face
  //      （含 default policy 调用，语料 default-policy 页 sink 断言仅 string 面
  //      触发）→ 仍受 CSP face；
  //   ③ String → TT enforce face（default policy createScript 四 block：缺回调/
  //      抛错/返 undefined·null/改值——violation(sink 'eval') + EvalError）→
  //      CSP face（__zwCspEvalBlockedQuery '1' → script-src 违规（无 'eval|'
  //      前缀）+ EvalError）→ 返串交语法调用点的原生 eval 编译（直接性本体）。
  // 薄装臂（刀㊵→㊶，no-csp TT-API 页）装置语义不变（其守卫读旗标同步跳过
  // armed 页）；createPolicy 先于 enforced 判定的序列由 arming 复位薄装处理。
  // 双路径：meta 三臂（part03 prepare 体前检查点）＝旗标形——globalThis.eval
  // 恒原生，armed 驱动 prepare 文本变换（直接性保形本体）；宿主 header-face
  //（webview eval-blocked 脚本 M2-s6 per-script 装卸）＝runtime 包装形保留
  //（install 在 exec 前、变换在 prepare——armed 迟到赶不上本文本，包装拦
  // eval 于调用时才完备；Function Proxy 半边不变）。
  var _zwEvalArgArm = function () {
    if (globalThis.__zwEvalGateInstalled) return;
    globalThis.__zwEvalGateInstalled = true;
    globalThis.__zwEvalArgArmed = true;
    // 刀㊾：薄装若在位（createPolicy 先于 enforced 判定的页内序列）→ 复位
    // 原生绑定（其 TS 臂由 __zwTtEvalArg face ② 承接；薄装守卫读旗标，此后
    // 不再复装）。
    if (globalThis.__zwEvalThinInstalled
        && typeof globalThis.__zwNativeEval === 'function') {
      try { globalThis.eval = globalThis.__zwNativeEval; } catch (_eEvR50) {}
    }
  };
  // 宿主 header-face 包装形（__zwInstallEvalGate 导出）：包装体前置
  // __zwTtEvalArg 三 face（语义与刀㉙ 包装逐面同形），native 尾调仍间接——
  // 该路径无直接 eval 保形诉求（语料 header 面＝CSP 阻断导向）。
  var _zwEvalWrapInstall = function () {
    if (globalThis.__zwEvalWrapInstalled) return;
    var nativeEval = globalThis.__zwNativeEval;
    if (typeof nativeEval !== 'function') return;
    globalThis.__zwEvalWrapInstalled = true;
    globalThis.__zwEvalGateInstalled = true;
    globalThis.eval = function (x) {
      var s = globalThis.__zwTtEvalArg(x);
      return typeof s === 'string'
        ? nativeEval.call(globalThis, s)
        : nativeEval.call(globalThis, x);
    };
  };
  // 门体（实参面）——只消费/校验/返回，不调 native eval（编译由语法调用点的
  // 原生 eval 承担＝直接性保形本体）。三 face 与刀㉙ 包装逐面同形（face 序、
  // 违规 sample 口径、四 block 语义不变；fresh parse 逐调用分派）。
  globalThis.__zwTtEvalArg = function (x) {
      var ttObj = globalThis.trustedTypes;
      var isScript = ttObj && typeof ttObj.isScript === 'function'
        ? function (v) { try { return ttObj.isScript(v); } catch (_eIs50) { return false; } }
        : null;
      var s;
      if (typeof x === 'string') {
        s = x;
        // TT enforce face——**只作用于 String 实参**（TrustedScript 免 face 含
        // default policy 调用，语料 default-policy 页 sink 断言仅 string 面
        // 触发；本 face 若放到三分支外会给 TS 面二次过门——首版钉测
        // FAIL:p4.throw/p3.called 同根因）。
        // fresh parse（非 _zwTtEnforced memo）：memo 刷新依赖微任务检查点，本引擎
        // 脚本间不清微任务——meta 位于 head 脚本之后的页面（语料 eval-csp-* 族
        // 形态）在同步 test() 求值窗内 memo 恒 stale-false（k29 全腿判读：
        // tt-block-eval/mutate/FC-throw 等页 TT face 静默旁路实证）。eval 门非
        // 热路径（worker OOM 域无关），逐调用解析经指纹缓存仅属性读成本。
        // https://html.spec.whatwg.org/multipage/semantics.html#meta-http-equiv-content-security-policy
        if (_zwTtParseCsp().enforced) {
          var def = ttObj && ttObj.defaultPolicy;
          // raw 回调寄存面与 _zwTtSinkValue 同口径（包装方法 undefined/null
          // 语义只归直调面）。
          var _evCbs = def && (def.__zwRawCbs || def);
          var fn = _evCbs && _evCbs.createScript;
          if (typeof fn !== 'function') {
            _zwTtViolation('eval', s, _zwTtParseCsp().enforcePolicy, 'enforce');
            throw new EvalError('Failed to compile a string: This document requires a TrustedScript for compilation.');
          }
          var out;
          var inS = s;
          try {
            out = fn.call(def, s, 'TrustedScript', 'eval');
          } catch (_eEv) {
            _zwTtViolation('eval', inS, _zwTtParseCsp().enforcePolicy, 'enforce');
            throw new EvalError('Failed to compile a string: default policy callback threw.');
          }
          if (out === undefined || out === null) {
            _zwTtViolation('eval', inS, _zwTtParseCsp().enforcePolicy, 'enforce');
            throw new EvalError('Failed to compile a string: default policy returned undefined.');
          }
          s = String(out);
          if (s !== inS) {
            // eval face 不可改写（mutate → block——语料
            // trusted-types-reporting-for-Window-eval modify 样本族）；violation
            // sample 载原输入（同 sink 管道口径——管道违规 sample 恒管道前原串）。
            _zwTtViolation('eval', inS, _zwTtParseCsp().enforcePolicy, 'enforce');
            throw new EvalError('Failed to compile a string: default policy modified the value.');
          }
        } else if (_zwTtParseCsp().reportOnly) {
          // 刀㊻（wptfyi-dom s10k47）：eval RO 监测臂——全包装 RO 装置臂自刀㉖
          // 在位，但 String 臂此前只认 enforce 面，RO 页违规从不派发（语料
          // trusted-types-eval-reporting-report-only「evaluating a string」F
          // 根因）。Report-Only require-trusted-types-for 页 String eval 走
          // _zwTtRoSink：默认政策存在则运行 createScript（成功返值继续、无违
          // 规）；无政策/返 null·undefined → disposition 'report' 违规（sample
          // 'eval|' 前缀＝sinkName 载体）＋原串透传，不执法不抛；回调异常原样
          // 传播（同 sink 管道 RO 臂口径）。后续 CSP face 照走——RO TT＋
          // enforcing script-src 无 'unsafe-eval' 页（reporting-check-report
          // sink-mismatch .sub.headers 双头）RO report 后仍经 CSP face 抛
          // EvalError，assert_throws_js 臂两全。
          // https://w3c.github.io/webappsec-trusted-types/dist/spec/#eval-dom
          s = _zwTtRoSink('TrustedScript', s, 'eval');
        }
      } else if (isScript && isScript(x)) {
        // WebIDL wrapper 边界 ToString（TrustedScript 恒有 __zwTtData 原串，
        // String() 经 toString 取原串）；免 TT face，仍落 CSP face。
        s = String(x);
      } else {
        // face ①：非串非 TS 原样透传（PerformEval 非串保恒语义）。
        return x;
      }
      // CSP face——决策两源：宿主 __zwCspEvalBlockedQuery（webview header 面，
      // eval-blocked 脚本才在场；无派发副作用）优先；meta 政策面兜底（runner
      // 语料宿主桥缺席——首见 script-src 无 'unsafe-eval' token 即 block，
      // 语料 eval-csp-* 判据面）。阻止 → 违规派发（sinkName 空 = 无 'eval|'
      // 前缀，directive 'script-src'）+ EvalError，代码不落体。
      var _evCspSt = _zwTtParseCsp();
      if ((typeof globalThis.__zwCspEvalBlockedQuery === 'function'
            && globalThis.__zwCspEvalBlockedQuery(s) === '1')
          || (_evCspSt.scriptSrcPolicy !== '' && !_evCspSt.scriptSrcHasUnsafeEval)) {
        // script-src 违规（blockedURI 'eval'——CSP §create-violation-for-global
        // 对 eval 编译面的占位；originalPolicy＝script-src 政策原文）。
        _zwTtViolation('', s, _evCspSt.scriptSrcPolicy, 'enforce', 'script-src', 'eval');
        throw new EvalError('EvalError: call to eval() blocked by CSP');
      }
      // 返值＝变换后串（或 face ①② 透传值）；编译交由调用点语法原生 eval。
      return s;
  };
  // 刀㊾ 装置语义（承刀㉙返修二「装配期不安装」边界）：meta 三臂路径旗标形
  // ＝globalThis.eval 恒原生（刀㉙ 包装版 enforced 页直接 eval 退化边界在本
  // 形闭合——Range 框架 strict 闭包 eval 域与 reporting-for-Window-eval 双
  // 实证面）；宿主 header-face 路径包装形（__zwEvalWrapInstalled 独立旗标，
  // 该路径 armed 不置——变换赶不上 exec 前装置时点，见 5.5 头注）。双入口：
  //   · __zwMaybeInstallEvalGate——TT enforced／meta-RO／meta script-src 无
  //     'unsafe-eval' 即旗标化（part03 script prepare 每脚本前调用——meta
  //     解析先于其后脚本执行，静态与动态注入 meta 均覆盖）；
  //   · __zwInstallEvalGate／__zwUninstallEvalGate——宿主显式装/卸（webview
  //     header 面 eval-blocked 脚本装、非 blocked 脚本卸——M2-s6 per-script
  //     语义；卸＝双旗清零＋包装复位）。
  // 已知边界（FIXME 挂账）：armed 页非 prepare 文本域（内联事件处理器/timer
  // 串/Function 体串内 eval、别名 eval var e=eval、eval 返串嵌套）不经变换
  // ＝失控面（语料零形——timer 串/内联处理器 eval/别名形 grep 零命中；
  // Function ctor 面＝Function Proxy 半边承载不变）；混合页（meta armed＋
  // header 包装并存）变换文本走门体、包装拦非变换域，无同案双派（门体不落
  // globalThis.eval）；无前置 script 元素的 eval（纯内联事件处理器 TT 页）不
  // 触发装置——语料无此形态；runner 多页进程若复用全局需按页复位（四邻腿
  // 电池 SET-IDENTICAL 实证门）；script_gen.rs 跨页 __zwEvalGateInstalled
  // 泄漏 FIXME 与刀㉙ 直接 eval 边界同根闭合于本形（installed 语义转旗标化）。
  globalThis.__zwInstallEvalGate = _zwEvalWrapInstall;
  globalThis.__zwMaybeInstallEvalGate = function () {
    if (globalThis.__zwEvalGateInstalled) return;
    // 返修三（k29 残 8F 实证）：三臂读 _zwTtParseCsp() 直调状态而非
    // _zwTtEnforced()/_zwRoActive() memo 版——runner 同步 attach 全页的单块内，
    // memo 首查（首 script prepare，meta 尚未 attach）缓存 false，同块内后续
    // 脚本 prepare 读 stale false；parseCsp 直调走 head meta 指纹比对，meta
    // attach 即重扫（指纹缓存使静态页零解析成本；装置钩子每脚本一次非热
    // 路径）。eval-reporting-no-unsafe-eval（兜底臂直调）翻绿与 permissive-csp/
    // no-default-policy（enforced 臂 memo）残红的对照为本判读的三证。
    var _miSt = _zwTtParseCsp();
    if (_miSt.enforced || _miSt.reportOnly
        || (_miSt.scriptSrcPolicy !== '' && !_miSt.scriptSrcHasUnsafeEval)) {
      _zwEvalArgArm();
    }
  };
  globalThis.__zwUninstallEvalGate = function () {
    if (globalThis.__zwEvalWrapInstalled) {
      globalThis.__zwEvalWrapInstalled = false;
      if (typeof globalThis.__zwNativeEval === 'function') {
        globalThis.eval = globalThis.__zwNativeEval;
      }
    }
    globalThis.__zwEvalGateInstalled = false;
    globalThis.__zwEvalArgArmed = false;
  };
  // 刀㊵→㊶薄包装（trusted-types eval-no-csp-no-tt 族）：TT API 已用页
  //（createPolicy 调用置 __zwTtApiUsed，part12）的 eval(TrustedScript) 集成——
  // spec TT §eval()-integration 在 TT 支持即生效，而全包装装置条件
  //（CSP enforced/RO/script-src 无 unsafe-eval）不覆盖 no-csp 页。薄包装仅
  // 一臂：isScript 实参 → ToString 落 native 编译（WebIDL 边界转换——native
  // eval 非串不转换恒原样返回对象）；**其余一切实参原样透传 native**（含
  // String——CSP/TT enforce face 只归全包装承载，薄装不接管）。
  // k29 返修二边界承袭：包装在位即退化直接 eval 声明落点——故装置窄条件＝
  // 页面实际调用过 createPolicy（dom 语料非 TT 页零装置零退化）；TT eval 语
  // 料页自身无直接 eval 声明断言面（值语义 eval '1+1' 经 native 尾调结果等价）。
  // 全包装已在位 → no-op（其 TS 臂已覆盖同一语义）。卸载面（宿主 per-script
  // __zwUninstallEvalGate）与薄装无交互——语料无卸载后 TS eval 形态，薄装不
  // 记入 uninstall 契约（FIXME: 宿主卸载域薄装复位待证据）。
  // https://w3c.github.io/webappsec-trusted-types/dist/spec/#eval-integration
  var _installEvalThinGate = function () {
    if (globalThis.__zwEvalGateInstalled) return;
    if (globalThis.__zwEvalThinInstalled) return;
    var nativeEval = globalThis.__zwNativeEval;
    if (typeof nativeEval !== 'function') return;
    var ttObj = globalThis.trustedTypes;
    if (!ttObj || typeof ttObj.isScript !== 'function') return;
    globalThis.__zwEvalThinInstalled = true;
    globalThis.eval = function (x) {
      try {
        if (ttObj.isScript(x)) return nativeEval.call(globalThis, String(x));
      } catch (_eTk41) {}
      return nativeEval.call(globalThis, x);
    };
  };
  globalThis.__zwMaybeInstallEvalThinGate = _installEvalThinGate;


  // ===== 5.6 eval 调用点文本扫描器（刀㊾ s10k50：arg-transform 变换器）=====
  // 文本级守卫集（非解析器——管线无 AST，见 S1 评估 2026-11-27）：跳过行/块
  // 注释、单双引号串（转义）、模板串（转义＋${} 花括号深度；插值内串跳过、
  // 反引号→整文本放弃）、正则字面量（前显著字符启发：(,=:[!&|?{};+-*%^<>~
  // 后及起点判 regex，')' 后判除号；`return\n/re/` 形误判除号→只引发后续
  // skip/放弃，不改写不破坏）。调用点＝裸 `eval` 标识符（**紧邻**前字符非
  // `.`/$/字词——window.eval 别名与 myeval 后缀排除、换行/空格复位邻接；
  // `c ? eval(a) : b` 三元分支仍直接 eval）＋次一非空白 `(`（直接形）或 `?.`
  // 后 `(`（可选调用形＝间接 eval，语料 18 处/9 文件；其余间接形 call/apply/
  // 括号索引语料零形＝边界挂账）。改写＝括号
  // 内插 `__zwTtEvalArg(`（实参片递归同规则，初始 prev='('——正则上下文正
  // 确；深度上限 8）；已插 `__zwTtEvalArg` 的重扫（re-prepare 域）跳过防双
  // 包双派。不变式：输出除确认调用点插入外与输入逐字节相同；任何异常（未
  // 终结串/模板/括号失衡/深度超限）返原文零变化＝armed 页该文本退未门控域
  //（FIXME 边界挂账；语料 eval 页全裸形实证扫描无异常路径）。多实参整片包
  // 裹＝门取首参、native 丢余参（与旧包装 function(x) 同口径）；空参
  // `eval()` → 门收 undefined → face ① 透传。
  globalThis.__zwEvalArgTransform = function (text) {
    try {
      if (typeof text !== 'string' || text.indexOf('eval') < 0) return text;
      return _zwEvalArgScan(text, '', 0);
    } catch (_eTx50) {
      return text;
    }
  };
  var _zwEvalArgRegexOk = function (prev) {
    return prev === '' || '(,=:[!&|?{};+-*%^<>~'.indexOf(prev) >= 0;
  };
  var _zwEvalArgSkipQuoted = function (src, i) {
    // i 指向开引号；返闭合引号 index（转义跳过；未终结/行尾断开返 -1）。
    var c = src.charAt(i);
    var q = i + 1;
    var n = src.length;
    while (q < n) {
      var qc = src.charAt(q);
      if (qc === '\\') { q += 2; continue; }
      if (qc === c) return q;
      if (qc === '\n') return -1;
      q += 1;
    }
    return -1;
  };
  var _zwEvalArgSkipTemplate = function (src, i) {
    // i 指向反引号；返闭合反引号 index。跳过转义；${} 插值体只跟踪花括号
    // 深度（其内引号串跳过、反引号返 -1 放弃——语料零插值嵌套模板形）。
    var t = i + 1;
    var n = src.length;
    var bd = 0;
    while (t < n) {
      var tc = src.charAt(t);
      if (tc === '\\') { t += 2; continue; }
      if (bd > 0) {
        if (tc === '{') { bd += 1; t += 1; continue; }
        if (tc === '}') { bd -= 1; t += 1; continue; }
        if (tc === '"' || tc === "'") {
          var sq = _zwEvalArgSkipQuoted(src, t);
          if (sq < 0) return -1;
          t = sq + 1;
          continue;
        }
        if (tc === '`') return -1;
        t += 1;
        continue;
      }
      if (tc === '`') return t;
      if (tc === '$' && t + 1 < n && src.charAt(t + 1) === '{') {
        bd = 1;
        t += 2;
        continue;
      }
      t += 1;
    }
    return -1;
  };
  var _zwEvalArgSkipComment = function (src, i) {
    // i 指向 '/'；返注释终结后继 index（行注释＝换行符位、块注释＝'*/' 后
    // 一位）；未终结返 -1。
    var c2 = src.charAt(i + 1);
    if (c2 === '/') {
      var j = src.indexOf('\n', i);
      return j < 0 ? src.length : j;   // 行注释至 EOF 合法终结
    }
    if (c2 === '*') {
      var k = src.indexOf('*/', i + 2);
      return k < 0 ? -1 : k + 2;
    }
    return -1;
  };
  var _zwEvalArgMatchParen = function (src, open) {
    // open 指向 '('；返平衡 ')' 的 index；串/模板/注释内括号不计；失衡/异常
    // 返 -1。
    var d = 0;
    var i = open;
    var n = src.length;
    while (i < n) {
      var c = src.charAt(i);
      if (c === '"' || c === "'") {
        var q = _zwEvalArgSkipQuoted(src, i);
        if (q < 0) return -1;
        i = q + 1;
        continue;
      }
      if (c === '`') {
        var t = _zwEvalArgSkipTemplate(src, i);
        if (t < 0) return -1;
        i = t + 1;
        continue;
      }
      if (c === '/') {
        var cm = _zwEvalArgSkipComment(src, i);
        if (cm >= 0) { i = cm; continue; }
      }
      if (c === '(') { d += 1; i += 1; continue; }
      if (c === ')') {
        d -= 1;
        if (d === 0) return i;
        i += 1;
        continue;
      }
      i += 1;
    }
    return -1;
  };
  var _zwEvalArgScan = function (src, initPrev, depth) {
    if (depth > 8) return src;
    var out = '';
    var i = 0;
    var n = src.length;
    var sig = initPrev;   // 前显著字符（正则判定用，空白不刷新）
    var prevCh = initPrev; // 紧邻前字符（字词邻接判定用，逐字符刷新）
    while (i < n) {
      var c = src.charAt(i);
      if ((c === '"' || c === "'")) {
        var q = _zwEvalArgSkipQuoted(src, i);
        if (q < 0) return src;
        out += src.slice(i, q + 1);
        sig = c;
        prevCh = c;
        i = q + 1;
        continue;
      }
      if (c === '`') {
        var t = _zwEvalArgSkipTemplate(src, i);
        if (t < 0) return src;
        out += src.slice(i, t + 1);
        sig = '`';
        prevCh = '`';
        i = t + 1;
        continue;
      }
      if (c === '/') {
        var cm = _zwEvalArgSkipComment(src, i);
        if (cm >= 0) {
          out += src.slice(i, cm);
          prevCh = src.charAt(cm - 1);
          i = cm;
          continue;
        }
        if (_zwEvalArgRegexOk(sig)) {
          var r = i + 1;
          var rdone = -1;
          while (r < n) {
            var rc = src.charAt(r);
            if (rc === '\\') { r += 2; continue; }
            if (rc === '/') { rdone = r; break; }
            if (rc === '\n') break;
            r += 1;
          }
          if (rdone < 0) return src;
          out += src.slice(i, rdone + 1);
          sig = ')';
          prevCh = '/';
          i = rdone + 1;
          continue;
        }
      }
      if (src.slice(i, i + 4) === 'eval'
          && prevCh !== '.' && prevCh !== '$'
          && !/[A-Za-z0-9_$]/.test(prevCh)) {
        var p = i + 4;
        while (p < n && /\s/.test(src.charAt(p))) p += 1;
        // 调用开括号两形：`eval(`（直接）与 `eval?.(`（可选调用——ECMA-262 可
        // 选调用非 IdentifierReference callee＝间接 eval，WebIDL TT 门经 global
        // 绑定值拦（旧刀㉙ 包装运行时接管）；变换后 `eval?.(__zwTtEvalArg(x))`
        // native 仍经可选调用＝全局作用域间接语义保形）。`?.` 后非 `(`（如
        // `eval?.5:0.3` 条件小数形）不匹配。
        var q2 = -1;
        if (p < n && src.charAt(p) === '(') {
          q2 = p;
        } else if (p + 1 < n && src.charAt(p) === '?' && src.charAt(p + 1) === '.') {
          var o = p + 2;
          while (o < n && /\s/.test(src.charAt(o))) o += 1;
          if (o < n && src.charAt(o) === '(') q2 = o;
        }
        if (q2 >= 0 && !src.startsWith('__zwTtEvalArg', q2 + 1)) {
          var close = _zwEvalArgMatchParen(src, q2);
          if (close < 0) return src;
          if (depth >= 8) return src;
          var inner = _zwEvalArgScan(src.slice(q2 + 1, close), '(', depth + 1);
          out += src.slice(i, q2 + 1) + '__zwTtEvalArg(' + inner + ')';
          sig = ')';
          prevCh = ')';
          i = close;   // 闭 ')' 由下一轮普通输出（闭合双括号成形）
          continue;
        }
      }
      if (!/\s/.test(c)) sig = c;
      prevCh = c;
      out += c;
      i += 1;
    }
    return out;
  };


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
      // 刀㊷：空体且无 default policy → 静默不执行（零违规）——block-text spot 面
      // 判据（innerHTML TypeError 中止后 append 的空体脚本 + 无政策 = 零违规零
      // 执行）；有政策时空体仍进转换（script-enforcement-005 空体救援判据，
      // policy 以 ('',type,sink) 调用）。非空无政策照走管道（违规+阻断，
      // script-enforcement-001 判据族）。
      // https://w3c.github.io/webappsec-trusted-types/dist/spec/#enforcement-in-scripts
      if (String(text) === ''
          && !(globalThis.trustedTypes && globalThis.trustedTypes.defaultPolicy)) {
        return '';
      }
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
              // 刀㉖：RO 面同管道（监测臂处置，不执法）。
              var enforced = _zwTtEnforced() || _zwRoActive();
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
  // 刀㉖：RO monitoring 活性查询导出（part04/05 等 IDL sink 前守卫扩臂消费——
  // 同 _zwTtEnforced 的导出契约形）。
  Object.defineProperty(globalThis, '__zwTtRoActive', { value: _zwRoActive, writable: true, configurable: true });
  // 刀㉗：违规派发桥（part12 createPolicy 名单 block 面消费——同 task 队列
  // 实现保 setTimeout FIFO 单源；sink 面 sinkName 传空串即无前缀体）。
  Object.defineProperty(globalThis, '__zwTtViolation', { value: _zwTtViolation, writable: true, configurable: true });
  // 返修②失效钩：HTML sink 写面完成解析/插入后调用，置 enforcement 脏位——
  // 下一次守卫查询必重算指纹（动态 meta 注入 face 013 同块内生效语义）。memo
  // 只合并无变异间隔的重复查询，不吞注入。
  Object.defineProperty(globalThis, '__zwTtCspDirty', { value: function () { _zwTtEnfDirty = true; _zwRoDirty = true; }, writable: true, configurable: true });
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
  // 刀㊷（s10k42）：JS 侧融合遍历导出——part03 prepare 门 text 回落读消费
  //（enforcement 页 sel 形代理 textContent 读的 pending 窗口陈旧快照面，见
  // part03 消费点注记）。§6.2b 同源（childNodes 直走，零 trap 属性读）。
  Object.defineProperty(globalThis, '__zwTtChildTextFused', { value: _zwTtChildTextFused, writable: true, configurable: true });
  // 刀㊸（wptfyi-dom s10k43）：per-document policy 名册通道——spec 每 Document
  // 独立 TrustedTypePolicyFactory（createPolicy 名单/重名唯一性 per-factory，
  // #dom-trustedtypepolicyfactory-createpolicy 的 factory 概念域）。单一 sandbox
  // 全局近似：主文档用 part12 主名册（返回 null 由调用方回落），帧文档用
  // per-doc WeakMap 名册——帧上下文键 = `__zwActiveIframeDoc` 槽位（R206 脚本
  // 执行窗 / s339 注册体执行窗 / 刀㊸ 帧消息派发窗三处在场）。返回名册数组
  //（可变引用，调用方 dup 检查 + push 同引用）。
  var _zwTtNamesByDoc43 = typeof WeakMap === 'function' ? new WeakMap() : null;
  var _zwTtActiveNames43 = function () {
    var ad = null;
    try { ad = globalThis.__zwActiveIframeDoc || null; } catch (_e43ad) { ad = null; }
    if (ad && _zwTtNamesByDoc43 && typeof ad === 'object') {
      var arr = _zwTtNamesByDoc43.get(ad);
      if (!arr) { arr = []; _zwTtNamesByDoc43.set(ad, arr); }
      return arr;
    }
    return null;
  };
  Object.defineProperty(globalThis, '__zwTtActivePolicyNames', { value: _zwTtActiveNames43, writable: true, configurable: true });
  // 刀㊹b（wptfyi-dom s10k44b）face P 落册口——part01 finish 管线对**网络交付**
  // 帧文档调用（spec CSP §policy-containers 网络交付自有容器）。记录与
  // _zwTtParseCsp 产物同构的中性态（无 CSP 交付：无 enforcement／无名单／无
  // script-src／无 RO——宿主 fetch 桥未 wire 帧响应 CSP 头，FIXME 待头 wire 后
  // 按头解析升级）。本地 scheme 帧（about:blank/srcdoc/data:/blob:）不落册＝
  // 继承父容器（全局读零变化）。
  var _zwTtFrameOwnCsp44b = function (doc) {
    if (!doc || !_zwTtPerDocCsp44b) return;
    _zwTtPerDocCsp44b.set(doc, {
      enforced: false, hasList: false, lists: [], allowDuplicates: false,
      reportOnly: false, roPolicy: '', enforcePolicy: '', listPolicy: '',
      scriptSrcPolicy: '', scriptSrcHasUnsafeEval: false,
    });
  };
  Object.defineProperty(globalThis, '__zwTtFrameOwnCsp44b', { value: _zwTtFrameOwnCsp44b, writable: true, configurable: true });
})();
