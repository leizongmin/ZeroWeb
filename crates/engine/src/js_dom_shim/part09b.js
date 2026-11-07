  // ── B6c：animateTransform 采样（SMIL #AnimateTransformElement + SVG2
  //    coords.html #AnimateTransformElement；消费面 = part03
  //    SVGAnimatedTransformList.animVal parse 的 _zwSmilAnimTransform 钩子）──
  // type 域组件向量插值：translate(1-2)/scale(1-2)/rotate(1-3)/skewX/skewY(1)
  // 数值分量；type 缺省 translate（SVG1.1 animateTransform type 缺省值）、
  // 未知 type 无采样（属性枚举封闭）。calcMode linear 缺省/discrete/spline
  // 同数值域（paced 对 transform 未定义 → linear 回落 FIXME(svg-compat-v2
  // M3)）；accumulate="sum" 迭代传播 k×(last−first) 逐分量。by-only 起点取
  // 零向量（SMIL #AnimateTransformElement by 语义——additive-type-by 定谳：
  // rotate(45) base 上 by="-45" 采样 0→-45，不基于 base）；set/animate 元素
  // 与 to-only 无语料定谳面 → 无采样 FIXME(svg-compat-v2 M3)。

  // 采样相位：active 简单时间 repeat 取模 / freeze 冻结 activeEnd 浮点钳制
  //（与数值/类型域采样器同式——本域新增，前两采样器路径保持原样不回改）。
  // 返回 {frac, iter, em, dm} 或 null（不活跃且无冻结）；em/dm = B7 path d
  // 域 EDM 整数 ms 相位（simpleTime/dur ×1000 取整，null = degenerate/无限
  // 相位——数值域消费方忽略）。
  var _smilPhase = function (animEl, g, t) {
    var cur = _activeInterval(animEl, t);
    var d = _parseDur(animEl);
    if (d.kind === 'invalid') return null;
    var frac;
    var iter;
    var em = null;
    var dm = null;
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
        em = Math.round(simpleTime * 1000);
        dm = Math.round(cur.sd * 1000);
      }
    } else {
      var fillAttr = g('fill');
      if (fillAttr !== 'freeze') return null;
      var sd0 = d.kind === 'finite' ? d.s : Infinity;
      var rec0 = _chainOf(animEl, t);
      var lastEnd = -Infinity;
      var lastB = 0;
      if (rec0) {
        for (var j = 0; j < rec0.ivs.length; j++) {
          var iv0 = rec0.ivs[j];
          // 剪枝 interval（e ≤ 0）非 real interval（first-interval 剪枝恒持）。
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
      // 冻结迭代边界浮点钳制（animate-end-attribute-numeric-precision 同式）。
      if (span > 0 && span >= sd0 && simpleTime0 < 1e-9 * sd0) simpleTime0 = sd0;
      frac = Math.min(simpleTime0 / sd0, 1);
      iter = Math.floor((span - (simpleTime0 === sd0 ? 1e-9 : 0)) / sd0);
      // 冻结钳制后 simpleTime0 = sd0 → em = dm → 插值精确落 to 侧终值。
      em = Math.round(simpleTime0 * 1000);
      dm = Math.round(sd0 * 1000);
    }
    return { frac: frac, iter: iter, em: em, dm: dm };
  };

  // token → type 域组件向量（comma-wsp 分隔，分量全数值——任一不可解析/
  // 分量数越界 → null 整动画无效）。
  var _zwTrVecOf = function (tok, type) {
    if (tok == null) return null;
    var parts = String(tok).trim().split(/[\s,]+/);
    var nums = [];
    for (var i = 0; i < parts.length; i++) {
      if (parts[i] === '') continue;
      var v = _parseFloatTok(parts[i]);
      if (isNaN(v)) return null;
      nums.push(v);
    }
    if (type === 2) {          // translate：tx [ty=0]
      if (!nums.length || nums.length > 2) return null;
      return [nums[0], nums.length > 1 ? nums[1] : 0];
    }
    if (type === 3) {          // scale：sx [sy=sx]
      if (!nums.length || nums.length > 2) return null;
      return [nums[0], nums.length > 1 ? nums[1] : nums[0]];
    }
    if (type === 4) {          // rotate：angle [cx=0, cy=0]
      if (!nums.length || nums.length > 3) return null;
      return [nums[0], nums.length > 1 ? nums[1] : 0, nums.length > 2 ? nums[2] : 0];
    }
    if (!nums.length || nums.length > 1) return null;   // skewX/skewY：angle
    return [nums[0]];
  };

  // 组件向量 → transform 态（公式同 part03 _zwSvgTransformParse——闭包不可
  // 反向引用，式样复制；rotate 中心点 e = cx(1−cos) + cy·sin）。
  var _zwTrStateOf = function (v, type) {
    if (type === 2) return { type: 2, a: 1, b: 0, c: 0, d: 1, e: v[0], f: v[1] };
    if (type === 3) return { type: 3, a: v[0], b: 0, c: 0, d: v[1], e: 0, f: 0 };
    if (type === 4) {
      var r = v[0] * Math.PI / 180;
      var co = Math.cos(r), si = Math.sin(r);
      return { type: 4, a: co, b: si, c: -si, d: co,
        e: v[1] * (1 - co) + v[2] * si, f: v[2] * (1 - co) - v[1] * si, ang: v[0] };
    }
    if (type === 5) return { type: 5, a: 1, b: 0, c: Math.tan(v[0] * Math.PI / 180), d: 1, e: 0, f: 0, ang: v[0] };
    return { type: 6, a: 1, b: Math.tan(v[0] * Math.PI / 180), c: 0, d: 1, e: 0, f: 0, ang: v[0] };
  };

  var _sampleAnimTransform = function (animEl, target, attr, t) {
    var an = null;
    try { an = animEl.getAttribute('attributeName'); } catch (_eA) { an = null; }
    if (an !== attr) return null;
    var localName = '';
    try { localName = String(animEl.localName || '').toLowerCase(); } catch (_eL) {}
    if (localName !== 'animatetransform') return null;
    var g = function (name) {
      var v = null;
      try { v = animEl.getAttribute(name); } catch (_eG) { v = null; }
      return v == null ? null : String(v);
    };
    var ph = _smilPhase(animEl, g, t);
    if (!ph) return null;
    var frac = ph.frac;
    var iter = ph.iter;
    var typeStr = (g('type') || '').replace(/^\s+|\s+$/g, '');
    var type = typeStr === '' ? 2
      : (typeStr === 'translate' ? 2 : typeStr === 'scale' ? 3
        : typeStr === 'rotate' ? 4 : typeStr === 'skewX' ? 5
          : typeStr === 'skewY' ? 6 : 0);
    if (type === 0) return null;
    var toTok = g('to');
    var fromTok = g('from');
    var byTok = g('by');
    var valuesTok = g('values');
    var vals = null;
    var startV = null;
    var endV = null;
    if (valuesTok != null) {
      var parts = _parseSemiList(valuesTok);
      if (!parts) return null;
      vals = [];
      for (var p = 0; p < parts.length; p++) {
        var pv = _zwTrVecOf(parts[p], type);
        if (pv == null) return null;
        vals.push(pv);
      }
    } else if (fromTok != null && toTok != null) {
      startV = _zwTrVecOf(fromTok, type);
      endV = _zwTrVecOf(toTok, type);
      if (startV == null || endV == null) return null;
    } else if (fromTok != null && byTok != null) {
      // from-by：f(t) = from + by·p（SMIL #FromByAnimation 逐分量）。
      startV = _zwTrVecOf(fromTok, type);
      var bv = _zwTrVecOf(byTok, type);
      if (startV == null || bv == null) return null;
      endV = [];
      for (var fb = 0; fb < startV.length; fb++) endV.push(startV[fb] + bv[fb]);
    } else if (byTok != null) {
      // by-only：零向量起点（additive-type-by 定谳——不基于 base）。
      var bv2 = _zwTrVecOf(byTok, type);
      if (bv2 == null) return null;
      startV = [];
      for (var zb = 0; zb < bv2.length; zb++) startV.push(0);
      endV = bv2;
    } else {
      return null;   // to-only：FIXME(svg-compat-v2 M3) 无语料定谳面
    }
    var list = vals != null ? vals : [startV, endV];
    var n1 = list.length;
    if (n1 === 1) return _zwTrStateOf(list[0], type);
    var calcMode = (g('calcMode') || '').replace(/^\s+|\s+$/g, '').toLowerCase();
    if (!calcMode) calcMode = 'linear';
    if (calcMode !== 'linear' && calcMode !== 'discrete'
        && calcMode !== 'spline' && calcMode !== 'paced') return null;
    if (calcMode === 'paced') calcMode = 'linear';   // FIXME(svg-compat-v2 M3)
    var kt = null;
    var ks = null;
    if (g('keyTimes') != null) {
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
    var accum = g('accumulate') === 'sum';
    // 迭代传播 k×(last−first) 逐分量（accumulate 同数值域）。
    var accDelta = null;
    if (accum) {
      accDelta = [];
      for (var ac = 0; ac < list[n1 - 1].length; ac++) accDelta.push(list[n1 - 1][ac] - list[0][ac]);
    }
    if (calcMode === 'discrete') {
      var idx = 0;
      if (kt) {
        for (var di = 0; di < kt.length; di++) {
          if (frac >= kt[di]) idx = di; else break;
        }
      } else {
        idx = Math.min(Math.floor(frac * n1), n1 - 1);
      }
      return _zwTrStateOf(_zwTrVecShift(list[idx], accDelta, iter), type);
    }
    // 段定位（linear/spline，与数值域同构；spline 段内 _bezSolve 映射）。
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
    var aV = list[segI];
    var bV = list[segI + 1];
    var outV = [];
    for (var ov = 0; ov < aV.length; ov++) {
      outV.push(_lerp(aV[ov], bV[ov], segF) + (accDelta ? accDelta[ov] * iter : 0));
    }
    return _zwTrStateOf(outV, type);
  };
  var _zwTrVecShift = function (v, accDelta, iter) {
    if (!accDelta) return v.slice();
    var out = [];
    for (var i = 0; i < v.length; i++) out.push(v[i] + accDelta[i] * iter);
    return out;
  };

  // transform 域三明治钩子（part03 _zwMakeSvgTransformList animVal parse
  // 接线——globalThis 契约）。文档序动画项合成：replace 重置动画项集、
  // additive="sum" 追加（SMIL #AdditiveAnimation sandwich——transform 域的
  // 「加」= 后乘，表项级表示为列表追加）。返回 { items: [态], clearBase }
  // 或 null（无活跃/冻结动画）。clearBase：gradientTransform/patternTransform
  // 的 replace 整表重置（animate-gradient-transform 定谳 animVal 单项）；
  // transform 属性 replace 不清 base——SVG2 transform property 动画恒与
  // base 复合（additive-type-by 定谳 animVal = base 项 + 动画项 = 2）。
  globalThis._zwSmilAnimTransform = function (el, attr) {
    var doc = null;
    try { doc = el.ownerDocument; } catch (_eD) { doc = null; }
    if (!doc) return null;
    var st = _docTimelines.get(doc);
    if (st && st.tickEverRan && !st.smilSeen) return null;
    var t = st ? _currentTimeOf(st) : 0;
    var kids = null;
    try { kids = el.children; } catch (_eC) { kids = null; }
    var hrefElsX = null;
    if (!st || !st.tickEverRan || st.smilSeen) {
      try { hrefElsX = _zwHrefAnimList(doc, el); } catch (_eHX) { hrefElsX = null; }
    }
    if ((!kids || !kids.length) && !hrefElsX) return null;
    var items = null;
    var clearBase = false;
    var runTrX = function (list, isHrefPass) {
      if (!list) return;
      for (var i = 0; i < list.length; i++) {
        var c = list[i];
        var ln = '';
        try { ln = String(c.localName || '').toLowerCase(); } catch (_eL) {}
        if (!_animLocalNames[ln]) continue;
        if (!isHrefPass && _zwAnimHrefOf(c) != null) continue;    // href 指靶归 href 面处理
        var stv = null;
        try { stv = _sampleAnimTransform(c, el, attr, t); } catch (_eSv) { stv = null; }
        if (stv == null) continue;
        var isSum = false;
        try { isSum = c.getAttribute('additive') === 'sum'; } catch (_eAd) { isSum = false; }
        if (isSum) {
          if (!items) items = [];
          items.push(stv);
        } else {
          items = [stv];
          clearBase = attr !== 'transform';
        }
      }
    };
    runTrX(kids, false);
    runTrX(hrefElsX, true);
    if (!items) return null;
    return { items: items, clearBase: clearBase };
  };

  // 动画元素 href 值（SVG2 href 优先、xlink:href 回落——markup 冒号名归
  // local、setAttributeNS 字面量保 qname，两名并查）。
  var _zwAnimHrefOf = function (c) {
    var href = null;
    try { href = c.getAttribute('href'); } catch (_eH0) { href = null; }
    if (href == null || String(href) === '') {
      try { href = c.getAttribute('xlink:href'); } catch (_eH1) { href = null; }
    }
    return href == null ? null : String(href).replace(/^\s+|\s+$/g, '');
  };
  // href 显式靶候选列表（SMIL #TargetElement：href/heref 指靶——靶不必是
  // 动画元素父级；additive-type-by 等 href 语料面。读时解析——
  // change-target-while-animating setAttributeNS 改靶即时生效）。无 id 靶
  // 不可被 href 指到 → null（零扫描快速回落）。children 隐式定位与 href
  // 两类语料不混用同属性，合并顺序 children 先、href 后覆盖（文档序近似
  // FIXME(svg-compat-v2 M3)）。
  var _zwHrefAnimList = function (doc, el) {
    var elId = null;
    try { elId = el.getAttribute('id'); } catch (_eHi) { elId = null; }
    if (elId == null || String(elId) === '') return null;
    var els = _scanDoc(doc);
    var out = null;
    for (var i = 0; i < els.length; i++) {
      var frag = _zwAnimHrefOf(els[i]);
      if (frag == null || frag.charAt(0) !== '#' || frag.slice(1) !== String(elId)) continue;
      if (!out) out = [];
      out.push(els[i]);
    }
    return out;
  };

  // 目标元素 + 属性名 → 文档序动画采样值三明治合成（SMIL #AdditiveAnimation
  // sandwich model：replace 重置运行值、additive="sum" 累加——文档序优先级
  // 后者高；首个 sum 无前置 replace 时以目标 base 起底）。
  var _animatedValue = function (target, attr, numeric) {
    var doc = null;
    try { doc = target.ownerDocument; } catch (_eD) { doc = null; }
    if (!doc) return null;
    var st = _docTimelines.get(doc);
    var t = st ? _currentTimeOf(st) : 0;
    var kids = null;
    try { kids = target.children; } catch (_eC) { kids = null; }
    var hrefElsN = null;
    if (!st || !st.tickEverRan || st.smilSeen) {
      try { hrefElsN = _zwHrefAnimList(doc, target); } catch (_eHN) { hrefElsN = null; }
    }
    if ((!kids || !kids.length) && !hrefElsN) return null;
    var result = null;
    var runAvN = function (list, isHrefPass) {
      if (!list) return;
      for (var i = 0; i < list.length; i++) {
        var c = list[i];
        var ln = '';
        try { ln = String(c.localName || '').toLowerCase(); } catch (_eL) {}
        if (!_animLocalNames[ln]) continue;
        if (!isHrefPass && _zwAnimHrefOf(c) != null) continue;    // href 指靶归 href 面处理
        var v = _sampleAnim(c, target, attr, t, result);
        if (v == null) continue;
        var isSum = false;
        try { isSum = c.getAttribute('additive') === 'sum'; } catch (_eAd) { isSum = false; }
        if (isSum) {
          if (result == null) {
            var b0 = _baseNumeric(target, attr);
            result = (b0 != null ? b0 : 0) + v;
          } else {
            result = result + v;
          }
        } else {
          result = v;    // replace 重置
        }
      }
    };
    runAvN(kids, false);
    runAvN(hrefElsN, true);
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


  // 类型域 token 反射消费单点（part03 B6b 接线——globalThis 契约）。文档序
  // 动画元素取最后非空采样（replace 语义）；fast-path 与数值域同口径
  //（首 tick 后无 SMIL 元素文档零开销返——tickEverRan/smilSeen）。
  globalThis._zwSmilAnimToken = function (el, attr) {
    var doc = null;
    try { doc = el.ownerDocument; } catch (_eD) { doc = null; }
    if (!doc) return null;
    var st = _docTimelines.get(doc);
    if (st && st.tickEverRan && !st.smilSeen) return null;
    var t = st ? _currentTimeOf(st) : 0;
    var kids = null;
    try { kids = el.children; } catch (_eC) { kids = null; }
    var hrefElsT = null;
    try { hrefElsT = _zwHrefAnimList(doc, el); } catch (_eHT) { hrefElsT = null; }
    if ((!kids || !kids.length) && !hrefElsT) return null;
    var out = null;
    var runTkT = function (list, isHrefPass) {
      if (!list) return;
      for (var i = 0; i < list.length; i++) {
        var c = list[i];
        var ln = '';
        try { ln = String(c.localName || '').toLowerCase(); } catch (_eL) {}
        if (!_animLocalNames[ln]) continue;
        if (!isHrefPass && _zwAnimHrefOf(c) != null) continue;    // href 指靶归 href 面处理
        var v = _sampleAnimStr(c, el, attr, t);
        if (v == null) continue;
        var isSumTk = false;
        try { isSumTk = c.getAttribute('additive') === 'sum'; } catch (_eAdT) { isSumTk = false; }
        if (isSumTk) {
          // additive=sum 三明治（token 域逐项 user units——svglengthlist-5 面
          // base+采样 delta；任一侧不可解析回落 replace 语义）。
          var biT = null;
          try { biT = el.getAttribute(attr); } catch (_eBaT) { biT = null; }
          var baseItems = biT != null ? _zwListNumItems(biT) : null;
          var vItems = _zwListNumItems(v);
          if (baseItems && vItems) {
            var cntS = Math.max(baseItems.length, vItems.length);
            var outS = [];
            for (var sq = 0; sq < cntS; sq++) {
              outS.push(_zwNumFmt6(baseItems[sq % baseItems.length] + vItems[sq % vItems.length]));
            }
            out = outS.join(' ');
          } else {
            out = v;
          }
        } else {
          out = v;    // replace 重置
        }
      }
    };
    runTkT(kids, false);
    runTkT(hrefElsT, true);
    return out;
  };

  // ── B7：path 'd' 插值（SMIL path data 域 + SVG1.1 SVGPathSeg 消费面）──
  // 语义定谳（svg/animations path 语料 33 案，svg-compat-v2 M3/B7）：
  // ① 兼容 = 段数相同且逐段大写同型；from-to 不兼容 → 强制 discrete（half
  //   含边界 frac≥0.5→to；mismatch-same-seg-count/diff-seg-counts/
  //   empty-from/empty-to 四面定谳）；by/from-by 不兼容或一侧空 → 无效
  //   回落 base（by/from-by-mismatch、empty-from/empty-by/both-empty、
  //   by-empty-base 定谳）。
  // ② 隐式 moveto 重复组保留 M 型不转 L（mismatch-same-seg-count 定谳：
  //   from "M…L…M…M…" vs to "M…L…L…L…" discrete——与 part03
  //   _zwSvgPathDataOf 的 M→L 转换有意分叉，彼为消费语义、此为兼容判定）；
  //   几何行走按新子路径 moveto（Mm-Aa-Z 定谳 Z 回 repeat-m 点）。
  // ③ 插值 = 端点绝对化后逐参数；输出字母大小写随 to/by 侧；输出字母大写
  //   产绝对坐标、小写产相对 delta（in_array 双形态：小写 to 侧自动落
  //   formA 相对形态、大写落 formB 绝对形态/inverse-equals——Ll-Vv-Hh、
  //   Mm-Aa-Z、Cc-Ss、Qq-Tt 八面定谳）。
  // ④ by/from-by = 原始逐参数分量加法（underlying + p·by，含 A 全 7 参、
  //   无需绝对化换算——svgpath-animation-1 定谳；accumulate 系数
  //   k = segF+iter）；from-to 插值 = lerp(fromAbs,toAbs,segF)，
  //   accumulate 迭代叠加 = iter·toAbs（from-to-accumulate-sum-empty-base
  //   定谳「Iteration 1 adds 1×to」，与 by 域分叉）且 additive 缺省
  //   replace 不混叠 base。
  // ⑤ additive="sum" 三明治：全值逐参数加 prior-or-base（from-to-additive
  //   定谳 base+lerp(from,to,p)）；空采样值加非空 prior → prior 保全
  //   （additive-sum-both-empty-over-base 定谳）；加不了 → 该采样无效
  //   跳过（by-additive-mismatch 定谳）。
  // ⑥ 消费面：assert_animated_path_* → 'animatedPathSegList' in target
  //   分支（animated-path-helpers.js）：pathSegList/animatedPathSegList
  //   段对象（pathSegTypeAsLetter + 类型域字段 + Array.from 迭代——R374
  //   iterable 同构）。CSS d property/inherit 面（svgpath-animation-2）=
  //   挂账 #4 不入本批。values 列表（非 inherit 形态）FIXME(svg-compat-v2
  //   M4)。
  var _zwPathArity = function (U) {
    return U === 'H' || U === 'V' ? 1
      : U === 'C' ? 6 : U === 'S' || U === 'Q' ? 4
      : U === 'A' ? 7 : U === 'Z' ? 0 : 2;  // M/L/T
  };
  // d 字符串 → 原始段表 [{L,u,a}]（L 原文大小写、u 大写型、a 原始参数）或
  // null（非法 token）。逗号/空白 = comma-wsp 分隔符；首命令前数字 → null；
  // 尾组不完整 → 丢弃（前缀保留，§PathDataErrorHandling）；Z 后数字 →
  // 截断保留前缀（part03 同口径）；'' / 纯空白 → []（合法空路径）；首段
  // 非 moveto（bare 'Z'）→ []。隐式 moveto 重复组保留 M 型不转 L（定谳②
  // mismatch-same-seg-count 兼容判定面）；几何行走按新子路径 moveto（
  // Mm-Aa-Z 定谳 Z 回 repeat-m 点）。
  var _zwPathSegsOf = function (d) {
    if (d == null) return null;
    var str = String(d);
    var segs = [];
    var curL = null;
    var curU = null;
    var sawZ = false;
    var args = [];
    var re = /([MmLlHhVvCcSsQqTtAaZz])|([+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?)|,|(\S)|(\s+)/g;
    var m;
    while ((m = re.exec(str)) !== null) {
      if (m[1] != null) {                             // 命令字母
        if (curU != null && args.length % _zwPathArity(curU) !== 0) {
          args = [];                                  // 尾组不完整 → 丢弃
        }
        curL = m[1];
        curU = curL >= 'A' && curL <= 'Z' ? curL : String.fromCharCode(curL.charCodeAt(0) - 32);
        if (curU === 'Z') {
          segs.push({ L: curL, u: 'Z', a: [] });
          sawZ = true;
          curL = null;
          curU = null;
        }
      } else if (m[2] != null) {                      // 数字
        if (curU == null) {
          if (sawZ) break;                            // Z 后数字 → 截断
          return null;                                // 首命令前数字 → 非法
        }
        args.push(Number(m[2]));
        var ar = _zwPathArity(curU);
        if (ar > 0 && args.length === ar) {
          segs.push({ L: curL, u: curU, a: args });
          args = [];
        }
      } else if (m[3] != null) {
        return null;                                  // 非命令非数字非分隔 → 非法
      }
      // 逗号（裸备选）/空白（m[4]）= 分隔符跳过（comma-wsp）
    }
    if (segs.length && segs[0].u !== 'M') return [];  // 非 moveto 开头 → 空序列
    return segs;
  };
  // 原始段表 → 绝对化段表（相对端点经 current point 折算；A 仅 x,y 折算、
  // 其余 5 参保持；Z 复位 current point 至子路径起点；M 恒重置子路径起点
  // ——含隐式重复组）。字母/型保持——原始参数面由加法/插值消费。
  var _zwPathAbsOf = function (segs) {
    var out = [];
    var cx = 0;
    var cy = 0;
    var sx = 0;
    var sy = 0;
    for (var i = 0; i < segs.length; i++) {
      var sg = segs[i];
      var rel = sg.L !== sg.u;
      var v = sg.a.slice();
      var U = sg.u;
      if (U === 'M') {
        if (rel) { v[0] += cx; v[1] += cy; }
        sx = v[0]; sy = v[1];
        cx = v[0]; cy = v[1];
      } else if (U === 'L' || U === 'T') {
        if (rel) { v[0] += cx; v[1] += cy; }
        cx = v[0]; cy = v[1];
      } else if (U === 'H') {
        v[0] = rel ? cx + v[0] : v[0];
        cx = v[0];
      } else if (U === 'V') {
        v[0] = rel ? cy + v[0] : v[0];
        cy = v[0];
      } else if (U === 'C') {
        if (rel) {
          for (var ci = 0; ci < 6; ci += 2) { v[ci] += cx; v[ci + 1] += cy; }
        }
        cx = v[4]; cy = v[5];
      } else if (U === 'S' || U === 'Q') {
        if (rel) {
          for (var qi = 0; qi < 4; qi += 2) { v[qi] += cx; v[qi + 1] += cy; }
        }
        cx = v[2]; cy = v[3];
      } else if (U === 'A') {
        if (rel) { v[5] += cx; v[6] += cy; }
        cx = v[5]; cy = v[6];
      } else {                                        // Z
        cx = sx; cy = sy;
      }
      out.push({ L: sg.L, u: U, a: v });
    }
    return out;
  };
  // 兼容判定：段数相同且逐段大写同型（大小写不敏感同型——定谳①）。
  var _zwPathCompat = function (a, b) {
    if (!a || !b || a.length !== b.length) return false;
    for (var i = 0; i < a.length; i++) {
      if (a[i].u !== b[i].u) return false;
    }
    return true;
  };
  // 单参数插值：B7 EDM 整数 ms 相位在场 → (a·dm + (b−a)·em)/dm——分子
  // 全精确整数、单次除法 correctly-rounded，与 Blink 全部 toFixed(2) 边界
  // 一致（corpus 字面量定谳；预计算 frac 双精度偏上会越界，弃）；否则回落
  // a + (b−a)·k（keyTimes/spline 预映射 segF 面）。
  var _zwPathBlendV = function (a, b, k, em, dm) {
    return em != null ? (a * dm + (b - a) * em) / dm : a + (b - a) * k;
  };
  // from-to 线性插值 → 绝对结果段表（字母随 to 侧；全参数插值——A 全
  // 7 参，定谳④；em/dm = EDM 相位，null 走 segF）。
  var _zwPathLerpSegs = function (fAbs, tAbs, k, em, dm) {
    var out = [];
    for (var i = 0; i < tAbs.length; i++) {
      var t = tAbs[i];
      var f = fAbs[i].a;
      var a = t.a.slice();
      for (var j = 0; j < a.length; j++) a[j] = _zwPathBlendV(f[j], a[j], k, em, dm);
      out.push({ L: t.L, u: t.u, a: a });
    }
    return out;
  };
  // k·by 增量段表（原始参数逐分量缩放，字母保持——delta 采样面，
  // 定谳④ by/from-by 原始分量加法；em/dm = EDM：by·em/dm 单除法）。
  var _zwPathScaleSegs = function (rawSegs, k, em, dm) {
    var out = [];
    for (var i = 0; i < rawSegs.length; i++) {
      var a = rawSegs[i].a.slice();
      for (var j = 0; j < a.length; j++) a[j] = em != null ? a[j] * em / dm : a[j] * k;
      out.push({ L: rawSegs[i].L, u: rawSegs[i].u, a: a });
    }
    return out;
  };
  // 增量/全值加法 → 绝对结果段表（字母随采样值侧；原始逐参数分量加——
  // 定谳④⑤；delta=true 时采样值 a 已持 k·by 增量）。
  var _zwPathSumSegs = function (uAbs, fSegs) {
    var out = [];
    for (var i = 0; i < fSegs.length; i++) {
      var f = fSegs[i].a;
      var a = uAbs[i].a.slice();
      for (var j = 0; j < a.length; j++) a[j] += f[j];
      out.push({ L: fSegs[i].L, u: fSegs[i].u, a: a });
    }
    return out;
  };
  // 绝对结果段表 → path 字符串 token：输出字母大写产绝对坐标、小写产相对
  // delta（相对转换经输出 current point 行走——定谳③ in_array 双形态自动
  // 落位）；数值全精度 String（helper toFixed(2) 双侧同构吸收舍入差）。
  var _zwPathEmitStr = function (segs) {
    var out = [];
    var cx = 0;
    var cy = 0;
    var sx = 0;
    var sy = 0;
    for (var i = 0; i < segs.length; i++) {
      var sg = segs[i];
      var a = sg.a;
      var U = sg.u;
      if (U === 'Z') {
        cx = sx; cy = sy;
        out.push(sg.L);
        continue;
      }
      var v = a;
      if (sg.L !== sg.u) {
        v = a.slice();
        if (U === 'H') {
          v[0] -= cx;
        } else if (U === 'V') {
          v[0] -= cy;
        } else if (U === 'A') {
          v[5] -= cx; v[6] -= cy;
        } else {
          var pr = U === 'C' ? 6 : U === 'S' || U === 'Q' ? 4 : 2;  // M/L/T
          for (var pi = 0; pi < pr; pi += 2) { v[pi] -= cx; v[pi + 1] -= cy; }
        }
      }
      var parts = [sg.L];
      for (var pj = 0; pj < v.length; pj++) parts.push(String(v[pj]));
      out.push(parts.join(' '));
      if (U === 'M') {
        sx = a[0]; sy = a[1];
        cx = a[0]; cy = a[1];
      } else if (U === 'L' || U === 'T') {
        cx = a[0]; cy = a[1];
      } else if (U === 'H') {
        cx = a[0];
      } else if (U === 'V') {
        cy = a[0];
      } else if (U === 'C') {
        cx = a[4]; cy = a[5];
      } else if (U === 'S' || U === 'Q') {
        cx = a[2]; cy = a[3];
      } else if (U === 'A') {
        cx = a[5]; cy = a[6];
      }
    }
    return out.join(' ');
  };
  // from-to/from-by 段进度（n1=2 同构 B6a：keyTimes 校验 + spline 段内
  // _bezSolve；discrete = half 切换口径——定谳①）。返回 segF 或 null（文法
  // 违例整动画无效）。
  var _zwPathSegF = function (g, frac, calcMode) {
    var segF = frac;
    if (g('keyTimes') != null) {
      var ktRaw = _parseSemiList(g('keyTimes'));
      var ktNums = null;
      if (ktRaw) {
        ktNums = [];
        for (var i = 0; i < ktRaw.length; i++) {
          var kv = _parseFloatTok(ktRaw[i]);
          if (isNaN(kv)) { ktNums = null; break; }
          ktNums.push(kv);
        }
      }
      var kt = ktNums ? _validKeyTimes(ktNums, 2, calcMode) : null;
      if (kt == null) return null;
      var span = kt[1] - kt[0];
      segF = span > 0 ? (frac - kt[0]) / span : 1;
    }
    if (calcMode === 'spline' && g('keySplines') != null) {
      var ks = _validKeySplines(g('keySplines'), 1);
      if (ks == null) return null;
      segF = _bezSolve(segF, ks[0][0], ks[0][1], ks[0][2], ks[0][3]);
    }
    return segF;
  };
  // path 'd' 域采样器 → null（无采样/无效）| { kind:'raw', token } |
  // { kind:'segs', segs, delta }。kind='segs'：绝对结果段表（delta=true 时
  // a 持 k·by 原始增量、由三明治层加 underlying 后产出）；kind='raw'：
  // discrete/set 直通 token（to-only 段型不配/discrete 时同为 half 切换
  // 直通，配平时线性 lerp——B7）。复用 B6a/B6c 相位与文法 machinery。
  var _sampleAnimPathStr = function (animEl, target, attr, t) {
    var an = null;
    try { an = animEl.getAttribute('attributeName'); } catch (_eA) { an = null; }
    if (an !== attr || attr !== 'd') return null;
    var localName = '';
    try { localName = String(animEl.localName || '').toLowerCase(); } catch (_eL) {}
    if (localName !== 'animate' && localName !== 'set') return null;
    var g = function (name) {
      var v = null;
      try { v = animEl.getAttribute(name); } catch (_eG) { v = null; }
      return v == null ? null : String(v);
    };
    var ph = _smilPhase(animEl, g, t);
    if (!ph) return null;
    var frac = ph.frac;
    var iter = ph.iter;
    var toTok = g('to');
    var fromTok = g('from');
    var byTok = g('by');
    var valuesTok = g('values');
    var accum = g('accumulate') === 'sum';
    var baseTok = null;
    try { baseTok = target.getAttribute('d'); } catch (_eB) { baseTok = null; }
    if (localName === 'set') {
      return toTok == null ? null : { kind: 'raw', token: toTok };
    }
    if (valuesTok != null) {
      return null;   // FIXME(svg-compat-v2 M4)：values 列表非 inherit 形态
    }
    var calcMode = (g('calcMode') || '').replace(/^\s+|\s+$/g, '').toLowerCase();
    if (!calcMode) calcMode = 'linear';
    if (calcMode !== 'linear' && calcMode !== 'discrete'
        && calcMode !== 'spline' && calcMode !== 'paced') return null;
    if (calcMode === 'paced') calcMode = 'linear';   // FIXME(svg-compat-v2 M3)
    // B7 EDM 门控：整数 ms 相位在场且纯 linear 全程插值才走 EDM；
    // accumulate 迭代比与 keyTimes/spline 预映射 segF 面保持原路径。
    var useEDM = ph.em != null && ph.dm != null && !accum
      && g('keyTimes') == null && calcMode === 'linear';
    if (fromTok != null && toTok != null) {
      var fRaw = _zwPathSegsOf(fromTok);
      var tRaw = _zwPathSegsOf(toTok);
      if (fRaw == null || tRaw == null) return null;   // 非法 token → 无效不采样
      if (!_zwPathCompat(fRaw, tRaw) || calcMode === 'discrete') {
        // 段数不等/段型不配或显式 discrete → 强制 discrete（half 含边界）
        return { kind: 'raw', token: frac >= 0.5 ? toTok : fromTok };
      }
      var segF = null;
      if (!useEDM) {
        segF = _zwPathSegF(g, frac, calcMode);
        if (segF == null) return null;
      }
      var fAbs = _zwPathAbsOf(fRaw);
      var tAbs = _zwPathAbsOf(tRaw);
      var lerped = _zwPathLerpSegs(fAbs, tAbs, segF,
        useEDM ? ph.em : null, useEDM ? ph.dm : null);
      if (accum && iter > 0) {
        // accumulate=sum 迭代叠加 = iter·toAbs（from-to-accumulate-sum-
        // empty-base 定谳「Iteration 1 adds 1×to」——非 k×(to−from) delta
        // 叠加，与 by 域 (iter+segF)·by 分叉）
        lerped = _zwPathSumSegs(lerped, _zwPathScaleSegs(tAbs, iter));
      }
      return { kind: 'segs', segs: lerped, delta: false };
    }
    if (fromTok != null && byTok != null) {
      // from-by：f(t) = from + p·by（原始分量加，定谳④）；空 from/by/段型
      // 不配 → 无效（定谳①三面）。
      var fRaw2 = _zwPathSegsOf(fromTok);
      var bRaw2 = _zwPathSegsOf(byTok);
      if (!fRaw2 || !fRaw2.length || !bRaw2 || !bRaw2.length) return null;
      if (!_zwPathCompat(fRaw2, bRaw2)) return null;
      if (calcMode === 'discrete') {
        return { kind: 'raw', token: frac >= 0.5
          ? _zwPathEmitStr(_zwPathSumSegs(_zwPathAbsOf(fRaw2), _zwPathScaleSegs(bRaw2, 1)))
          : fromTok };
      }
      var segF2 = null;
      if (!useEDM) {
        segF2 = _zwPathSegF(g, frac, calcMode);
        if (segF2 == null) return null;
      }
      var k2 = useEDM ? null : segF2 + (accum ? iter : 0);
      return {
        kind: 'segs',
        segs: _zwPathSumSegs(_zwPathAbsOf(fRaw2), _zwPathScaleSegs(bRaw2, k2,
          useEDM ? ph.em : null, useEDM ? ph.dm : null)),
        delta: false,
      };
    }
    if (byTok != null) {
      // by-only：underlying + p·by（delta=true——预乘 k·by 增量、三明治层加
      // base/prior，replace/sum 单动画同值定谳）；空/无 base、段型不配 →
      // 无效（定谳①）。
      var bRaw3 = _zwPathSegsOf(byTok);
      if (!bRaw3 || !bRaw3.length) return null;
      var uRaw = _zwPathSegsOf(baseTok == null ? '' : baseTok);
      if (!uRaw || !uRaw.length) return null;
      if (!_zwPathCompat(uRaw, bRaw3)) return null;
      if (calcMode === 'discrete') {
        return { kind: 'raw', token: frac >= 0.5
          ? _zwPathEmitStr(_zwPathSumSegs(_zwPathAbsOf(uRaw), _zwPathScaleSegs(bRaw3, 1)))
          : baseTok };
      }
      var segF3 = null;
      if (!useEDM) {
        segF3 = _zwPathSegF(g, frac, calcMode);
        if (segF3 == null) return null;
      }
      var k3 = useEDM ? null : segF3 + (accum ? iter : 0);
      return {
        kind: 'segs',
        segs: _zwPathScaleSegs(bRaw3, k3, useEDM ? ph.em : null, useEDM ? ph.dm : null),
        delta: true,
      };
    }
    if (toTok != null) {
      // to-only：base/to 段型匹配且非 discrete → 线性 lerp(base, to)（B7
      // 定谳——animate-path-to-animation @2.0/dur 4s = lerp@0.5 中值非半程
      // 切换）；不配或 discrete → base→to half 切换 raw token（to-empty-base
      // corpus 注释明示 structurally incompatible → discrete at 50%；空
      // base = [] 对非空 to 恒不配，自然落此支）。base 非法 → 无效。
      var tRaw2 = _zwPathSegsOf(toTok);
      if (!tRaw2 || !tRaw2.length) return null;
      var bRaw4 = _zwPathSegsOf(baseTok == null ? '' : baseTok);
      if (bRaw4 == null) return null;
      if (!_zwPathCompat(bRaw4, tRaw2) || calcMode === 'discrete') {
        return { kind: 'raw', token: frac >= 0.5 ? toTok : baseTok };
      }
      var segF4 = null;
      if (!useEDM) {
        segF4 = _zwPathSegF(g, frac, calcMode);
        if (segF4 == null) return null;
      }
      return {
        kind: 'segs',
        segs: _zwPathLerpSegs(_zwPathAbsOf(bRaw4), _zwPathAbsOf(tRaw2), segF4,
          useEDM ? ph.em : null, useEDM ? ph.dm : null),
        delta: false,
      };
    }
    return null;
  };
  // path 'd' 三明治钩子（globalThis 契约，SVGPathElement.animatedPathSegList
  // 消费）。文档序：replace 重置运行值；additive="sum" 全值加 prior-or-base
  // （delta 采样加 underlying/prior——replace/sum 单动画同值）；空采样值加
  // 非空 prior → prior 保全（定谳⑤）；加不了 → 该采样无效跳过。children +
  // href 双路候选合并同三钩子口径（fast-path 同——tickEverRan&&!smilSeen）。
  globalThis._zwSmilAnimPath = function (el, attr) {
    var doc = null;
    try { doc = el.ownerDocument; } catch (_eD) { doc = null; }
    if (!doc) return null;
    var st = _docTimelines.get(doc);
    if (st && st.tickEverRan && !st.smilSeen) return null;
    var t = st ? _currentTimeOf(st) : 0;
    var kids = null;
    try { kids = el.children; } catch (_eC) { kids = null; }
    var hrefElsP = null;
    if (!st || !st.tickEverRan || st.smilSeen) {
      try { hrefElsP = _zwHrefAnimList(doc, el); } catch (_eHP) { hrefElsP = null; }
    }
    if ((!kids || !kids.length) && !hrefElsP) return null;
    var result = null;                              // 当前运行值 token
    var baseRawP = function () {
      var b = null;
      try { b = el.getAttribute('d'); } catch (_eB) { b = null; }
      return _zwPathSegsOf(b == null ? '' : String(b));
    };
    var runPk = function (list, isHrefPass) {
      if (!list) return;
      for (var i = 0; i < list.length; i++) {
        var c = list[i];
        var ln = '';
        try { ln = String(c.localName || '').toLowerCase(); } catch (_eL) {}
        if (!_animLocalNames[ln]) continue;
        if (!isHrefPass && _zwAnimHrefOf(c) != null) continue;    // href 指靶归 href 面处理
        var r = null;
        try { r = _sampleAnimPathStr(c, el, attr, t); } catch (_eS) { r = null; }
        if (r == null) continue;
        var isSumP = false;
        try { isSumP = c.getAttribute('additive') === 'sum'; } catch (_eAdP) { isSumP = false; }
        if (r.kind === 'raw') {
          if (isSumP) {
            // sum + 空采样值加非空 prior → prior 保全（定谳⑤）
            var rs = _zwPathSegsOf(r.token);
            var prior0 = result != null ? _zwPathSegsOf(result) : baseRawP();
            if (rs && rs.length === 0 && prior0 && prior0.length > 0) continue;
          }
          result = r.token;
          continue;
        }
        if (r.delta || isSumP) {
          var priorSegs = null;
          if (result != null) {
            priorSegs = _zwPathSegsOf(result);
          }
          if (priorSegs == null) priorSegs = baseRawP();
          var uAbsP = _zwPathAbsOf(priorSegs || []);
          if (!_zwPathCompat(uAbsP, r.segs)) continue;   // 加不了 → 采样无效
          if (!r.segs.length) continue;
          result = _zwPathEmitStr(_zwPathSumSegs(uAbsP, r.segs));
          continue;
        }
        result = _zwPathEmitStr(r.segs);            // replace 重置
      }
    };
    runPk(kids, false);
    runPk(hrefElsP, true);
    return result;
  };
  // SVG1.1 SVGPathSeg 消费面（animated-path-helpers assert_animated_path_*
  // 'animatedPathSegList' in target 分支——SVG2 已移除、Chromium 兼容保有
  // 的最小读面，仅喂 helper 序列化所需字段）。段对象持原文大小写字母 +
  // pathSegType 常量（SVG1.1 §PathElement 表）+ kCommandDescriptor 类型域
  // 字段（M/L/T x,y；C 6；S/Q 4；A 7 含 flags；H x；V y）。
  var _zwPathSegTypeNum = { M: 1, m: 2, L: 3, l: 4, C: 5, c: 6, S: 7, s: 8,
    Q: 9, q: 10, A: 11, a: 12, H: 13, h: 14, V: 15, v: 16, Z: 17, z: 17 };
  var _zwPSegCtor = globalThis.SVGPathSeg;
  if (!_zwPSegCtor) {
    // 构造器仅作段对象工厂，不挂 globalThis——WPT svg/historical.html 断言
    // `'SVGPathSeg' in window === false`（SVG2 移除清单）。
    _zwPSegCtor = function SVGPathSeg() {};
    try { _zwPSegCtor.prototype[Symbol.toStringTag] = 'SVGPathSeg'; } catch (_eT) {}
  }
  var _zwMakePathSeg = function (sg) {
    var s = new _zwPSegCtor();
    s.pathSegTypeAsLetter = sg.L;
    s.pathSegType = _zwPathSegTypeNum[sg.L] || 0;
    var a = sg.a;
    var U = sg.u;
    if (U === 'M' || U === 'L' || U === 'T') {
      s.x = a[0]; s.y = a[1];
    } else if (U === 'C') {
      s.x1 = a[0]; s.y1 = a[1]; s.x2 = a[2]; s.y2 = a[3]; s.x = a[4]; s.y = a[5];
    } else if (U === 'Q') {
      s.x1 = a[0]; s.y1 = a[1]; s.x = a[2]; s.y = a[3];
    } else if (U === 'S') {
      s.x2 = a[0]; s.y2 = a[1]; s.x = a[2]; s.y = a[3];
    } else if (U === 'A') {
      s.r1 = a[0]; s.r2 = a[1]; s.angle = a[2];
      s.largeArcFlag = a[3]; s.sweepFlag = a[4]; s.x = a[5]; s.y = a[6];
    } else if (U === 'H') {
      s.x = a[0];
    } else if (U === 'V') {
      s.y = a[0];
    }
    return s;
  };
  var _zwMakePathSegList = function (segs) {
    var arr = [];
    for (var i = 0; i < segs.length; i++) arr.push(_zwMakePathSeg(segs[i]));
    var list = {};
    list.length = arr.length;
    for (var j = 0; j < arr.length; j++) list[j] = arr[j];
    list.getItem = function (idx) { return arr[idx]; };
    list.item = list.getItem;
    // Array.from(list) 消费面（helper serializePathSegList）——R374 iterable
    // 同构：快照数组迭代器。
    list[Symbol.iterator] = (function (snapshot) {
      return function () { return snapshot[Symbol.iterator](); };
    })(arr);
    return list;
  };
  var _zwPathElCtor = globalThis.SVGPathElement;
  if (_zwPathElCtor && _zwPathElCtor.prototype) {
    var _zwProtoP = _zwPathElCtor.prototype;
    if (!Object.prototype.hasOwnProperty.call(_zwProtoP, 'pathSegList')) {
      Object.defineProperty(_zwProtoP, 'pathSegList', {
        configurable: true,
        get: function () {
          var d = null;
          try { d = this.getAttribute('d'); } catch (_eD) { d = null; }
          var segs = _zwPathSegsOf(d == null ? '' : String(d));
          return _zwMakePathSegList(segs || []);
        },
      });
    }
    if (!Object.prototype.hasOwnProperty.call(_zwProtoP, 'animatedPathSegList')) {
      Object.defineProperty(_zwProtoP, 'animatedPathSegList', {
        configurable: true,
        get: function () {
          var tok = null;
          try { tok = globalThis._zwSmilAnimPath(this, 'd'); } catch (_eS) { tok = null; }
          var segs = tok != null ? _zwPathSegsOf(tok) : null;
          if (segs == null) {
            var d = null;
            try { d = this.getAttribute('d'); } catch (_eD2) { d = null; }
            segs = _zwPathSegsOf(d == null ? '' : String(d));
          }
          return _zwMakePathSegList(segs || []);
        },
      });
    }
  }
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
                s.repK = 0;
              }
              // repeatEvent 自然播放面（B5）：迭代边界 b + k·sd（k ≥ 1 且
              // k·sd < e）入窗 (max(w0, b), min(t, e)] 按 k 升序补派——仅
              // 当前跟踪 interval（begin 前不派、end 后不派）；repK 已派
              // 迭代水位去重（seek 合并派发同水位，双路不重不漏）。千次
              // 封顶 = OPTIMIZATION：节流追赶窗口内病态 repeatCount=
              // indefinite × 毫秒级 dur 的边界风暴防失控（语料真实面 ≤
              // 个位数边界）。
              if (s.ib === iv1.b && s.began && !s.ended &&
                  isFinite(rec1.sd) && rec1.sd > 0) {
                var k0 = Math.ceil((Math.max(w0, iv1.b) - iv1.b) / rec1.sd);
                if (k0 <= s.repK) k0 = s.repK + 1;
                var fired = 0;
                for (; iv1.b + k0 * rec1.sd < iv1.e && iv1.b + k0 * rec1.sd <= t;
                     k0++) {
                  if (++fired > 1000) break;
                  _fireSmilEvent(el, 'repeatEvent');
                }
                if (fired > 0) s.repK = k0 - 1;
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
      var seekW0 = s.lastT;
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
        s.repK = 0;
      }
      // repeatEvent seek 合并派发面（seek-repeat-event-condition 定谳）：
      // 前向 seek 跨 ≥1 迭代边界 → 合并派一次（跳过迭代不逐个补）；paused
      // 时间线不派（seeking-events-5/6 定谳——EventWatcher 未期待事件即
      // fail）；水位 repK 恒推进（抑制时同推进——往返 seek 不重放）。
      // kEnd = 最大 k ≥ 1 且 b+k·sd < e 且 b+k·sd ≤ t（边界恰落 e 不算
      // repeat）。
      if (isSeek && s.ib != null && s.began && !s.ended &&
          active && active.b === s.ib && isFinite(active.sd) && active.sd > 0 &&
          (seekW0 == null || t >= seekW0)) {
        var kEndS = Math.floor((t - s.ib) / active.sd);
        var kByE = Math.ceil((s.ie - s.ib) / active.sd) - 1;
        if (kEndS > kByE) kEndS = kByE;
        if (kEndS > s.repK) {
          var stS = _docTimelines.get(doc);
          if (!stS || !stS.paused) _fireSmilEvent(el, 'repeatEvent');
          s.repK = kEndS;
        }
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
      // per-element 暂停集（Blink 每容器独立 paused——pause-and-seek-
      // before-begin 双 svg 同文档各自 pause/unpause 消费面）；文档钟冻结
      // 条件 = 集非空。
      var k = _elKey(this);
      if (!st.pausedEls) st.pausedEls = {};
      if (k) st.pausedEls[k] = true;
      st.frozenDoc = _currentTimeOf(st);
      st.paused = true;
      if (st.tickTimer) { clearTimeout(st.tickTimer); st.tickTimer = 0; }
    };
    _zwSvgP.unpauseAnimations = function () {
      var doc = this.ownerDocument || this;
      var st = _timelineFor(doc);
      var k = _elKey(this);
      if (k && st.pausedEls) delete st.pausedEls[k];
      if (st.pausedEls && Object.keys(st.pausedEls).length > 0) return;
      if (!st.paused) return;
      st.paused = false;
      if (st.started) {
        st.baseDoc = st.frozenDoc;
        st.baseReal = Date.now();
        _wake(doc);
      }
    };
    // pause-and-seek-before-begin 消费门——Blink 每容器独立 paused：只读
    // 本元素暂停集位（文档钟冻结由 pause/unpause 集合非空条件维护，不回读
    // ——另一 svg 持暂停时本元素 unpause 后须报 false）。
    _zwSvgP.animationsPaused = function () {
      var doc = this.ownerDocument || this;
      var st = _timelineFor(doc);
      var k = _elKey(this);
      return !!(k && st.pausedEls && st.pausedEls[k]);
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
      // Blink SMILTime 内部存 float——秒值经 float32 往返（smil-clock-value-
      // large-hours：1193047h=4294969200s → fround 4294969344；常规值 float32
      // 恒等零外溢）。
      return Math.fround(d.s);
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
