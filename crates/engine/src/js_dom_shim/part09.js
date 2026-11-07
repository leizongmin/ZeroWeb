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
//   ④ 采样（B6 值域感知插值管线）：calcMode linear（缺省）/discrete/
//      spline（三次贝塞尔求解）/paced（数值累计距离）四模；keyTimes/
//      keySplines 文法校验（违例 → 整动画无效——keysplines-x-limits 定
//      谳）；accumulate="sum" 迭代传播 k×(last−first)；fill=freeze 冻结
//      于 activeEnd（activeDur 非 dur 整数倍时 simpleTime = activeDur
//      mod dur）。长度 token 按 px + 物理单位换算进 user units（part03
//      S2a 常量表同源复制——闭包不可反向引用）；em/ex/% 经 target
//      font-size/视口上下文解析（_lenUserUnits——svglength px-to-ems 定
//      谳）。三明治合成 replace/additive=sum（_animatedValue）。类型化域
//      enum/string/angle/list/bool/integer/rect = B6b；transform = ⑪
//     （B6c）；color = FIXME（style 协同面）。
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
//   ⑪ animateTransform 采样（B6c，2026-11-07 第五批）：type 域组件向量
//      插值（translate/scale/rotate/skewX/skewY；type 缺省 translate）+
//      calcMode linear 缺省/discrete/spline + accumulate 迭代传播逐分量 +
//      by-only 零向量起点（additive-type-by 定谳）；三明治钩子
//      _zwSmilAnimTransform（globalThis 契约）——animVal 表项级合成
//      replace 重置/sum 追加：gradientTransform/patternTransform replace
//      清 base（animate-gradient-transform 定谳 animVal 单项）、transform
//      属性 replace 恒与 base 复合（additive-type-by 定谳 animVal 2 项）。
//      part03 SVGAnimatedTransformList.animVal parse 接线 + SVGTransform
//      angle 态（rotate/skewX/skewY 原始角度）。另有 _sampleAnimStr by-only
//      角度域度数合成（orient by-only——additive-type-by 定谳）。
//      ⑪ 起物理在 part09b（2026-11-07 2000 行准则按行边界字节精确切分，
//      R2963 先例——本文件开 IIFE、part09b 续段同闭合并收尾，桥接层
//      part09 后紧接 include，拼接字节序零变化）。
//
// 作用域：part07 末尾已闭合外层 IIFE，本文件顶层作用域（part08 先例）；
// 本文件 + part09b 合成同一 IIFE（part09b 尾收 })();）。
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
      // restart=always（缺省）中途替换：新实例 b 落入已建活跃区间 → 旧区
      // 间截断至 b（SMIL #Timing-RestartMechanisms「restarted」语义——活
      // 跃 interval 被新实例终结重建；multiple-begin-additive 定谳面：x
      // 的 to 值自第二实例起点 +半程起算，非旧实例 4s 处续用）。
      for (var q3 = r.ivs.length - 1; q3 >= 0; q3--) {
        var ivp = r.ivs[q3];
        if (!ivp.pruned && ivp.b < b && ivp.e > b) ivp.e = b;
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
    // 跨族 wrapper 键回落（挂账 #32 邻接——querySelector 产物根 svg 的
    // children 是 sel 族 wrapper（'#id\0\0smil'），scan 产物是 handle 族
    //（'null\0__n0\0smil'），两侧 key 不同源致链查落空——svgrect viewBox
    // 根目标面探针实证）。链 rec 存有元素 id 属性：id 非空时按 id 对齐
    //（语料动画元素恒带 id——runAnimationTest 按 getElementById 寻元素）。
    if (!r || r.dead) {
      var aid = '';
      try { aid = String(animEl.getAttribute('id') || ''); } catch (_eI) { aid = ''; }
      if (aid) {
        for (var kk in map) {
          var rr = map[kk];
          if (rr && !rr.dead && rr.id === aid) { r = rr; break; }
        }
      }
    }
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

  // ── 值采样（B6 采样引擎：calcMode/keyTimes/keySplines/additive/
  //    accumulate，SMIL Animation #CalcModeValueAttributes）───────────────
  // 长度 token → user units。em/ex/% 上下文单位经 _lenUserUnits 解析
  //（挂账 #3 采样侧部分清偿——SVGLength IDL 上下文单位另批）。
  var _lenUnitsPx = { cm: 96 / 2.54, mm: 96 / 25.4, in: 96, pt: 96 / 72, pc: 16, q: 96 / 25.4 / 4 };
  // 长度文法：数字与单位间**不容空白**（SMIL #ToAttribute「argument-values
  // 非法 → 动画无效」——svglength-animation-invalid-value-1/5..9
  // "100   px" 形定谳；前后缘空白允许）。
  var _lenTokenRe = /^\s*([+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?)([a-zA-Z%]*)\s*$/;
  var _parseLenToken = function (tok) {
    var m = _lenTokenRe.exec(String(tok));
    if (!m) return null;
    var v = Number(m[1]);
    var u = m[2].toLowerCase();
    if (u === '' || u === 'px') return v;
    var f = _lenUnitsPx[u];
    return f ? v * f : null;
  };
  var _parseNumToken = function (tok) {
    var str = String(tok).replace(/^\s+|\s+$/g, '');
    if (!str || !/^[+-]?(\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?$/.test(str)) return null;
    var v = Number(str);
    return isFinite(v) ? v : null;
  };

  // B6：目标 font-size（em/ex 上下文）——font-size 属性读 → computed
  // style → 16px 缺省（CSS Fonts font-size 初始值 medium = 16px）。
  var _targetFontSize = function (target) {
    var raw = null;
    try { raw = target.getAttribute('font-size'); } catch (_eF) { raw = null; }
    if (raw != null) {
      var v = _parseLenToken(raw);
      if (v != null) return Math.abs(v);
    }
    try {
      var dv = target.ownerDocument ? target.ownerDocument.defaultView : null;
      var cs = dv && dv.getComputedStyle ? dv.getComputedStyle(target) : null;
      if (cs && cs.fontSize) {
        var fv = parseFloat(cs.fontSize);
        if (isFinite(fv) && fv > 0) return fv;
      }
    } catch (_eF2) {}
    return 16;
  };

  // B6：% 单位视口解析——viewBox 宽高 → width/height 属性 → 300×150
  //（spec SVG2 §7.2 initial viewport 常规缺省）。轴向按属性名：x/width 族
  // 取宽、y/height 族取高、其余取宽。
  var _HORIZONTAL_ATTRS = { x: 1, cx: 1, rx: 1, x1: 1, x2: 1, fx: 1, width: 1, sx: 1 };
  var _VERTICAL_ATTRS = { y: 1, cy: 1, ry: 1, y1: 1, y2: 1, fy: 1, height: 1 };
  var _targetViewportLen = function (target, attr) {
    var w = 300;
    var h = 150;
    // 最近祖先 svg 视口（spec SVG2 §7.3：% 长度对最近祖先 svg 建立的视口
    // 解析——svglength-additive-by-7 定谳：HTML 内嵌 svg width="800" 时
    // % 对 800 解析，documentElement（html）落 300 缺省不符）。
    var svg = null;
    try {
      var n = target;
      while (n) {
        var lnn = '';
        try { lnn = String(n.localName || '').toLowerCase(); } catch (_eL2) { lnn = ''; }
        if (lnn === 'svg') { svg = n; break; }
        n = n.parentNode;
      }
    } catch (_eS) { svg = null; }
    if (!svg) {
      try { svg = target.ownerDocument ? target.ownerDocument.documentElement : null; } catch (_eS2) { svg = null; }
    }
    if (svg) {
      var vb = null;
      try { vb = svg.getAttribute('viewBox'); } catch (_eV) { vb = null; }
      if (vb) {
        var vp = String(vb).split(/[\s,]+/);
        if (vp.length === 4 && isFinite(Number(vp[2])) && isFinite(Number(vp[3])) && Number(vp[2]) > 0 && Number(vp[3]) > 0) {
          w = Number(vp[2]);
          h = Number(vp[3]);
        }
      } else {
        var wa = null;
        var ha = null;
        try { wa = svg.getAttribute('width'); ha = svg.getAttribute('height'); } catch (_eW) {}
        var wv = wa != null ? _parseLenToken(wa) : null;
        var hv = ha != null ? _parseLenToken(ha) : null;
        if (wv != null && wv > 0) w = wv;
        if (hv != null && hv > 0) h = hv;
      }
    }
    return _HORIZONTAL_ATTRS[attr] ? w : (_VERTICAL_ATTRS[attr] ? h : w);
  };

  var _lenUserUnits = function (tok, target, attr) {
    if (tok == null) return null;
    var s = String(tok).replace(/^\s+|\s+$/g, '');
    // CSS calc() 包裹：项级加减求值（SMIL 长度域 calc = 数×单位项序列，
    // % 项按视口、em/ex 按字号、物理单位按 px 因子逐项折算求和——
    // svglength-additive-by-7/8 定谳 calc(4% + 8px) = 0.04×800+8 = 40）。
    // FIXME(svg-compat-v2 B6b)：乘除/嵌套 calc 未展开（语料无此形态）。
    var cm = /^calc\((.*)\)$/.exec(s);
    if (cm) {
      var body = cm[1];
      var re = /([+-]?)\s*((?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?)([a-zA-Z%]*)/g;
      var mm;
      var total = 0;
      var any = false;
      while ((mm = re.exec(body)) !== null) {
        var v2 = Number(mm[2]);
        var u2 = mm[3].toLowerCase();
        var val;
        if (u2 === '' || u2 === 'px' || u2 === 'user') val = v2;
        else if (u2 === '%') val = v2 / 100 * _targetViewportLen(target, attr);
        else {
          var f2 = _lenUnitsPx[u2];
          if (f2) val = v2 * f2;
          else if (u2 === 'em' || u2 === 'ex') val = v2 * _targetFontSize(target) * (u2 === 'ex' ? 0.5 : 1);
          else return null;
        }
        total += (mm[1] === '-' ? -1 : 1) * val;
        any = true;
      }
      return any ? total : null;
    }
    var m = _lenTokenRe.exec(s);
    if (!m) return null;
    var v = Number(m[1]);
    var u = m[2].toLowerCase();
    if (u === '' || u === 'px' || u === 'user') return v;
    var f = _lenUnitsPx[u];
    if (f) return v * f;
    if (u === 'em' || u === 'ex') {
      var fs = _targetFontSize(target);
      return v * fs * (u === 'ex' ? 0.5 : 1);
    }
    if (u === '%') return v / 100 * _targetViewportLen(target, attr);
    return null;
  };

  var _lerp = function (a, b, f) { return a + (b - a) * f; };

  // B6：三次贝塞尔 easing 求值（CSS easing cubic-bezier 同法）——解
  // bezier_x(t)=x 取 bezier_y(t)：牛顿迭代 8 轮 + 二分 24 轮兜底
  //（animate-calcMode-spline-from-to 定谳 0.5 → 0.812 面）。
  var _bezPt = function (t, p1, p2) {
    var u = 1 - t;
    return 3 * u * u * t * p1 + 3 * u * t * t * p2 + t * t * t;
  };
  var _bezSolve = function (x, x1, y1, x2, y2) {
    if (x <= 0) return 0;
    if (x >= 1) return 1;
    var t = x;
    var i;
    for (i = 0; i < 8; i++) {
      var bx = _bezPt(t, x1, x2) - x;
      if (Math.abs(bx) < 1e-7) return _bezPt(t, y1, y2);
      var dx = 3 * (1 - t) * (1 - t) * x1 + 6 * (1 - t) * t * (x2 - x1) + 3 * t * t * (1 - x2);
      if (Math.abs(dx) < 1e-7) break;
      t -= bx / dx;
      if (t < 0) t = 0;
      if (t > 1) t = 1;
    }
    var lo = 0;
    var hi = 1;
    t = x;
    for (i = 0; i < 24; i++) {
      var bx2 = _bezPt(t, x1, x2) - x;
      if (Math.abs(bx2) < 1e-7) break;
      if (bx2 > 0) hi = t; else lo = t;
      t = (lo + hi) / 2;
    }
    return _bezPt(t, y1, y2);
  };

  // B6：分号列表解析——尾分号/尾空段忽略、中段空 → null（属性无效）
  //（keytimes/keypoints-attribute-trailing-semi 与 smil-values-empty-
  // segments 的 trailing-semi 定谳面）。
  var _parseSemiList = function (s) {
    if (s == null) return null;
    var parts = String(s).split(';');
    var arr = [];
    for (var i = 0; i < parts.length; i++) {
      var tok = parts[i].replace(/^\s+|\s+$/g, '');
      if (tok === '' && i === parts.length - 1) break;
      if (tok === '') return null;
      arr.push(tok);
    }
    return arr.length ? arr : null;
  };

  var _parseFloatTok = function (tok) {
    var m = /^\s*([+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?)\s*$/.exec(String(tok));
    return m ? Number(m[1]) : NaN;
  };

  // B6：keyTimes 校验（SMIL #KeyTimesAttributeSyntax）：项数 = 值数；
  // linear/spline 首 0 末 1、discrete 首 0（末不须 1——animate-from-to-
  // keyTimes keyTimes="0;0.25" 定谳）；非降序且各 ∈[0,1]。违例 → 整动画
  // 无效不采样（keysplines-x-limits 同语义域定谳）。
  var _validKeyTimes = function (nums, n, mode) {
    if (!nums || nums.length !== n) return null;
    if (nums[0] !== 0) return null;
    if (mode !== 'discrete' && nums[nums.length - 1] !== 1) return null;
    for (var i = 1; i < nums.length; i++) {
      if (!(nums[i] >= nums[i - 1]) || nums[i] < 0 || nums[i] > 1) return null;
    }
    return nums;
  };

  // B6：keySplines 校验（SMIL #KeySplinesAttributeSyntax）：组数 = 段数、
  // 每组 4 数、四值全部 ∈[0,1]（SMIL 文法「The values must all be in the
  // range 0 to 1」——与 CSS cubic-bezier 仅限 x 不同；keysplines-x-limits
  // /keysplines-y-limits 双面定谳，违例整动画无效）。
  var _validKeySplines = function (s, segs) {
    var groups = _parseSemiList(s);
    if (!groups || groups.length !== segs) return null;
    var out = [];
    for (var i = 0; i < groups.length; i++) {
      // 组内分隔符 = comma-wsp（SMIL 文法「separator: comma-wsp」——逗号
      // 与空白混用合法，animate-keySplines 语料形态「0 ,0  1 , 1」）。
      var nums = groups[i].split(/[\s,]+/);
      if (nums.length !== 4) return null;
      var q = [];
      for (var j = 0; j < 4; j++) {
        var v = _parseFloatTok(nums[j]);
        if (isNaN(v) || v < 0 || v > 1) return null;
        q.push(v);
      }
      out.push(q);
    }
    return out;
  };

  // 目标属性当前 base 数值（by/to 动画的 underling 起点，单位上下文解析）。
  var _baseNumeric = function (target, attr) {
    var raw = null;
    try { raw = target.getAttribute(attr); } catch (_eG) { raw = null; }
    if (raw == null) return 0;
    return _lenUserUnits(raw, target, attr);
  };

  // 单动画元素在 t 的采样值（数值域）→ number | null（不支持/不活跃）。
  // calcMode 线性（缺省）；discrete/spline = B6 FIXME（暂按线性——对应案
  // 现势恒 F，无翻转面）。
  var _sampleAnim = function (animEl, target, attr, t, underlying) {
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
    // B6：calcMode 元素缺省（set → discrete；animateMotion → paced——motion
    // 域 B6b；其余 linear）。未知值 → 无采样（spec 属性枚举封闭）。
    var calcMode = (g('calcMode') || '').replace(/^\s+|\s+$/g, '').toLowerCase();
    if (!calcMode) {
      calcMode = localName === 'set' ? 'discrete'
        : (localName === 'animateMotion' ? 'paced' : 'linear');
    }
    if (calcMode !== 'linear' && calcMode !== 'discrete'
        && calcMode !== 'spline' && calcMode !== 'paced') return null;
    var accum = g('accumulate') === 'sum';
    // frac/iter：iter = repeat 迭代序（accumulate 传播 k×(last−first)——
    // accumulate-values-width 定谳面）。
    var frac;
    var iter;
    if (cur) {
      var actTime = t - cur.b;
      var actDur = cur.e - cur.b;
      if (cur.sd === Infinity) {
        frac = 0;                             // 无 dur：恒活 from 常值
        iter = 0;
      } else if (actDur <= 0) {
        frac = 1;
        iter = 0;
      } else {
        // repeat 边界采样取下一 iteration 起点（f(t mod d)——simpleTime=0）。
        var simpleTime = actTime % cur.sd;
        frac = Math.min(simpleTime / cur.sd, 1);
        iter = Math.floor(actTime / cur.sd);
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
      // 浮点精度钳制：span ≈ k·sd（残差 < 1e-9·sd）取整迭代末（frac=1——
      // animate-end-attribute-numeric-precision 定谳：begin/dur 尾数浮点
      // 残差致 span % sd ≈ 1e-16，冻结采样值塌向迭代起点）。
      if (span > 0 && span >= sd0 && simpleTime0 < 1e-9 * sd0) simpleTime0 = sd0;
      frac = Math.min(simpleTime0 / sd0, 1);
      iter = Math.floor((span - (simpleTime0 === sd0 ? 1e-9 : 0)) / sd0);
    }
    if (localName === 'set') {
      // set：离散持有 to 常值（spec SVG2 animate.html §SetElement）。
      var sv = _lenUserUnits(toTok, target, attr);
      if (sv == null) sv = _parseNumToken(toTok);
      return sv;
    }
    // B6：spec 值表组装（values 优先于 from/to/by；全部 token 须可解析为
    // 数值域，否则整动画无采样——类型化域 color/transform/enum 走 B6b，
    // 现势同 F 无回归面）。
    var vals = null;
    var startV = null;
    var endV = null;
    if (valuesTok != null) {
      var parts = _parseSemiList(valuesTok);
      if (!parts) return null;
      vals = [];
      for (var p = 0; p < parts.length; p++) {
        var pv = _lenUserUnits(parts[p], target, attr);
        if (pv == null) return null;
        vals.push(pv);
      }
    } else if (fromTok != null && toTok != null) {
      startV = _lenUserUnits(fromTok, target, attr);
      endV = _lenUserUnits(toTok, target, attr);
      if (startV == null || endV == null) return null;
    } else if (fromTok != null && byTok != null) {
      startV = _lenUserUnits(fromTok, target, attr);
      var b0 = _lenUserUnits(byTok, target, attr);
      if (startV == null || b0 == null) return null;
      endV = startV + b0;
    } else if (byTok != null) {
      // by-only：additive="sum" 贡献 = by×f 的**增量**（起点 0——三明治
      // 加法面；svglength-additive-by 定谳）；replace 起点为 underlying
      //（SMIL #AdditiveAnimation by-without-from 语义——from 隐式取
      // underlying 值非裸 base，svglength-additive-by-2 定谳：an1 冻结
      // 50 + by×0.5 = 75）。accumulate 的迭代传播由 accDelta 承担，不改
      // 起点。
      // FIXME(svg-compat-v2 B6b)：spec underlying 取 interval 起点快照，
      // 现按采样时运行值近似——前置动画冻结/常值场景恒等，活跃前置动
      // 画待 interval 起点三明治求值。
      var byv = _lenUserUnits(byTok, target, attr);
      if (byv == null) return null;
      if (g('additive') === 'sum') {
        startV = 0;
      } else {
        startV = underlying != null ? underlying : _baseNumeric(target, attr);
        if (startV == null) return null;
      }
      endV = startV + byv;
    } else if (toTok != null) {
      // to-animation：from 取目标 base 值（SMIL to-anim 的 underlying 起点
      // 近似——Blink to-anim 特例语义 = FIXME(svg-compat-v2 B6b)）。
      startV = _baseNumeric(target, attr);
      endV = _lenUserUnits(toTok, target, attr);
      if (startV == null || endV == null) return null;
    } else {
      return null;
    }
    var list = vals != null ? vals : [startV, endV];
    var n1 = list.length;
    if (n1 === 1) return list[0];
    // B6：keyTimes/keySplines 校验（paced 忽略 keyTimes——paced-value-
    // animation-overwrites-keyTimes 定谳）；违例 → 整动画无效。
    var kt = null;
    var ks = null;
    if (calcMode !== 'paced' && g('keyTimes') != null) {
      var ktRaw = _parseSemiList(g('keyTimes'));
      var ktNums = null;
      if (ktRaw) {
        ktNums = [];
        for (var k2 = 0; k2 < ktRaw.length; k2++) {
          var kv = _parseFloatTok(ktRaw[k2]);
          if (isNaN(kv)) { ktNums = null; break; }
          ktNums.push(kv);
        }
      }
      kt = ktNums ? _validKeyTimes(ktNums, n1, calcMode) : null;
      if (kt == null) return null;
    }
    if (calcMode === 'spline' && g('keySplines') != null) {
      ks = _validKeySplines(g('keySplines'), n1 - 1);
      if (ks == null) return null;
    }
    var accDelta = accum ? (list[n1 - 1] - list[0]) : 0;
    if (calcMode === 'discrete') {
      // discrete：kt 在场值 i 自 kt[i] 起持有、否则均匀 n 段
      //（animate-number-calcMode-discrete 两值半分定谳）。
      var idx = 0;
      if (kt) {
        for (var di = 0; di < kt.length; di++) {
          if (frac >= kt[di]) idx = di; else break;
        }
      } else {
        idx = Math.min(Math.floor(frac * n1), n1 - 1);
      }
      return list[idx] + iter * accDelta;
    }
    if (calcMode === 'paced' && n1 > 2) {
      // paced：数值域按累计 |Δ| 均匀（keyTimes 忽略）。
      var totalD = 0;
      var accD = [0];
      var pi;
      for (pi = 1; pi < n1; pi++) {
        totalD += Math.abs(list[pi] - list[pi - 1]);
        accD.push(totalD);
      }
      if (totalD === 0) return list[0] + iter * accDelta;
      var dist = frac * totalD;
      for (pi = 1; pi < n1; pi++) {
        if (dist <= accD[pi] || pi === n1 - 1) {
          var segD = accD[pi] - accD[pi - 1];
          var fD = segD > 0 ? (dist - accD[pi - 1]) / segD : 0;
          return _lerp(list[pi - 1], list[pi], fD) + iter * accDelta;
        }
      }
    }
    // linear / spline（spline 段内经 _bezSolve 映射）。
    var segI;
    var segF;
    if (kt) {
      segI = n1 - 2;
      segF = 1;
      for (var li = 0; li <= n1 - 2; li++) {
        if (frac <= kt[li + 1]) { segI = li; break; }
      }
      var ktSpan = kt[segI + 1] - kt[segI];
      segF = ktSpan > 0 ? (frac - kt[segI]) / ktSpan : 1;
    } else {
      var segPos = frac * (n1 - 1);
      segI = Math.floor(segPos);
      if (segI >= n1 - 1) {
        segI = n1 - 2;
        segF = 1;
      } else {
        segF = segPos - segI;
      }
    }
    if (ks) segF = _bezSolve(segF, ks[segI][0], ks[segI][1], ks[segI][2], ks[segI][3]);
    return _lerp(list[segI], list[segI + 1], segF) + iter * accDelta;
  };

  // ── 类型域 token 采样（B6b：enum/boolean/integer/string/angle/list 反射
  // 消费面）。与数值域共享 interval 解析助手，值表按 raw token 组装；插值
  // 域分类（SMIL 3 #InterpolationDefaults：不可插值类型 linear/spline/paced
  // 回落 discrete——from-to 半程切换）：
  // ① discrete：按 kt/均匀段持有原 token；
  // ② 全 token 角度域（deg/grad/rad 可选后缀）：度数线性插值 + 首 token
  //   后缀（svgangle-*-deg-grad/rad 面——animVal 反射经角度解析还原
  //   unitType/度数）；
  // ③ 全 token 数值域：段端点列表 tiling（短表重复——SMIL 列表插值语义）
  //   逐项线性空格连接（lengthlist/numberlist/viewBox/point 面）；
  // ④ 其余（enum/boolean/string/URI）：离散半程切换兜底。
  // 三明治 = replace（文档序取最后非空——类型域无 sum 合成面，spec
  // #AdditiveAnimation 非数值域 additive 恒无效按 replace 处理）。
  var _zwAngleTokRe = /^\s*([+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?)\s*(deg|grad|rad)?\s*$/i;
  var _zwAngleDegOf = function (tok) {
    var m = _zwAngleTokRe.exec(String(tok));
    if (!m) return null;
    var v = Number(m[1]);
    var u = (m[2] || '').toLowerCase();
    if (u === 'grad') return v * 0.9;
    if (u === 'rad') return v * 180 / Math.PI;
    return v;
  };
  var _zwNumFmt6 = function (v) { return String(Math.round(v * 1e6) / 1e6); };
  // 列表 token → user units 数组（长度单位经 _parseLenToken 折算；任一项
  // 不可解析 → null——整动画无效负向面同数值域口径）。
  var _zwListNumItems = function (tok) {
    var parts = String(tok).trim().split(/[\s,]+/);
    var items = [];
    for (var i = 0; i < parts.length; i++) {
      if (parts[i] === '') continue;
      var v = _parseLenToken(parts[i]);
      if (v == null) return null;
      items.push(v);
    }
    return items.length ? items : null;
  };
  // 数值列表 token 逐项求和（from-by/by-only 段端点合成——SMIL
  // #FromByAnimation f(t) = from + by·p 的 b 侧；tile 短表）。
  var _zwListSumTokens = function (aTok, byTok) {
    var sa = _zwListNumItems(aTok);
    var sb = _zwListNumItems(byTok);
    if (!sa || !sb) return null;
    var n = Math.max(sa.length, sb.length);
    var out = [];
    for (var i = 0; i < n; i++) out.push(_zwNumFmt6(sa[i % sa.length] + sb[i % sb.length]));
    return out.join(' ');
  };
  var _sampleAnimStr = function (animEl, target, attr, t) {
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
    var frac;
    var iter;
    if (cur) {
      var actTime = t - cur.b;
      var actDur = cur.e - cur.b;
      if (cur.sd === Infinity) {
        frac = 0;                             // 无 dur：恒活 from 常值
        iter = 0;
      } else if (actDur <= 0) {
        frac = 1;
        iter = 0;
      } else {
        var simpleTime = actTime % cur.sd;
        frac = Math.min(simpleTime / cur.sd, 1);
        iter = Math.floor(actTime / cur.sd);
      }
    } else {
      // 冻结（fill=freeze 且存在已结束的 last interval）——同数值域精度钳制。
      var fillAttr = g('fill');
      if (fillAttr !== 'freeze') return null;
      var sd0 = d.kind === 'finite' ? d.s : Infinity;
      var rec0 = _chainOf(animEl, t);
      var lastEnd = -Infinity;
      var lastB = 0;
      if (rec0) {
        for (var j = 0; j < rec0.ivs.length; j++) {
          var iv0 = rec0.ivs[j];
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
      if (span > 0 && span >= sd0 && simpleTime0 < 1e-9 * sd0) simpleTime0 = sd0;
      frac = Math.min(simpleTime0 / sd0, 1);
      iter = Math.floor((span - (simpleTime0 === sd0 ? 1e-9 : 0)) / sd0);
    }
    var toTok = g('to');
    var fromTok = g('from');
    var byTok = g('by');
    var valuesTok = g('values');
    if (localName === 'set') return toTok;
    // raw token 值表（类型域无 by 增量语义——from-by/by-only 无效 → null）。
    var toks = null;
    if (valuesTok != null) {
      toks = _parseSemiList(valuesTok);
      if (!toks) return null;
    } else if (fromTok != null && toTok != null) {
      toks = [fromTok, toTok];
    } else if (fromTok != null && byTok != null) {
      // from-by：f(t) = from + by·p（SMIL #FromByAnimation——列表域逐项
      // user units 合成段端点，svglengthlist-4/svgpointlist-2/svgrect-2 面）。
      var sumTok = _zwListSumTokens(fromTok, byTok);
      if (sumTok == null) return null;
      toks = [fromTok, sumTok];
    } else if (byTok != null) {
      // by-only：起点取目标属性现值逐项 + by（属性缺席/不可解析 → 无采样）；
      // 列表合成不达（单位化角度 token）→ 角度域度数合成（SMIL
      // #FromByAnimation——B6c：additive-type-by orient by-only 定谳
      // -45deg + 45deg·p，段端点角度插值共用②路径）。
      var baseRaw2 = null;
      try { baseRaw2 = target.getAttribute(attr); } catch (_eB2) { baseRaw2 = null; }
      if (baseRaw2 == null) return null;
      var sumTok2 = _zwListSumTokens(String(baseRaw2), byTok);
      if (sumTok2 != null) {
        toks = [String(baseRaw2), sumTok2];
      } else {
        var daB = _zwAngleDegOf(baseRaw2);
        var dbB = _zwAngleDegOf(byTok);
        if (daB == null || dbB == null) return null;
        toks = [String(baseRaw2), _zwNumFmt6(daB + dbB)];
      }
    } else if (toTok != null) {
      // to-only：起点取目标属性现值（SMIL to-anim 语义 base→to；属性缺席
      // 保 null——采样返 null 时消费面回落 base 反射即类型缺省值，
      // svgboolean-1 preserveAlpha 缺席 false 面）。
      var baseRaw = null;
      try { baseRaw = target.getAttribute(attr); } catch (_eB) { baseRaw = null; }
      toks = [baseRaw != null ? String(baseRaw) : null, toTok];
    } else {
      return null;
    }
    var n1 = toks.length;
    if (n1 === 1) return toks[0];
    var calcMode = (g('calcMode') || '').replace(/^\s+|\s+$/g, '').toLowerCase();
    if (!calcMode) calcMode = 'linear';
    if (calcMode !== 'linear' && calcMode !== 'discrete'
        && calcMode !== 'spline' && calcMode !== 'paced') return null;
    var kt = null;
    var ks = null;
    if (calcMode !== 'paced' && g('keyTimes') != null) {
      var ktRaw = _parseSemiList(g('keyTimes'));
      var ktNums = null;
      if (ktRaw) {
        ktNums = [];
        for (var k2 = 0; k2 < ktRaw.length; k2++) {
          var kv = _parseFloatTok(ktRaw[k2]);
          if (isNaN(kv)) { ktNums = null; break; }
          ktNums.push(kv);
        }
      }
      kt = ktNums ? _validKeyTimes(ktNums, n1, calcMode) : null;
      if (kt == null) return null;
    }
    if (calcMode === 'spline' && g('keySplines') != null) {
      ks = _validKeySplines(g('keySplines'), n1 - 1);
      if (ks == null) return null;
    }
    if (calcMode === 'discrete') {
      // discrete：kt 在场值 i 自 kt[i] 起持有、否则均匀 n 段。
      var idx = 0;
      if (kt) {
        for (var di = 0; di < kt.length; di++) {
          if (frac >= kt[di]) idx = di; else break;
        }
      } else {
        idx = Math.min(Math.floor(frac * n1), n1 - 1);
      }
      return toks[idx];
    }
    // 段定位（与数值域同构；spline 段内经 _bezSolve 映射）。
    var segI;
    var segF;
    if (kt) {
      segI = n1 - 2;
      segF = 1;
      for (var li = 0; li <= n1 - 2; li++) {
        if (frac <= kt[li + 1]) { segI = li; break; }
      }
      var ktSpan = kt[segI + 1] - kt[segI];
      segF = ktSpan > 0 ? (frac - kt[segI]) / ktSpan : 1;
    } else {
      var segPos = frac * (n1 - 1);
      segI = Math.floor(segPos);
      if (segI >= n1 - 1) {
        segI = n1 - 2;
        segF = 1;
      } else {
        segF = segPos - segI;
      }
    }
    if (ks) segF = _bezSolve(segF, ks[segI][0], ks[segI][1], ks[segI][2], ks[segI][3]);
    var a = toks[segI];
    var b = toks[segI + 1];
    // ②角度域：度数域插值 + 段端点单位回写（SVGAngle.value 恒为度数；
    // animVal 单位段内半程前取 from、后半程取 to——unitType 面 deg→200grad
    // t=2.0 DEG / t=3.999 GRAD 定谳；插值度数按回写单位折算：90 度按 grad
    // 表达写 100grad）。to-only 起点属性缺席按角度缺省 0（无单位=度）。
    var da = _zwAngleDegOf(a);
    var db = _zwAngleDegOf(b);
    if (db != null && da == null && a == null) {
      a = '0';
      da = 0;
    }
    if (da != null && db != null) {
      var sufRe = /^\s*[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?\s*([a-zA-Z]*)\s*$/;
      var sufA = (sufRe.exec(String(a)) || ['', ''])[1];
      var sufB = (sufRe.exec(String(b)) || ['', ''])[1];
      var suf = segF <= 0.5 ? sufA : sufB;
      var sufL = suf.toLowerCase();
      var uf = sufL === 'grad' ? 0.9 : sufL === 'rad' ? 180 / Math.PI : 1;
      return _zwNumFmt6(_lerp(da, db, segF) / uf) + suf;
    }
    // ③数值列表域：逐项 user units 折算 + 段线性。列表形态（任一端多
    // token）而任一项不可解析 → 整动画无效（SMIL invalid-value 负向面——
    // svglengthlist/numberlist/pointlist-invalid-value 族，不落 discrete
    // 兜底）；tile 短表；to 端项数多于 from → 常值 to 列表不插值
    //（svglengthlist-3 数对不齐定谳 WebKit 行为）；to 端更少按 from tile
    // 逐项插值（svgnumberoptionalnumber-3 面）。单 token 数值对直线性。
    var aParts = String(a).trim().split(/[\s,]+/);
    var bParts = String(b).trim().split(/[\s,]+/);
    var aN = 0;
    var bN = 0;
    for (var ap = 0; ap < aParts.length; ap++) if (aParts[ap] !== '') aN++;
    for (var bp = 0; bp < bParts.length; bp++) if (bParts[bp] !== '') bN++;
    var na = _zwListNumItems(a);
    var nb = _zwListNumItems(b);
    if ((aN > 1 || bN > 1) && (!na || !nb)) return null;
    if (na && nb) {
      if (nb.length > na.length) return b;
      var n2 = Math.max(na.length, nb.length);
      var out = [];
      for (var ii = 0; ii < n2; ii++) {
        out.push(_zwNumFmt6(_lerp(na[ii % na.length], nb[ii % nb.length], segF)));
      }
      return out.join(' ');
    }
    // ④不可插值（enum/boolean/string/URI）：SMIL #InterpolationDefaults——
    // 不可插值类型按 discrete 语义整值表持有（svgenum-4 五值五秒整秒切换
    // 面）。无 keyTimes 时均匀段边界取严格 >（harness setCurrentTime 精确
    // seek 半程——svgboolean-1 t=2.0 归前段 false 定谳，ceil-1 与 WebKit
    // 虚拟 keyTimes 语义一致）；keyTimes 在场按 SMIL [kt[i], kt[i+1]) 持有。
    // from-to（n=2）半程前 from 后 to。
    var di2 = 0;
    if (kt) {
      for (var dq = 0; dq < kt.length; dq++) {
        if (frac >= kt[dq]) di2 = dq; else break;
      }
    } else {
      var segRaw = Math.ceil(frac * n1) - 1;
      if (segRaw < 0) segRaw = 0;
      di2 = Math.min(segRaw, n1 - 1);
    }
    return toks[di2];
  };

