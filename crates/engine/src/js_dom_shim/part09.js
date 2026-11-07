// ============================================================================
// part09 — SMIL 文档级时间线引擎（svg-compat-v2 goal M1/M2，2026-11-07）。
//
// 范围（控制面 docs/goal/svg-compat-v2.md）：M1 = B1 时钟值文法 + B2
// interval 模型首批；M2 首批 = B3 seek 事件语义——
//   ① clock-value 严格文法：Full/Partial-clock-val（分/秒必须两位数且 <60、
//      分数点后至少一位数字、小时两位起不限位）+ Timecount-val（h/min/s/ms，
//      缺省 s）——B1 非法矩阵（越界分秒/分数缺数字/前导零/缺字段）逐条
//      定谳（WPT smil-clock-value-parsing 7 合法 + 11 非法定谳面）。
//   ② begin/end 列表：offset-value（± clock-val）+ indefinite；syncbase/
//      eventbase/repeat-value/wallclock/accessKey 语法识别但实例不解析
//      （FIXME(svg-compat-v2 M2)：B4 syncbase / B5 eventbase 接实例源——
//      WPT SVGAnimationElement-exceptions 的 begin="foo.begin" 依赖本口径：
//      识别但零实例 → getStartTime InvalidStateError）。
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
      st = { started: false, paused: false, frozenDoc: 0, baseDoc: 0, baseReal: 0, tickTimer: 0, seekTimer: 0 };
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

  // begin/end 列表 → 可解析 offset 实例数组。识别但 M2 未接实例源的形态
  // （syncbase/eventbase/repeat 等）返回零实例不报错（exceptions 案口径）。
  var _parseTimingList = function (s) {
    var out = [];
    if (s == null) return out;
    var parts = String(s).split(';');
    for (var i = 0; i < parts.length; i++) {
      var seg = parts[i].replace(/^\s+|\s+$/g, '');
      if (!seg || seg === 'indefinite') continue;
      // M2 实例源占位识别：syncbase（id.begin/end）、repeat-value
      //（id.repeat(n)）、eventbase（裸事件名 / id.event）、wallclock、
      // accessKey——FIXME(svg-compat-v2 M2)。
      if (/[.(]/.test(seg)) continue;
      if (/^[a-zA-Z][a-zA-Z0-9_-]*$/.test(seg)) continue;
      var v = _parseOffsetValue(seg);
      if (v != null) out.push(v);
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
    if (!_elStateCache[k]) _elStateCache[k] = { dynBegins: [], ib: null, ie: null, began: false, ended: false, seen: false };
    return _elStateCache[k];
  };

  // interval end：end 属性首个 > begin 的实例与 begin+activeDur 取 min；
  // end 实例 ≤ begin（零/负时长 interval）→ null（该 instance 不成 interval）。
  var _intervalEnd = function (animEl, b, actDur) {
    var ends = null;
    try { ends = _parseTimingList(animEl.getAttribute('end')); } catch (_eG) { ends = null; }
    var cap = b + actDur;
    for (var i = 0; i < ends.length; i++) {
      if (ends[i] > b) return Math.min(ends[i], cap);
    }
    return cap;
  };

  // 全部 begin 实例（属性 offset 集 + beginElement 动态集）升序；begin 属性
  // 缺省/空白才回落 '0s'（spec #Timing-DefaultBegin）——属性在但零实例
  //（indefinite / 未接实例源的 syncbase/eventbase 形态，M1 口径识别不解析）
  // 保持无实例：元素不自动开始（SVGAnimationElement-exceptions 的
  // begin="foo.begin" → getStartTime InvalidStateError 定谳面；M0 stub
  // 恒抛不可回退）。
  var _allBegins = function (animEl) {
    var raw = null;
    try { raw = animEl.getAttribute('begin'); } catch (_eB) { raw = null; }
    var begins = _parseTimingList(raw);
    if (!begins.length &&
        String(raw == null ? '' : raw).replace(/^\s+|\s+$/g, '') === '') begins = [0];
    var st = _elementState(animEl);
    var all = begins.slice();
    for (var i = 0; i < st.dynBegins.length; i++) all.push(st.dynBegins[i]);
    all.sort(function (x, y) { return x - y; });
    return all;
  };

  // current interval：instance 升序首个 end > t（SVGAnimationElement-
  // getStartTime 定谳；frozen 不算 current、gap 期取下一 interval）。
  var _currentInterval = function (animEl, t) {
    var d = _parseDur(animEl);
    if (d.kind === 'invalid') return null;
    var sd = d.kind === 'finite' ? d.s : Infinity;
    var actDur = d.kind === 'finite' ? _activeDuration(animEl, sd) : Infinity;
    var all = _allBegins(animEl);
    for (var j = 0; j < all.length; j++) {
      var e = _intervalEnd(animEl, all[j], actDur);
      if (e > t) return { b: all[j], e: e, sd: sd };
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
      var actDur0 = d.kind === 'finite' ? _activeDuration(animEl, sd0) : Infinity;
      var all = _allBegins(animEl);
      var lastEnd = -Infinity;
      var lastB = 0;
      for (var j = 0; j < all.length; j++) {
        var e0 = _intervalEnd(animEl, all[j], actDur0);
        // 修剪：end ≤ 文档起点（0）的 interval 非 real interval（SMIL
        // first-interval 剪枝——begin="-1s" dur=1s 的 [-1,0] 不贡献冻结值）。
        if (e0 > 0 && e0 <= t && e0 > lastEnd) { lastEnd = e0; lastB = all[j]; }
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
  // SMIL 事件处理器 IDL 名 ↔ 事件类型错位映射（spec SVG2 animate.html：
  // onbegin/onend/onrepeat 分别是 beginEvent/endEvent/repeatEvent 的处理器
  // ——event-listeners 案定谳：onendEvent 不是处理器名，恒不触发）。
  var _smilHandlerNames = { beginEvent: 'onbegin', endEvent: 'onend', repeatEvent: 'onrepeat' };

  var _fireSmilEvent = function (el, type) {
    try {
      var ev = null;
      if (typeof globalThis.__zwMakeTrustedEvent === 'function') {
        ev = globalThis.__zwMakeTrustedEvent(type, { bubbles: false, cancelable: false });
      } else if (typeof globalThis.Event === 'function') {
        ev = new globalThis.Event(type, { bubbles: false, cancelable: false });
      }
      // on* 处理器（IDL 属性 JS 设值 + 内容属性 inline 编译——经 proxy get
      // trap 的 _ensureInlineHandler 惰性编译）单点直调；类型名错位故不走
      // dispatch 的 on<type> 泛型路。addEventListener 注册面经 dispatchEvent。
      if (ev) {
        var hn = _smilHandlerNames[type];
        if (hn) {
          var h = null;
          try { h = el[hn]; } catch (_eH) { h = null; }
          if (typeof h === 'function') {
            try { h.call(el, ev); } catch (_eHC) {}
          }
        }
        el.dispatchEvent(ev);
      }
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

  // 转移扫描：对每动画元素比较上一观测态 {ib, began, ended} 与 t 时刻
  // active interval（b ≤ t < e），差分出事件。语义（seeking-events-1..8
  // 定谳矩阵）： seek（isSeek）越过的 interval（首见即整体在过去）不派
  //  任何事件； 曾 began 的 interval 失去活跃态（自然到 end / seek 出界 /
  //  seek 回退越过 begin / 被新 instance 替代）→ endEvent； seek 落点
  //  所在 interval（含回退重入已 ended interval）→ beginEvent； 同
  //  interval 内 seek（跨 repeat 边界也一样）不派 repeatEvent（repeatEvent
  //  = B5 自然播放面）。首见自然 tick（!isSeek）且已整体走完的 real
  //  interval（e > 0 ≤ t）补派 begin+end——文档时间真实流过该 interval
  //（event-listeners 的 [0,5ms] 案：泵首 tick 前已自然结束；多 interval
  //  首个命中即止的近似）。e ≤ 0 的 interval 例外（first-interval 剪枝，
  //  dont-contribute 定谳：begin=-1s dur=1s 不贡献）。
  var _scanDocEvents = function (doc, t, isSeek) {
    var els = _scanDoc(doc);
    for (var i = 0; i < els.length; i++) {
      var el = els[i];
      var active = null;
      try { active = _activeInterval(el, t); } catch (_eA) { active = null; }
      var s = _elementState(el);
      if (!s.seen) {
        s.seen = true;
        if (!isSeek && !active) {
          var d0 = _parseDur(el);
          if (d0.kind === 'finite') {
            var actDur0 = _activeDuration(el, d0.s);
            var begins0 = _allBegins(el);
            for (var j0 = 0; j0 < begins0.length; j0++) {
              var e0 = _intervalEnd(el, begins0[j0], actDur0);
              if (e0 > 0 && e0 <= t) {
                _fireSmilEvent(el, 'beginEvent');
                _fireSmilEvent(el, 'endEvent');
                s.ib = begins0[j0];
                s.ie = e0;
                s.began = true;
                s.ended = true;
                s.everBegan = true;
                break;
              }
            }
          }
          continue;
        }
      }
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
    // 轮询节律：下一边界 ≤50ms 提前到点，否则 50ms 保底（属性变更无钩子、
    // 变更感知靠轮询——begin-attribute-mutation 消费面）。
    var next = Infinity;
    var els = _scanDoc(doc);
    for (var i = 0; i < els.length; i++) {
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
