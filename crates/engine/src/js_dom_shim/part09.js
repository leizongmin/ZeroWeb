// ============================================================================
// part09 — SMIL 文档级时间线引擎（svg-compat-v2 goal M1/M2，2026-11-07）。
//
// 范围（控制面 docs/goal/svg-compat-v2.md）：M1 = B1 时钟值文法 + B2
// interval 模型首批；M2 = B3 seek 事件语义 + B4 syncbase 同步基 + B5
// 事件基/动画事件时基——
//   ① clock-value 严格文法：Full/Partial-clock-val（分/秒必须两位数且 <60、
//      分数点后至少一位数字、小时两位起不限位）+ Timecount-val（h/min/s/ms，
//      缺省 s）——B1 非法矩阵（越界分秒/分数缺数字/前导零/缺字段）逐条
//      定谳（WPT smil-clock-value-parsing 7 合法 + 11 非法定谳面）。
//   ② begin/end 列表：offset-value（± clock-val）+ indefinite + syncbase
//      （id.begin / id.end 带可选 ±offset，id 内 '.' 用 '\' 转义——B4 接
//      实例源）+ repeat-value（id.repeat(k)，k ≥ 1 迭代起点静态传播——B5）
//      + eventbase（裸事件名 / id.event——B5 接派发通知实例源）；wallclock /
//      accessKey 语法识别但实例不解析（FIXME(svg-compat-v2 后续)：无
//      wallclock 时基与 accessKey 派发面——WPT SVGAnimationElement-
//      exceptions 的 begin="foo.begin" 语义保持：查无目标 → 零实例 →
//      getStartTime InvalidStateError）。end 侧 event 形态识别但零实例
//     （FIXME(svg-compat-v2 B5+)：dynEnds 实例集——现势语料仅 begin 侧）。
//   ③ interval 解析：instance 升序首个 end > t 的 interval 为 current——
//      WPT SVGAnimationElement-getStartTime 定谳（gap 期取下一 interval、
//      fill=freeze 不延长 current、begin="1s; 3s" 在 t=4 后恒
//      InvalidStateError）。interval end = min(end 实例, begin + activeDur)
//      （activeDur = repeatCount×dur / repeatDur，双给取 min）。
//   ④ 采样：calcMode=linear（缺省）from/to、from/by、by、to、values（均匀
//      keyTimes 线段插值）；fill=freeze 冻结于 activeEnd（activeDur 非 dur
//      整数倍时 simpleTime = activeDur mod dur）。calcMode=discrete/
//      spline/additive/accumulate = B6（FIXME）；keyTimes/keySplines = B6。
//      长度 token 按 px + 物理单位换算进 user units（part03 S2a 常量表
//      同源复制——闭包不可反向引用）；%/em/ex 上下文单位 = 挂账 #3 域
//      （不采样 → 维持 base 行为）。
//   ⑤ SVGSVGElement pauseAnimations/unpauseAnimations/getCurrentTime/
//      setCurrentTime 文档级共享时间线（spec SVG2 struct.html
//      §InterfaceSVGSVGElement——part03 M2 S3 per-element stub 的合流兑现，
//      本文件加载序在后无条件覆写；currentTranslate/currentScale/deselectAll
//      留 part03 不动）。文档时间线起点 = window load（part06 派发点挂钩
//      _zwSmilDocStart）；load 前 SMIL API 读恒 0（WPT
//      SVGAnimationElement-getStartTime 的 pre-load pause 断言
//      getCurrentTime()===0 定谳）；pre-load seek 落 frozenDoc、start 时结转。
//   ⑥ SVGAnimationElement getStartTime/getSimpleDuration 真值（spec SVG2
//      animate.html §InterfaceSVGAnimationElement——part03 M4 异常 stub 覆写，
//      exceptions 案语义保持：零 instance → InvalidStateError、无/非有限
//      simple duration → NotSupportedError）+ beginElement/beginElementAt
//      动态 instance（float 校验保持；restart=never/whenNotActive 门控）。
//   ⑦ 事件模型（B3，2026-11-07 第二批）：转移扫描——每元素比较上一观测
//      态 {ib, began, ended} 与当前 active interval（b ≤ t < e），差分派发：
//      seek 越过的 interval 零事件（seeking-events-1 定谳）；曾 began 的
//      interval 失活（自然 end / seek 出界 / seek 回退 / 被替代）→
//      endEvent；seek 落点所在 interval（含回退重入已 ended interval）→
//      beginEvent；同 interval 内 seek 不派 repeatEvent（seeking-events-5/6
//      定谳；repeatEvent 自然播放面 = B5 FIXME）。派发 task 粒度
//      （setTimeout ≤50ms 轮询 + setCurrentTime 的一次性 seek 扫描——
//      learnings #30 禁 microtask；同步派发会早于 seek 后才挂的
//      EventWatcher）。onbegin/onend/onrepeat 处理器 IDL（JS 设值 + 内容
//      属性 inline 编译）经 _fireSmilEvent 错位名映射直调（onendEvent 非
//      处理器名恒不触发——event-listeners 定谳）；addEventListener 面走
//      dispatchEvent。first-interval 剪枝：end ≤ 文档起点的 interval 非
//      real interval，冻结采样不贡献（first-interval-in-the-past-
//      dont-contribute 定谳）。
//   ⑧ animVal 反射消费：globalThis._zwSmilAnimValue（SVGLength 只读面 →
//      {ut, vu} 态）/ _zwSmilAnimNumber（SVGAnimatedNumber.animVal → 数值）
//      单点，part03 S2a cur()/animVal getter 挂钩；动画不写 DOM 属性
//      （animVal-basics 定谳：baseVal 恒 base 值）。多动画同 (target, attr)
//      文档序后者胜（sandwich 近似——additive 合成 = B6 FIXME）。
//   ⑨ syncbase 同步基（B4，2026-11-07 第三批）：interval 链按文档级
//      时间序离散事件模拟求值——每元素 begin/end 实例集 = 静态 offset +
//      beginElement 动态实例 + syncbase 传播；目标 interval 一经创建即向
//      依赖者传播 b/e（SMIL #Timing-SyncbaseValueSemantics）；自引用
//      （0s;anim.end 自续链）与互指环（a↔b）按时间序自然展开，零实例源
//      的环恒死锁（SMIL 未解析口径）。传播的 end 实例截断已建 interval
//      （dependent-end 面）；剪枝 interval（e ≤ 0 文档起点前）不传播
//      （first-interval dont-contribute 定谳）。静态/dyn 实例元素与 M1
//      口径逐字节同链（首个 end > t + min(end 实例, b+actDur)——零回归面）。
//      链缓存 per (doc, t)（beginElement 作废；属性变更靠泵轮询下 tick
//      重算，同 tick 变更窗口 = 已知近似 FIXME(svg-compat-v2 B4)）。
//   ⑩ 事件基 + repeat 时基（B5，2026-11-07 第四批）：eventbase 动态实例
//      源——part05 派发尾通知（_zwSmilNotifyEvent）对现行文档全量扫描
//      begin 列表事件形态，base 匹配（id 名目标 / 缺省 = 目标元素，非 SVG
//      元素允许）即写 dynBegins（SMIL #Eventvalue semantics；每次通知重建
//      = 移除/重建自动反映，条件门元素不入实例源）。repeatEvent 自然播放
//      面入窗口重放（迭代边界 b + k·sd，k ≥ 1 且 k·sd < e，仅当前跟踪
//      interval——seek 不派，seeking-events-5/6 恒持）；repeat(k) 值为静态
//      传播（k=0 无实例——repeatn-remove-add 定谳）。restart 门入模拟
//      （never = 首个 interval 后恒忽略 / whenNotActive = 活跃期实例忽略，
//      spec #Timing-RestartMechanisms）。条件处理门：祖先链 systemLanguage
//      判否（BCP47 前缀）或 requiredExtensions 非空 → 元素不进时间线
//      （无 interval/无事件/无采样——conditional-processing-02 定谳；
//      requiredFeatures SVG2 已移除恒过；switch 首可接受子选择 = 渲染域
//      不在此）。onbegin/onend/onrepeat 处理器改经 part05 错位名映射统一
//      派发（直调面移除——泛型 on<type> 查找对 beginEvent→onbegin 别名 +
//      裸 begin/end/repeat 跳过，custom-events/event-listeners 定谳）。
//
// 作用域：part07 末尾已闭合外层 IIFE，本文件顶层作用域（part08 先例）；
// 引用面仅 globalThis 契约（SVGSVGElement/SVGAnimationElement prototype、
// __zwMakeTrustedEvent、setTimeout、WeakMap、DOMException、Date.now）。
// per-element 键形态镜像 part03 _zwSvgAnimKey（__zwSelector+\0+__zwHandle）。
// spec：SMIL Animation REC
//   https://www.w3.org/TR/2001/REC-smil-animation-20010904/
//   （#Timing-ClockValueSyntax 文法 / #Timing-BeginEnd-LC-Start 插入 /
//    #Timing-ComputingActiveDuration）；SVG2 animate
//   https://www.w3.org/TR/SVG2/animate.html
// ============================================================================

(function () {
  'use strict';

  var _animLocalNames = { animate: 1, set: 1, animatetransform: 1, animatemotion: 1 };
  var _docTimelines = new WeakMap();
  var _elStateCache = {};

  var _domErr = function (msg, name) {
    return new (globalThis.DOMException || Error)(msg, name);
  };

  // per-element 键（part03 _zwSvgAnimKey 同形态镜像——拓扑同源才能命中同一
  // proxy 家族缓存槽；#32 跨族 identity 恒败，此处只做键一致性不做对象比较）。
  var _elKey = function (el) {
    try {
      return el.__zwSelector + '\u0000' + (el.__zwHandle || '') + '\u0000smil';
    } catch (_eK) { return null; }
  };

  // ── 时间线状态（per document）────────────────────────────────────────
  // { started, paused, frozenDoc, baseDoc, baseReal, tickTimer, seekTimer }
  var _timelineFor = function (doc) {
    var st = null;
    try { st = _docTimelines.get(doc); } catch (_eG) { st = null; }
    if (!st) {
      st = { started: false, paused: false, frozenDoc: 0, baseDoc: 0, baseReal: 0,
        tickTimer: 0, seekTimer: 0, tickEverRan: false, smilSeen: false };
      try { _docTimelines.set(doc, st); } catch (_eS) {}
    }
    return st;
  };

  var _currentTimeOf = function (st) {
    // 未 started：frozenDoc 即文档时（初始 0；pre-load seek 经 setCurrentTime
    // 落此——WPT seeking-events 的 pause+seek 驱动面与 getStartTime 案的
    // pre-load getCurrentTime()===0 断言两满足）。
    if (!st.started) return st.frozenDoc;
    if (st.paused) return st.frozenDoc;
    return st.baseDoc + (Date.now() - st.baseReal) / 1000;
  };

  // ── clock-value 文法（SMIL Animation #Timing-ClockValueSyntax + Blink
  //    语义定谳面——smil-clock-value-parsing 18 用例逐条）────────────────
  // Full/Partial：小时 ≥1 位不限长，分/秒必须恰两位且 <60，分数 '.' 后
  // ≥1 位；Timecount：数字 + 缺省 s。负号在 offset 层处理。
  var _parseClockValue = function (s) {
    var str = String(s).replace(/^\s+|\s+$/g, '');
    if (!str) return null;
    // Timecount-val（数字 + 可选分数 + 单位缺省 s）
    var m = /^(\d+)(?:\.(\d+))?(h|min|ms|s|)$/.exec(str);
    if (m) {
      var v = Number(m[1]) + (m[2] ? Number('0.' + m[2]) : 0);
      var u = m[3];
      if (u === 'h') return v * 3600;
      if (u === 'min') return v * 60;
      if (u === 'ms') return v / 1000;
      return v;
    }
    // Full/Partial-clock-val：分/秒严格两位（前导零规则——'5:30'/'9.9' 非法）
    m = /^(\d+):(\d{2}):(\d{2})(?:\.(\d+))?$/.exec(str);
    if (m) {
      var mins = Number(m[2]);
      var secs = Number(m[3]);
      if (mins > 59 || secs > 59) return null;
      return Number(m[1]) * 3600 + mins * 60 + secs + (m[4] ? Number('0.' + m[4]) : 0);
    }
    m = /^(\d{2}):(\d{2})(?:\.(\d+))?$/.exec(str);
    if (m) {
      var m2 = Number(m[1]);
      var s2 = Number(m[2]);
      if (m2 > 59 || s2 > 59) return null;
      return m2 * 60 + s2 + (m[3] ? Number('0.' + m[3]) : 0);
    }
    return null;
  };

  // offset-value：± clock-value。'indefinite' → null（特判由调用方区分）。
  var _parseOffsetValue = function (s) {
    var str = String(s).replace(/^\s+|\s+$/g, '');
    var sign = 1;
    if (str.charAt(0) === '+') str = str.slice(1);
    else if (str.charAt(0) === '-') { sign = -1; str = str.slice(1); }
    var v = _parseClockValue(str);
    return v == null ? null : sign * v;
  };

  // ── 时序 specifier 解析（SMIL #Timing-ParsingTimingSpecifiers）──────
  // 单段 → spec：{ kind:'offset', t } | { kind:'syncbase', id, base, off }
  // | { kind:'event', id, name, off }（repeat(k)/事件名——B5 接实例源：
  // repeat 静态传播、事件名走派发通知）
  // | null（indefinite / 无法识别 → 无实例）。id 内 '.' 以 '\' 转义
  //（syncbase-escaped-dots 定谳面），首个未转义 '.' 为 id 分隔符。
  var _parseTimingSpec = function (seg) {
    var s = String(seg).replace(/^\s+|\s+$/g, '');
    if (!s || s === 'indefinite') return null;
    var off0 = _parseOffsetValue(s);
    if (off0 != null) return { kind: 'offset', t: off0 };
    // id 扫描：'\.' → 字面点；未转义 '.' → 进入 base/事件段
    var id = '';
    var rest = '';
    for (var i = 0; i < s.length; i++) {
      var ch = s.charAt(i);
      if (ch === '\\' && s.charAt(i + 1) === '.') { id += '.'; i++; continue; }
      if (ch === '.') { rest = s.slice(i + 1); break; }
      id += ch;
    }
    if (rest) {
      var m = /^(begin|end|repeat\(\d+\)|[a-zA-Z][a-zA-Z0-9_-]*)(?:\s*([+-])\s*(\S+))?$/.exec(rest);
      if (!m) return null;
      var off = 0;
      if (m[2]) {
        off = _parseOffsetValue(m[2] + m[3]);
        if (off == null) return null;
      }
      if (m[1] === 'begin' || m[1] === 'end') {
        return { kind: 'syncbase', id: id, base: m[1], off: off };
      }
      return { kind: 'event', id: id, name: m[1], off: off };   // repeat(n)/事件名 = B5 实例源
    }
    // 无分隔符：裸事件名（B5 实例源——base = 目标元素）
    if (/^[a-zA-Z][a-zA-Z0-9_-]*$/.test(s)) return { kind: 'event', id: null, name: s, off: 0 };
    return null;
  };

  var _parseTimingSpecs = function (raw) {
    var out = [];
    if (raw == null) return out;
    var parts = String(raw).split(';');
    for (var i = 0; i < parts.length; i++) {
      var sp = _parseTimingSpec(parts[i]);
      if (sp) out.push(sp);
    }
    return out;
  };

  // dur → { kind: 'finite'|'indef'|'invalid', s }。absent/indefinite/media →
  // indef（恒活采样 from 常值）；非法 → invalid（B1 口径：整体无动画）。
  var _parseDur = function (animEl) {
    var raw = null;
    try { raw = animEl.getAttribute('dur'); } catch (_eG) { raw = null; }
    if (raw == null || raw === '') return { kind: 'indef', s: 0 };
    var str = String(raw).replace(/^\s+|\s+$/g, '');
    if (str === 'indefinite' || str === 'media') return { kind: 'indef', s: 0 };
    var v = _parseClockValue(str);
    if (v == null || v <= 0) return { kind: 'invalid', s: 0 };
    return { kind: 'finite', s: v };
  };

  // repeatCount/repeatDur → active duration 秒数（Infinity = 不定）。
  // spec #Timing-ComputingActiveDuration：缺省 = simple duration（单次）；
  // repeatCount=indefinite → 不定；双给取 min。
  var _activeDuration = function (animEl, simpleDur) {
    var rc = null, rd = null;
    try { rc = animEl.getAttribute('repeatCount'); } catch (_eRc) {}
    try { rd = animEl.getAttribute('repeatDur'); } catch (_eRd) {}
    var byCount = simpleDur;
    var rcStr = rc == null ? '' : String(rc).replace(/^\s+|\s+$/g, '');
    if (rcStr === 'indefinite') byCount = Infinity;
    else if (rcStr !== '') {
      var rcN = Number(rcStr);
      if (isFinite(rcN) && rcN > 0) byCount = rcN * simpleDur;
    }
    var byDur = Infinity;
    var rdV = rd == null ? null : _parseClockValue(String(rd));
    if (rdV != null && rdV > 0) byDur = rdV;
    return Math.min(byCount, byDur);
  };

  // ── instance 集 + current interval ──────────────────────────────────
  // 动态 instance（beginElement/beginElementAt）存 per-element 状态槽。
  var _elementState = function (animEl) {
    var k = _elKey(animEl);
    if (!k) return { dynBegins: [] };
    if (!_elStateCache[k]) _elStateCache[k] = { dynBegins: [], ib: null, ie: null, began: false, ended: false, seen: false, lastT: null };
    return _elStateCache[k];
  };

  // ── B5：条件处理门（spec SVG2 struct.html #ConditionalProcessing）────
  // 祖先链任一条件属性判否 → 元素不进文档级时间线（无 interval/无事件/
  // 无采样——conditional-processing-02 定谳面）。requiredFeatures SVG2 已
  // 移除恒过；requiredExtensions 空串过、非空判否（引擎无命名空间扩展）；
  // systemLanguage 按 RFC 4647 basic filtering（tag 与用户语言 '-' 边界
  // 前缀互配）对 navigator.languages 逐 tag 匹配。switch 首可接受子选择
  // 是渲染域语义不在此（FIXME(svg-compat-v2 渲染协同)）。
  var _condOK = function (el) {
    var n = el;
    var guard = 0;
    while (n && guard++ < 200) {
      var re = null;
      var sl = null;
      try {
        if (typeof n.getAttribute === 'function') {
          re = n.getAttribute('requiredExtensions');
          sl = n.getAttribute('systemLanguage');
        }
      } catch (_eC) { re = null; sl = null; }
      if (re != null && String(re) !== '') return false;
      if (sl != null && String(sl) !== '') {
        var nav = [];
        try {
          nav = (globalThis.navigator && navigator.languages && navigator.languages.length)
            ? navigator.languages
            : (globalThis.navigator && navigator.language ? [navigator.language] : []);
        } catch (_eN) { nav = []; }
        var tags = String(sl).split(',');
        var hit = false;
        for (var i = 0; i < tags.length && !hit; i++) {
          var tv = String(tags[i]).replace(/^\s+|\s+$/g, '').toLowerCase();
          if (!tv) continue;
          for (var j = 0; j < nav.length; j++) {
            var nv = String(nav[j]).toLowerCase();
            if (nv === tv || nv.indexOf(tv + '-') === 0 || tv.indexOf(nv + '-') === 0) {
              hit = true;
              break;
            }
          }
        }
        if (!hit) return false;
      }
      try { n = n.parentNode; } catch (_eP) { n = null; }
    }
    return true;
  };

  // ── B4：interval 链 = 文档级时间序离散事件模拟 ─────────────────────
  // 每元素记录：pendB/pendE（升序实例集，传播可增长）、ib（pendB 消费
  // 指针）、ivs（按 b 升序已建 interval，含 e ≤ 0 剪枝标记）。传播值恒 ≥
  // 当前全局最小事件时刻，sorted-splice 插入不会落到 ib 之前（指针安全）。
  var _insertSorted = function (arr, v) {
    var i = 0;
    while (i < arr.length && arr[i] < v) i++;
    if (arr[i] === v) return;    // 同刻实例去重（cyclic-syncbase-events 的
    arr.splice(i, 0, v);         // c.end 与 a.begin 同刻双源）
  };

  var _docChains = function (doc, t) {
    var st = _timelineFor(doc);
    if (st.chainCache && st.chainCache.t === t) return st.chainCache.map;
    var els = _scanDoc(doc);
    var recs = [];
    var deps = {};   // 目标 id → [{ rec(依赖者), asBegin, timeKind, off }]——
    // 键为引用目标的 id，每查询从现行文档重建（syncbase-remove-add 的
    // 移除/重建自动重挂；查无目标 = 无实例源，exceptions 案口径）。
    for (var i = 0; i < els.length; i++) {
      var el = els[i];
      if (!_condOK(el)) continue;   // 条件处理判否 → 不建 rec（无链/无采样）
      var d = _parseDur(el);
      var sd = d.kind === 'finite' ? d.s : Infinity;
      var actDur = d.kind === 'finite' ? _activeDuration(el, sd) : Infinity;
      var rawB = null;
      var rawE = null;
      try { rawB = el.getAttribute('begin'); } catch (_eB) { rawB = null; }
      try { rawE = el.getAttribute('end'); } catch (_eE) { rawE = null; }
      var bSpecs = _parseTimingSpecs(rawB);
      var eSpecs = _parseTimingSpecs(rawE);
      var key = _elKey(el);
      var rec = {
        key: key, sd: sd, actDur: actDur, dead: d.kind === 'invalid',
        pendB: [], pendE: [], ib: 0, ivs: [], ivsBeyond: 0, id: '',
        restart: '', startedN: 0
      };
      try { rec.id = String(el.getAttribute('id') || ''); } catch (_eI) { rec.id = ''; }
      try { rec.restart = String(el.getAttribute('restart') || ''); } catch (_eRt) { rec.restart = ''; }
      // begin 属性缺省/空白才回落 '0s'（spec #Timing-DefaultBegin）——
      // 属性在但零实例（indefinite / 查无目标的 syncbase / 查无目标的
      // eventbase）保持无实例（SVGAnimationElement-exceptions 定谳面不可
      // 回退）。
      if (!bSpecs.length &&
          String(rawB == null ? '' : rawB).replace(/^\s+|\s+$/g, '') === '') {
        bSpecs = [{ kind: 'offset', t: 0 }];
      }
      var j;
      for (j = 0; j < bSpecs.length; j++) {
        var sp = bSpecs[j];
        if (sp.kind === 'offset') rec.pendB.push(sp.t);
        else if (sp.kind === 'syncbase') {
          var slotB = deps[sp.id] || (deps[sp.id] = []);
          slotB.push({ rec: rec, asBegin: true, timeKind: sp.base, off: sp.off });
        } else if (sp.kind === 'event') {
          // repeat(k) 值 = 静态传播（B5）：k 次迭代起点 = b + k·sd，迭代
          // 存在性在传播时判（k·sd < e − b）；k = 0 无实例（repeatn-remove-
          // add 定谳——Chromium/Firefox 首迭代起点非 repeat 事件源，SMIL
          // REC 未明文 k=0）；id 缺省自引用（SMIL repeat-value 语义）。
          var rm = /^repeat\((\d+)\)$/.exec(sp.name);
          if (rm && Number(rm[1]) >= 1) {
            var repTarget = sp.id || rec.id;
            if (repTarget) {
              var slotR = deps[repTarget] || (deps[repTarget] = []);
              slotR.push({ rec: rec, asBegin: true, timeKind: 'repeat',
                repK: Number(rm[1]), off: sp.off });
            }
          }
          // 其余事件形态 = 动态实例源（派发通知 _zwSmilNotifyEvent），不入
          // 静态链——零实例保持 exceptions 定谳面。
        }
      }
      for (j = 0; j < eSpecs.length; j++) {
        var se = eSpecs[j];
        if (se.kind === 'offset') rec.pendE.push(se.t);
        else if (se.kind === 'syncbase') {
          var slotE = deps[se.id] || (deps[se.id] = []);
          slotE.push({ rec: rec, asBegin: false, timeKind: se.base, off: se.off });
        }
        // end 侧 event 形态零实例（FIXME(svg-compat-v2 B5+)：dynEnds 面——
        // 现势语料仅 begin 侧）
      }
      var dyn = _elementState(el).dynBegins;
      for (j = 0; j < dyn.length; j++) rec.pendB.push(dyn[j]);
      rec.pendB.sort(function (x, y) { return x - y; });
      rec.pendE.sort(function (x, y) { return x - y; });
      recs.push(rec);
    }
    // 时间序主循环：反复取全文档最早未消费 begin 实例建 interval。
    // 传播只来自已建 interval；剪枝 interval（e ≤ 0 文档起点前，
    // dont-contribute 定谳）建而不传播；t 之后每元素至多建一个 pending
    //（gap 期 getStartTime 取下一 interval 的定谳面 + 终止界）。
    var guard = 0;
    for (;;) {
      if (++guard > 2000) break;
      var pick = null;
      for (var i2 = 0; i2 < recs.length; i2++) {
        var r2 = recs[i2];
        if (r2.dead) continue;
        var b2 = r2.pendB[r2.ib];
        if (b2 === undefined) continue;
        if (b2 > t && r2.ivsBeyond >= 1) continue;
        if (b2 === Infinity) continue;
        if (!pick || b2 < pick.b) pick = { r: r2, b: b2 };
      }
      if (!pick) break;
      var r = pick.r;
      var b = pick.b;
      r.ib++;
      // restart 门（spec #Timing-RestartMechanisms——模拟级实例过滤，属性
      // 实例与 dyn 实例统一；_beginElementAt 入口门 = 同语义两道）。never =
      // 首个 interval 后恒忽略（startedN 只计非剪枝 interval——观测时域外
      // 剪枝 interval 不锁重启）；whenNotActive = 实例落活跃区间内忽略。
      if (r.restart === 'never' && r.startedN > 0) continue;
      if (r.restart === 'whenNotActive') {
        var blocked = false;
        for (var q2 = 0; q2 < r.ivs.length; q2++) {
          if (r.ivs[q2].b <= b && b < r.ivs[q2].e) { blocked = true; break; }
        }
        if (blocked) continue;
      }
      // interval end：首个 > b 的 end 实例与 b+actDur 取 min（M1 口径恒持）；
      // 无 end 实例 → b + actDur。
      var e = b + r.actDur;
      for (var k2 = 0; k2 < r.pendE.length; k2++) {
        if (r.pendE[k2] > b) { e = Math.min(r.pendE[k2], e); break; }
      }
      if (!(e > b)) continue;    // 零/负时长不成 interval（实例已消费，无环）
      var pruned = e <= 0;
      r.ivs.push({ b: b, e: e, pruned: pruned });
      if (b > t) r.ivsBeyond++;
      if (pruned) continue;
      r.startedN++;
      // 传播（SMIL #Timing-SyncbaseValueSemantics）：b → '.begin' 依赖、
      // e → '.end' 依赖、迭代起点 → repeat(k) 依赖（k·sd < e − b 才存在）
      // ——实例写给依赖者 dep.rec（键是目标 id，依赖者收值；自引用 = 目标
      // 即依赖者，同槽自然成立）；end 实例截断依赖者已建含 v 区间
      //（dependent-end）。
      var slot2 = deps[r.id];
      if (slot2) {
        for (var d2 = 0; d2 < slot2.length; d2++) {
          var dep = slot2[d2];
          var depRec = dep.rec;
          if (!depRec || depRec.dead) continue;
          var v;
          if (dep.timeKind === 'repeat') {
            if (!isFinite(r.sd)) continue;
            if (dep.repK * r.sd >= e - b) continue;   // 迭代 k 不存在
            v = b + dep.repK * r.sd + dep.off;
          } else {
            v = (dep.timeKind === 'begin' ? b : e) + dep.off;
          }
          if (!isFinite(v)) continue;
          if (dep.asBegin) {
            _insertSorted(depRec.pendB, v);
          } else {
            _insertSorted(depRec.pendE, v);
            for (var q = depRec.ivs.length - 1; q >= 0; q--) {
              var ivq = depRec.ivs[q];
              if (v > ivq.b && v < ivq.e) { ivq.e = v; break; }
            }
          }
        }
      }
    }
    var map = {};
    for (var i3 = 0; i3 < recs.length; i3++) map[recs[i3].key] = recs[i3];
    st.chainCache = { t: t, map: map };
    return map;
  };

  var _chainOf = function (animEl, t) {
    var doc = null;
    try { doc = animEl.ownerDocument; } catch (_eD) { doc = null; }
    if (!doc) return null;
    var map = _docChains(doc, t);
    var r = map[_elKey(animEl)];
    return r && !r.dead ? r : null;
  };

  // current interval：链上首个非剪枝且 e > t 的 interval（SVGAnimationElement-
  // getStartTime 定谳；frozen 不算 current、gap 期取下一 interval）。
  var _currentInterval = function (animEl, t) {
    var r = _chainOf(animEl, t);
    if (!r) return null;
    for (var j = 0; j < r.ivs.length; j++) {
      var iv = r.ivs[j];
      if (!iv.pruned && iv.e > t) return { b: iv.b, e: iv.e, sd: r.sd };
    }
    return null;
  };

  // active interval：b ≤ t < e（事件模型与采样用——pending（b > t）不算
  // 活跃；t ≥ 0 文档时域下 e > t 即 e > 0，first-interval-in-the-past-
  // dont-contribute 的 end ≤ 文档起点 interval 在此自然修剪）。
  var _activeInterval = function (animEl, t) {
    var cur = _currentInterval(animEl, t);
    if (cur && cur.b <= t) return cur;
    return null;
  };

  // ── 值采样 ──────────────────────────────────────────────────────────
  // 长度 token → user units（px 域）。%/em/ex 上下文单位不采样（挂账 #3 域）。
  var _lenUnitsPx = { cm: 96 / 2.54, mm: 96 / 25.4, in: 96, pt: 96 / 72, pc: 16, q: 96 / 25.4 / 4 };
  var _parseLenToken = function (tok) {
    var m = /^\s*([+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?)\s*([a-zA-Z%]*)\s*$/.exec(String(tok));
    if (!m) return null;
    var v = Number(m[1]);
    var u = m[2].toLowerCase();
    if (u === '' || u === 'px') return v;
    var f = _lenUnitsPx[u];
    return f ? v * f : null;
  };
  var _parseNumToken = function (tok) {
    var str = String(tok).replace(/^\s+|\s+$/g, '');
    if (!str || !/^[+-]?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?$/.test(str)) return null;
    var v = Number(str);
    return isFinite(v) ? v : null;
  };

  var _lerp = function (a, b, f) { return a + (b - a) * f; };

  // 线性采样：normalized 值对（全部可解析才插值，单 token 非法 → null）。
  var _sampleLinear = function (vals, frac) {
    for (var i = 0; i < vals.length; i++) if (vals[i] == null) return null;
    if (vals.length === 1) return vals[0];
    var seg = frac * (vals.length - 1);
    var i2 = Math.floor(seg);
    if (i2 >= vals.length - 1) return vals[vals.length - 1];
    return _lerp(vals[i2], vals[i2 + 1], seg - i2);
  };

  // 目标属性当前 base 数值（by/to 动画的 underling 起点）。
  var _baseNumeric = function (target, attr) {
    var raw = null;
    try { raw = target.getAttribute(attr); } catch (_eG) { raw = null; }
    return raw == null ? 0 : _parseNumToken(raw);
  };

  // 单动画元素在 t 的采样值（数值域）→ number | null（不支持/不活跃）。
  // calcMode 线性（缺省）；discrete/spline = B6 FIXME（暂按线性——对应案
  // 现势恒 F，无翻转面）。
  var _sampleAnim = function (animEl, target, attr, t) {
    var an = null;
    try { an = animEl.getAttribute('attributeName'); } catch (_eA) { an = null; }
    if (an !== attr) return null;
    var cur = _activeInterval(animEl, t);
    var d = _parseDur(animEl);
    if (d.kind === 'invalid') return null;
    var localName = '';
    try { localName = String(animEl.localName || '').toLowerCase(); } catch (_eL) {}
    var g = function (name) {
      var v = null;
      try { v = animEl.getAttribute(name); } catch (_eG) { v = null; }
      return v == null ? null : String(v);
    };
    var toTok = g('to');
    var fromTok = g('from');
    var byTok = g('by');
    var valuesTok = g('values');
    var vals;
    var frac;
    if (cur) {
      var actTime = t - cur.b;
      var actDur = cur.e - cur.b;
      if (cur.sd === Infinity) {
        frac = 0;                             // 无 dur：恒活 from 常值
      } else if (actDur <= 0) {
        frac = 1;
      } else {
        // repeat 边界采样取下一 iteration 起点（f(t mod d)——simpleTime=0）。
        var simpleTime = actTime % cur.sd;
        frac = Math.min(simpleTime / cur.sd, 1);
      }
    } else {
      // 冻结（fill=freeze 且存在已结束的 last interval）——冻结于 activeEnd
      //（span 恰为 dur 整数倍时取 simpleTime=sd → to 值）。
      var fillAttr = g('fill');
      if (fillAttr !== 'freeze') return null;
      var sd0 = d.kind === 'finite' ? d.s : Infinity;
      var rec0 = _chainOf(animEl, t);
      var lastEnd = -Infinity;
      var lastB = 0;
      if (rec0) {
        for (var j = 0; j < rec0.ivs.length; j++) {
          var iv0 = rec0.ivs[j];
          // 剪枝 interval（e ≤ 0）非 real interval（SMIL first-interval
          // 剪枝——begin="-1s" dur=1s 的 [-1,0] 不贡献冻结值）。
          if (!iv0.pruned && iv0.e > 0 && iv0.e <= t && iv0.e > lastEnd) {
            lastEnd = iv0.e;
            lastB = iv0.b;
          }
        }
      }
      if (lastEnd === -Infinity) return null;
      if (sd0 === Infinity) return null;
      var span = lastEnd - lastB;
      var simpleTime0 = span % sd0;
      if (simpleTime0 === 0 && span > 0 && span >= sd0) simpleTime0 = sd0;
      frac = Math.min(simpleTime0 / sd0, 1);
    }
    if (localName === 'set') {
      // set：离散持有 to 常值（spec SVG2 animate.html §SetElement）。
      var sv = _parseNumToken(toTok) != null ? _parseNumToken(toTok) : _parseLenToken(toTok);
      return sv;
    }
    if (valuesTok != null) {
      var parts = valuesTok.split(';');
      vals = [];
      for (var p = 0; p < parts.length; p++) {
        var tok = parts[p].replace(/^\s+|\s+$/g, '');
        vals.push(_parseLenToken(tok) != null ? _parseLenToken(tok) : _parseNumToken(tok));
      }
      return _sampleLinear(vals, frac);
    }
    if (fromTok != null && toTok != null) {
      vals = [_parseLenToken(fromTok) != null ? _parseLenToken(fromTok) : _parseNumToken(fromTok),
        _parseLenToken(toTok) != null ? _parseLenToken(toTok) : _parseNumToken(toTok)];
      return _sampleLinear(vals, frac);
    }
    if (fromTok != null && byTok != null) {
      var f0 = _parseLenToken(fromTok) != null ? _parseLenToken(fromTok) : _parseNumToken(fromTok);
      var b0 = _parseLenToken(byTok) != null ? _parseLenToken(byTok) : _parseNumToken(byTok);
      if (f0 == null || b0 == null) return null;
      return _lerp(f0, f0 + b0, frac);
    }
    if (byTok != null) {
      var base = _baseNumeric(target, attr);
      var by = _parseLenToken(byTok) != null ? _parseLenToken(byTok) : _parseNumToken(byTok);
      if (base == null || by == null) return null;
      return _lerp(base, base + by, frac);
    }
    if (toTok != null) {
      // to-animation：from 取目标 base 值（SMIL to-anim 的 underlying 起点
      // 近似——Blink to-anim 特例语义 = B6 FIXME）。
      var base0 = _baseNumeric(target, attr);
      var tv = _parseLenToken(toTok) != null ? _parseLenToken(toTok) : _parseNumToken(toTok);
      if (base0 == null || tv == null) return null;
      return _lerp(base0, tv, frac);
    }
    return null;
  };

  // 目标元素 + 属性名 → 文档序动画采样值合成（后者胜 sandwich 近似）。
  var _animatedValue = function (target, attr, numeric) {
    var doc = null;
    try { doc = target.ownerDocument; } catch (_eD) { doc = null; }
    if (!doc) return null;
    var st = _docTimelines.get(doc);
    var t = st ? _currentTimeOf(st) : 0;
    var kids = null;
    try { kids = target.children; } catch (_eC) { kids = null; }
    if (!kids || !kids.length) return null;
    var result = null;
    for (var i = 0; i < kids.length; i++) {
      var c = kids[i];
      var ln = '';
      try { ln = String(c.localName || '').toLowerCase(); } catch (_eL) {}
      if (!_animLocalNames[ln]) continue;
      var v = _sampleAnim(c, target, attr, t);
      if (v != null) result = v;    // 文档序后者胜
    }
    if (result == null) return null;
    return numeric ? result : { ut: 1, vu: result };
  };

  // animVal 反射消费单点（part03 S2a 挂钩——globalThis 契约）。
  globalThis._zwSmilAnimValue = function (el, attr) {
    return _animatedValue(el, attr, false);
  };
  globalThis._zwSmilAnimNumber = function (el, attr) {
    return _animatedValue(el, attr, true);
  };

  // ── 事件泵（task 粒度 setTimeout 轮询；paused 靠 seek 扫描驱动）──────
  // SMIL 事件统一经 dispatch 派发：onbegin/onend/onrepeat 处理器（IDL JS
  // 设值 + 内容属性 inline 编译）由 part05 派发的错位名映射触发
  //（beginEvent→onbegin / endEvent→onend / repeatEvent→onrepeat；裸
  // begin/end/repeat 类型查找跳过——onendEvent 非处理器名恒不触发，
  // custom-events/event-listeners 定谳面），addEventListener 注册面走
  // listeners 循环——单通道无双 fire。
  var _fireSmilEvent = function (el, type) {
    try {
      var ev = null;
      if (typeof globalThis.__zwMakeTrustedEvent === 'function') {
        ev = globalThis.__zwMakeTrustedEvent(type, { bubbles: false, cancelable: false });
      } else if (typeof globalThis.Event === 'function') {
        ev = new globalThis.Event(type, { bubbles: false, cancelable: false });
      }
      if (ev) el.dispatchEvent(ev);
    } catch (_eF) {}
  };

  var _scanDoc = function (doc) {
    var els = [];
    try {
      var found = doc.querySelectorAll('animate, set, animateTransform, animateMotion');
      for (var i = 0; i < found.length; i++) els.push(found[i]);
    } catch (_eQ) { els = []; }
    return els;
  };
  // FIXME(挂账-引擎查询层): doc.querySelectorAll 对动态 append 的动画元素
  // 同步不可见（detached 快照/复合选择器缓存代际——探针实证单标签元素作用
  // 域活络、文档作用域与复合形态延迟代际化），child 链手工遍历可即时发现
  // 但产物 wrapper 族与消费端（target.children/_elKey）跨族 identity 失配
  //（#32），17 子 P→F 回归实证。SMIL 动态元素发现（spec 时序目标活解析）
  // 待查询层统一 wrapper 归一后重接——syncbase-remove-add-while-running
  // 挂此门（F 显形记账，非 Pass 丢失）。

  var _scheduleTick = function (doc, st, delayMs) {
    if (st.tickTimer) return;
    st.tickTimer = setTimeout(function () {
      st.tickTimer = 0;
      _pumpTick(doc);
    }, delayMs);
  };

  // 泵激活入口（load/beginElement 唤醒；paused/未 started no-op）。
  var _wake = function (doc) {
    var st = _docTimelines.get(doc);
    if (!st || !st.started || st.paused) return;
    _scheduleTick(doc, st, 5);
  };

  // seek 一次性扫描（setCurrentTime 驱动；paused 也派——WPT seeking 案
  // pause+seek 是唯一时间驱动源）。合并窗口：已排程则复用（执行时读最新
  // frozenDoc，幂等）。事件经 task 粒度派发——同步派发会早于测试的
  // EventWatcher 挂钩（seeking-events-1 watcher 在 seek 后才建）。
  var _scheduleSeekScan = function (doc, st) {
    if (st.seekTimer) return;
    st.seekTimer = setTimeout(function () {
      st.seekTimer = 0;
      _scanDocEvents(doc, _currentTimeOf(st), true);
    }, 0);
  };

  // 转移扫描：seek（isSeek）走差分（seeking-events-1..8 定谳矩阵：越过
  // 零事件、出界 endEvent、落点 beginEvent、同 interval 不派 repeat）；
  // 自然 tick 走时间序窗口重放——对 (lastT, t]（首见 lastT=-∞）内真实
  // 流逝的 interval 边界按 b 升序补派：窗口内整体走完 → begin+end 成对；
  // 跨窗口结束（begin 在窗前）→ 仅 endEvent；窗口内开始且 ≤ t →
  // beginEvent（被替代的旧 interval 若未报 end 先补 end——restart 语义）。
  // 泵 tick 抖动跨过多个边界时事件不丢、不乱序（correct-events-for-short-
  // animations-with-syncbases 定谳面）；seek 只推进 lastT 不重放。
  var _scanDocEvents = function (doc, t, isSeek) {
    var els = _scanDoc(doc);
    for (var i = 0; i < els.length; i++) {
      var el = els[i];
      if (!_condOK(el)) continue;   // 条件判否元素零事件（02 定谳面）
      var active = null;
      try { active = _activeInterval(el, t); } catch (_eA) { active = null; }
      var s = _elementState(el);
      s.seen = true;
      if (!isSeek) {
        var w0 = s.lastT == null ? -Infinity : s.lastT;
        if (t >= w0) {
          var rec1 = _chainOf(el, t);
          if (rec1) {
            for (var j0 = 0; j0 < rec1.ivs.length; j0++) {
              var iv1 = rec1.ivs[j0];
              if (iv1.pruned || iv1.e <= 0) continue;
              // interval 起点入窗：被替代旧 interval 先补 end 再 begin
              //（restart 语义——B4 口径恒持）
              if (iv1.b > w0 && iv1.b <= t && s.ib !== iv1.b) {
                if (s.ib != null && s.began && !s.ended) {
                  _fireSmilEvent(el, 'endEvent');    // 被替代/截断
                  s.ended = true;
                }
                _fireSmilEvent(el, 'beginEvent');
                s.ib = iv1.b;
                s.ie = iv1.e;
                s.began = true;
                s.ended = false;
                s.everBegan = true;
              }
              // repeatEvent 自然播放面（B5）：迭代边界 b + k·sd（k ≥ 1 且
              // k·sd < e）入窗 (max(w0, b), min(t, e)] 按 k 升序补派——仅
              // 当前跟踪 interval（begin 前不派、end 后不派）；seek 不派
              //（isSeek 臂外——seeking-events-5/6 恒持）。千次封顶 =
              // OPTIMIZATION：节流追赶窗口内病态 repeatCount=indefinite ×
              // 毫秒级 dur 的边界风暴防失控（语料真实面 ≤ 个位数边界）。
              if (s.ib === iv1.b && s.began && !s.ended &&
                  isFinite(rec1.sd) && rec1.sd > 0) {
                var k0 = Math.ceil((Math.max(w0, iv1.b) - iv1.b) / rec1.sd);
                if (k0 < 1) k0 = 1;
                var fired = 0;
                for (; iv1.b + k0 * rec1.sd < iv1.e && iv1.b + k0 * rec1.sd <= t;
                     k0++) {
                  if (++fired > 1000) break;
                  _fireSmilEvent(el, 'repeatEvent');
                }
              }
              // 窗口内结束（首见 w0=-∞ 覆盖全部已过 interval——整体成对；
              // begin 已在本轮建立跟踪的 interval 同轮内 end 成对）
              if (iv1.e > w0 && iv1.e <= t && s.ib === iv1.b && s.began && !s.ended) {
                _fireSmilEvent(el, 'endEvent');
                s.ended = true;
              }
            }
          }
        }
      }
      s.lastT = t;
      // 差分兜底：seek 语义面 + 链突变（属性轮询/beginElement）后的对齐。
      if (s.ib != null && s.began && !s.ended && (!active || active.b !== s.ib)) {
        _fireSmilEvent(el, 'endEvent');
        s.ended = true;
      }
      if (active && (s.ib !== active.b || !s.began || s.ended)) {
        _fireSmilEvent(el, 'beginEvent');
        s.ib = active.b;
        s.ie = active.e;
        s.began = true;
        s.ended = false;
        s.everBegan = true;
      }
    }
  };

  var _pumpTick = function (doc) {
    var st = _docTimelines.get(doc);
    if (!st || !st.started || st.paused) return;
    var t = _currentTimeOf(st);
    _scanDocEvents(doc, t, false);
    st.tickEverRan = true;
    // 轮询节律：下一边界 ≤50ms 提前到点，否则 50ms 保底（属性变更无钩子、
    // 变更感知靠轮询——begin-attribute-mutation 消费面）。
    var next = Infinity;
    var els = _scanDoc(doc);
    st.smilSeen = els.length > 0;
    for (var i = 0; i < els.length; i++) {
      if (!_condOK(els[i])) continue;   // 条件判否元素无边界
      var cur = null;
      try { cur = _currentInterval(els[i], t); } catch (_eI) { cur = null; }
      if (cur) {
        if (cur.b > t && cur.b < next) next = cur.b;
        if (cur.e > t && cur.e < next) next = cur.e;
      }
    }
    var delay = 50;
    if (next < Infinity) {
      var ms = (next - t) * 1000;
      if (ms < 5) ms = 5;
      if (ms < delay) delay = ms;
    }
    _scheduleTick(doc, st, delay);
  };

  // ── 文档时间线起点（part06 window load 派发点挂钩）───────────────────
  globalThis._zwSmilDocStart = function (doc) {
    if (!doc) return;
    var st = _timelineFor(doc);
    if (st.started) return;
    st.started = true;
    st.baseDoc = st.frozenDoc;   // pre-load seek 结转
    st.baseReal = Date.now();
    if (!st.paused) _scheduleTick(doc, st, 5);
  };

  // ── B5：eventbase 动态实例源（part05 派发尾通知契约）─────────────────
  // spec SMIL #Eventvalue semantics：事件真发即对引用该事件基的动画元素写
  // dynBegins 实例（当前文档时 + offset）。base 解析：id 名 → 全文档
  // getElementById（非 SVG 元素允许——eventbase-non-svg-element 定谳）；
  // 裸事件名 → 目标元素（animate 的父元素——与采样目标解析同口径）。每次
  // 通知从现行文档全量扫描 = 移除/重建自动反映（syncbase deps 同口径）；
  // 条件判否元素不入实例源。repeat(k) 形态是静态传播不走此路。性能面：
  // 首 tick 后无 SMIL 元素的文档快路径返（tickEverRan/smilSeen 标记——首
  // tick 前恒扫描，覆盖 onload 同步派发）。pre-load 通知：时间线状态经
  // _timelineFor 现建（eventbase-non-svg-element 定谳——inline 脚本 load 前
  // 派发实例不丢，文档时间线起点后 interval [0,…) 生效）。
  globalThis._zwSmilNotifyEvent = function (target, type) {
    try {
      var doc = null;
      try { doc = target && target.ownerDocument; } catch (_eD) { doc = null; }
      if (!doc) {
        try { if (target && target.documentElement) doc = target; } catch (_eT) { doc = null; }
      }
      if (!doc) return;
      var st = _timelineFor(doc);
      if (!st) return;
      if (st.tickEverRan && !st.smilSeen) return;
      var t = _currentTimeOf(st);
      var els = _scanDoc(doc);
      var hit = false;
      for (var i = 0; i < els.length; i++) {
        var el = els[i];
        if (!_condOK(el)) continue;
        var rawB = null;
        try { rawB = el.getAttribute('begin'); } catch (_eB) { rawB = null; }
        var specs = _parseTimingSpecs(rawB);
        for (var j = 0; j < specs.length; j++) {
          var sp = specs[j];
          if (sp.kind !== 'event' || /^repeat\(\d+\)$/.test(sp.name)) continue;
          var base = null;
          if (sp.id) {
            try { base = doc.getElementById(sp.id); } catch (_eI) { base = null; }
          } else {
            try { base = el.parentElement; } catch (_eP) { base = null; }
          }
          if (!base || sp.name !== type) continue;
          // base 与派发 target 同节点判定：对象 identity 跨访问族恒败
          //（#32），键（__zwSelector+__zwHandle，节点稳定）一致兜底。
          var same = base === target;
          if (!same) {
            var kt = null;
            try { kt = _elKey(target); } catch (_eK2) { kt = null; }
            var kb = _elKey(base);
            same = kb != null && kt != null && kb === kt;
          }
          if (!same) continue;
          var s = _elementState(el);
          s.dynBegins.push(t + sp.off);
          hit = true;
        }
      }
      if (hit) {
        st.chainCache = null;   // 实例入集——下查询重模拟
        _wake(doc);
      }
    } catch (_eN) {}
  };

  // ── SVGSVGElement 文档级时间线控制（覆写 part03 M2 S3 per-element stub）─
  if (globalThis.SVGSVGElement && globalThis.SVGSVGElement.prototype) {
    var _zwSvgP = globalThis.SVGSVGElement.prototype;
    _zwSvgP.pauseAnimations = function () {
      var doc = this.ownerDocument || this;
      var st = _timelineFor(doc);
      st.frozenDoc = _currentTimeOf(st);
      st.paused = true;
      if (st.tickTimer) { clearTimeout(st.tickTimer); st.tickTimer = 0; }
    };
    _zwSvgP.unpauseAnimations = function () {
      var doc = this.ownerDocument || this;
      var st = _timelineFor(doc);
      if (!st.paused) return;
      st.paused = false;
      if (st.started) {
        st.baseDoc = st.frozenDoc;
        st.baseReal = Date.now();
        _wake(doc);
      }
    };
    _zwSvgP.getCurrentTime = function () {
      var st = _timelineFor(this.ownerDocument || this);
      return _currentTimeOf(st);
    };
    _zwSvgP.setCurrentTime = function (seconds) {
      var n = Number(seconds);
      if (!isFinite(n)) throw new TypeError('The provided float value is non-finite');
      var doc = this.ownerDocument || this;
      var st = _timelineFor(doc);
      if (!st.started || st.paused) {
        st.frozenDoc = n;
      } else {
        st.baseDoc = n;
        st.baseReal = Date.now();
        _wake(doc);
      }
      // seek 事件转移（task 粒度——同步派发会早于 seek 后才挂的 EventWatcher）
      _scheduleSeekScan(doc, st);
    };
  }

  // ── SVGAnimationElement 真值接口（覆写 part03 M4 异常 stub）─────────
  if (globalThis.SVGAnimationElement && globalThis.SVGAnimationElement.prototype) {
    var _zwAnimP = globalThis.SVGAnimationElement.prototype;
    _zwAnimP.getStartTime = function () {
      var doc = null;
      try { doc = this.ownerDocument; } catch (_eD) { doc = null; }
      var st = doc ? _docTimelines.get(doc) : null;
      var t = st ? _currentTimeOf(st) : 0;
      var cur = null;
      try { cur = _currentInterval(this, t); } catch (_eI) { cur = null; }
      if (!cur) {
        throw _domErr('getStartTime: the current interval is unresolved.', 'InvalidStateError');
      }
      return cur.b;
    };
    _zwAnimP.getSimpleDuration = function () {
      var d = _parseDur(this);
      if (d.kind !== 'finite') {
        throw _domErr('getSimpleDuration: the simple duration is undefined or indefinite.', 'NotSupportedError');
      }
      return d.s;
    };
    var _beginElementAt = function (offset) {
      var n = Number(offset);
      if (!isFinite(n)) throw new TypeError('The provided float value is non-finite');
      var doc = null;
      try { doc = this.ownerDocument; } catch (_eD) { doc = null; }
      var st = doc ? _timelineFor(doc) : _timelineFor(this);
      var now = _currentTimeOf(st);
      var restart = '';
      try { restart = String(this.getAttribute('restart') || 'always'); } catch (_eR) { restart = 'always'; }
      var cur = null;
      try { cur = _currentInterval(this, now); } catch (_eI) { cur = null; }
      // restart 门控（spec #Timing-RestartMechanisms——beginElement 动态实例面；
      // begin 属性实例的 restart 语义 = M2 FIXME）。never：任一 interval 一经
      // 开始即永不再启（everBegan 含已过 interval）。
      var s = _elementState(this);
      if (restart === 'never' && (s.everBegan || cur)) return;
      if (restart === 'whenNotActive' && cur && cur.b <= now) return;
      s.dynBegins.push(now + n);
      st.chainCache = null;   // 动态实例入集——链缓存作废，下查询重模拟
      // 差分状态不手动重置——下轮转移扫描按新 instance 集派
      // endEvent（旧 interval 被替代）+ beginEvent（restart=always 语义）。
      _wake(doc || this);
    };
    _zwAnimP.beginElement = function () { _beginElementAt.call(this, 0); };
    _zwAnimP.beginElementAt = function (offset) { _beginElementAt.call(this, offset); };
    _zwAnimP.endElement = function () {
      // 动态 end 实例 = M2 FIXME（interval-restart-events / end 属性操作面）；
      // 控制面保持可调用（part03 stub 同口径）。
    };
    _zwAnimP.endElementAt = function (offset) {
      var n = Number(offset);
      if (!isFinite(n)) throw new TypeError('The provided float value is non-finite');
    };
  }
})();
