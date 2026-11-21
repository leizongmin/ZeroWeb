// part10（2026-11-10 wptfyi-css DC-2 首批）：CSS Typed OM API 入口面。
//
// 目标面：css/css-typed-om 上游语料（DC-1 evidence
// docs/goal/wptfyi-css/evidence/2026-11-10-dc1-baseline-and-classification.md
// 失败聚类：CSSKeywordValue undefined 243 / CSS[unit] not a function 50 /
// CSSUnitValue 41 / computedStyleMap 34 / CSSStyleValue 21 / MathValue 族等
// ——API 整体未实现，页脚本首个断言即 ReferenceError 中断，ZW 子测试密度
// 2.2/案 vs Chrome ~37/案）。
//
// 规范：https://drafts.css-houdini.org/css-typed-om-1/
//
// 面：CSSStyleValue 基类（parse/parseAll）+ CSSKeywordValue/CSSUnitValue
//（to/toSum/equals/算术四则/min/max）+ CSSMathValue 族（Sum/Product/Min/Max/
// Clamp/Negate/Invert/Abs/Sign/Hypot，type() 数值类型记录）+ CSSUnparsedValue
//（Array 子类，var() 词元化）+ CSSVariableReferenceValue + CSSTransformValue
// 族（Translate/Rotate/Scale/Skew/SkewX/SkewY/Perspective/MatrixComponent，
// toMatrix→DOMMatrix）+ CSSImageValue（opaque）+ 色值类补挂（HWB/LCH/Lab/
// OKLCH/OKLab/ColorValue——CSSRGB/CSSHSL 已由 part05 color 内部定义，不覆盖）
// + CSS 命名空间数值工厂（含 CSS.Q/Hz/kHz 大小写映射）+
// StylePropertyMap(ReadOnly)（attributeStyleMap declared 面 / computedStyleMap
// computed 面，底层挂 el.style 与 __zw_get_computed_style 既有回调，零新
// Rust 桥）+ 属性语法表（225 属性，语料 runPropertyTests 逐页提取——
// .acceptance/wptfyi-css/typed-om-prop-table.js.txt 对账）。
//
// 划界：CSSStyleRule.styleMap（declared 规则面）不入本批——shim 无
// CSSStyleRule 类（part06 FIXME，CSSOM 规则接口族未建），declared/ 语料页
// 在前置缺口处失败同现状；`.any.js` 变体通道（DC-1 三分类「通道外」桶）同
// 后续批次。computed 面单位 canonical 化依赖 host computed 串（px/s/deg），
// host 未支持属性的 computed face 回落 declared 解析（keyword 面）。
(function () {
  'use strict';
  if (!globalThis.CSS) globalThis.CSS = {};
  if (globalThis.CSSStyleValue) return; // 幂等（shim 重装配边界）

  // ===== 1. 单位分类与换算表 =====
  // family: length/angle/time/frequency/resolution/flex/percent/number。
  // 绝对族带 toBase 换算（base: px / deg / s / hz / dppx）。
  var _zwOmUnitFamily = {
    // absolute length（px base）
    px: ['length', 1], cm: ['length', 96 / 2.54], mm: ['length', 96 / 25.4],
    q: ['length', 96 / 101.6], in: ['length', 96], pt: ['length', 96 / 72],
    pc: ['length', 16],
    // font/viewport relative length（相对族：to() 同族异单位 TypeError）
    em: ['length'], rem: ['length'], ex: ['length'], cap: ['length'],
    ch: ['length'], ic: ['length'], lh: ['length'], rlh: ['length'],
    rcap: ['length'], rch: ['length'], rex: ['length'], ric: ['length'],
    vw: ['length'], vh: ['length'], vi: ['length'], vb: ['length'],
    vmin: ['length'], vmax: ['length'],
    svw: ['length'], svh: ['length'], svi: ['length'], svb: ['length'],
    lvw: ['length'], lvh: ['length'], lvi: ['length'], lvb: ['length'],
    dvw: ['length'], dvh: ['length'], dvi: ['length'], dvb: ['length'],
    cqw: ['length'], cqh: ['length'], cqi: ['length'], cqb: ['length'],
    cqmin: ['length'], cqmax: ['length'],
    // angle（deg base）
    deg: ['angle', 1], grad: ['angle', 360 / 400],
    rad: ['angle', 180 / Math.PI], turn: ['angle', 360],
    // time（s base）
    s: ['time', 1], ms: ['time', 1 / 1000],
    // frequency（hz base）
    hz: ['frequency', 1], khz: ['frequency', 1000],
    // resolution（dppx base）
    dppx: ['resolution', 1], x: ['resolution', 1],
    dpi: ['resolution', 1 / 96], dpcm: ['resolution', 2.54 / 96],
    // flex / number / percent
    fr: ['flex'], number: ['number'], percent: ['percent'],
  };
  function _zwOmUnitInfo(unit) { return Object.prototype.hasOwnProperty.call(_zwOmUnitFamily, unit) ? _zwOmUnitFamily[unit] : null; }
  // 刀 OM-13：族规范单位（css-values-4 §canonical-unit：基单位因子 1 即规范
  // 形——angle→deg/time→s/frequency→hz/resolution→dppx/length→px）。
  var _zwOmCanonFamUnit = { angle: 'deg', time: 's', frequency: 'hz', resolution: 'dppx', length: 'px' };
  // 单位大小写 canonical 化（'Q'→'q'、'Hz'→'hz'；语料断言 unit === input.toLowerCase()）。
  function _zwOmCanonUnit(u) {
    if (u === '%') return 'percent';
    if (_zwOmUnitInfo(u)) return u;
    var l = u.toLowerCase();
    return _zwOmUnitInfo(l) ? l : null;
  }

  // ===== 2. CSSStyleValue 基类 =====
  // 不可直接构造语义（spec abstract）——语料 assert_class_string 仅探
  // constructor.name 与 instanceof；不支持语法经 StylePropertyMap.get 返回
  // 本类实例（_zwOmText 携原文，toString 回原文，set 回写保形）。
  function CSSStyleValue() {
    // spec abstract：不可直接构造（stylevalue-objects 构造断言面）。
    throw new TypeError("Illegal constructor");
  }
  // opaque 实例工厂（内部替代 new CSSStyleValue()——基类不可构造）。
  function _zwOmOpaque() { var v = {}; v.__proto__ = CSSStyleValue.prototype; return v; }
  // 类型判别（StylePropertyMap 归一消费）。
  CSSStyleValue.prototype._zwOmKind = function () { return 'opaque'; };
  // 刀 P：assert_class_string 按 Object.prototype.toString tag 判类（spec
  // WebIDL interface 对象 toStringTag）——缺 tag 类断言 face 全灭（b0
  // unsupported 337 面 + em/cm/calc 族）。子类 tag 覆写基类。
  try { CSSStyleValue.prototype[Symbol.toStringTag] = 'CSSStyleValue'; } catch (_eT) {}
  Object.defineProperty(CSSStyleValue, 'parse', {
    value: function (property, cssText) {
      if (typeof property !== 'string' || property === '') throw new TypeError("Invalid property name");
      if (typeof cssText !== 'string') throw new TypeError("Invalid cssText");
      var prop = _zwOmPropName(property);
      // 未知属性 TypeError；parse-invalid 语料全部断言 TypeError（无
      // SyntaxError 面）。
      var inTable = Object.prototype.hasOwnProperty.call(_zwOmPropSyntax, prop);
      var known = inTable || _zwOmKnownExtra[prop] === 1 || prop.slice(0, 2) === '--';
      if (!known) throw new TypeError("Invalid property name");
      var t = String(cssText).trim();
      if (t === '') throw new TypeError("Invalid value");
      // 自定义属性非空文本恒 CSSUnparsedValue。
      if (prop.slice(0, 2) === '--') return new CSSUnparsedValue(_zwOmTokenizeVar(t));
      // 简写：逐 token 语法校验（margin '10deg' → TypeError），过校验按
      // CSSUnparsedValue；值含 var() 引用时无法静态校验（css-typed-om-1
      // §normalize-tokens：简写值 var → 词元形，normalize-tokens 语料面）。
      if (inTable && _zwOmPropSyntax[prop].length === 0) {
        if (/var\(/i.test(t)) return new CSSUnparsedValue(_zwOmTokenizeVar(t));
        _zwOmCheckShorthandTokens(t);
        return new CSSUnparsedValue([t]);
      }
      // 列表值（transition-duration '1s, 2s, 3s'）：顶层逗号逐段解析，parse 语义
      // 取首段（css-typed-om-1 § parse；任一段非法 → TypeError）。
      var segs = _zwOmTopSplit(t, ',');
      if (segs.length > 1) {
        var first = null;
        for (var si = 0; si < segs.length; si++) {
          var seg = segs[si].trim();
          var sv = _zwOmParseWithText(prop, seg);
          if (sv == null || (inTable && sv._zwOmKind() === 'opaque' && !_zwOmOpaqueMatchesTable(prop, seg, _zwOmPropSyntax[prop]))) throw new TypeError("Invalid value");
          if (si === 0) first = sv;
        }
        return first;
      }
      var v = _zwOmParseWithText(prop, t);
      if (v == null) throw new TypeError("Invalid value");
      // 表内属性解析不出 typed 形态时按词形裁决：数值 token 族配表（width
      // '10deg' → TypeError）；keyword-only 表项的合法 ident 收 base
      //（color 'red' → CSSStyleValue，Chrome 同形）。
      if (inTable && v._zwOmKind() === 'opaque' && !_zwOmOpaqueMatchesTable(prop, t, _zwOmPropSyntax[prop])) throw new TypeError("Invalid value");
      return v;
    },
    writable: true, configurable: true,
  });
  Object.defineProperty(CSSStyleValue, 'parseAll', {
    value: function (property, cssText) {
      // 列表值（'1s, 2s, 3s'）：顶层逗号逐段解析全量返回（css-typed-om-1 §
      // parseAll 步）；其余形态单元素列表（parse 面语义）。
      var prop = _zwOmPropName(property);
      var t = String(cssText).trim();
      var inTable = Object.prototype.hasOwnProperty.call(_zwOmPropSyntax, prop);
      if (inTable && _zwOmPropSyntax[prop].length !== 0 && _zwOmTopSplit(t, ',').length > 1) {
        var out = [];
        var segs = _zwOmTopSplit(t, ',');
        for (var i = 0; i < segs.length; i++) {
          var seg = segs[i].trim();
          var sv = _zwOmParseWithText(prop, seg);
          if (sv == null || (sv._zwOmKind && sv._zwOmKind() === 'opaque' && !_zwOmOpaqueMatchesTable(prop, seg, _zwOmPropSyntax[prop]))) throw new TypeError("Invalid value");
          out.push(sv);
        }
        return out;
      }
      var v = CSSStyleValue.parse(property, cssText);
      return [v];
    },
    writable: true, configurable: true,
  });
  globalThis.CSSStyleValue = CSSStyleValue;

  // ===== 3. CSSKeywordValue =====
  function CSSKeywordValue(value) {
    if (arguments.length === 0) throw new TypeError("CSSKeywordValue requires a value");
    if (typeof value !== 'string') throw new TypeError("Invalid keyword value");
    this._zwOmValue = value;
  }
  // 语料 assert_true(x instanceof CSSStyleValue)——全部子类挂基类原型链。
  CSSKeywordValue.prototype = Object.create(CSSStyleValue.prototype);
  CSSKeywordValue.prototype.constructor = CSSKeywordValue;
  Object.defineProperty(CSSKeywordValue.prototype, 'value', {
    get: function () { return this._zwOmValue; },
    set: function (v) { if (typeof v !== 'string') throw new TypeError("Invalid keyword value"); this._zwOmValue = v; },
    enumerable: true, configurable: true,
  });
  CSSKeywordValue.prototype._zwOmKind = function () { return 'keyword'; };
  CSSKeywordValue.prototype.toString = function () { return this._zwOmValue; };
  try { CSSKeywordValue.prototype[Symbol.toStringTag] = 'CSSKeywordValue'; } catch (_eT) {}
  globalThis.CSSKeywordValue = CSSKeywordValue;

  // ===== CSSNumericValue（abstract；Unit/Math 基接口 + 算术面）=====
  // testhelper 数值类型保持块裸引该标识符（spec CSSNumericValue : CSSStyleValue）。
  function CSSNumericValue() { throw new TypeError("Illegal constructor"); }
  CSSNumericValue.prototype = Object.create(CSSStyleValue.prototype);
  CSSNumericValue.prototype.constructor = CSSNumericValue;
  // to(unit)：绝对族跨单位换算；相对/不同族 TypeError。
  CSSNumericValue.prototype.to = function (unit) {
    var u = _zwOmCanonUnit(String(unit));
    var b = u ? _zwOmUnitInfo(u) : null;
    // 刀 OM-12：非法单位 DOMException SyntaxError（css-typed-om-1 §dom-cssnumericvalue-to
    // 步 1，与 toSum 同门；语料 to.tentative 'lemon' assert_throws_dom 面）。
    if (!b) throw new (globalThis.DOMException || Error)("Invalid unit", 'SyntaxError');
    // 刀 face C part 1：math 值 to()（css-typed-om-1 §dom-cssnumericvalue-to：
    // 展开逐成员按因子换算并加总，全成员可转换时返单 CSSUnitValue——语料
    // to.tentative 'Converting a CSSMathSum to a single unit adds the values'
    // + stylepropertymap 钳形回调 input.to('percent') 面；余量/异族成员
    // TypeError 保留）。percent/number/flex 同名直取不加因子（与 Unit 路径同门）。
    if (!(this instanceof CSSUnitValue)) {
      // 刀 OM-12：math 值 to() 全形态数值求值（css-typed-om-1 §dom-cssnumericvalue-to
      // ＋§convert-a-cssmathvalue）。乘除树符号级指数代数：Unit 叶按族 base 因子
      // 折算，Product/Invert 指数累并、Sum/Negate 同维代数和、Min/Max/Clamp 同维
      // 数值取小大/钳（成员含相对单位叶 TypeError——语料 Clamp(px,em,px) 面）；
      // 末态族维度须恰合目标单位型（绝对族指数 1、number 全零、percent 单族），
      // 否则 TypeError。复式语料面 max((s·s·px·px)/(s·px), ms·cm)/cm − min(ms,s)
      // →to('ms') 3500（cm 同符号对消 + 比较面取 ms·cm 分支）。
      var t12 = _zwOmEvalMathUnit(this);
      var bFam12 = b[0];
      if (b.length < 2 && bFam12 !== 'number' && bFam12 !== 'percent') {
        throw new TypeError("Relative units can not be converted");
      }
      if (t12.rel) throw new TypeError("Relative units can not be converted");
      var fams12 = {}, want12 = {}, k12;
      for (k12 in t12.exp) {
        var fi12 = _zwOmUnitInfo(k12);
        var f12 = fi12 ? fi12[0] : k12;
        fams12[f12] = (fams12[f12] || 0) + t12.exp[k12];
      }
      if (bFam12 !== 'number') want12[bFam12] = 1;
      for (k12 in fams12) {
        if ((fams12[k12] || 0) !== (want12[k12] || 0)) throw new TypeError("Incompatible units");
      }
      for (k12 in want12) {
        if ((fams12[k12] || 0) !== 1) throw new TypeError("Incompatible units");
      }
      var final12 = t12.val;
      if (bFam12 !== 'number' && b.length >= 2) final12 = t12.val / b[1];
      return new CSSUnitValue(_zwOmNumClean(final12), u);
    }
    var a = _zwOmUnitInfo(this._zwOmUnit);
    if (!a || a[0] !== b[0]) throw new TypeError("Incompatible units");
    var factor = 1;
    if (a[0] !== 'percent' && a[0] !== 'number' && a[0] !== 'flex') {
      if (a.length < 2 || b.length < 2) {
        if (this._zwOmUnit !== u) throw new TypeError("Relative units can not be converted");
      } else {
        factor = a[1] / b[1];
      }
    } else if (this._zwOmUnit !== u) {
      throw new TypeError("Incompatible units");
    }
    return new CSSUnitValue(this._zwOmValue * factor, u);
  };
  // toSum（css-typed-om-1 § convert a CSSNumericValue to a sum）：请求单位按序
  // 贪心填充——同名单元直取、绝对族换算（factor 表）、相对单元按名直取（未命中
  // 记余量）、多余请求单位补 0；余量未清空 TypeError；无参返回按单位名排序的
  // Sum；非法单位串 DOMException SyntaxError。
  CSSNumericValue.prototype.toSum = function () {
    var units = [];
    for (var i = 0; i < arguments.length; i++) {
      var u = _zwOmCanonUnit(String(arguments[i]));
      if (!u) throw new (globalThis.DOMException || Error)("Invalid unit", 'SyntaxError');
      units.push(u);
    }
    if (units.length === 0) {
      var flat0 = _zwOmFlattenToUnits(this);
      var entries0 = [];
      for (var k0 = 0; k0 < flat0.length; k0++) entries0.push(new CSSUnitValue(flat0[k0].value, flat0[k0].unit));
      entries0.sort(function (a, b) { return a.unit < b.unit ? -1 : (a.unit > b.unit ? 1 : 0); });
      return new CSSMathSum(entries0);
    }
    // 请求单位可加性：全部同族（px 'px' 's' 混族 TypeError）。
    if (units.length > 1) {
      var fams = {};
      for (var j = 0; j < units.length; j++) {
        var fi = _zwOmUnitInfo(units[j]);
        fams[fi ? fi[0] : units[j]] = 1;
      }
      var keys = Object.keys(fams);
      if (keys.length > 1) throw new TypeError("Units are not addable");
    }
    var flat = _zwOmFlattenToUnits(this);
    var consumed = [];
    for (var c = 0; c < flat.length; c++) consumed.push(false);
    var out = [];
    for (var ui = 0; ui < units.length; ui++) {
      var tu = units[ui];
      var tinfo = _zwOmUnitInfo(tu);
      var amount = 0;
      for (var ei = 0; ei < flat.length; ei++) {
        if (consumed[ei]) continue;
        var e = flat[ei];
        var einfo = _zwOmUnitInfo(e.unit);
        if (e.unit === tu) { amount += e.value; consumed[ei] = true; continue; }
        if (tinfo && tinfo.length >= 2 && einfo && einfo.length >= 2 && einfo[0] === tinfo[0]) {
          amount += _zwOmNumClean(e.value * einfo[1] / tinfo[1]);
          consumed[ei] = true;
        }
      }
      out.push(new CSSUnitValue(amount, tu));
    }
    for (var li = 0; li < flat.length; li++) {
      if (!consumed[li]) throw new TypeError("Leftover units cannot be converted");
    }
    return new CSSMathSum(out);
  };
  // 源值展开为 {value, unit} 列表（Unit → 自身；Sum 逐成员递归；其余 math 形态
  // 语料 toSum 面按 TypeError 裁决——Max 混合型/Product 复合型不可展平）。
  function _zwOmFlattenToUnits(v) {
    if (v instanceof CSSUnitValue) return [{ value: v._zwOmValue, unit: v._zwOmUnit }];
    if (v instanceof CSSMathValue && v.operator === 'sum') {
      var out = [];
      for (var i = 0; i < v._zwOmValues.length; i++) out = out.concat(_zwOmFlattenToUnits(v._zwOmValues[i]));
      return out;
    }
    throw new TypeError("Invalid sum value");
  }
  // 刀 OM-12：math 全形态 to() 求值器——符号级指数代数（css-typed-om-1 §convert
  // a CSSNumericValue：乘除树叶按族 base 因子折算，指数带号累并；Product 空
  // 防御积 1、Invert 指数取负值取倒、Negate 取负、Sum 同维代数和、Min/Max/
  // Clamp 同维按折算值取小大/钳且成员禁相对叶）。返回 {exp:{符号:带号指数},
  // val, rel}；未知 math 形态（hypot/abs/sign…语料 to() 面外）TypeError。
  function _zwOmEvalMathUnit(node) {
    if (node instanceof CSSUnitValue) {
      var ui = _zwOmUnitInfo(node._zwOmUnit);
      if (!ui) throw new TypeError("Invalid unit");
      var out = { exp: {}, val: node._zwOmValue, rel: false };
      if (ui[0] === 'number') { /* number 维外（css-typed-om-1 §type 全零型） */ }
      else if (ui.length >= 2) { out.exp[node._zwOmUnit] = 1; out.val *= ui[1]; }
      else {
        // percent/flex 是自有族维（css-typed-om-1 §converting-cssunitvalue：
        // 同名单位直取不加因子）不置 rel；仅 factorless length（em/vw…相对族）
        // 置 rel——to() 拒换算（语料 Clamp(px,em,px) 面与 Sum(0%,0%).to('percent')
        // stylepropertymap set 钳形回调放行面）。
        out.exp[node._zwOmUnit] = 1;
        if (ui[0] === 'length') out.rel = true;
      }
      return out;
    }
    if (node instanceof CSSMathValue) {
      var op = node.operator, ms = node._zwOmValues || [], i, acc, cur;
      if (op === 'product') {
        acc = null;
        for (i = 0; i < ms.length; i++) {
          cur = _zwOmEvalMathUnit(ms[i]);
          if (acc === null) { acc = cur; continue; }
          for (var sy in cur.exp) acc.exp[sy] = (acc.exp[sy] || 0) + cur.exp[sy];
          acc.val *= cur.val;
          acc.rel = acc.rel || cur.rel;
        }
        if (acc === null) acc = { exp: {}, val: 1, rel: false };
        return acc;
      }
      if (op === 'invert') {
        acc = _zwOmEvalMathUnit(ms[0]);
        for (var sy2 in acc.exp) acc.exp[sy2] = -acc.exp[sy2];
        acc.val = 1 / acc.val;
        return acc;
      }
      if (op === 'negate') {
        acc = _zwOmEvalMathUnit(ms[0]);
        acc.val = -acc.val;
        return acc;
      }
      if (op === 'sum') {
        acc = null;
        for (i = 0; i < ms.length; i++) {
          cur = _zwOmEvalMathUnit(ms[i]);
          if (acc === null) { acc = cur; continue; }
          if (!_zwOmExpDimsEq(acc.exp, cur.exp)) throw new TypeError("Incompatible units");
          acc.val += cur.val;
          acc.rel = acc.rel || cur.rel;
        }
        if (acc === null) acc = { exp: {}, val: 0, rel: false };
        return acc;
      }
      if (op === 'min' || op === 'max' || op === 'clamp') {
        // 比较面：成员须同维且不含相对叶（语料 Min/Max 异单位/Clamp 含 em 面
        // TypeError；比较按族 base 因子折算值——Min(cm,mm).to('mm') 取 mm 支）。
        var ev = [];
        for (i = 0; i < ms.length; i++) {
          cur = _zwOmEvalMathUnit(ms[i]);
          if (cur.rel) throw new TypeError("Relative units can not be converted");
          ev.push(cur);
        }
        for (i = 1; i < ev.length; i++) {
          if (!_zwOmExpDimsEq(ev[0].exp, ev[i].exp)) throw new TypeError("Incompatible units");
        }
        acc = ev[op === 'clamp' ? 1 : 0];
        if (op === 'min' || op === 'max') {
          for (i = 1; i < ev.length; i++) {
            if (op === 'min' ? ev[i].val < acc.val : ev[i].val > acc.val) acc = ev[i];
          }
        } else {
          var cv = Math.min(Math.max(ev[1].val, ev[0].val), ev[2].val);
          acc = { exp: ev[1].exp, val: cv, rel: ev[1].rel };
        }
        return acc;
      }
      throw new TypeError("Incompatible units");
    }
    throw new TypeError("Incompatible units");
  }
  // 族维度等价（指数表按符号记，族维度=族内符号指数和；css-typed-om-1 §type
  // 相同型判定——Sum/Min/Max/Clamp 成员同维门）。
  function _zwOmExpDimsEq(a, b) {
    var fa = {}, fb = {}, k, fi, f;
    for (k in a) { fi = _zwOmUnitInfo(k); f = fi ? fi[0] : k; fa[f] = (fa[f] || 0) + a[k]; }
    for (k in b) { fi = _zwOmUnitInfo(k); f = fi ? fi[0] : k; fb[f] = (fb[f] || 0) + b[k]; }
    var keys = {}, kn;
    for (kn in fa) keys[kn] = 1;
    for (kn in fb) keys[kn] = 1;
    for (kn in keys) if ((fa[kn] || 0) !== (fb[kn] || 0)) return false;
    return true;
  }
  // equals（css-typed-om-1 § dom-cssnumericvalue-equals，语料 equals 面）：数值
  // 结构化递归比对——Unit 比值/单位、Math 比 operator + 逐成员。
  CSSNumericValue.prototype.equals = function () {
    for (var i = 0; i < arguments.length; i++) {
      if (!_zwOmNumEq(this, arguments[i])) return false;
    }
    return true;
  };
  function _zwOmNumEq(a, b) {
    if (a === b) return true;
    if (!(a instanceof CSSNumericValue) || !(b instanceof CSSNumericValue)) return false;
    if (a instanceof CSSUnitValue && b instanceof CSSUnitValue) {
      return a._zwOmValue === b._zwOmValue && a._zwOmUnit === b._zwOmUnit;
    }
    if (a instanceof CSSMathValue && b instanceof CSSMathValue) {
      if (a.operator !== b.operator) return false;
      var av = a._zwOmValues || [], bv = b._zwOmValues || [];
      if (av.length !== bv.length) return false;
      for (var i = 0; i < av.length; i++) if (!_zwOmNumEq(av[i], bv[i])) return false;
      return true;
    }
    return false;
  }
  // 算术四则（CSSMathValue 返回；css-typed-om-1 § arithmetic）。
  CSSNumericValue.prototype.add = function () { return _zwOmArith('sum', this, arguments); };
  CSSNumericValue.prototype.sub = function () { return _zwOmArith('sub', this, arguments); };
  CSSNumericValue.prototype.mul = function () { return _zwOmArith('mul', this, arguments); };
  CSSNumericValue.prototype.div = function () { return _zwOmArith('div', this, arguments); };
  CSSNumericValue.prototype.min = function () { return _zwOmArith('min', this, arguments); };
  CSSNumericValue.prototype.max = function () { return _zwOmArith('max', this, arguments); };
  // CSSNumericValue.parse（css-typed-om-1 § dom-cssnumericvalue-parse，语料
  // numeric-objects/parse 面）：裸数值 token → CSSUnitValue；min()/max()/clamp()
  // 裸形直返；calc()/乘除包裹 → Sum + 同绝对族折叠；sign()、除零、余 token、
  // 未知单位、复合/不可加单位均 DOMException SyntaxError。
  CSSNumericValue.parse = function (cssText) {
    var t = String(cssText).trim();
    var SE = function (msg) { return new (globalThis.DOMException || Error)(msg, 'SyntaxError'); };
    var p = new _zwOmCalcParser(t);
    var v;
    try {
      v = p.parseSum();
      p.ws();
      if (p.i < p.s.length) throw new SyntaxError("leftover tokens");
    } catch (_e) {
      throw SE("Invalid numeric value");
    }
    if (v == null) throw SE("Invalid numeric value");
    try { _zwOmNumericCheck(v); } catch (_eC) { throw SE("Invalid numeric value"); }
    var isCalc = /^calc\(/i.test(t);
    // 裸数值 token → UnitValue；裸 min()/max()/clamp() 直返；calc() 包裹：Unit
    // 原子 → Sum 包裹（'calc(10px)' → Sum[px]）、math 节点保形直返（'calc(2 *
    // min(…))' → Product 裸，语料 numeric parse 面）；Sum 折叠绝对族。
    if (!isCalc) {
      if (!(v instanceof CSSMathValue)) return v;
      if (v.operator === 'min' || v.operator === 'max' || v.operator === 'clamp') return v;
    }
    if (v instanceof CSSMathValue) return v.operator === 'sum' ? _zwOmFoldNumericSum(v) : v;
    return _zwOmFoldNumericSum(new CSSMathSum([v]));
  };
  // calc 类型代数检查（CSS Values §8.1）：非 percent 族至多一个（px*2s 复合、
  // px+2s 不可加 → SyntaxError）；sign() 未支持；除数为字面 0。
  function _zwOmNumericCheck(v) {
    if (v instanceof CSSUnitValue) {
      if (!_zwOmUnitInfo(v._zwOmUnit)) throw new SyntaxError("invalid unit");
      return;
    }
    if (!(v instanceof CSSMathValue)) throw new SyntaxError("non-numeric");
    var members = v._zwOmValues || [];
    var i;
    if (v.operator === 'sign') throw new SyntaxError("sign unsupported");
    for (i = 0; i < members.length; i++) _zwOmNumericCheck(members[i]);
    if (v.operator === 'invert') {
      var d = members[0];
      if (d instanceof CSSUnitValue && d._zwOmValue === 0) throw new SyntaxError("division by zero");
    }
    if (v.operator === 'product') {
      var pf = {};
      for (i = 0; i < members.length; i++) {
        var pt = _zwOmUnitTypeOf(members[i]);
        for (var f in pt) if (pt[f] && f !== 'percent' && f !== 'percentHint') pf[f] = 1;
      }
      if (Object.keys(pf).length > 1) throw new SyntaxError("compound units");
    }
    if (v.operator === 'sum' || v.operator === 'min' || v.operator === 'max' || v.operator === 'clamp') {
      var sf = {};
      for (i = 0; i < members.length; i++) {
        var st = _zwOmUnitTypeOf(members[i]);
        for (var g in st) if (st[g] && g !== 'percent' && g !== 'percentHint') sf[g] = 1;
      }
      if (Object.keys(sf).length > 1) throw new SyntaxError("incompatible units");
    }
  }
  // 换算浮点清尾（12 位有效数字回读）：因子比值残渣（cm→mm
  // 19.999999999999996）不外泄——语料 toSum 断言精确值。
  function _zwOmNumClean(x) {
    if (!isFinite(x)) return x;
    return parseFloat(x.toPrecision(12));
  }
  // ===== 刀 face C part 1：computed 面数值规范化 =====
  // em/rem 解析因子（css-values-4 §6 相对长度 computed 期按元素 font-size
  // 绝对化）——宿主 computed font-size 读 px 数；读取失败回落 16px 初始值。
  function _zwOmFontSizePx(getStyleObj) {
    try {
      var fs = String(getStyleObj().getPropertyValue('font-size') || '').trim();
      var m = /^([+-]?(?:\d+(?:\.\d+)?|\.\d+))px$/i.exec(fs);
      if (m) return parseFloat(m[1]);
    } catch (_eFS) {}
    return 16;
  }
  // grid 轨道属性负百分钳零集（css-grid-2 §7.1.1 track breadth：负
  // <percentage> 文法合法但 used/computed 期按 0 处理——testsuite
  // assert_is_equal_with_clamping_percentage 期望 CSSUnitValue(0,'percent')）。
  // 刀 OM-3：[0,1] number 族 computed 钳零/钳一 + percent→number 换算集
  //（css-fill-stroke §fill-opacity / filter-effects §flood-opacity /
  // SVG2 §stop-opacity / SVG2 §stroke-opacity：值域 [0,1]，computed 归一
  // CSSUnitValue(n,'number')，<percentage> 按 1/100 换算——语料四页
  // assert_is_equal_with_clamping_percentage 期望 CSSUnitValue(n,'number')；
  // opacity 本尊不入集——引擎计算串已钳化，specified % 保持引擎域）。
  var _zwOmOpacityClampNumProps = {
    'fill-opacity': 1, 'flood-opacity': 1, 'stop-opacity': 1, 'stroke-opacity': 1,
  };
  var _zwOmGridClampPctProps = {
    'grid-template-columns': 1, 'grid-template-rows': 1,
    'grid-auto-columns': 1, 'grid-auto-rows': 1,
    // 刀 G（css-align-3 §gap used/computed）：gap 负 % 与 grid 轨道同合同。
    'column-gap': 1, 'row-gap': 1,
    // 刀 OM-2（css-scroll-snap-1 §scroll-padding）：scroll-padding-* 负
    // <percentage> computed 期钳 0——语料 scroll-padding.html 同
    // assert_is_equal_with_clamping_percentage 期望 CSSUnitValue(0,'percent')。
    'scroll-padding-top': 1, 'scroll-padding-right': 1,
    'scroll-padding-bottom': 1, 'scroll-padding-left': 1,
    'scroll-padding-inline-start': 1, 'scroll-padding-block-start': 1,
    'scroll-padding-inline-end': 1, 'scroll-padding-block-end': 1,
    // 刀 OM-7（svg2 §stroke-dasharray/§stroke-width）：stroke-* 负
    // <percentage> computed 期钳 0——语料两页 percent 臂同
    // assert_is_equal_with_clamping_percentage 期望 Unit(0,'percent')。
    'stroke-dasharray': 1, 'stroke-width': 1,
    // 刀 OM-9（css-sizing-3 §width/height/§min/max sizing 负值 invalid＋
    // css-logical-1 block-size/inline-size 逻辑映射同合同＋svg2 §geometry
    // properties r/rx/ry 负值 invalid）：sizing/SVG geometry 族负
    // <percentage> computed 期钳 0——语料三页 percent 臂同
    // assert_is_equal_with_clamping_percentage 期望 Unit(0,'percent')，
    // specified 臂 assert_is_equal_with_range_handling 保形不走钳表
    // （探针 zz-om9probe 实证 GOT Unit(-3.14,'percent') 未钳、REC 形
    // 不经 canonicalize 钳位）。
    'block-size': 1, 'min-block-size': 1, 'max-block-size': 1,
    'inline-size': 1, 'min-inline-size': 1, 'max-inline-size': 1,
    'r': 1, 'rx': 1, 'ry': 1,
  };
  // 刀 OM-2：scroll-padding 族 computed 面 declared 文本源集——引擎计算串对
  // %/负值/calc 有损（探针 zz-om2probe 实证：'0%'/'3.14%'→'0px'、负值/calc→
  // 'auto'），computedStyleMap 恒从 inline 声明文本 reify（css-scroll-snap-1
  // §scroll-padding computed 值语义 shim 侧兑现 + 上表 percent 钳零）。
  var _zwOmScrollPadProps = {
    'scroll-padding-top': 1, 'scroll-padding-right': 1,
    'scroll-padding-bottom': 1, 'scroll-padding-left': 1,
    'scroll-padding-inline-start': 1, 'scroll-padding-block-start': 1,
    'scroll-padding-inline-end': 1, 'scroll-padding-block-end': 1,
  };
  // 刀 OM-5：逻辑角 radius 名集（css-logical-1 §border-radius logical 映射；
  // computed 百分比形走声明文本源，见 computed face 臂）。
  var _zwOmLogicalRadiusProps = {
    'border-start-start-radius': 1, 'border-start-end-radius': 1,
    'border-end-start-radius': 1, 'border-end-end-radius': 1,
  };
  // computed 值 typed-om reification：长度 computed 绝对化 px（em/rem 按宿主
  // font-size、绝对族按因子）、同族 calc Sum 折叠（length→px / percent /
  // angle→deg 三族；time 等宿主已 canonical 不折——P2F 收窄）、grid 负百分
  // 钳零。语料五形族靶：'-3.14em'→Unit(px)、'3.14cm'→Unit(px)、
  // 'calc(0px+0em)'→Unit(px)、'-3.14%'(grid)→Unit(0,'percent')、
  // 'calc(0%+0%)'→Unit(0,'percent')。折叠不可达形原样返回。
  function _zwOmComputedCanonicalize(p, v, getStyleObj) {
    if (v instanceof CSSUnitValue) {
      var uinfo = _zwOmUnitInfo(v._zwOmUnit);
      if (uinfo && uinfo[0] === 'length') {
        if (uinfo.length < 2) {
          // 字号相对：语料面仅 em/rem 需求（ex/ch/viewport 无因子不折）。
          if (v._zwOmUnit !== 'em' && v._zwOmUnit !== 'rem') return v;
          v = new CSSUnitValue(_zwOmNumClean(v._zwOmValue * _zwOmFontSizePx(getStyleObj)), 'px');
        } else if (v._zwOmUnit !== 'px') {
          v = new CSSUnitValue(_zwOmNumClean(v._zwOmValue * uinfo[1]), 'px');
        }
      }
      if (v._zwOmUnit === 'percent' && v._zwOmValue < 0 && _zwOmGridClampPctProps[p]) {
        v = new CSSUnitValue(0, 'percent');
      }
      // 刀 OM-6：line-height 百分比计算值绝对化 px（css-inline-3 §line-height
      // computed：percentage → absolute length 按宿主 font-size 解析——引擎
      // gCS 对正形已解析同值、负/calc 形声明弃置；语料 line-height.html
      // computed 臂 assert_is_unit('px')，负值保号不钳）。
      if (p === 'line-height' && v._zwOmUnit === 'percent') {
        v = new CSSUnitValue(_zwOmNumClean(v._zwOmValue * _zwOmFontSizePx(getStyleObj) / 100), 'px');
      }
      // 刀 OM-7：stroke-dasharray/stroke-width number 计算值用户单位折 px
      // （svg2 §stroke-dasharray/§stroke-width：number 计算值按用户单位即
      // px 类——语料两页 number 臂 assert_is_unit('px')，值保值不钳；引擎
      // gCS 对 number 形保形回显 unit 'number'，探针 zz-om7probe 挂载态实证）。
      // 刀 OM-8：stroke-dashoffset 同合同（svg2 §stroke-dashoffset——语料
      // number 四臂 assert_equals unit 'px' got 'number'；percent/length
      // 臂语料本就过、无钳合同，不入 clamp 表）。
      if ((p === 'stroke-dasharray' || p === 'stroke-width' || p === 'stroke-dashoffset') && v._zwOmUnit === 'number') {
        var swPx = v._zwOmValue;
        v = new CSSUnitValue(swPx, 'px');
      }
      // 刀 OM-7：tab-size 负 number 计算值钳 0（css-text-3 §tab-size 非负
      // 值域——语料 tab-size.html number 臂 assert_is_equal_with_clamping_
      // number 期望 Unit(0,'number')；正形保 number，引擎 gCS 声明弃置回
      // 默认 number 8）。<length> 相对单位走通用 em/rem 折 px 臂，px 保形。
      if (p === 'tab-size' && v._zwOmUnit === 'number' && v._zwOmValue < 0) {
        var tsN = 0;
        v = new CSSUnitValue(tsN, 'number');
      }
      // 刀 OM-3：[0,1] 族 computed 归一 number（percent 按 1/100 换算后钳
      // [0,1]——css-fill-stroke/filters/SVG2 上述四属性值域合同）。
      if (_zwOmOpacityClampNumProps[p] && (v._zwOmUnit === 'percent' || v._zwOmUnit === 'number')) {
        var nOp = (v._zwOmUnit === 'percent') ? v._zwOmValue / 100 : v._zwOmValue;
        if (nOp < 0) nOp = 0; else if (nOp > 1) nOp = 1;
        v = new CSSUnitValue(nOp, 'number');
      }
      // 刀 OM-11：整数族 computed 取整/钳 1（css-values-4 §<integer> 计算值
      // 四舍五入取整 ties +∞——Math.round 同判 3.5→4/-3.5→-3；
      // orphans/widows/column-count 值域下限 1 钳 1，order 不钳——语料两簇
      // computed 臂 assert_style_value_equals Unit 形，探针 G2 实证 -3.14
      // 期望 Unit(1)、G3 实证 order -3.14 期望 Unit(-3)）。
      if (_zwOmIntProps[p] && v._zwOmUnit === 'number') {
        var nInt = Math.round(v._zwOmValue);
        if (_zwOmIntMin1Props[p] && nInt < 1) nInt = 1;
        if (nInt !== v._zwOmValue) v = new CSSUnitValue(nInt, 'number');
      }
      return v;
    }
    if (v instanceof CSSMathValue && v.operator === 'sum') {
      var flat;
      try { flat = _zwOmFlattenToUnits(v); } catch (_eCF) { return v; }
      var baseUnit = { length: 'px', percent: 'percent', angle: 'deg', number: 'number' };
      var total = 0, famName = null, fs2 = null;
      for (var i = 0; i < flat.length; i++) {
        var mi = _zwOmUnitInfo(flat[i].unit);
        var mf = mi ? mi[0] : null;
        var mv;
        if (mf === 'percent') {
          mv = flat[i].value;
        } else if (mf === 'number') {
          // 刀 OM-3：number 族折叠（'calc(2 + 3)'→CSSUnitValue(5,'number')——
          // 语料 opacity 族 computed 面期望 Unit 形）。
          mv = flat[i].value;
        } else if (mi && mi.length >= 2 && baseUnit[mf]) {
          mv = flat[i].value * mi[1];
        } else if (mf === 'length' && (flat[i].unit === 'em' || flat[i].unit === 'rem')) {
          if (fs2 === null) fs2 = _zwOmFontSizePx(getStyleObj);
          mv = flat[i].value * fs2;
        } else {
          return v;
        }
        if (famName === null) { famName = mf; total = mv; }
        else if (famName === mf) { total += mv; }
        else { return v; }
      }
      if (famName === null) return v;
      var folded = new CSSUnitValue(_zwOmNumClean(total), baseUnit[famName]);
      if (folded._zwOmUnit === 'percent' && folded._zwOmValue < 0 && _zwOmGridClampPctProps[p]) {
        folded = new CSSUnitValue(0, 'percent');
      }
      // 刀 OM-6：line-height calc 百分比折叠后同走绝对化（与 Unit 臂同合同）。
      if (p === 'line-height' && folded._zwOmUnit === 'percent') {
        folded = new CSSUnitValue(_zwOmNumClean(folded._zwOmValue * _zwOmFontSizePx(getStyleObj) / 100), 'px');
      }
      // 刀 OM-7：折叠后同合同——stroke-* number 折 px、tab-size 负 number
      // 钳 0（'calc(2 + 3)' 折 Unit(5,'number') 后与 Unit 臂同变换）；
      // 刀 OM-8 承 stroke-dashoffset 同合同。
      if ((p === 'stroke-dasharray' || p === 'stroke-width' || p === 'stroke-dashoffset') && folded._zwOmUnit === 'number') {
        folded = new CSSUnitValue(folded._zwOmValue, 'px');
      }
      if (p === 'tab-size' && folded._zwOmUnit === 'number' && folded._zwOmValue < 0) {
        folded = new CSSUnitValue(0, 'number');
      }
      // 刀 OM-3：折叠后同走 [0,1] 族归一（number 直接钳、percent 换算后钳）。
      if (_zwOmOpacityClampNumProps[p] && (folded._zwOmUnit === 'percent' || folded._zwOmUnit === 'number')) {
        var nOf = (folded._zwOmUnit === 'percent') ? folded._zwOmValue / 100 : folded._zwOmValue;
        if (nOf < 0) nOf = 0; else if (nOf > 1) nOf = 1;
        folded = new CSSUnitValue(nOf, 'number');
      }
      // 刀 OM-11：折叠后同合同——整数族取整/钳 1（'calc(2 + 3)' 折
      // Unit(5,'number') 后与 Unit 臂同变换；column-count computed calc 臂
      // 语料 expected Unit(5)）。
      if (_zwOmIntProps[p] && folded._zwOmUnit === 'number') {
        var nInt2 = Math.round(folded._zwOmValue);
        if (_zwOmIntMin1Props[p] && nInt2 < 1) nInt2 = 1;
        if (nInt2 !== folded._zwOmValue) folded = new CSSUnitValue(nInt2, 'number');
      }
      return folded;
    }
    return v;
  }
  function _zwOmUnitTypeOf(v) {
    return (v && typeof v.type === 'function') ? v.type() : {};
  }
  // 同绝对族 Sum 顶层裸 Unit 折叠（'calc(1px + 1in)' → Sum[px(97)]；percent/
  // 相对族/Negate 包裹成员不折叠——语料 'calc(1px + 2em)' 不简化同面）。
  function _zwOmFoldNumericSum(sumNode, canon) {
    var members = sumNode._zwOmValues;
    var out = [];
    var firstIdx = {};
    // 刀 OM-13：number 因子积成员吸收（css-typed-om-1 §normalize-numeric
    // 化简：calc(1px + calc(1px * 2) + 1%) 内层积 1px·2 → 2px 并入同族
    // host——语料期望 Sum[px(4), percent(1)] 面）。仅单 Unit×单 number 形。
    var absorbed = [];
    for (var ai = 0; ai < members.length; ai++) {
      var am = members[ai];
      if (am instanceof CSSMathValue && am.operator === 'product' && am._zwOmValues.length === 2) {
        var pa = am._zwOmValues[0], pb = am._zwOmValues[1];
        var anum = null, aun = null;
        if (pa instanceof CSSUnitValue && pa._zwOmUnit === 'number') anum = pa;
        else if (pa instanceof CSSUnitValue) aun = pa;
        if (pb instanceof CSSUnitValue && pb._zwOmUnit === 'number') anum = pb;
        else if (pb instanceof CSSUnitValue) aun = pb;
        if (anum && aun) am = new CSSUnitValue(anum._zwOmValue * aun._zwOmValue, aun._zwOmUnit);
      }
      absorbed.push(am);
    }
    if (!canon) {
      // 数值通道既有族宿主增量折（firstIdx 单位为宿主；'calc(1px + 1in)' →
      // Sum[px(97)] 语料既有绿面，本刀不动）。
      for (var i = 0; i < absorbed.length; i++) {
        var m = absorbed[i];
        if (m instanceof CSSUnitValue) {
          var info = _zwOmUnitInfo(m._zwOmUnit);
          if (info && info.length >= 2) {
            var fi = firstIdx[info[0]];
            if (fi !== undefined) {
              var host = out[fi];
              host._zwOmValue += _zwOmNumClean(m._zwOmValue * info[1] / _zwOmUnitInfo(host._zwOmUnit)[1]);
              continue;
            }
            firstIdx[info[0]] = out.length;
          }
        }
        out.push(m);
      }
      return out.length === absorbed.length ? sumNode : new CSSMathSum(out);
    }
    // 刀 OM-13：declared/get 通道族规范单位折（css-values-4 §canonical-unit：
    // 基单位因子 1，桶和值即规范值）。合同面：normalize-numeric 三通道同断
    // 言 Sum[px(4), percent(1)]；set→get 文本回环平铺和式恒等（'calc(0% +
    // 0%)' 面族——percent/number/flex factorless 入不了桶）与 angle/time 例
    // alternateExpected（Sum[deg(0)]/Sum[s(0)]，testsuite 判读器 catch→
    // alternate 通路）双侧兼容。仅 count>1 桶折，孤桶保位保单位。
    var buckets = [];
    var bIdx = {};
    for (var bi = 0; bi < absorbed.length; bi++) {
      var bm = absorbed[bi];
      if (!(bm instanceof CSSUnitValue)) continue;
      var binfo = _zwOmUnitInfo(bm._zwOmUnit);
      if (!binfo || binfo.length < 2) continue;
      var bkey = bIdx[binfo[0]];
      if (bkey === undefined) {
        bkey = buckets.length;
        bIdx[binfo[0]] = bkey;
        buckets.push({ sum: 0, count: 0, pos: bi });
      }
      buckets[bkey].sum += bm._zwOmValue * binfo[1];
      buckets[bkey].count++;
    }
    var folded = false;
    for (var ck = 0; ck < buckets.length; ck++) folded = folded || buckets[ck].count > 1;
    if (!folded) return absorbed.length === members.length ? sumNode : new CSSMathSum(absorbed);
    var out2 = [];
    for (var k = 0; k < absorbed.length; k++) {
      var m3 = absorbed[k];
      if (m3 instanceof CSSUnitValue) {
        var info3 = _zwOmUnitInfo(m3._zwOmUnit);
        if (info3 && info3.length >= 2 && bIdx[info3[0]] !== undefined) {
          var bk = buckets[bIdx[info3[0]]];
          if (k === bk.pos) {
            if (bk.count > 1) {
              out2.push(new CSSUnitValue(_zwOmNumClean(bk.sum), _zwOmCanonFamUnit[info3[0]]));
              continue;
            }
            out2.push(m3);
            continue;
          }
          continue; // 桶内非首成员已并入
        }
      }
      out2.push(m3);
    }
    return new CSSMathSum(out2);
  }
  globalThis.CSSNumericValue = CSSNumericValue;

  // ===== CSSUnitValue =====
  function CSSUnitValue(value, unit) {
    var n = Number(value);
    if (!isFinite(n)) throw new TypeError("Invalid number value");
    var u = (unit == null) ? 'number' : _zwOmCanonUnit(String(unit));
    if (!u) throw new TypeError("Invalid unit");
    this._zwOmValue = n; this._zwOmUnit = u;
  }
  CSSUnitValue.prototype = Object.create(CSSNumericValue.prototype);
  CSSUnitValue.prototype.constructor = CSSUnitValue;
  Object.defineProperty(CSSUnitValue.prototype, 'value', {
    get: function () { return this._zwOmValue; },
    set: function (v) { var n = Number(v); if (!isFinite(n)) throw new TypeError("Invalid number value"); this._zwOmValue = n; },
    enumerable: true, configurable: true,
  });
  Object.defineProperty(CSSUnitValue.prototype, 'unit', {
    get: function () { return this._zwOmUnit; },
    set: function (v) { var u = _zwOmCanonUnit(String(v)); if (!u) throw new TypeError("Invalid unit"); this._zwOmUnit = u; },
    enumerable: true, configurable: true,
  });
  CSSUnitValue.prototype._zwOmKind = function () { return 'unit'; };
  CSSUnitValue.prototype.toString = function () {
    // number 面单位省略（canonical：'2' 非 '2number'）；percent 用 '%'。
    return _zwOmNumStr(this._zwOmValue) + (this._zwOmUnit === 'percent' ? '%' : (this._zwOmUnit === 'number' ? '' : this._zwOmUnit));
  };
  // type()：数值类型记录（css-typed-om-1 §cssnumericvalue-type；形状为仅非零
  // 指数键 + percentHint——零键省略，见 _zwOmUnitType）。
  CSSUnitValue.prototype.type = function () { return _zwOmUnitType(this._zwOmUnit); };
  try { CSSUnitValue.prototype[Symbol.toStringTag] = 'CSSUnitValue'; } catch (_eT) {}
  globalThis.CSSUnitValue = CSSUnitValue;

  // ===== 4. CSSMathValue 族 =====
  function CSSMathValue() { throw new TypeError("Illegal constructor"); }
  CSSMathValue.prototype = Object.create(CSSNumericValue.prototype);
  CSSMathValue.prototype.constructor = CSSMathValue;
  CSSMathValue.prototype._zwOmKind = function () { return 'math'; };
  globalThis.CSSMathValue = CSSMathValue;

  function _zwOmMathCtor(name, operator, wrap) {
    function Ctor() {
      var values = (arguments.length === 1 && Array.isArray(arguments[0])) ? arguments[0] : Array.prototype.slice.call(arguments);
      // 语料 cssMathValue 面（assert_throws_dom）：空值列表 DOMException
      // SyntaxError（css-typed-om-1 构造步语义，仅 variadic 族）；Clamp 参数量/
      // 类型面为 TypeError（wrap 内裁决，零参不抢先）。
      if (values.length === 0 && !wrap) {
        throw new (globalThis.DOMException || Error)(name + " requires at least one value", 'SyntaxError');
      }
      // 每个 value 接受 CSSNumberish：CSSUnitValue/CSSMathValue/number。
      var list = [];
      for (var i = 0; i < values.length; i++) {
        var v = values[i];
        if (typeof v === 'number') v = new CSSUnitValue(v, 'number');
        if (!(v instanceof CSSStyleValue)) throw new TypeError("Invalid value");
        list.push(v);
      }
      // 加法型构造器逐对消解校验（css-typed-om-1 §add-two-types：同族指数相等、
      // percent 归并/hint 一致；Sum/Min/Max/Hypot 面；clamp 另走 _zwOmClampCompat
      // 非-相等兼容面）。文法面（calc 解析器）经 _zwOmCtorStrict 旗标绕过——
      // 可加性在解析期裁决，构造期不重复抛。
      if (_zwOmCtorStrict && (operator === 'sum' || operator === 'min' || operator === 'max' || operator === 'hypot')) {
        var accT = _zwOmTypeOfVal(list[0]);
        for (var vi = 1; vi < list.length; vi++) accT = _zwOmTypeAdd(accT, _zwOmTypeOfVal(list[vi]), true);
      }
      // 刀 OM-13：乘积型构造器 percent-hint 面（css-typed-om-1
      // §multiplying-two-types：双方 percentHint 均非 null 且不同 → TypeError，
      // Sum(px,%)×Sum(s,%) 面）；文法面经 _zwOmCtorStrict 旗标绕过（解析期
      // 已裁决），与加法型同门。
      if (_zwOmCtorStrict && operator === 'product') {
        var hintSeen = null;
        for (var pi = 0; pi < list.length; pi++) {
          var ph = _zwOmTypeOfVal(list[pi]).percentHint;
          if (!ph) continue;
          if (hintSeen && hintSeen !== ph) throw new TypeError('percent hints must match');
          hintSeen = ph;
        }
      }
      if (wrap) wrap.call(this, list); // wrap 可裁移成员（Clamp/Invert），冻结在后
      this._zwOmValues = Object.freeze ? Object.freeze(list) : list;
    }
    Ctor.prototype = Object.create(CSSMathValue.prototype);
    Ctor.prototype.constructor = Ctor;
    // assert_class_string 消费 constructor.name——内部 Ctor 名外化为 spec 名。
    Object.defineProperty(Ctor, 'name', { value: name, configurable: true });
    Object.defineProperty(Ctor.prototype, 'operator', { get: function () { return operator; }, enumerable: true, configurable: true });
    Object.defineProperty(Ctor.prototype, 'values', { get: function () { return this._zwOmValues; }, enumerable: true, configurable: true });
    Ctor.prototype.type = function () { return _zwOmMathType(this); };
    Ctor.prototype.toString = function () { return _zwOmMathStr(this); };
    Ctor.prototype._zwOmKind = function () { return 'math'; };
    globalThis[name] = Ctor;
      try { Ctor.prototype[Symbol.toStringTag] = name; } catch (_eT) {}
return Ctor;
  }
  var CSSMathSum = _zwOmMathCtor('CSSMathSum', 'sum');
  var CSSMathProduct = _zwOmMathCtor('CSSMathProduct', 'product');
  var CSSMathMin = _zwOmMathCtor('CSSMathMin', 'min');
  var CSSMathMax = _zwOmMathCtor('CSSMathMax', 'max');
  var CSSMathHypot = _zwOmMathCtor('CSSMathHypot', 'hypot');
  var CSSMathAbs = _zwOmMathCtor('CSSMathAbs', 'abs');
  var CSSMathSign = _zwOmMathCtor('CSSMathSign', 'sign');
  // API 构造 strict 面（语料 cssMathValue：零参 TypeError；Clamp <3 参/混合单位
  // 类型 TypeError）。文法面（calc 解析器）经 _zwOmFnNode 关旗标绕过——CSS 文法
  // 允许混合 <length-percentage>（clamp(0px, 50%, 100px) 合法）。
  var _zwOmCtorStrict = true;
  function _zwOmTypeEq(a, b) {
    var k, seen = {};
    for (k in a) { seen[k] = 1; if ((a[k] || 0) !== (b[k] || 0)) return false; }
    for (k in b) if (!seen[k] && ((a[k] || 0) !== (b[k] || 0))) return false;
    return true;
  }
  // Clamp 加法兼容型（css-typed-om-1 type resolution）：percent 并入对方单族后
  // 全族相等；number↔percent 可加（hint number）；number↔带族不可加。
  function _zwOmClampCompat(a, b) {
    function resolve(t) {
      var fams = {}, famCount = 0, famName = null, hasPercent = false;
      for (var k in t) {
        if (k === 'percentHint' || !t[k]) continue;
        if (k === 'percent') { hasPercent = true; continue; }
        fams[k] = t[k]; famCount++; famName = k;
      }
      return { fams: fams, famCount: famCount, famName: famName, hasPercent: hasPercent };
    }
    var A = resolve(a), B = resolve(b);
    if (A.hasPercent && !B.hasPercent && B.famCount === 1) { A = resolve(a); A.fams = {}; A.fams[B.famName] = 1; A.famCount = 1; A.famName = B.famName; }
    else if (B.hasPercent && !A.hasPercent && A.famCount === 1) { B = resolve(b); B.fams = {}; B.fams[A.famName] = 1; B.famCount = 1; B.famName = A.famName; }
    else if (A.hasPercent !== B.hasPercent && (A.famCount === 0 || B.famCount === 0)) return true; // number↔percent
    return _zwOmTypeEq(A.fams, B.fams);
  }
  // Clamp：三参（min, value, max），max/min 可缺省。
  // Clamp 属性名按现行 spec/testhelper 为 lower/value/upper（min/max 只读别名）。
  var CSSMathClamp = _zwOmMathCtor('CSSMathClamp', 'clamp', function (list) {
    if (_zwOmCtorStrict) {
      if (list.length < 3) throw new TypeError("CSSMathClamp requires three values");
      // 兼容型（非相等型）：percent 可解析并入对方单族（clamp(10px, 10%, 20px)
      // 合法；number 与 px 不可加 TypeError，语料 cssMathValue/parse 双面）。
      if (!_zwOmClampCompat(list[0].type(), list[1].type()) || !_zwOmClampCompat(list[1].type(), list[2].type())) {
        throw new TypeError("CSSMathClamp arguments must have matching types");
      }
    }
    // 读位不裁员（values 须保持 [lower, value, upper]；pop 会清空冻结列表）。
    this._zwOmUpper = list[list.length - 1];
    this.value = list[list.length - 2];
    this._zwOmLower = list[list.length - 3];
  });
  Object.defineProperty(CSSMathClamp.prototype, 'lower', { get: function () { return this._zwOmLower; }, set: function (v) { this._zwOmLower = v; }, enumerable: true, configurable: true });
  Object.defineProperty(CSSMathClamp.prototype, 'upper', { get: function () { return this._zwOmUpper; }, set: function (v) { this._zwOmUpper = v; }, enumerable: true, configurable: true });
  Object.defineProperty(CSSMathClamp.prototype, 'min', { get: function () { return this._zwOmLower; }, enumerable: true, configurable: true });
  Object.defineProperty(CSSMathClamp.prototype, 'max', { get: function () { return this._zwOmUpper; }, enumerable: true, configurable: true });
  // Negate / Invert：单值（Invert 隐含除数 1）。
  var CSSMathNegate = _zwOmMathCtor('CSSMathNegate', 'negate', function (list) { this.value = list[0]; });
  var CSSMathInvert = _zwOmMathCtor('CSSMathInvert', 'invert', function (list) {
    this.value = list[0];
  });

  // ===== 5. CSSUnparsedValue / CSSVariableReferenceValue =====
  function CSSVariableReferenceValue(variable, fallback) {
    if (typeof variable !== 'string' || variable.slice(0, 2) !== '--') throw new TypeError("Invalid custom property name");
    this._zwOmVariable = variable;
    this._zwOmFallback = (fallback === undefined) ? null : fallback;
  }
  Object.defineProperty(CSSVariableReferenceValue.prototype, 'variable', {
    get: function () { return this._zwOmVariable; },
    // 刀 OM-14：variable 可写（css-typed-om-1 §dom-cssvariablereferencevalue-variable
    // setter 同构造器校验——非串/非 '--' 前缀 TypeError 且原值不变；语料
    // '--bar' 更新面；''/'bar' 面旧态 strict 赋值 TypeError 已绿）。
    set: function (v) {
      if (typeof v !== 'string' || v.slice(0, 2) !== '--') throw new TypeError("Invalid custom property name");
      this._zwOmVariable = v;
    },
    enumerable: true, configurable: true,
  });
  Object.defineProperty(CSSVariableReferenceValue.prototype, 'fallback', { get: function () { return this._zwOmFallback; }, enumerable: true, configurable: true });
  CSSVariableReferenceValue.prototype._zwOmKind = function () { return 'varref'; };
  CSSVariableReferenceValue.prototype.toString = function () {
    if (this._zwOmFallback == null) return 'var(' + this._zwOmVariable + ')';
    // 刀 OM-13：序列化逗号后无空白（css-typed-om-1 §varref-serialization
    // 'var(name,fallback)' 形；fallback 自身文本携带的空白保留——tokenizer
    // 保原文空白，'var(--A, 1em)' 回显不变）；语料 var-refs/mix 面。
    return 'var(' + this._zwOmVariable + ',' + this._zwOmFallback.toString() + ')';
  };
  globalThis.CSSVariableReferenceValue = CSSVariableReferenceValue;

  // 刀 OM-14：索引 set 陷阱（canonical 数值串键 n > length 抛 RangeError，
  // n == length 追加放行；length/负索引/符号键透传——Array exotic 索引写
  // 不可拦截，Proxy set 是合同唯一实现面）。
  function _zwOmUnparsedIndexTrap(t, k, v) {
    if (typeof k === 'string') {
      var n = Number(k);
      if (Number.isInteger(n) && n >= 0 && String(n) === String(k) && n > t.length) throw new RangeError("Index out of range");
    }
    t[k] = v;
    return true;
  }
  function CSSUnparsedValue() {
    var list = (arguments.length === 1 && Array.isArray(arguments[0])) ? arguments[0] : Array.prototype.slice.call(arguments);
    for (var i = 0; i < list.length; i++) {
      var s = list[i];
      if (typeof s !== 'string' && !(s instanceof CSSVariableReferenceValue)) throw new TypeError("Invalid unparsed segment");
    }
    var arr = list.slice();
    arr.__proto__ = CSSUnparsedValue.prototype;
    // 刀 OM-14：索引 setter 越界 RangeError（css-typed-om-1
    // §dom-cssunparsedvalue-__setter__：index > length 拒写 RangeError、
    // == length 追加；语料 out-of-range 面）。
    return new Proxy(arr, { set: _zwOmUnparsedIndexTrap });
  }
  // 原型链挂 CSSStyleValue（spec CSSUnparsedValue : CSSStyleValue；实例本体
  // 仍是 Array exotic 对象——length/index/isArray 天然成立），迭代器手动补。
  CSSUnparsedValue.prototype = Object.create(CSSStyleValue.prototype);
  try { CSSUnparsedValue.prototype[Symbol.toStringTag] = 'CSSUnparsedValue'; } catch (_eT) {}
  CSSUnparsedValue.prototype.constructor = CSSUnparsedValue;
  CSSUnparsedValue.prototype._zwOmKind = function () { return 'unparsed'; };
  // 刀 OM-13：ident 交界码点（css-syntax-3 §ident-token 起始/续接近似：ASCII
  // 字母数字、-、_ 与非 ASCII；转义面挂账 OM-14）。
  function _zwOmIdentEdge(ch) {
    return /[A-Za-z0-9_-\u0080-\uFFFF]/.test(ch);
  }
  // 刀 OM-13：段边界保 token 界（css-typed-om-1 §unparsedvalue-serialization：
  // 相邻段串接会并成单 token 时插入空注释 /**/——'lem'+'on'、'bar'+'var(' 面
  // 插；'bar '+'var('（空白已隔）与 ')'+'v'（括号界）不插。语料 strings/
  // var-refs/mix/round-trip 四面）。
  CSSUnparsedValue.prototype.toString = function () {
    var out = '';
    for (var i = 0; i < this.length; i++) {
      var seg = String(this[i]);
      if (out && _zwOmIdentEdge(out.charAt(out.length - 1)) && _zwOmIdentEdge(seg.charAt(0))) out += '/**/';
      out += seg;
    }
    return out;
  };
  globalThis.CSSUnparsedValue = CSSUnparsedValue;

  // ===== 6. CSSTransformValue 族 =====
  // DOMMatrixReadOnly 补挂（CSSMatrixComponent 语料消费；part05 仅有可变
  // DOMMatrix）。只读形状：is2D + m11..m44/a..f + toString 网格。
  if (!globalThis.DOMMatrixReadOnly) {
    globalThis.DOMMatrixReadOnly = function (init) {
      var m = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];
      if (typeof init === 'string') { /* cssText 形态暂不解析（FIXME） */ }
      else if (Array.isArray(init)) {
        if (init.length === 6) {
          m = [init[0], init[1], 0, 0, init[2], init[3], 0, 0, 0, 0, 1, 0, init[4], init[5], 0, 1];
        } else if (init.length === 16) { m = init.slice(); }
        else throw new TypeError("Invalid matrix init");
      } else if (init && typeof init === 'object') {
        var src = init._m || init;
        if (Array.isArray(src)) m = src.slice();
      }
      this._m = m;
    };
    var _zwDomMRP = globalThis.DOMMatrixReadOnly.prototype;
    Object.defineProperty(_zwDomMRP, 'is2D', { get: function () { var m = this._m; return m[2] === 0 && m[6] === 0 && m[10] === 1 && m[11] === 0 && m[14] === 0 && m[15] === 1; }, enumerable: true, configurable: true });
    Object.defineProperty(_zwDomMRP, 'isIdentity', { get: function () { var m = this._m; for (var i = 0; i < 16; i++) { if (m[i] !== (i % 5 === 0 ? 1 : 0)) return false; } return true; }, enumerable: true, configurable: true });
    var _zwMNames = ['m11', 'm12', 'm13', 'm14', 'm21', 'm22', 'm23', 'm24', 'm31', 'm32', 'm33', 'm34', 'm41', 'm42', 'm43', 'm44'];
    for (var _zwMi = 0; _zwMi < 16; _zwMi++) (function (idx) {
      Object.defineProperty(_zwDomMRP, _zwMNames[idx], { get: function () { return this._m[idx]; }, enumerable: true, configurable: true });
    })(_zwMi);
    var _zwAbcdef = ['a', 'b', 'c', 'd', 'e', 'f'];
    for (var _zwMi2 = 0; _zwMi2 < 6; _zwMi2++) (function (idx) {
      // a=m11 b=m12 c=m21 d=m22 e=m41 f=m42（2D 映射按 spec DOMMatrix）。
      var slot = [0, 1, 4, 5, 12, 13][idx];
      Object.defineProperty(_zwDomMRP, _zwAbcdef[idx], { get: function () { return this._m[slot]; }, enumerable: true, configurable: true });
    })(_zwMi2);
    _zwDomMRP.toFloat64Array = function () { return new Float64Array(this._m); };
    _zwDomMRP.toString = function () { return 'matrix(' + this._m.slice(0, 6).join(', ') + ')'; };
    // 变换方法族（stylevalue-subclasses toMatrix 展平语料 expectedMatrix 侧消费；
    // spec DOMMatrixReadOnly §transformation-methods。列主序 _m 与 part05
    // DOMMatrix 同存储约定，乘法同形——自洽后乘：identity × T）。
    var _zwOmMRIdentity = function () { return [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]; };
    // 4×4 列主序乘法（与 part05 _domMatrixMultiply 同索引形；IIFE 内共享，
    // 组件/容器 toMatrix 合成同源）。
    var _zwOmMatMul16 = function (a, b) {
      var r = new Array(16);
      for (var col = 0; col < 4; col++) {
        for (var row = 0; row < 4; row++) {
          r[col * 4 + row] = a[row] * b[col * 4] + a[4 + row] * b[col * 4 + 1] + a[8 + row] * b[col * 4 + 2] + a[12 + row] * b[col * 4 + 3];
        }
      }
      return r;
    };
    // FIXME(sv-compose): 乘积序为 other×self（与 part05 _domMatrixMultiply 同族
    // 约定，行向量 p·M 语义下相当于先 other 后 self）。toMatrix 语料期望侧
    // （part05 Self 族链）与组件侧同族相消故两侧恒一致；绝对序修正须与
    // part05 同步换序，不得单侧改序。
    var _zwOmMRWrap = function (m16) { return new globalThis.DOMMatrixReadOnly(m16); };
    var _zwOmAngleRad = function (v) {
      var u = v.unit, val = v.value;
      if (u === 'deg') return val * Math.PI / 180;
      if (u === 'grad') return val * Math.PI / 200;
      if (u === 'turn') return val * Math.PI * 2;
      return val; // rad 及其余按弧度数值直取
    };
    _zwDomMRP.multiply = function (other) { return _zwOmMRWrap(_zwOmMatMul16(this._m, other._m)); };
    // 不可变变换方法族（spec geometry-1 §DOMMatrixReadOnly transformation-methods：
    // 返回 this × 变换基阵的新矩阵，不修改 this；乘积序随 _zwOmMatMul16 族约定）。
    _zwDomMRP.translate = function (tx, ty, tz) {
      var t = _zwOmMRIdentity();
      t[12] = +tx || 0; t[13] = +ty || 0; t[14] = +tz || 0;
      return _zwOmMRWrap(_zwOmMatMul16(this._m, t));
    };
    _zwDomMRP.scale = function (sx, sy, sz) {
      var x = (sx == null) ? 1 : +sx, y = (sy == null) ? x : +sy, z = (sz == null) ? 1 : +sz;
      var s = _zwOmMRIdentity();
      s[0] = x; s[5] = y; s[10] = z;
      return _zwOmMRWrap(_zwOmMatMul16(this._m, s));
    };
    _zwDomMRP.skewX = function (ax) {
      var m = _zwOmMRIdentity();
      m[4] = Math.tan((+ax || 0) * Math.PI / 180);
      return _zwOmMRWrap(_zwOmMatMul16(this._m, m));
    };
    _zwDomMRP.skewY = function (ay) {
      var m = _zwOmMRIdentity();
      m[1] = Math.tan((+ay || 0) * Math.PI / 180);
      return _zwOmMRWrap(_zwOmMatMul16(this._m, m));
    };
    // Rodrigues（spec DOMMatrix rotateAxisAngle；轴 (0,0,0) 退化为绕 z）。
    _zwDomMRP.rotateAxisAngle = function (rx, ry, rz, angle) {
      var x = +rx || 0, y = +ry || 0, z = +rz || 0;
      var len = Math.sqrt(x * x + y * y + z * z);
      if (len === 0) { x = 0; y = 0; z = 1; } else { x /= len; y /= len; z /= len; }
      var rad = (+angle || 0) * Math.PI / 180;
      var c = Math.cos(rad), s = Math.sin(rad), t = 1 - c;
      var m = _zwOmMRIdentity();
      m[0] = t * x * x + c; m[4] = t * x * y - s * z; m[8] = t * x * z + s * y;
      m[1] = t * x * y + s * z; m[5] = t * y * y + c; m[9] = t * y * z - s * x;
      m[2] = t * x * z - s * y; m[6] = t * y * z + s * x; m[10] = t * z * z + c;
      return _zwOmMRWrap(_zwOmMatMul16(this._m, m));
    };
    // spec geometry-1 §rotate：仅 rx → rotateAxisAngle(0,0,1,rx)；否则绕
    // z/x/y 三轴依次合成（undefined 按 0）。
    _zwDomMRP.rotate = function (rx, ry, rz) {
      if (ry === undefined && rz === undefined) return this.rotateAxisAngle(0, 0, 1, rx);
      var m = this.rotateAxisAngle(0, 0, 1, rx);
      m = m.rotateAxisAngle(1, 0, 0, ry === undefined ? 0 : ry);
      m = m.rotateAxisAngle(0, 1, 0, rz === undefined ? 0 : rz);
      return m;
    };
  }

  function CSSTransformComponent() {}
  CSSTransformComponent.prototype = Object.create(CSSStyleValue.prototype);
  CSSTransformComponent.prototype._zwOmKind = function () { return 'transform-comp'; };
  globalThis.CSSTransformComponent = CSSTransformComponent;

  // 组件构造器：实例 `_` 前缀存储槽 + 原型 accessor；入参经 _zwOmTransSlot 校验
  //（CSSNumberish 裸数按槽位组归一；型不匹配 TypeError）；is2D 经 store/fixed
  // 两形暴露（testhelper assert_style_value_equals 组件分派消费 x/y/z/is2D/angle）。
  function _zwOmCompProp(proto, f, coerce) {
    Object.defineProperty(proto, f, {
      get: function () { return this['_' + f]; },
      set: function (v) { this['_' + f] = coerce ? coerce(v) : v; },
      enumerable: true, configurable: true,
    });
  }
  // ===== transform 组件槽位校验（css-typed-om-1 §cssrotate/§cssscale/
  // §csstranslate/§cssskew/§cssperspective：构造与 setter 入参不匹配槽位型 →
  // TypeError，setter 失败保旧值）。型判定复用 _zwOmTypeOfVal 的 math 消解
  //（negate 保形/invert 指数取负/product 指数累加/sum 逐对加法），单位消去
  //（CSSMathProduct(px, CSSMathInvert(px)) → 纯 <number>）随指数算术自然成立。
  function _zwOmTransMatch(v, group) {
    var t = _zwOmTypeOfVal(v), k, fams = [];
    for (k in t) {
      if (k === 'percentHint' || !t[k]) continue; // 零指数键（乘除消解残键）不计
      fams.push(k);
    }
    if (group === 'number') return fams.length === 0;
    if (fams.length !== 1) return false;
    var fam = fams[0];
    if (group === 'lengthPercent') {
      // calc(px + %) 加法面经 percentHint 归并（§add-two-types）；percent 族不收 hint
      return t[fam] === 1 && (fam === 'length' || (fam === 'percent' && !t.percentHint));
    }
    // 单族组（angle/length）：恰该族指数 1，无 pending percent（hint/percent 键）
    return fam === group && t[fam] === 1 && !t.percentHint && !t.percent;
  }
  // 槽位写入（ctor 与 setter 同走）：裸数按组归一（<number> 保 number、
  // length→px、angle→deg；skew/perspective 槽位不收裸数——语料 'a double'
  // TypeError 钉）；keyword 仅 perspective 'none' 臂前置转换，其余拒。
  function _zwOmTransSlot(v, group, name, numUnit) {
    if (typeof v === 'number') {
      if (!numUnit) throw new TypeError("Invalid " + name);
      return new CSSUnitValue(v, numUnit);
    }
    if (!(v instanceof CSSUnitValue) && !(v instanceof CSSMathValue)) throw new TypeError("Invalid " + name);
    if (!_zwOmTransMatch(v, group)) throw new TypeError("Invalid " + name);
    return v;
  }
  // is2D：rotate/scale/translate 可写（spec is2D attribute；语料 'Modifying
  // ... is2D can be updated to true or false'）；skew/perspective 固定形——
  // 赋值 no-op 不抛（语料 'Modifying ... is2D is a no-op' 直赋面）。
  function _zwOmIs2DStoreProp(proto) {
    Object.defineProperty(proto, 'is2D', {
      get: function () { return this._is2D; },
      set: function (v) { this._is2D = !!v; },
      enumerable: true, configurable: true,
    });
  }
  function _zwOmIs2DFixedProp(proto, val) {
    Object.defineProperty(proto, 'is2D', {
      get: function () { return val; },
      set: function () {},
      enumerable: true, configurable: true,
    });
  }

  function CSSTranslate(x, y, z) {
    // x/y <length-percentage>、z <length>（% 拒——语料 z percent TypeError 钉）；
    // is2D 按参数量定（2 参 true / 3 参 false，z=px(0) 直赋不改）。
    this._x = _zwOmTransSlot(x, 'lengthPercent', 'x', 'px');
    this._y = _zwOmTransSlot(y, 'lengthPercent', 'y', 'px');
    this._z = _zwOmTransSlot(arguments.length > 2 ? z : 0, 'length', 'z', 'px');
    this._is2D = arguments.length < 3;
  }
  CSSTranslate.prototype = Object.create(CSSTransformComponent.prototype);
  CSSTranslate.prototype.constructor = CSSTranslate;
  _zwOmCompProp(CSSTranslate.prototype, 'x', function (v) { return _zwOmTransSlot(v, 'lengthPercent', 'x', 'px'); });
  _zwOmCompProp(CSSTranslate.prototype, 'y', function (v) { return _zwOmTransSlot(v, 'lengthPercent', 'y', 'px'); });
  _zwOmCompProp(CSSTranslate.prototype, 'z', function (v) { return _zwOmTransSlot(v, 'length', 'z', 'px'); });
  _zwOmIs2DStoreProp(CSSTranslate.prototype);
  CSSTranslate.prototype.toString = function () {
    // css-typed-om-1 §csstranslate serialization：is2D → 两参 translate 形，
    // 否则恒 translate3d 形（'translate3d(1px, 2%, 3px)' 语料钉）。
    if (this._is2D) return 'translate(' + this._x + ', ' + this._y + ')';
    return 'translate3d(' + this._x + ', ' + this._y + ', ' + this._z + ')';
  };
  globalThis.CSSTranslate = CSSTranslate;

  function CSSRotate(xOrAngle, y, z, angle) {
    if (arguments.length <= 2) {
      // CSSRotate(angle)：绕 z 轴；x/y/z 归一 CSS.number（语料 number(0)/number(0)/
      // number(1) 钉）。
      this._x = new CSSUnitValue(0, 'number');
      this._y = new CSSUnitValue(0, 'number');
      this._z = new CSSUnitValue(1, 'number');
      this._angle = _zwOmTransSlot(xOrAngle, 'angle', 'angle', 'deg');
    } else {
      // CSSRotate(x, y, z, angle)：轴向量 CSSNumberish <number>（裸数→CSS.number、
      // CSS.number/math 直存——语料 'constructed from numberish coordinates' 钉）。
      this._x = _zwOmTransSlot(xOrAngle, 'number', 'x', 'number');
      this._y = _zwOmTransSlot(y, 'number', 'y', 'number');
      this._z = _zwOmTransSlot(z, 'number', 'z', 'number');
      this._angle = _zwOmTransSlot(angle, 'angle', 'angle', 'deg');
    }
    // css-typed-om-1 §cssrotate is2D：1-2 参恒 true、4 参恒 false（与轴向量
    // 零无关——rotate3d(0, 0, 0, …) 语料钉恒 3d 序列化形）。
    this._is2D = arguments.length <= 2;
  }
  CSSRotate.prototype = Object.create(CSSTransformComponent.prototype);
  CSSRotate.prototype.constructor = CSSRotate;
  _zwOmCompProp(CSSRotate.prototype, 'x', function (v) { return _zwOmTransSlot(v, 'number', 'x', 'number'); });
  _zwOmCompProp(CSSRotate.prototype, 'y', function (v) { return _zwOmTransSlot(v, 'number', 'y', 'number'); });
  _zwOmCompProp(CSSRotate.prototype, 'z', function (v) { return _zwOmTransSlot(v, 'number', 'z', 'number'); });
  _zwOmCompProp(CSSRotate.prototype, 'angle', function (v) { return _zwOmTransSlot(v, 'angle', 'angle', 'deg'); });
  _zwOmIs2DStoreProp(CSSRotate.prototype);
  CSSRotate.prototype.toString = function () {
    // css-typed-om-1 §cssrotate serialization：is2D → 'rotate(angle)'，否则恒
    // 'rotate3d(x, y, z, angle)'（spec 无 rotatex/rotatey 序列化形）。
    if (this._is2D) return 'rotate(' + this._angle + ')';
    return 'rotate3d(' + this._x + ', ' + this._y + ', ' + this._z + ', ' + this._angle + ')';
  };
  globalThis.CSSRotate = CSSRotate;

  function CSSScale(x, y, z) {
    // x/y/z <number>（CSSNumberish：裸数/CSS.number/number 型 math）；is2D 按参数量
    //（2 参 true / 3 参 false——语料 CSSScale(-3.14, CSS.number(3.14)) true 钉）。
    this._x = _zwOmTransSlot(x, 'number', 'x', 'number');
    this._y = _zwOmTransSlot(y, 'number', 'y', 'number');
    this._z = _zwOmTransSlot(arguments.length > 2 ? z : 1, 'number', 'z', 'number');
    this._is2D = arguments.length < 3;
  }
  CSSScale.prototype = Object.create(CSSTransformComponent.prototype);
  CSSScale.prototype.constructor = CSSScale;
  _zwOmCompProp(CSSScale.prototype, 'x', function (v) { return _zwOmTransSlot(v, 'number', 'x', 'number'); });
  _zwOmCompProp(CSSScale.prototype, 'y', function (v) { return _zwOmTransSlot(v, 'number', 'y', 'number'); });
  _zwOmCompProp(CSSScale.prototype, 'z', function (v) { return _zwOmTransSlot(v, 'number', 'z', 'number'); });
  _zwOmIs2DStoreProp(CSSScale.prototype);
  CSSScale.prototype.toString = function () {
    // css-typed-om-1 §cssscale serialization：is2D → 两参 scale 形，否则恒
    // scale3d 形（'scale3d(1, 2, 3)' 语料钉）。
    if (this._is2D) return 'scale(' + this._x + ', ' + this._y + ')';
    return 'scale3d(' + this._x + ', ' + this._y + ', ' + this._z + ')';
  };
  globalThis.CSSScale = CSSScale;

  function CSSSkew(ax, ay) {
    // ax/ay <angle>（裸数/keyword/string 拒——语料 'a double' TypeError 钉）。
    this._ax = _zwOmTransSlot(ax, 'angle', 'ax', null);
    this._ay = _zwOmTransSlot(ay == null ? new CSSUnitValue(0, 'deg') : ay, 'angle', 'ay', null);
  }
  CSSSkew.prototype = Object.create(CSSTransformComponent.prototype);
  CSSSkew.prototype.constructor = CSSSkew;
  _zwOmCompProp(CSSSkew.prototype, 'ax', function (v) { return _zwOmTransSlot(v, 'angle', 'ax', null); });
  _zwOmCompProp(CSSSkew.prototype, 'ay', function (v) { return _zwOmTransSlot(v, 'angle', 'ay', null); });
  _zwOmIs2DFixedProp(CSSSkew.prototype, true);
  CSSSkew.prototype.toString = function () {
    // css-typed-om-1 §cssskew serialization：ay 零角省略（'skew(90deg)' 语料钉）。
    if (this._ay instanceof CSSUnitValue && this._ay.value === 0) return 'skew(' + this._ax + ')';
    return 'skew(' + this._ax + ', ' + this._ay + ')';
  };
  globalThis.CSSSkew = CSSSkew;

  function CSSSkewX(ax) { this._ax = _zwOmTransSlot(ax, 'angle', 'ax', null); }
  CSSSkewX.prototype = Object.create(CSSTransformComponent.prototype);
  CSSSkewX.prototype.constructor = CSSSkewX;
  _zwOmCompProp(CSSSkewX.prototype, 'ax', function (v) { return _zwOmTransSlot(v, 'angle', 'ax', null); });
  _zwOmIs2DFixedProp(CSSSkewX.prototype, true);
  // skewX/skewY 序列化形大写 X/Y（css-typed-om-1 §cssskew serialization）。
  CSSSkewX.prototype.toString = function () { return 'skewX(' + this._ax + ')'; };
  globalThis.CSSSkewX = CSSSkewX;

  function CSSSkewY(ay) { this._ay = _zwOmTransSlot(ay, 'angle', 'ay', null); }
  CSSSkewY.prototype = Object.create(CSSTransformComponent.prototype);
  CSSSkewY.prototype.constructor = CSSSkewY;
  _zwOmCompProp(CSSSkewY.prototype, 'ay', function (v) { return _zwOmTransSlot(v, 'angle', 'ay', null); });
  _zwOmIs2DFixedProp(CSSSkewY.prototype, true);
  CSSSkewY.prototype.toString = function () { return 'skewY(' + this._ay + ')'; };
  globalThis.CSSSkewY = CSSSkewY;

  function CSSPerspective(length) {
    // length <length> 或 keyword 'none'（'none' 字符串转换面——语料
    // gConvertingValidTestCases）；裸数/单位数/其他 keyword 全拒。
    this._length = _zwOmPerspSlot(length);
  }
  function _zwOmPerspSlot(v) {
    if (v === 'none') v = new CSSKeywordValue('none');
    if (v instanceof CSSKeywordValue) {
      if (v.value === 'none') return v;
      throw new TypeError("Invalid length");
    }
    return _zwOmTransSlot(v, 'length', 'length', null);
  }
  CSSPerspective.prototype = Object.create(CSSTransformComponent.prototype);
  CSSPerspective.prototype.constructor = CSSPerspective;
  _zwOmCompProp(CSSPerspective.prototype, 'length', function (v) { return _zwOmPerspSlot(v); });
  _zwOmIs2DFixedProp(CSSPerspective.prototype, false);
  CSSPerspective.prototype.toString = function () {
    // css-typed-om-1 §cssperspective serialization：负 <length> 包 calc
    //（'perspective(calc(-1px))' 语料钉；math 节点负值判位 FIXME）。
    if (this._length instanceof CSSUnitValue && this._length.value < 0) return 'perspective(calc(' + this._length + '))';
    return 'perspective(' + this._length + ')';
  };
  globalThis.CSSPerspective = CSSPerspective;
  function CSSMatrixComponent(matrix, options) {
    if (!(matrix instanceof globalThis.DOMMatrixReadOnly) && !(matrix instanceof globalThis.DOMMatrix) && typeof matrix !== 'object') throw new TypeError("Invalid matrix");
    this._matrix = matrix;
    this._options = options || { is2D: matrix.is2D };
  }
  CSSMatrixComponent.prototype = Object.create(CSSTransformComponent.prototype);
  CSSMatrixComponent.prototype.constructor = CSSMatrixComponent;
  Object.defineProperty(CSSMatrixComponent.prototype, 'matrix', { get: function () { return this._matrix; }, set: function (m) { if (typeof m !== 'object') throw new TypeError("Invalid matrix"); this._matrix = m; }, enumerable: true, configurable: true });
  Object.defineProperty(CSSMatrixComponent.prototype, 'is2D', {
    get: function () { return !!(this._options && this._options.is2D); },
    // is2D 可写（spec CSSMatrixComponent is2D attribute；cssTransformValue-toMatrix
    // 语料 'respects is2D changes' 赋值面——仅 getter 时 setter 缺失赋值抛 strict 错）。
    set: function (v) {
      if (!this._options) this._options = {};
      this._options.is2D = !!v;
    },
    enumerable: true, configurable: true,
  });
  CSSMatrixComponent.prototype.toString = function () {
    var m = this._matrix && this._matrix._m ? this._matrix._m : null;
    if (m && this.is2D !== false) return 'matrix(' + m[0] + ', ' + m[1] + ', ' + m[4] + ', ' + m[5] + ', ' + m[12] + ', ' + m[13] + ')';
    if (m) return 'matrix3d(' + m.join(', ') + ')';
    return 'matrix(1, 0, 0, 1, 0, 0)';
  };
  CSSMatrixComponent.prototype._zwOmKind = function () { return 'transform-comp'; };
  globalThis.CSSMatrixComponent = CSSMatrixComponent;

  // ===== 组件 toMatrix 族（css-typed-om-1 §dom-csstransformcomponent-tomatrix；
  // cssTransformComponent-toMatrix / cssTransformValue-toMatrix 语料）。
  // 列主序 16 元素（与 part05 DOMMatrix / DOMMatrixReadOnly 同约定），is2D
  // 槽消费：true 面 2D 形（第三行/列恒等），false 面含 z/轴向量。
  function _zwOmDomRO(m16) { return new (globalThis.DOMMatrixReadOnly || globalThis.DOMMatrix)(m16); }
  var _zwOmAngleRadOf = function (v) {
    var u = v.unit, val = v.value;
    if (u === 'deg') return val * Math.PI / 180;
    if (u === 'grad') return val * Math.PI / 200;
    if (u === 'turn') return val * Math.PI * 2;
    return val; // rad 及其余按弧度数值直取
  };
  function _zwOmRotateMatrix(axisX, axisY, axisZ, rad) {
    var m = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];
    var len = Math.sqrt(axisX * axisX + axisY * axisY + axisZ * axisZ);
    var x = axisX, y = axisY, z = axisZ;
    if (len === 0) { x = 0; y = 0; z = 1; } else { x /= len; y /= len; z /= len; }
    var c = Math.cos(rad), s = Math.sin(rad), t = 1 - c;
    m[0] = t * x * x + c; m[4] = t * x * y - s * z; m[8] = t * x * z + s * y;
    m[1] = t * x * y + s * z; m[5] = t * y * y + c; m[9] = t * y * z - s * x;
    m[2] = t * x * z - s * y; m[6] = t * y * z + s * x; m[10] = t * z * z + c;
    return m;
  }
  // toMatrix 相对单位守门（spec css-typed-om-1 §CSSTranslate.toMatrix /
  // §CSSPerspective.toMatrix：组件含相对长度（em/ex/ch/rem/vw/vh 等）或
  // 百分比 → TypeError；仅绝对长度可无布局上下文解析为矩阵）。
  function _zwOmAbsLen(v) {
    return v instanceof CSSUnitValue && ['px', 'cm', 'mm', 'q', 'in', 'pt', 'pc'].indexOf(v.unit) !== -1;
  }
  CSSTranslate.prototype.toMatrix = function () {
    if (!_zwOmAbsLen(this._x) || !_zwOmAbsLen(this._y) || (this._z && !_zwOmAbsLen(this._z))) {
      throw new TypeError('CSSTranslate.toMatrix: relative length or percentage component');
    }
    var m = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];
    m[12] = this._x.value; m[13] = this._y.value;
    if (!this._is2D) m[14] = this._z.value;
    return _zwOmDomRO(m);
  };
  CSSRotate.prototype.toMatrix = function () {
    var rad = _zwOmAngleRadOf(this._angle);
    var m;
    if (this._is2D) {
      // is2D 面：轴向量忽略，绕 z（spec cssrotate is2D 语义）。
      m = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];
      m[0] = Math.cos(rad); m[1] = Math.sin(rad); m[4] = -Math.sin(rad); m[5] = Math.cos(rad);
    } else {
      m = _zwOmRotateMatrix(this._x.value, this._y.value, this._z.value, rad);
    }
    return _zwOmDomRO(m);
  };
  CSSScale.prototype.toMatrix = function () {
    var m = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];
    m[0] = this._x.value; m[5] = this._y.value;
    if (!this._is2D) m[10] = this._z.value;
    return _zwOmDomRO(m);
  };
  CSSSkew.prototype.toMatrix = function () {
    var m = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];
    m[4] = Math.tan(_zwOmAngleRadOf(this._ax));
    m[1] = Math.tan(_zwOmAngleRadOf(this._ay));
    return _zwOmDomRO(m);
  };
  CSSSkewX.prototype.toMatrix = function () {
    var m = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];
    m[4] = Math.tan(_zwOmAngleRadOf(this._ax));
    return _zwOmDomRO(m);
  };
  CSSSkewY.prototype.toMatrix = function () {
    var m = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];
    m[1] = Math.tan(_zwOmAngleRadOf(this._ay));
    return _zwOmDomRO(m);
  };
  CSSPerspective.prototype.toMatrix = function () {
    if (this._length instanceof CSSUnitValue && !_zwOmAbsLen(this._length)) {
      throw new TypeError('CSSPerspective.toMatrix: relative length component');
    }
    var m = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];
    if (this._length instanceof CSSUnitValue) m[11] = -1 / Math.max(1, this._length.value);
    return _zwOmDomRO(m); // 'none' keyword → identity
  };
  CSSMatrixComponent.prototype.toMatrix = function () {
    var mx = this._matrix;
    var m = mx && mx._m ? mx._m.slice() : [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];
    if (this.is2D) {
      // is2D 强制 → 2D 投影（m11/m12/m21/m22/m41/m42，余恒等——
      // cssTransformValue-toMatrix 'respects is2D changes' 期望形）。
      var p = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];
      p[0] = m[0]; p[1] = m[1]; p[4] = m[4]; p[5] = m[5]; p[12] = m[12]; p[13] = m[13];
      m = p;
    }
    return _zwOmDomRO(m);
  };

  function CSSTransformValue(components) {
    // components 为必备序列且非空（缺参/空表 → TypeError——语料
    // 'Constructing a CSSTransformValue with no components throws TypeError'）。
    var list = components == null ? [] : components;
    if (!Array.isArray(list) || list.length === 0) throw new TypeError("Invalid components");
    for (var i = 0; i < list.length; i++) {
      if (!(list[i] instanceof CSSTransformComponent)) throw new TypeError("Invalid transform component");
    }
    var arr = list.slice();
    arr.__proto__ = CSSTransformValue.prototype;
    return arr;
  }
  CSSTransformValue.prototype = Object.create(CSSStyleValue.prototype);
  try { CSSTransformValue.prototype[Symbol.toStringTag] = 'CSSTransformValue'; } catch (_eT) {}
  CSSTransformValue.prototype.constructor = CSSTransformValue;
  CSSTransformValue.prototype._zwOmKind = function () { return 'transform'; };
  Object.defineProperty(CSSTransformValue.prototype, 'is2D', {
    get: function () {
      for (var i = 0; i < this.length; i++) { if (this[i].is2D === false) return false; }
      return true;
    },
    enumerable: true, configurable: true,
  });
  CSSTransformValue.prototype.toMatrix = function () {
    // 逐分量 toMatrix 按序后乘（css-typed-om-1 §dom-csstransformvalue-tomatrix；
    // cssTransformValue-toMatrix 语料：expectedMatrix = 期望组件链 multiplySelf
    // 同序——列向量约定 M = M1 × M2 × … × Mn）。
    var m = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];
    for (var i = 0; i < this.length; i++) {
      m = _zwOmMatMul16(m, this[i].toMatrix()._m);
    }
    return _zwOmDomRO(m);
  };
  CSSTransformValue.prototype.toString = function () {
    var out = [];
    for (var i = 0; i < this.length; i++) out.push(String(this[i]));
    return out.join(' ');
  };
  globalThis.CSSTransformValue = CSSTransformValue;

  // Array-like 迭代器（两 Array 子类的原型已改挂 CSSStyleValue，不再继承
  // Array.prototype 的内置迭代器——iterable 语料面手动补）。
  function _zwOmArrIterator(obj) {
    var idx = 0;
    return { next: function () { return idx < obj.length ? { value: obj[idx++], done: false } : { value: undefined, done: true }; } };
  }
  CSSUnparsedValue.prototype[Symbol.iterator] = function () { return _zwOmArrIterator(this); };
  CSSTransformValue.prototype[Symbol.iterator] = function () { return _zwOmArrIterator(this); };

  // ===== 7. CSSImageValue / 色值类补挂 =====
  function CSSImageValue(text) {
    this._zwOmText = String(text);
  }
  CSSImageValue.prototype = Object.create(CSSStyleValue.prototype);
  CSSImageValue.prototype.constructor = CSSImageValue;
  CSSImageValue.prototype._zwOmKind = function () { return 'image'; };
  try { CSSImageValue.prototype[Symbol.toStringTag] = 'CSSImageValue'; } catch (_eT) {}
  CSSImageValue.prototype.toString = function () { return this._zwOmText; };
  globalThis.CSSImageValue = CSSImageValue;

  // part05 已有 CSSRGB/CSSHSL（color 内部鸭子面）——不覆盖；本批补齐其余
  // 色值构造器（语料 stylevalue-subclasses 直接构造断言通道/序列化面）。
  // 刀 CO：fields 升 [名, kind] 对——构造/赋值双走 part05 `__zwOmColorChannel`
  // 验证归一（css-typed-om-1 §csshwb/§csslch/§csslab 构造步与属性步；kind 表
  // 注释见 part05 色值段）。alpha 缺省 percent(100)（三参构造 alpha 100% 面）。
  function _zwOmColorCtor(name, fields) {
    if (globalThis[name]) return globalThis[name];
    function Ctor() {
      var args = Array.prototype.slice.call(arguments);
      for (var i = 0; i < fields.length; i++) {
        var f = fields[i][0], kind = fields[i][1];
        var v = args[i];
        if (f === 'alpha' && v === undefined) v = new CSSUnitValue(100, 'percent');
        this['_' + f] = globalThis.__zwOmColorChannel(kind, v);
      }
    }
    Object.defineProperty(Ctor, 'name', { value: name, configurable: true });
    Ctor.prototype = Object.create(CSSStyleValue.prototype);
    Ctor.prototype.constructor = Ctor;
    Ctor.prototype._zwOmKind = function () { return 'color'; };
    for (var i = 0; i < fields.length; i++) (function (f, kind) {
      Object.defineProperty(Ctor.prototype, f, {
        get: function () { return this['_' + f]; },
        set: function (v) { this['_' + f] = globalThis.__zwOmColorChannel(kind, v); },
        enumerable: true, configurable: true,
      });
    })(fields[i][0], fields[i][1]);
    globalThis[name] = Ctor;
    return Ctor;
  }
  _zwOmColorCtor('CSSHWB', [['h', 'hueStrict'], ['w', 'pctOnly'], ['b', 'pctOnly'], ['alpha', 'pctOnly']]);
  _zwOmColorCtor('CSSLCH', [['l', 'pctOnly'], ['c', 'pctOnly'], ['h', 'hueNumAng'], ['alpha', 'pctOnly']]);
  _zwOmColorCtor('CSSLab', [['l', 'pctOnly'], ['a', 'numPctKeep'], ['b', 'numPctKeep'], ['alpha', 'pctOnly']]);
  _zwOmColorCtor('CSSOKLCH', [['l', 'pctOnly'], ['c', 'pctOnly'], ['h', 'hueNumAng'], ['alpha', 'pctOnly']]);
  _zwOmColorCtor('CSSOKLab', [['l', 'pctOnly'], ['a', 'numPctKeep'], ['b', 'numPctKeep'], ['alpha', 'pctOnly']]);
  if (!globalThis.CSSColorValue) globalThis.CSSColorValue = function CSSColorValue() {};

  // ===== 8. CSS 命名空间数值工厂 =====
  // 工厂名 → 单位名大小写映射（spec numeric-factory：CSS.Q→'q'、CSS.Hz→'hz'、
  // CSS.kHz→'khz'；其余同名）。
  var _zwOmFactories = {
    number: 'number', percent: 'percent', em: 'em', ex: 'ex', ch: 'ch', cap: 'cap',
    ic: 'ic', lh: 'lh', rlh: 'rlh', rcap: 'rcap', rch: 'rch', rex: 'rex', ric: 'ric',
    rem: 'rem', vb: 'vb', vi: 'vi', vw: 'vw', vh: 'vh', vmin: 'vmin', vmax: 'vmax',
    svw: 'svw', svh: 'svh', svi: 'svi', svb: 'svb', lvw: 'lvw', lvh: 'lvh', lvi: 'lvi',
    lvb: 'lvb', dvw: 'dvw', dvh: 'dvh', dvi: 'dvi', dvb: 'dvb', cqw: 'cqw', cqh: 'cqh',
    cqi: 'cqi', cqb: 'cqb', cqmin: 'cqmin', cqmax: 'cqmax', cm: 'cm', mm: 'mm',
    Q: 'q', in: 'in', pt: 'pt', pc: 'pc', px: 'px', deg: 'deg', grad: 'grad',
    rad: 'rad', turn: 'turn', s: 's', ms: 'ms', Hz: 'hz', kHz: 'khz',
    dpi: 'dpi', dpcm: 'dpcm', dppx: 'dppx', x: 'x', fr: 'fr',
  };
  var _zwOmFactoryNames = Object.keys(_zwOmFactories);
  for (var fi = 0; fi < _zwOmFactoryNames.length; fi++) (function (name, unit) {
    // 无条件覆盖（part05 color 内部 percent/deg 鸭子 stub——{value,unit} 形状
    // 与 CSSUnitValue 实例鸭子兼容，消费面不变）。
    globalThis.CSS[name] = function (v) { return new CSSUnitValue(v, unit); };
  })(_zwOmFactoryNames[fi], _zwOmFactories[_zwOmFactoryNames[fi]]);

  // 简写 token 语法校验（parse 简写面与 validateSet 共用）：长度/百分比/
  // keyword/calc/var 过，其余族 TypeError（margin '10s' 语料面）。
  // 顶层单字符分隔切分（括号深度感知；calc()/var() 内部不拆）——parse 列表值
  // 逗号切分消费。
  function _zwOmTopSplit(s, ch) {
    var parts = [], cur = '', depth = 0;
    for (var i = 0; i < s.length; i++) {
      var c = s.charAt(i);
      if (c === '(') depth++; else if (c === ')') depth--;
      if (depth === 0 && c === ch) { parts.push(cur); cur = ''; continue; }
      cur += c;
    }
    parts.push(cur);
    return parts;
  }
  function _zwOmCheckShorthandTokens(text) {
    // calc()/var()/url() 跨段先折叠为占位 token（内部空格不参与分词）。
    var s = String(text).replace(/(calc|var|url)\((?:[^()]|\([^()]*\))*\)/gi, '@');
    var toks = s.trim().split(/\s+/);
    for (var ti = 0; ti < toks.length; ti++) {
      var tk = toks[ti];
      if (_zwOmCssWide[tk.toLowerCase()]) continue;
      // 折叠占位段（calc()/var()/url() 跨段）恒语法合法（css-typed-om-1
      // §set 值解析步：var() 引用值归 CSSUnparsedValue，不作文法校验）。
      if (tk === '@') continue;
      var um = /^[-+]?(?:\d+\.?\d*|\.\d+)([a-zA-Z%]+)$/.exec(tk);
      if (um) {
        var u2 = um[1].toLowerCase();
        var inf2 = _zwOmUnitInfo(u2 === '%' ? 'percent' : u2);
        var fam2 = inf2 ? inf2[0] : null;
        if (fam2 !== 'length' && fam2 !== 'percent') throw new TypeError("Invalid value for shorthand");
        continue;
      }
      if (/^(calc|var)\(/i.test(tk)) continue;
      if (/^[-a-zA-Z][-\w]*$/.test(tk)) continue;
      throw new TypeError("Invalid value for shorthand");
    }
  }
  // 简写 set 的长槽展开（语料 set-shorthand 面：margin 1-4 值语义）——宿主
  // style 声明存原文，长槽读数由本层补写。FIXME: all/background 简写展开。
  function _zwOmExpandShorthand(styleObj, p, text) {
    if (p !== 'margin') return;
    // 顶层空白分词（calc()/var() 括号跨段内部空格不拆）。
    var s = String(text).trim(), parts = [], cur = '', depth = 0;
    for (var ci = 0; ci < s.length; ci++) {
      var ch = s.charAt(ci);
      if (ch === '(') depth++; else if (ch === ')') depth--;
      if (depth === 0 && /\s/.test(ch)) { if (cur !== '') { parts.push(cur); cur = ''; } continue; }
      cur += ch;
    }
    if (cur !== '') parts.push(cur);
    if (parts.length === 0 || parts.length > 4) return;
    var t = parts[0], r = parts.length > 1 ? parts[1] : t;
    var b = parts.length > 2 ? parts[2] : t, l = parts.length > 3 ? parts[3] : r;
    var four = [t, r, b, l];
    var props = ['margin-top', 'margin-right', 'margin-bottom', 'margin-left'];
    for (var i = 0; i < 4; i++) { try { styleObj.setProperty(props[i], four[i]); } catch (_eM) {} }
  }
  // opaque 词形 ↔ 属性表裁决（parse 面专用）。
  function _zwOmOpaqueMatchesTable(prop, t, table) {
    var fams = {};
    for (var i = 0; i < table.length; i++) if (table[i].charAt(0) === '<') fams[table[i]] = 1;
    var pm = /^([-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?)([a-zA-Z%]+)$/.exec(t);
    if (pm) {
      var u = pm[2].toLowerCase() === '%' ? 'percent' : pm[2].toLowerCase();
      var inf = _zwOmUnitInfo(u);
      var fam = inf ? inf[0] : null;
      if (fam === 'length') return !!(fams['<length>'] || fams['<length-percentage>']);
      if (fam === 'percent') return !!(fams['<percentage>'] || fams['<length-percentage>']);
      if (fam) return !!fams['<' + fam + '>'];
      return false;
    }
    if (/^[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?$/.exec(t)) return !!(fams['<number>'] || fams['<integer>']);
    if (/^[-a-zA-Z][-\w]*$/.test(t)) {
      var hasFams = false;
      for (var f in fams) { hasFams = true; break; }
      if (!hasFams) return true; // keyword-only 表项：合法 ident 收 base（color 'red'）
      var hasCustomIdent = table.indexOf('<custom-ident>') >= 0 || table.indexOf('custom-ident') >= 0;
      var lower = t.toLowerCase();
      return hasCustomIdent || table.indexOf(lower) >= 0 || table.indexOf(t) >= 0;
    }
    return false;
  }

  // ===== 9. 文法解析（text → typed）=====
  var _zwOmCssWide = { 'initial': 1, 'inherit': 1, 'unset': 1, 'revert': 1, 'revert-layer': 1 };
  function _zwOmPropName(name) {
    var n = String(name).trim();
    if (n.slice(0, 2) === '--') return n;
    var lower = n.toLowerCase();
    if (lower === n) return n;
    // IDL camelCase（paddingTop → padding-top）dasherize 归一；交替大小写形
    //（'tRaNsItIoN-dUrAtIoN'/'wIdTh'）dasherize 产物非法——CSS 属性名解析面
    // 大小写不敏感（css-properties-values-api parse 语料 not-case-sensitive
    // 面；StylePropertyMap set/append/get 语料 case-insensitive 面），已知
    // 语法表（_zwOmPropSyntax/_zwOmKnownExtra）按 plain-lowercase 收敛。
    var dashed = /^[a-z]/.test(n) && /[A-Z]/.test(n)
      ? n.replace(/([A-Z])/g, function (_m, c, off) { return (off === 0 ? '' : '-') + c.toLowerCase(); }).toLowerCase()
      : lower;
    var dashedKnown = Object.prototype.hasOwnProperty.call(_zwOmPropSyntax, dashed) || _zwOmKnownExtra[dashed] === 1;
    if (!dashedKnown
      && (Object.prototype.hasOwnProperty.call(_zwOmPropSyntax, lower) || _zwOmKnownExtra[lower] === 1)) {
      return lower;
    }
    return dashed;
  }
  function _zwOmNumStr(n) {
    return String(n);
  }
  // 词元化 var() 段（CSSUnparsedValue）：字符串段与 CSSVariableReferenceValue 交替。
  function _zwOmTokenizeVar(text) {
    var segments = [];
    var buf = '';
    var i = 0;
    var qch = null;
    while (i < text.length) {
      var ch = text[i];
      if (qch) { buf += ch; if (ch === qch) qch = null; i++; continue; }
      if (ch === '"' || ch === '\'') { qch = ch; buf += ch; i++; continue; }
      if (ch === '/' && text[i + 1] === '*') {
        // 刀 OM-13：注释不产段内容且作段界（css-syntax-3 §comments +
        // css-typed-om-1 §normalize-tokens：语料 'bar/*comment*/var(--foo)'
        // 段 = ['bar', varref]、'round-trips correctly, though the comment
        // is gone' 面；未闭合注释吞至尾）。段间 token 界由 toString 插 /**/。
        var end = text.indexOf('*/', i + 2);
        if (buf) { segments.push(buf); buf = ''; }
        i = (end < 0) ? text.length : end + 2;
        continue;
      }
      var rest = text.slice(i);
      var m = /^var\(/i.exec(rest);
      if (!m) { buf += ch; i++; continue; }
      if (buf) { segments.push(buf); buf = ''; }
      var depth = 1; var j = m[0].length;
      while (j < rest.length && depth > 0) {
        if (rest[j] === '(') depth++;
        else if (rest[j] === ')') depth--;
        j++;
      }
      var inner = rest.slice(m[0].length, j - 1);
      // var(--A[, fallback])
      var commaIdx = _zwOmTopLevelComma(inner);
      var name = commaIdx >= 0 ? inner.slice(0, commaIdx) : inner;
      var fb = commaIdx >= 0 ? inner.slice(commaIdx + 1) : null;
      var ref = new CSSVariableReferenceValue(name.trim());
      if (fb != null) {
        var fbSegs = _zwOmTokenizeVar(fb);
        var fbArr = [];
        for (var k = 0; k < fbSegs.length; k++) fbArr.push(fbSegs[k]);
        // fallback 恒 CSSUnparsedValue 且保原文空白（css-typed-om-1
        // §normalize-tokens：'var(--A, 1em)' fallback = CSSUnparsedValue([' 1em'])）。
        ref._zwOmFallback = new CSSUnparsedValue(fbArr);
      }
      segments.push(ref);
      i += j;
    }
    if (buf) segments.push(buf);
    return segments;
  }
  function _zwOmTopLevelComma(s) {
    var depth = 0;
    for (var i = 0; i < s.length; i++) {
      if (s[i] === '(') depth++;
      else if (s[i] === ')') depth--;
      else if (s[i] === ',' && depth === 0) return i;
    }
    return -1;
  }
  // calc/数学函数文法 → CSSMathValue 树。
  function _zwOmCalcParser(text) {
    this.s = text; this.i = 0;
  }
  _zwOmCalcParser.prototype.ws = function () { while (this.i < this.s.length && /\s/.test(this.s[this.i])) this.i++; };
  _zwOmCalcParser.prototype.peek = function () { this.ws(); return this.i < this.s.length ? this.s[this.i] : ''; };
  _zwOmCalcParser.prototype.parseSum = function () {
    var nodes = [this.parseProduct()];
    for (;;) {
      var c = this.peek();
      if (c === '+' || c === '-') {
        this.i++;
        var rhs = this.parseProduct();
        if (c === '-') rhs = new CSSMathNegate(rhs);
        nodes.push(rhs);
      } else break;
    }
    if (nodes.length === 1) return nodes[0];
    return new CSSMathSum(nodes);
  };
  _zwOmCalcParser.prototype.parseProduct = function () {
    var nodes = [this.parseAtom()];
    for (;;) {
      this.ws();
      var c = this.s[this.i];
      if (c === '*') { this.i++; nodes.push(this.parseAtom()); }
      else if (c === '/') { this.i++; nodes.push(new CSSMathInvert(this.parseAtom())); }
      else break;
    }
    if (nodes.length === 1) return nodes[0];
    return new CSSMathProduct(nodes);
  };
  _zwOmCalcParser.prototype.parseAtom = function () {
    this.ws();
    var rest = this.s.slice(this.i);
    var m = /^([-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?)([a-zA-Z%]*)/.exec(rest);
    if (m) {
      this.i += m[0].length;
      var unit = (m[2] || 'number').toLowerCase();
      // '%' 归一 percent（单位面 canonical 化，_zwOmCanonUnit 小写单位直通）。
      var cu = _zwOmCanonUnit(unit);
      if (cu) unit = cu;
      return new CSSUnitValue(parseFloat(m[1]), unit);
    }
    if (rest[0] === '(') {
      this.i++;
      var inner = this.parseSum();
      this.ws();
      if (this.s[this.i] === ')') this.i++;
      return inner;
    }
    var fm = /^(calc|min|max|clamp|hypot|abs|sign|round|mod|rem)\(/i.exec(rest);
    if (fm) {
      this.i += fm[0].length;
      var fname = fm[1].toLowerCase();
      var args = [this.parseSum()];
      for (;;) {
        this.ws();
        if (this.s[this.i] === ',') { this.i++; args.push(this.parseSum()); }
        else break;
      }
      this.ws();
      if (this.s[this.i] === ')') this.i++;
      return _zwOmFnNode(fname, args);
    }
    throw new SyntaxError("calc parse error");
  };
  function _zwOmFnNode(name, args) {
    // 文法面构造绕过构造器 strict 校验（CSS 文法允许混合 <length-percentage> 的
    // clamp/min/max——strict 面只管 API 侧，见 _zwOmCtorStrict 注）。
    _zwOmCtorStrict = false;
    try { return _zwOmFnNodeDispatch(name, args); } finally { _zwOmCtorStrict = true; }
  }
  function _zwOmFnNodeDispatch(name, args) {
    switch (name) {
      case 'calc': return args.length === 1 ? args[0] : new CSSMathSum(args);
      case 'min': return new CSSMathMin(args);
      case 'max': return new CSSMathMax(args);
      case 'clamp': return new CSSMathClamp(args);
      case 'hypot': return new CSSMathHypot(args);
      case 'abs': return new CSSMathAbs(args);
      case 'sign': return new CSSMathSign(args);
      case 'round': case 'mod': case 'rem': return new CSSMathSum(args);
      default: return new CSSMathSum(args);
    }
  }
  // 声明文本 → typed 值（declared/computed 两面共用；face 仅影响回落链）。
  function _zwOmParseDeclared(prop, text) {
    var t = String(text).trim();
    if (t === '') return null;
    if (_zwOmCssWide[t.toLowerCase()]) return new CSSKeywordValue(t.toLowerCase());
    var lower = t.toLowerCase();
    if (/var\(/i.test(lower)) {
      // 值含 var() 引用 → 词元形（css-typed-om-1 §normalize-tokens；中间位
      // var（calc 包 var）同走此道，normalize-tokens 语料 calc 面）。
      var segs = _zwOmTokenizeVar(t);
      return new CSSUnparsedValue(segs);
    }
    if (prop.slice(0, 2) === '--') {
      // 自定义属性恒 CSSUnparsedValue（spec custom properties 面）。
      return new CSSUnparsedValue(_zwOmTokenizeVar(t));
    }
    var table = _zwOmPropSyntax[prop];
    if (!table) return _zwOmOpaque(); // 未知属性面：opaque 原文（_zwOmText 由调用方回填）
    if (table.length === 0) {
      // 空语法表 = 简写属性（margin/all 等）：无 typed 形态，CSSUnparsedValue。
      return new CSSUnparsedValue([t]);
    }
    var fams = {};
    for (var i = 0; i < table.length; i++) {
      var e = table[i];
      if (e.charAt(0) === '<') fams[e] = 1;
    }
    // url()/image
    if (fams['<image>'] || fams['<url>']) {
      if (/^url\(/i.test(t)) return new CSSImageValue(t);
    }
    // transform list
    if (fams['<transform>'] || fams['<transform-list>']) {
      var tv = _zwOmParseTransform(t);
      if (tv) return tv;
    }
    // calc()
    if (/^calc\(/i.test(t)) {
      try {
        var p = new _zwOmCalcParser(t.slice(5, -1));
        var node = p.parseSum();
        p.ws();
        if (p.i >= p.s.length) {
          if (!_zwOmValidateMath(node, fams)) return new CSSStyleValue();
          // 刀 OM-13：declared/get 通道同折（css-typed-om-1 §normalize-numeric
          // 三通道合同——normalize-numeric 语料 calc 面对数值/declared/get 三
          // 通道同断言 Sum[px(4), percent(1)]；canon 臂＝族规范单位折，
          // 仅并族桶动、孤桶恒——set→get 文本回环平铺和式恒等面不受扰）。
          return (node instanceof CSSMathSum) ? _zwOmFoldNumericSum(node, true) : node;
        }
      } catch (_eCalc) { /* 落 opaque */ }
    }
    // 单 token 数值（含百分比/单位维度）
    var dm = /^([-+]?(?:\d+\.?\d*|\.\d+))(%)$/.exec(t);
    if (dm) {
      if (fams['<percentage>'] || fams['<length-percentage>']) return new CSSUnitValue(parseFloat(dm[1]), 'percent');
      return _zwOmOpaque();
    }
    // 无单位纯数字（<number>/<integer> 面：z-index/order/flex-grow 等）。
    var pm = /^([-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?)$/.exec(t);
    if (pm) {
      if (fams['<number>'] || fams['<integer>']) return new CSSUnitValue(parseFloat(pm[1]), 'number');
      // 刀 OM-13：unitless zero（css-values-4 §lengths：<length> 文法允许零省
      // 略单位 → 0px；css-typed-om-1 §normalize-numeric declared/compute 面
      // width:'0' → CSS.px(0)。get 与 parse 同道，zzP4/zzP5 实证原先两通道
      // 均拒收）。
      if (parseFloat(pm[1]) === 0 && (fams['<length>'] || fams['<length-percentage>'])) return new CSSUnitValue(0, 'px');
      return _zwOmOpaque();
    }
    var nm = /^([-+]?(?:\d+\.?\d*|\.\d+))(?:[eE][-+]?\d+)?([a-zA-Z]+)$/.exec(t);
    if (nm) {
      var unit = nm[2].toLowerCase();
      var fam = _zwOmUnitInfo(unit) ? _zwOmUnitInfo(unit)[0] : null;
      if (fam === 'length' && (fams['<length>'] || fams['<length-percentage>'])) return new CSSUnitValue(parseFloat(nm[1]), unit);
      if (fam === 'angle' && fams['<angle>']) return new CSSUnitValue(parseFloat(nm[1]), unit);
      if (fam === 'time' && fams['<time>']) return new CSSUnitValue(parseFloat(nm[1]), unit);
      if (fam === 'frequency' && fams['<frequency>']) return new CSSUnitValue(parseFloat(nm[1]), unit);
      if (fam === 'resolution' && fams['<resolution>']) return new CSSUnitValue(parseFloat(nm[1]), unit);
      if (fam === 'flex' && fams['<flex>']) return new CSSUnitValue(parseFloat(nm[1]), unit);
      if (fam === 'number' && (fams['<number>'] || fams['<integer>'])) return new CSSUnitValue(parseFloat(nm[1]), 'number');
      if (fam === 'number' && (fams['<length>'] || fams['<percentage>'] || fams['<angle>'] || fams['<time>'])) return _zwOmOpaque();
      return _zwOmOpaque();
    }
    // 单 ident（keyword 门：须在属性 keyword 表内；custom-ident 任意 ident）
    if (/^[-a-zA-Z][-\w]*$/.test(t)) {
      var isCustomIdent = table.indexOf('<custom-ident>') >= 0 || table.indexOf('custom-ident') >= 0;
      if (isCustomIdent || table.indexOf(t.toLowerCase()) >= 0 || table.indexOf(t) >= 0) return new CSSKeywordValue(t.toLowerCase());
      return _zwOmOpaque();
    }
    return _zwOmOpaque();
  }
  function _zwOmValidateMath(node, fams) {
    // 数值树与族表粗校验：任一叶单位族在表内即通过（粗粒度；详表校验归
    // set 面 type 检查）。
    var famsMap = fams || {};
    function walk(n) {
      if (n instanceof CSSUnitValue) {
        var info = _zwOmUnitInfo(n.unit);
        var f = info ? info[0] : null;
        // 刀 OM-3：length 叶只配 <length>/<length-percentage> 表——旧粗校验
        // 放行 <percentage> 表致 opacity 族 calc 长度混入（语料
        // testPropertyInvalid 'Setting ... to a length throws TypeError' 期望
        // TypeError；<percentage> 文法不含长度形，css-values-4 §calc）。
        if (f === 'length') return !!(famsMap['<length>'] || famsMap['<length-percentage>']);
        if (f === 'percent') return !!(famsMap['<percentage>'] || famsMap['<length-percentage>']);
        if (f) return !!famsMap['<' + f + '>'];
        return false;
      }
      if (n instanceof CSSMathValue) {
        var vs = n._zwOmValues || [];
        for (var i = 0; i < vs.length; i++) if (walk(vs[i])) return true;
      }
      return false;
    }
    return walk(node);
  }
  // rotate3d 轴向量取裸 number（CSSRotate 轴属性为 unrestricted double）。
  function _zwOmAxisNum(v) {
    if (typeof v === 'number') return v;
    if (v instanceof CSSUnitValue && v.unit === 'number') return v.value;
    throw new SyntaxError("Invalid axis number");
  }
  function _zwOmParseTransform(t) {
    // 函数括号支持一层嵌套（calc() 参数形——'\([^)]*\)' 会在 calc 内 '(' 提前
    // 截断致整列表解析 null）。
    var parts = t.match(/(matrix3d|matrix|translate3d|translate[XYZ]|translate|scale3d|scale[XYZ]|scale|rotate3d|rotate[XYZ]|rotate|skew[XY]|skew|perspective)\((?:[^()]|\([^()]*\))*\)/gi);
    if (!parts || parts.join(' ') !== t.replace(/\s+/g, ' ').trim()) return null;
    var comps = [];
    for (var i = 0; i < parts.length; i++) {
      var m = /^([a-z]+3d|[a-z]+)\((.*)\)$/i.exec(parts[i].trim());
      if (!m) return null;
      var fn = m[1].toLowerCase();
      var argText = m[2];
      var argParts = argText.split(',');
      // perspective(none)（css-typed-om-1 §transformvalue-normalization 语料钉
      // ——keyword none 归一 CSSPerspective(CSSKeywordValue('none'))）。
      if (fn === 'perspective' && argParts.length === 1 && argParts[0].trim().toLowerCase() === 'none') {
        comps.push(new CSSPerspective('none'));
        continue;
      }
      var nums = [];
      for (var k = 0; k < argParts.length; k++) {
        // 变换参数直接数值文法（不在属性语法表内，无 declared 面）；calc() 形
        // 经 calc 解析器 math 组件化（'translate(calc(1px + 1em))' 语料钉）。
        var tok = argParts[k].trim();
        var pm2 = /^([-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?)(%)?$/.exec(tok);
        if (pm2) {
          nums.push(pm2[2] ? new CSSUnitValue(parseFloat(pm2[1]), 'percent') : new CSSUnitValue(parseFloat(pm2[1]), 'number'));
          continue;
        }
        var dm2 = /^([-+]?(?:\d+\.?\d*|\.\d+))(deg|grad|rad|turn|px|em|rem|ex|ch|vw|vh|vmin|vmax|q|cm|mm|in|pt|pc|s|ms)$/i.exec(tok);
        if (dm2) { nums.push(new CSSUnitValue(parseFloat(dm2[1]), dm2[2].toLowerCase())); continue; }
        if (/^calc\(/i.test(tok)) {
          try {
            var cp = new _zwOmCalcParser(tok.slice(5, -1));
            var mnode = cp.parseSum();
            cp.ws();
            if (cp.i >= cp.s.length) { nums.push(mnode); continue; }
          } catch (_eC) { /* 落拒 */ }
        }
        nums = null; break;
      }
      if (!nums) return null;
      switch (fn) {
        case 'translate': comps.push(new CSSTranslate(nums[0], nums.length > 1 ? nums[1] : new CSSUnitValue(0, 'px'))); break;
        case 'translatex': comps.push(new CSSTranslate(nums[0], new CSSUnitValue(0, 'px'))); break;
        case 'translatey': comps.push(new CSSTranslate(new CSSUnitValue(0, 'px'), nums[0])); break;
        case 'translate3d': comps.push(new CSSTranslate(nums[0], nums[1], nums[2])); break;
        case 'translatez': comps.push(new CSSTranslate(new CSSUnitValue(0, 'px'), new CSSUnitValue(0, 'px'), nums[0])); break;
        case 'scale': comps.push(new CSSScale(nums[0], nums.length > 1 ? nums[1] : nums[0])); break;
        case 'scalex': comps.push(new CSSScale(nums[0], 1)); break;
        case 'scaley': comps.push(new CSSScale(1, nums[0])); break;
        case 'scale3d': comps.push(new CSSScale(nums[0], nums[1], nums[2])); break;
        case 'scalez': comps.push(new CSSScale(1, 1, nums[0])); break;
        case 'rotate': comps.push(new CSSRotate(nums[0])); break;
        case 'rotatex': comps.push(new CSSRotate(1, 0, 0, nums[0])); break;
        case 'rotatey': comps.push(new CSSRotate(0, 1, 0, nums[0])); break;
        case 'rotatez': comps.push(new CSSRotate(0, 0, 1, nums[0])); break;
        case 'rotate3d':
          if (nums.length !== 4) return null;
          comps.push(new CSSRotate(_zwOmAxisNum(nums[0]), _zwOmAxisNum(nums[1]), _zwOmAxisNum(nums[2]), nums[3]));
          break;
        case 'skew': comps.push(new CSSSkew(nums[0], nums.length > 1 ? nums[1] : new CSSUnitValue(0, 'deg'))); break;
        case 'skewx': comps.push(new CSSSkewX(nums[0])); break;
        case 'skewy': comps.push(new CSSSkewY(nums[0])); break;
        case 'perspective': comps.push(new CSSPerspective(nums[0])); break;
        case 'matrix':
          if (nums.length !== 6) return null;
          comps.push(new CSSMatrixComponent(new DOMMatrixReadOnly(nums.map(function (n) { return n.value; }))));
          break;
        case 'matrix3d':
          if (nums.length !== 16) return null;
          comps.push(new CSSMatrixComponent(new DOMMatrixReadOnly(nums.map(function (n) { return n.value; })), { is2D: false }));
          break;
        default: return null;
      }
    }
    return comps.length ? new CSSTransformValue(comps) : null;
  }
  // opaque 原文回填：_zwOmParseDeclared 返回裸 CSSStyleValue 时由外层携带文本。
  var _zwOmParseWithText = function (prop, text) {
    var v = _zwOmParseDeclared(prop, text);
    if (v && v instanceof CSSStyleValue && v._zwOmKind() === 'opaque' && v._zwOmText === undefined) {
      v._zwOmText = String(text).trim();
      v.toString = function () { return this._zwOmText; };
    }
    // opaque 实例携来源属性（css-typed-om-1 §set/§append 值裁决关联属性
    // 错配门——get/parse 产物不得跨属性并置；chrome oracle append 恒
    // TypeError 'Invalid type for property'，inline append 语料
    // 'a CSSVariableReferenceValue property' 面依赖此门）。
    if (v && v instanceof CSSStyleValue && v._zwOmKind() === 'opaque' && v._zwOmAssociatedProp === undefined) {
      v._zwOmAssociatedProp = prop;
    }
    return v;
  };

  // ===== 10. 数值类型记录 =====
  // 类型 map 只保留非零指数键（css-typed-om-1 §cssnumericvalue-type；语料
  // assert_numeric_type_equals 逐键 assert_equals——零指数键缺省语义，键须省略
  // 而非 0）。percentHint 仅在加法消解归并时出现（Unit 型 percent 无 hint）。
  function _zwOmUnitType(unit) {
    var t = {};
    if (unit === 'percent') { t.percent = 1; return t; }
    var info = _zwOmUnitInfo(unit);
    if (info && info[0] !== 'number') t[info[0]] = 1;
    return t;
  }
  function _zwOmTypeOfVal(v) {
    return (v instanceof CSSMathValue) ? v.type()
      : _zwOmUnitType(v instanceof CSSUnitValue ? v._zwOmUnit : 'number');
  }
  // 加法消解（css-typed-om-1 §add-two-types）：同族指数须逐键相等；percent +
  // 指数 1 单具体族 → percentHint 归并（族键保留、percent 键吸收）；% + number
  // → percent 保形；双侧 % 指数相等保留；hint 冲突/指数不等/任一键缺于一侧
  // （number↔具体族、跨族）→ strict 抛 TypeError；宽容面（type() 恒不抛）并集
  // 近似。
  function _zwOmTypeAdd(a, b, strict) {
    var ha = a.percentHint || null, hb = b.percentHint || null;
    var pa = a.percent || 0, pb = b.percent || 0;
    var k;
    function bestEffort() {
      var m = {};
      for (k in a) if (k !== 'percentHint' && a[k]) m[k] = a[k];
      for (k in b) if (k !== 'percentHint' && b[k] && !m[k]) m[k] = b[k];
      if (ha || hb) m.percentHint = ha || hb;
      return m;
    }
    if (ha && hb && ha !== hb) {
      if (strict) throw new TypeError("percent hints must match");
      return bestEffort();
    }
    var fams = {}, count = 0, single = null;
    for (k in a) if (k !== 'percent' && k !== 'percentHint' && a[k]) { fams[k] = a[k]; count++; single = k; }
    for (k in b) if (k !== 'percent' && k !== 'percentHint' && b[k]) { if (!fams[k]) { count++; single = k; } fams[k] = b[k]; }
    for (k in a) if (a[k] && b[k] && k !== 'percent' && k !== 'percentHint' && a[k] !== b[k]) {
      if (strict) throw new TypeError("type exponents must match");
      return bestEffort();
    }
    var hint = ha || hb, out = {};
    if (pa > 0 && pb > 0) {
      if (count > 0 || pa !== pb) { if (strict) throw new TypeError("percent types are not addable"); return bestEffort(); }
      out.percent = pa;
    } else if (pa > 0 || pb > 0) {
      if (count === 1 && fams[single] === 1) { out[single] = 1; if (!hint) hint = single; }
      else if (count === 0) out.percent = (pa > 0 ? pa : pb);
      else { if (strict) throw new TypeError("percent types are not addable"); return bestEffort(); }
    } else {
      // 无 percent：任一键缺于一侧即不可加（count 为并集键数）
      var aCount = 0;
      for (k in a) if (k !== 'percent' && k !== 'percentHint' && a[k]) aCount++;
      if (aCount !== count) { if (strict) throw new TypeError("types are not addable"); return bestEffort(); }
      for (k in fams) out[k] = fams[k];
    }
    if (hint) out.percentHint = hint;
    return out;
  }
  // CSSMathValue 型（css-typed-om-1 §type-of-a-cssmathvalue）：negate 保形、
  // invert 指数取负、product 指数累加；sum/min/max/clamp/hypot/abs 逐对加法
  // 消解（不可加已由构造器 strict 面与解析期检查挡住，type() 恒不抛）。
  function _zwOmMathType(node) {
    var op = node.operator;
    var members = node._zwOmValues || [];
    var i, k;
    if (op === 'sign') return {};
    if (op === 'negate') return _zwOmTypeOfVal(members[0]);
    if (op === 'invert') {
      var inv = _zwOmTypeOfVal(members[0]);
      var neg = {};
      for (k in inv) if (k !== 'percentHint') neg[k] = -inv[k];
      if (inv.percentHint) neg.percentHint = inv.percentHint;
      return neg;
    }
    if (op === 'product') {
      var prod = {}, hint = null;
      for (i = 0; i < members.length; i++) {
        var mt = _zwOmTypeOfVal(members[i]);
        if (mt.percentHint) hint = mt.percentHint;
        for (k in mt) if (k !== 'percentHint') prod[k] = (prod[k] || 0) + mt[k];
      }
      if (hint) prod.percentHint = hint;
      return prod;
    }
    var acc = {};
    for (i = 0; i < members.length; i++) acc = _zwOmTypeAdd(acc, _zwOmTypeOfVal(members[i]), false);
    return acc;
  }
  // CSSMathValue 序列化（css-typed-om-1 § calc-serialization，语料
  // stylevalue-serialization 面）：顶层 calc()/min()/max()/clamp() 包裹；嵌套
  // Sum/Product 成员加括号（多成员），min/max/clamp 参数为裸表达式；Sum 后位
  // Negate 融合 ' - '、Product 后位 Invert 融合 ' / '；首位 Negate/Invert 括号
  // 保形（仅多成员——parsed 'calc(-3.14em)' 单 Negate 精确往返）。
  // ctx：'top'（顶层）| 'term'（Sum/Product 成员位）| 'arg'（min/max/clamp 参数位）。
  function _zwOmMathStr(node, ctx) {
    if (ctx === undefined) ctx = 'top';
    var op = node.operator;
    var members = node._zwOmValues || [];
    var i, m;
    if (op === 'sum' || op === 'product') {
      var mul = (op === 'product');
      // term 位 Negate/Invert 恒括号（css-typed-om-1 序列化步；单成员 Sum[Negate]
      // → 'calc((-1))' 同形）；多成员后位融合 ' - '/' / '。
      var parts = [];
      for (i = 0; i < members.length; i++) {
        m = members[i];
        if (!mul && i > 0 && m instanceof CSSMathValue && m.operator === 'negate') {
          parts.push('- ' + _zwOmValStr(m._zwOmValues[0], 'term'));
          continue;
        }
        if (mul && i > 0 && m instanceof CSSMathValue && m.operator === 'invert') {
          parts.push('/ ' + _zwOmValStr(m._zwOmValues[0], 'term'));
          continue;
        }
        parts.push(_zwOmValStr(m, 'term'));
      }
      var body = parts.join(mul ? ' * ' : ' + ')
        .replace(mul ? / \* \/ /g : / \+ - /g, mul ? ' / ' : ' - ');
      if (ctx === 'arg') return body;
      if (ctx === 'term') return members.length > 1 ? '(' + body + ')' : body;
      return 'calc(' + body + ')';
    }
    if (op === 'negate') {
      var nv = '-' + _zwOmValStr(members[0], 'term');
      if (ctx === 'arg') return nv;
      if (ctx === 'term') return '(' + nv + ')';
      return 'calc(' + nv + ')';
    }
    if (op === 'invert') {
      var iv = '1 / ' + _zwOmValStr(members[0], 'term');
      if (ctx === 'arg') return iv;
      if (ctx === 'term') return '(' + iv + ')';
      return 'calc(' + iv + ')';
    }
    var heads = { min: 'min', max: 'max', clamp: 'clamp', hypot: 'hypot', abs: 'abs', sign: 'sign' };
    var head = heads[op] || op;
    var argParts = [];
    for (i = 0; i < members.length; i++) argParts.push(_zwOmValStr(members[i], 'arg'));
    return head + '(' + argParts.join(', ') + ')';
  }
  function _zwOmValStr(v, ctx) {
    return (v && typeof v.operator === 'string') ? _zwOmMathStr(v, ctx) : String(v);
  }
  // 算术 API（css-typed-om-1 §dom-cssnumericvalue-add/sub/mul/div/min/max）：
  // 无参恒等返回 this；numberish 展开；sum/sub/min/max 逐对加法消解（不可加
  // TypeError）；mul/div 乘除消解（percentHint 冲突 TypeError；div 除数 number
  // 型 0 值 RangeError）。UnitValue 全体时按运算化简：加/减/取小取大同单位、
  // mul 非 number 成员至多一个、div 除数全 number 型。sub 逐参取负（Unit 直取
  // 负、Negate 剥壳——双重取负抵消）；div 逐参取倒（number 直取倒数、Invert
  // 剥壳）。
  function _zwOmArith(mode, self, args) {
    if (args.length === 0) return self;
    var list = [self];
    for (var i = 0; i < args.length; i++) {
      var v = args[i];
      if (typeof v === 'number') v = new CSSUnitValue(v, 'number');
      list.push(v);
    }
    // 刀 OM-12：单成员同构 math 接收体剥壳（Sum(x).add(y) ≡ x.add(y)——
    // css-typed-om-1 算术步 Sum([x]) 与 x 等价；语料 'new mathType(CSS.number(1))
    // .add(CSS.number(2), CSS.number(3)) simplifies to a CSSUnitValue' 面）。
    var selfOp12 = ({ sum: 'sum', sub: 'sum', mul: 'product', div: 'product', min: 'min', max: 'max' })[mode];
    if (self instanceof CSSMathValue && self.operator === selfOp12 && self._zwOmValues && self._zwOmValues.length === 1) {
      list[0] = self._zwOmValues[0];
    }
    var i2;
    if (mode === 'div') {
      // 除数 number 型 0 值恒 RangeError（css-typed-om-1 divide 步；语料零除面）
      for (i2 = 1; i2 < list.length; i2++) {
        if (list[i2] instanceof CSSUnitValue && list[i2]._zwOmUnit === 'number' && list[i2]._zwOmValue === 0) throw new RangeError("Cannot divide by zero");
      }
    }
    if (mode === 'sum' || mode === 'sub' || mode === 'min' || mode === 'max') {
      var accT = _zwOmTypeOfVal(list[0]);
      for (i2 = 1; i2 < list.length; i2++) accT = _zwOmTypeAdd(accT, _zwOmTypeOfVal(list[i2]), true);
    } else {
      // mul/div 乘除消解：族指数任并，percentHint 冲突才 TypeError
      // （css-typed-om-1 §multiply-two-types/§divide-two-types）。
      // 刀 OM-12：跨实族乘除 TypeError（语料 'px(0).mul(px(1), CSS.s(2))
      // incompatible types throws' 面；number/percent 维外豁免——number(10)
      // .mul(px(5), px(2)) 仍归 Product 不化简）。
      var hint = null;
      var fams12 = {}, famN12 = 0;
      for (i2 = 0; i2 < list.length; i2++) {
        var h = _zwOmTypeOfVal(list[i2]).percentHint;
        if (h) { if (hint && hint !== h) throw new TypeError("percent hints must match"); hint = h; }
        if (list[i2] instanceof CSSUnitValue) {
          var ui12 = _zwOmUnitInfo(list[i2]._zwOmUnit);
          var uf12 = ui12 ? ui12[0] : null;
          if (uf12 && uf12 !== 'number' && uf12 !== 'percent' && !fams12[uf12]) {
            fams12[uf12] = 1; famN12++;
          }
        }
      }
      if (famN12 > 1) throw new TypeError("Incompatible types");
    }
    var allUnit = true;
    for (i2 = 0; i2 < list.length; i2++) if (!(list[i2] instanceof CSSUnitValue)) { allUnit = false; break; }
    if (allUnit) {
      var fold = true, resUnit = list[0]._zwOmUnit;
      if (mode === 'sum' || mode === 'sub' || mode === 'min' || mode === 'max') {
        for (i2 = 1; i2 < list.length; i2++) if (list[i2]._zwOmUnit !== resUnit) { fold = false; break; }
      } else if (mode === 'mul') {
        var nonNums = 0;
        for (i2 = 0; i2 < list.length; i2++) {
          if (list[i2]._zwOmUnit !== 'number') { nonNums++; resUnit = list[i2]._zwOmUnit; }
        }
        fold = nonNums <= 1;
      } else { // div：除数全 number 型才化简（商单位即 self 单位）
        for (i2 = 1; i2 < list.length; i2++) if (list[i2]._zwOmUnit !== 'number') { fold = false; break; }
      }
      if (fold) {
        var acc = list[0]._zwOmValue;
        for (i2 = 1; i2 < list.length; i2++) {
          var mv = list[i2]._zwOmValue;
          if (mode === 'sum') acc += mv;
          else if (mode === 'sub') acc -= mv;
          else if (mode === 'mul') acc *= mv;
          else if (mode === 'div') acc /= mv;
          else if (mode === 'min') acc = Math.min(acc, mv);
          else acc = Math.max(acc, mv);
        }
        return new CSSUnitValue(_zwOmNumClean(acc), resUnit);
      }
    }
    switch (mode) {
      case 'sum': return new CSSMathSum(list);
      case 'sub': {
        var negs = [list[0]];
        for (i2 = 1; i2 < list.length; i2++) {
          var sv = list[i2];
          if (sv instanceof CSSUnitValue) sv = new CSSUnitValue(-sv._zwOmValue, sv._zwOmUnit);
          else if (sv instanceof CSSMathValue && sv.operator === 'negate') sv = sv._zwOmValues[0];
          else sv = new CSSMathNegate(sv);
          negs.push(sv);
        }
        return new CSSMathSum(negs);
      }
      case 'mul': return new CSSMathProduct(list);
      case 'div': {
        var invs = [list[0]];
        for (i2 = 1; i2 < list.length; i2++) {
          var dv = list[i2];
          if (dv instanceof CSSUnitValue && dv._zwOmUnit === 'number') dv = new CSSUnitValue(1 / dv._zwOmValue, 'number');
          else if (dv instanceof CSSMathValue && dv.operator === 'invert') dv = dv._zwOmValues[0];
          else dv = new CSSMathInvert(dv);
          invs.push(dv);
        }
        return new CSSMathProduct(invs);
      }
      case 'min': return new CSSMathMin(list);
      case 'max': return new CSSMathMax(list);
    }
    return self;
  }

  // ===== 11. 属性语法表（225 属性）=====
  // 语料 runPropertyTests 逐页提取（.acceptance/wptfyi-css/
  // typed-om-prop-table.js.txt 对账）；'<x>' = 数值/形态族，裸名 = keyword。
  var _zwOmPropSyntax = {
    'accent-color':['currentcolor','auto'],
    'alignment-baseline':['baseline','text-bottom','alphabetic','ideographic','middle','central','mathematical','text-top'],
    'all':[],
    'anchor-scope':['none','all','--a'],
    'animation-delay':['<time>'],
    'animation-direction':['normal','reverse','alternate-reverse','alternate'],
    'animation-duration':['<time>'],
    'animation-fill-mode':['none','forwards','backwards','both'],
    'animation-iteration-count':['infinite','<number>'],
    'animation-name':['none','custom-ident'],
    'animation-play-state':['running','paused'],
    'animation-timing-function':['linear','ease','ease-in','ease-out','ease-in-out','step-start','step-end'],
    'backdrop-filter':['none'],
    'backface-visibility':['visible','hidden'],
    'background':[],
    'background-attachment':['scroll','fixed','local'],
    'background-blend-mode':['normal','multiply','screen','overlay','darken','lighten','color-dodge','color-burn','hard-light','soft-light','difference','exclusion','hue','saturation','color','luminosity'],
    'background-clip':['border-box','padding-box','content-box'],
    'background-color':['currentcolor'],
    'background-image':['none','<image>'],
    'background-origin':['border-box','padding-box','content-box'],
    'background-position':['<position>'],
    'background-repeat':['repeat-x','repeat-y','repeat','space','round','no-repeat'],
    'background-size':['<length>','<percentage>','auto','cover','contain'],
    'baseline-shift':['bottom','center','top','sub','super','<percentage>','<length>'],
    'block-size':['auto','<percentage>','<length>'],
    'border-collapse':['separate','collapse'],
    'border-image-outset':['<length>','<number>'],
    'border-image-repeat':['stretch','repeat','round','space'],
    'border-image-slice':['<number>','<percentage>'],
    'border-image-source':['none','<image>'],
    'border-image-width':['<length>','<percentage>','<number>','auto'],
    'bottom':['auto','<percentage>','<length>'],
    'box-shadow':['none'],
    'box-sizing':['content-box','border-box'],
    // break-after/before 12 关键字面（css-break-3 §break-between；WPT
    // css-typed-om break 页 runPropertyTests 全集）。
    'break-after':['auto','avoid','avoid-column','avoid-page','avoid-region','column','left','page','recto','region','right','verso'],
    'break-before':['auto','avoid','avoid-column','avoid-page','avoid-region','column','left','page','recto','region','right','verso'],
    'break-inside':['auto','avoid','avoid-column','avoid-page','avoid-region'],
    'caption-side':['top','bottom'],
    'caret-color':['currentcolor','auto'],
    'clear':['none','left','right','both'],
    'clip':['auto'],
    'clip-path':['none','fill-box','stroke-box','view-box','margin-box','border-box','padding-box','content-box','<url>'],
    'clip-rule':['nonzero','evenodd'],
    'color':['currentcolor'],
    'color-interpolation':['auto','srgb','linearrgb'],
    'column-count':['auto','<number>'],
    'column-height':['auto','<length>'],
    // 刀 G（css-box-gap/css-align-3 §gap）：非负 <length>|<percentage> ∨
    // normal；负 % computed 期钳 0（_zwOmGridClampPctProps 同合同）。
    'column-gap':['normal','<length>','<percentage>'],
    'column-rule-color':['currentcolor'],
    'column-rule-style':['none','hidden','dotted','dashed','solid','double','groove','ridge','inset','outset'],
    'column-rule-width':['thin','medium','thick','<length>'],
    'column-span':['none','all'],
    'column-width':['auto','<length>'],
    'contain':['none','strict','content','size','layout','style','paint','inline-size'],
    'container-name':['none','my-container'],
    'container-type':['normal','size','inline-size'],
    'counter-increment':['none'],
    'counter-reset':['none'],
    'counter-set':['none'],
    'cursor':['auto','default','none','context-menu','help','pointer','progress','wait','cell','crosshair','text','vertical-text','alias','copy','move','no-drop','not-allowed','grab','grabbing','e-resize','n-resize','ne-resize','nw-resize','s-resize','se-resize','sw-resize','w-resize','ew-resize','ns-resize','nesw-resize','nwse-resize','col-resize','row-resize','all-scroll','zoom-in','zoom-out'],
    'cx':['<percentage>','<length>'],
    'cy':['<percentage>','<length>'],
    'd':['none'],
    'direction':['ltr','rtl'],
    'display':['none','block','inline','flow-root','table','flex','grid','list-item','table-row-group','table-header-group','table-footer-group','table-row','table-cell','table-column-group','table-column','table-caption','contents','inline-block','inline-table','inline-flex','inline-grid'],
    'dominant-baseline':['auto','text-bottom','alphabetic','ideographic','middle','central','mathematical','hanging','text-top'],
    'empty-cells':['show','hide'],
    'fill-color':['currentcolor'],
    'fill-opacity':['<number>','<percentage>'],
    'fill-rule':['nonzero','evenodd'],
    'filter':['none'],
    'flex-basis':['auto','content','fit-content','min-content','max-content','<length>','<percentage>'],
    'flex-direction':['row','row-reverse','column','column-reverse'],
    'flex-grow':['<number>'],
    'flex-shrink':['<number>'],
    'flex-wrap':['nowrap','wrap','wrap-reverse'],
    'float':['left','right','none'],
    'flood-color':['currentcolor'],
    'flood-opacity':['<number>','<percentage>'],
    'font-feature-settings':['normal'],
    'font-kerning':['auto','normal','none'],
    'font-language-override':['normal'],
    'font-optical-sizing':['auto','none'],
    'font-palette':['normal','light','dark'],
    'font-presentation':['auto','text','emoji'],
    'font-size':['xx-small','x-small','small','medium','large','x-large','xx-large','xxx-large','larger','smaller','<length>','<percentage>'],
    'font-size-adjust':['none','<number>'],
    'font-stretch':['normal','ultra-condensed','extra-condensed','condensed','semi-condensed','semi-expanded','expanded','extra-expanded','ultra-expanded','<percentage>'],
    // 刀 G（css-fonts-4 §font-synthesis）：简写三形 token 面走
    // _zwOmCheckShorthandTokens（同 background 形），长手 auto|none。
    'font-synthesis':[],
    'font-synthesis-small-caps':['auto','none'],
    'font-synthesis-style':['auto','none'],
    'font-synthesis-weight':['auto','none'],
    'font-style':['normal','italic','oblique'],
    'font-variant-alternates':['normal','historical-forms'],
    'font-variant-caps':['normal','small-caps','all-small-caps','petite-caps','all-petite-caps','unicase','titling-caps'],
    'font-variant-east-asian':['normal','jis78','jis83','jis90','jis04','simplified','traditional','full-width','proportional-width','ruby'],
    'font-variant-emoji':['normal','text','emoji','unicode'],
    'font-variant-ligatures':['normal','none','common-ligatures','no-common-ligatures','discretionary-ligatures','no-discretionary-ligatures','historical-ligatures','no-historical-ligatures','contextual','no-contextual'],
    'font-variant-numeric':['normal','lining-nums','oldstyle-nums','proportional-nums','tabular-nums','diagonal-fractions','stacked-fractions','ordinal','slashed-zero'],
    'font-variation-settings':['normal'],
    'font-weight':['normal','bold','bolder','lighter','<number>'],
    // grid-auto-columns/rows：auto track list 文法（css-grid-1 §7.4
    // <track-size>#；WPT css-typed-om grid-auto-columns-rows 页——minmax()/
    // fit-content() 形不在 typed 分型域，get 走 opaque 回落）。
    'grid-auto-columns':['auto','min-content','max-content','<length>','<percentage>','<flex>'],
    'grid-auto-rows':['auto','min-content','max-content','<length>','<percentage>','<flex>'],
    // 刀 TO-GRID face B（css-grid-2 §track-sizing）：template 轨道表——'none'
    // 关键字 + <length>/<percentage>/<flex> typed 分型（WPT grid-template-
    // columns-rows 页 runPropertyTests 全集）；line-name/repeat 组合走 opaque
    // 往返（set 侧 kind=opaque 恒放行 + CSSOM parse 期接受）。负 flex 由
    // _zwOmValidateSet 通用负 flex 门拒（表无 <negative-flex>）。
    'grid-template-columns':['none','<length>','<percentage>','<flex>'],
    'grid-template-rows':['none','<length>','<percentage>','<flex>'],
    'grid-auto-flow':['row','column'],
    // grid 行列放置长手（css-grid-2 §line-placement；WPT css-typed-om
    // grid-start-end 页仅 auto 入 typed 分型域，<grid-line> 其余形 opaque）。
    'grid-column-end':['auto'],
    'grid-column-start':['auto'],
    'grid-row-end':['auto'],
    'grid-row-start':['auto'],
    'grid-template-areas':['none'],
    'height':['auto','<percentage>','<length>'],
    'hyphens':['none','manual','auto'],
    'image-rendering':['auto','smooth','high-quality','crisp-edges','pixelated'],
    'inline-size':['auto','<percentage>','<length>'],
    'isolation':['auto','isolate'],
    'left':['auto','<percentage>','<length>'],
    'letter-spacing':['normal','<length>','<percentage>'],
    'lighting-color':['currentcolor'],
    'line-break':['auto','loose','normal','strict','anywhere'],
    'line-height':['normal','<length>','<number>','<percentage>'],
    'list-style-image':['none','<image>'],
    'list-style-position':['inside','outside'],
    'list-style-type':['none','custom-ident'],
    'margin':[],
    'mask-image':['none','<image>'],
    'mask-type':['luminance','alpha'],
    'max-block-size':['none','<percentage>','<length>'],
    'max-height':['none','<percentage>','<length>'],
    'max-inline-size':['none','<percentage>','<length>'],
    'max-width':['none','<percentage>','<length>'],
    'min-block-size':['<percentage>','<length>'],
    'min-height':['<percentage>','<length>'],
    'min-inline-size':['<percentage>','<length>'],
    'min-width':['<percentage>','<length>'],
    'mix-blend-mode':['normal','multiply','screen','overlay','darken','lighten','color-dodge','color-burn','hard-light','soft-light','difference','exclusion','hue','saturation','color','luminosity'],
    'object-fit':['fill','contain','cover','none','scale-down'],
    'object-position':['<position>'],
    'offset-anchor':['auto','<position>'],
    'offset-distance':['<length>','<percentage>'],
    'offset-path':['none'],
    'offset-position':['normal','auto','<position>'],
    'offset-rotate':['auto','reverse','<angle>'],
    'opacity':['<number>','<percentage>'],
    'order':['<number>'],
    'orphans':['<number>'],
    'outline-color':['currentcolor'],
    'outline-offset':['<length>'],
    'outline-style':['auto','none','dotted','dashed','solid','double','groove','ridge','inset','outset'],
    'outline-width':['thin','medium','thick','<length>'],
    'overflow-anchor':['auto','none'],
    'overflow-clip-margin':['border-box','content-box','padding-box','<length>'],
    'overflow-wrap':['normal','break-word','break-spaces'],
    // 刀 G（css-overflow-3 §overflow-control + css-overscroll-1
    // §overscroll-behavior-properties）：longhand 关键字面；overflow 简写
    // 既有 KnownExtra 不动。
    'overflow-x':['visible','hidden','clip','scroll','auto'],
    'overflow-y':['visible','hidden','clip','scroll','auto'],
    'overscroll-behavior-x':['auto','contain','none'],
    'overscroll-behavior-y':['auto','contain','none'],
    'page':['auto','custom-ident'],
    'paint-order':['normal','fill','stroke','markers'],
    'perspective':['none','<length>'],
    'perspective-origin':['none','<position>'],
    'pointer-events':['bounding-box','visiblepainted','visiblefill','visiblestroke','visible','painted','fill','stroke','all','none'],
    'position':['relative','absolute'],
    'position-visibility':['always','anchors-valid','anchors-visible','no-overflow'],
    'quotes':['none'],
    'r':['<percentage>','<length>'],
    'resize':['none','both','horizontal','vertical'],
    'right':['auto','<percentage>','<length>'],
    // 刀 G：row-gap 与 column-gap 同合同（css-box-gap）。
    'row-gap':['normal','<length>','<percentage>'],
    'rx':['auto','<percentage>','<length>'],
    'ry':['auto','<percentage>','<length>'],
    'scroll-behavior':['auto','smooth'],
    'scroll-snap-align':['none','start','end','center'],
    'scroll-snap-stop':['normal','always'],
    'scroll-snap-type':['none','x','y','block','inline','both'],
    'scrollbar-gutter':['auto','stable'],
    'scrollbar-width':['auto','thin','none'],
    'shape-image-threshold':['<number>'],
    'shape-margin':['<length>','<percentage>'],
    'shape-outside':['none','margin-box','border-box','padding-box','content-box','<image>'],
    'shape-rendering':['auto','optimizespeed','crispedges','geometricprecision'],
    'speak':['auto','never','always'],
    'stop-color':['currentcolor'],
    'stop-opacity':['<number>','<percentage>'],
    'stroke-dasharray':['none','<length>','<percentage>','<number>'],
    'stroke-dashoffset':['<length>','<percentage>','<number>'],
    'stroke-linecap':['butt','round','square'],
    'stroke-linejoin':['crop','arcs','miter','bevel','round','stupid'],
    'stroke-miterlimit':['<number>'],
    'stroke-opacity':['<number>','<percentage>'],
    'stroke-width':['<length>','<percentage>','<number>'],
    'tab-size':['<number>','<length>'],
    'table-layout':['auto','fixed'],
    'text-align':['start','end','left','right','center','justify'],
    'text-align-last':['auto','start','end','left','right','center','justify'],
    'text-anchor':['start','middle','end'],
    'text-autospace':['normal','no-autospace','ideograph-alpha','ideograph-numeric','insert','auto'],
    'text-box-trim':['none','trim-both','trim-start','trim-end'],
    'text-combine-upright':['none','all'],
    'text-decoration-color':['currentcolor'],
    'text-decoration-line':['none','underline','overline','line-through','blink','spelling-error','grammar-error'],
    'text-decoration-skip':['none','objects','edges','box-decoration','spaces'],
    'text-decoration-skip-ink':['auto','none'],
    'text-decoration-style':['solid','double','dotted','dashed','wavy'],
    'text-decoration-thickness':['auto','<length>','<percentage>'],
    'text-emphasis-color':['currentcolor'],
    'text-emphasis-position':['auto','over','under'],
    'text-indent':['<length>','<percentage>'],
    'text-justify':['auto','none','inter-word','inter-character'],
    'text-orientation':['mixed','upright','sideways'],
    'text-overflow':['clip','ellipsis','fade'],
    'text-rendering':['auto','optimizespeed','optimizelegibility','geometricprecision'],
    'text-shadow':['none'],
    'text-size-adjust':['none','auto','<percentage>'],
    'text-transform':['none','capitalize','uppercase','lowercase','full-width'],
    'text-underline-offset':['auto','<length>','<percentage>'],
    'text-underline-position':['auto','under','left','right'],
    'text-wrap-mode':['wrap','nowrap'],
    'text-wrap-style':['auto','balance','stable','pretty'],
    'top':['auto','<percentage>','<length>'],
    'touch-action':['auto','none','pan-x','pan-left','pan-right','pan-y','pan-up','pan-down','pinch-zoom','manipulation'],
    'transform':['none','<transform>'],
    'transform-box':['border-box','fill-box','view-box'],
    'transform-style':['auto','flat','preserve-3d'],
    'transition-delay':['<time>'],
    'transition-duration':['<time>'],
    'transition-property':['none'],
    'transition-timing-function':['linear','ease','ease-in','ease-out','ease-in-out','step-start','step-end'],
    'unicode-bidi':['normal','embed','isolate','bidi-override','isolate-override','plaintext'],
    'user-select':['auto','text','none','contain','all'],
    'vector-effect':['non-scaling-stroke','none'],
    'visibility':['visible','hidden','collapse'],
    'white-space-collapse':['collapse','discard','preserve','preserve-breaks','preserve-spaces','break-spaces'],
    'white-space-trim':['none','discard-before','discard-after','discard-inner'],
    'widows':['<number>'],
    'width':['auto','<percentage>','<length>'],
    'will-change':['auto'],
    'word-break':['normal','keep-all','break-all'],
    'word-spacing':['normal','<length>','<percentage>'],
    'word-wrap':['normal','break-word','break-spaces'],
    'writing-mode':['horizontal-tb','vertical-rl','vertical-lr','sideways-rl','sideways-lr'],
    'x':['<percentage>','<length>'],
    'y':['<percentage>','<length>'],
    'z-index':['auto','<number>'],
    // 刀 P（css-typed-om-1 §set 属性面扩维）：物理长手 + 逻辑属性
    //（css-logical-1 §logical-prop）+ scroll 长手（css-scroll-snap-3
    // §scroll-padding）——logical.html 46 属性与 margin/padding/border-*/
    // scroll-* 简写页长手 runPropertyTests 全形此前落 SET known 门
    //（"Unsupported property"）；语法族逐页 testCases 精确镜像（页未测
    // 文法不入表——invalid-rejection 子测靠拒收保持）。
    'margin-top':['auto','<percentage>','<length>'],
    'margin-right':['auto','<percentage>','<length>'],
    'margin-bottom':['auto','<percentage>','<length>'],
    'margin-left':['auto','<percentage>','<length>'],
    'margin-block-start':['<percentage>','<length>'],
    'margin-block-end':['<percentage>','<length>'],
    'margin-inline-start':['<percentage>','<length>'],
    'margin-inline-end':['<percentage>','<length>'],
    'margin-block':['<percentage>','<length>'],
    'margin-inline':['<percentage>','<length>'],
    // 刀 OM-1：marker-* 长手入表（css-masking-1 §marker-*，语料
    // marker.html runPropertyTests 18F 面——none keyword + <url> 形；
    // 简写 marker 不入表保持 opaque 往返面）。
    'marker-start':['none','<url>'],
    'marker-mid':['none','<url>'],
    'marker-end':['none','<url>'],
    'padding-top':['<percentage>','<length>'],
    'padding-right':['<percentage>','<length>'],
    'padding-bottom':['<percentage>','<length>'],
    'padding-left':['<percentage>','<length>'],
    'padding-block-start':['<percentage>','<length>'],
    'padding-block-end':['<percentage>','<length>'],
    'padding-inline-start':['<percentage>','<length>'],
    'padding-inline-end':['<percentage>','<length>'],
    'padding-block':['<percentage>','<length>'],
    'padding-inline':['<percentage>','<length>'],
    'inset-block-start':['<percentage>','<length>'],
    'inset-block-end':['<percentage>','<length>'],
    'inset-inline-start':['<percentage>','<length>'],
    'inset-inline-end':['<percentage>','<length>'],
    'inset-block':['<percentage>','<length>'],
    'inset-inline':['<percentage>','<length>'],
    'border-top-width':['thin','medium','thick','<length>'],
    'border-right-width':['thin','medium','thick','<length>'],
    'border-bottom-width':['thin','medium','thick','<length>'],
    'border-left-width':['thin','medium','thick','<length>'],
    'border-block-start-width':['thin','medium','thick','<length>'],
    'border-block-end-width':['thin','medium','thick','<length>'],
    'border-inline-start-width':['thin','medium','thick','<length>'],
    'border-inline-end-width':['thin','medium','thick','<length>'],
    'border-block-width':['thin','medium','thick','<length>'],
    'border-inline-width':['thin','medium','thick','<length>'],
    'border-top-color':['currentcolor'],
    'border-right-color':['currentcolor'],
    'border-bottom-color':['currentcolor'],
    'border-left-color':['currentcolor'],
    'border-block-start-color':['currentcolor'],
    'border-block-end-color':['currentcolor'],
    'border-inline-start-color':['currentcolor'],
    'border-inline-end-color':['currentcolor'],
    'border-block-color':['currentcolor'],
    'border-inline-color':['currentcolor'],
    'border-top-style':['none','solid'],
    'border-right-style':['none','solid'],
    'border-bottom-style':['none','solid'],
    'border-left-style':['none','solid'],
    'border-block-start-style':['none','solid'],
    'border-block-end-style':['none','solid'],
    'border-inline-start-style':['none','solid'],
    'border-inline-end-style':['none','solid'],
    'border-block-style':['none','solid'],
    'border-inline-style':['none','solid'],
    // border-radius 简写：空表 opaque 往返（border-radius.html
    // runUnsupportedPropertyTests——set 收基类实例、get 基类 CSSStyleValue、
    // 文本逐字往返；'25% 10%'/'10% / 50%' 双值对非 Typed OM L1 形态，
    // css-typed-om-1 §get 回落形态）。
    'border-radius':[],
    'border-top-left-radius':['<percentage>','<length>'],
    'border-top-right-radius':['<percentage>','<length>'],
    'border-bottom-left-radius':['<percentage>','<length>'],
    'border-bottom-right-radius':['<percentage>','<length>'],
    'border-start-start-radius':['<percentage>','<length>'],
    'border-start-end-radius':['<percentage>','<length>'],
    'border-end-start-radius':['<percentage>','<length>'],
    'border-end-end-radius':['<percentage>','<length>'],
    'border-block-start':['none'],
    'border-block-end':['none'],
    'border-inline-start':['none'],
    'border-inline-end':['none'],
    'border-block':['none'],
    'border-inline':['none'],
    'scroll-margin-top':['<length>'],
    'scroll-margin-right':['<length>'],
    'scroll-margin-bottom':['<length>'],
    'scroll-margin-left':['<length>'],
    'scroll-margin-inline-start':['<length>'],
    'scroll-margin-block-start':['<length>'],
    'scroll-margin-inline-end':['<length>'],
    'scroll-margin-block-end':['<length>'],
    'scroll-padding-top':['<percentage>','<length>'],
    'scroll-padding-right':['<percentage>','<length>'],
    'scroll-padding-bottom':['<percentage>','<length>'],
    'scroll-padding-left':['<percentage>','<length>'],
    'scroll-padding-inline-start':['<percentage>','<length>'],
    'scroll-padding-block-start':['<percentage>','<length>'],
    'scroll-padding-inline-end':['<percentage>','<length>'],
    'scroll-padding-block-end':['<percentage>','<length>']
  };
  // 已知属性扩面（opaque roundtrip set 面；语料 225 之外常见属性）。
  // 刀 TO-GRID face B：grid-template-columns/rows 升级白名单表（typed 分型面
  // ——WPT grid-template-columns-rows 页 CSSUnitValue/CSSKeywordValue 期望）；
  // scroll-padding 简写入集（testUnsupportedValue opaque 往返面——多分量值
  // Chrome 同形不解析）。
  var _zwOmKnownExtra = { color: 1, display: 1, background: 1, 'background-color': 1, 'background-image': 1, opacity: 1, visibility: 1, content: 1, transform: 1, 'z-index': 1, overflow: 1, cursor: 1, outline: 1, flex: 1, 'flex-direction': 1, 'scroll-padding': 1, 'scroll-margin': 1, transition: 1, animation: 1,
    // 刀 G opaque 兜底面（testUnsupportedPropertyTests：computed 往返期望
    // 基类 opaque——PropSyntax 优先命中，本表仅兜底；简写族 computed 多值
    // 形 Typed OM L1 无分型）。
    'marker': 1, 'grid-area': 1, 'offset': 1, 'grid-template': 1,
    'grid-gap': 1, 'font': 1, 'font-variant': 1, 'text-decoration': 1,
    'grid': 1, 'font-family': 1, 'flex-flow': 1, 'stroke': 1, 'mask': 1,
    'fill': 1, 'vertical-align': 1 };

  // 刀 OM-1：marker 长手族（computed 兜底判定用——css-masking-1 §marker-*）
  var _zwOmMarkerProps = { 'marker-start': 1, 'marker-mid': 1, 'marker-end': 1 };

  // ===== 12. StylePropertyMap =====
  // A/B 快照二进制 provenance 指纹（zw-typedom-listvalued-2026-11-17-r2）：刀 H
  // Typed OM 校验面轮 r2（var() 引用 get 归一 + 折叠占位接受）的源侧标记
  // ——strings 抽验新 shim 装载（include_str 全量入二进制）。
  function _zwOmValueToText(v) {
    if (typeof v === 'string') return v.trim();
    if (v instanceof CSSStyleValue) return v.toString();
    if (v && typeof v.toString === 'function' && v.toString !== Object.prototype.toString) return String(v);
    if (typeof v === 'number') return String(v);
    throw new TypeError("Invalid value");
  }
  function _zwOmValidateSet(prop, value) {
    // 值裁决步关联属性错配门（css-typed-om-1 §set：携关联属性的基类实例
    // 不得跨属性并置——语料 'Setting a property with a CSSStyleValue whose
    // associated property does not match' 面）。
    if (value instanceof CSSStyleValue && value._zwOmAssociatedProp !== undefined && value._zwOmAssociatedProp !== prop) {
      throw new TypeError("Cannot set a CSSStyleValue with a different associated property");
    }
    var kind = (value instanceof CSSStyleValue) ? value._zwOmKind() : (typeof value === 'string' ? 'text' : 'other');
    if (kind === 'other' && value && typeof value.toString === 'function' && value.toString !== Object.prototype.toString) {
      // 外部鸭子形态（part05 CSSRGB/CSSHSL 等）：有自有 ToString 按 opaque 放行。
      kind = 'foreign';
    }
    if (kind === 'other') throw new TypeError("Invalid value");
    var known = Object.prototype.hasOwnProperty.call(_zwOmPropSyntax, prop) || _zwOmKnownExtra[prop] === 1 || (typeof prop === 'string' && prop.slice(0, 2) === '--');
    if (!known) throw new TypeError("Unsupported property");
    var table = _zwOmPropSyntax[prop];
    // 简写字符串面逐 token 校验（margin '10s' → TypeError 语料面）；须在
    // text 早退之前裁决，否则为死分支。
    if (kind === 'text' && table && table.length === 0) {
      _zwOmCheckShorthandTokens(value);
      return;
    }
    if (kind === 'text') {
      // list-valued string 门（§set 值解析步：非 list-valued 属性值不得含
      // 顶层逗号——语料 'Setting a non list-valued property with list-valued
      // string' 面）+ 单 token 数值形客观族裁决（'10s'@width 面）。自定义属性
      // 豁免（spec 自定义属性任意串合法——css-properties-values-api-1
      // typedom.html 'Specified string "10px, 11px" accepted by set() for
      // syntax <length>#' 刀 TD1 面）。
      var segsV = _zwOmTopSplit(String(value).trim(), ',');
      if (segsV.length > 1 && !_zwOmIsListValued(prop) && prop.slice(0, 2) !== '--') throw new TypeError("Cannot set a list of values on a non list-valued property");
      for (var vi = 0; vi < segsV.length; vi++) {
        var segT = segsV[vi].trim();
        if (segT === '') throw new TypeError("Invalid value");
        if (/^[^\s(,]+$/.test(segT)) _zwOmValidateNumericText(prop, segT);
      }
      return;
    }
    if (kind === 'unparsed' || kind === 'varref') return;
    if (prop.slice(0, 2) === '--') {
      // css-typed-om-1 §set 值解析步：自定义属性仅受理 string/CSSUnparsedValue
      //（含 var 引用）——分型 CSSStyleValue（keyword/unit/math/image/transform/
      // opaque 基类）恒 TypeError（css-properties-values-api-1 typedom.html
      // 'CSSUnitValue rejected by set()' 族，刀 TD1）。
      throw new TypeError("Only strings and CSSUnparsedValues are accepted for custom properties");
    }
    if (kind === 'opaque' || kind === 'foreign' || kind === 'color') return;
    if (!table) return; // 已知但无表：opaque/keyword 面放行（css-wide 面已过）
    var t = value;
    // css-wide keyword 恒可。
    if (kind === 'keyword' && _zwOmCssWide[String(t.value).toLowerCase()]) return;
    var fams = {};
    for (var i = 0; i < table.length; i++) if (table[i].charAt(0) === '<') fams[table[i]] = 1;
    var hasCustomIdent = table.indexOf('<custom-ident>') >= 0;
    if (kind === 'keyword') {
      var kw = String(t.value).toLowerCase();
      if (hasCustomIdent || table.indexOf(kw) >= 0 || table.indexOf(String(t.value)) >= 0) return;
      throw new TypeError("Invalid keyword for property");
    }
    if (kind === 'unit') {
      // 刀 G2-a：famName 从 unit 名直取（_zwOmUnitFamily 族表 [0]）——旧法
      // _zwOmUnitType map 迭代（fam[d]===1）在 ARITH 刀零指数键省略形状下对
      // number 产 {} → famName=null → want=[] 恒 throw（css-properties-values-
      // api-1 'to a number' 断言面全灭：opacity/flex-grow/flex-shrink/order/
      // column-count/line-height/border-image-*/fill|flood|stop|stroke-opacity
      // 等 <number> 族页；HEAD 考古 pre-face 同炸＝旧债非 ARITH 回归）。
      var uinfo = _zwOmUnitInfo(t.unit);
      var famName = uinfo ? uinfo[0] : null;
      // 负 flex 值门（css-grid-1 §7.2.3 <flex> 负值非法——track sizing fr
      // breadth 非负；testsuite <negative-flex> 面期望 CSSUnitValue(-3,'fr')
      // set 恒 TypeError，表中无 <negative-flex> 语法即拒）。
      if (famName === 'flex' && t.value < 0 && !fams['<negative-flex>']) throw new TypeError("Invalid negative flex for property");
      var map = { length: ['<length>', '<length-percentage>'], percent: ['<percentage>', '<length-percentage>'], angle: ['<angle>'], time: ['<time>'], frequency: ['<frequency>'], resolution: ['<resolution>'], flex: ['<flex>'], number: ['<number>', '<integer>'] };
      var want = map[famName] || [];
      for (var j = 0; j < want.length; j++) if (fams[want[j]]) return;
      throw new TypeError("Invalid unit for property");
    }
    if (kind === 'math') {
      var ok = _zwOmValidateMath(t, fams);
      if (!ok) throw new TypeError("Invalid math value for property");
      return;
    }
    if (kind === 'image') { if (fams['<image>'] || fams['<url>']) return; throw new TypeError("Invalid image for property"); }
    if (kind === 'transform') { if (fams['<transform>'] || fams['<transform-list>']) return; throw new TypeError("Invalid transform for property"); }
    return;
  }
  function _zwOmDeclTextFromValue(prop, values, sep) {
    // list-valued 属性 set/append 顶层逗号并置（css-typed-om-1 §set/§append
    // 列表值串形——'1s, 2s' 语料面）；其余单空格并置。
    var texts = [];
    for (var i = 0; i < values.length; i++) texts.push(_zwOmValueToText(values[i]));
    return texts.join(sep || ' ');
  }
  // 非负属性集（declared/computed association 负值按 spec 包 CSSMathSum——
  // 语料面 assert_is_equal_with_range_handling 期望；清单自 36 个 range-
  // handling 页的 runPropertyTests 属性名派生）。
  var _zwOmNonNegativeProps = {
    'animation-iteration-count': 1, 'background-size': 1, 'block-size': 1,
    'border-block-end-width': 1, 'border-block-start-width': 1, 'border-bottom-left-radius': 1,
    'border-bottom-right-radius': 1, 'border-bottom-width': 1, 'border-image-outset': 1,
    'border-image-slice': 1, 'border-image-width': 1, 'border-inline-end-width': 1,
    'border-inline-start-width': 1, 'border-left-width': 1, 'border-radius': 1,
    'border-right-width': 1, 'border-top-left-radius': 1, 'border-top-right-radius': 1,
    'border-top-width': 1, 'border-width': 1, 'column-gap': 1, 'column-height': 1,
    'column-rule-width': 1, 'column-width': 1, 'flex-basis': 1, 'flex-grow': 1,
    'flex-shrink': 1, 'font-size': 1, 'font-size-adjust': 1, 'font-stretch': 1,
    // 刀 P：轴简写宽度与逻辑 corner radius 入集（logical.html/border-width.html
    // range-handling 期望负值包 Sum）；inset-*/scroll-padding-* 长手剔除——
    // 负值合法（css-logical-1 inset / css-scroll-snap-3 允负形不存在但语料
    // 页 default-equal 期望原文），包 Sum 反断该两页负值面。
    'border-block-width': 1, 'border-inline-width': 1,
    'border-start-start-radius': 1, 'border-start-end-radius': 1,
    'border-end-start-radius': 1, 'border-end-end-radius': 1,
    gap: 1, 'grid-auto-columns': 1, 'grid-auto-rows': 1, 'grid-template-columns': 1,
    'grid-template-rows': 1, height: 1, 'inline-size': 1, 'line-height': 1,
    'max-block-size': 1, 'max-height': 1, 'max-inline-size': 1, 'max-width': 1,
    'min-block-size': 1, 'min-height': 1, 'min-inline-size': 1, 'min-width': 1,
    'outline-width': 1, 'overflow-clip-margin': 1, padding: 1, 'padding-block': 1,
    'padding-block-end': 1, 'padding-block-start': 1, 'padding-bottom': 1,
    'padding-inline': 1, 'padding-inline-end': 1, 'padding-inline-start': 1,
    'padding-left': 1, 'padding-right': 1, 'padding-top': 1, perspective: 1,
    r: 1, 'row-gap': 1, rx: 1, ry: 1, 'scroll-padding': 1,
    'scroll-padding-block': 1, 'scroll-padding-block-end': 1,
    'scroll-padding-block-start': 1, 'scroll-padding-bottom': 1,
    'scroll-padding-inline': 1, 'scroll-padding-inline-end': 1,
    'scroll-padding-inline-start': 1, 'scroll-padding-left': 1,
    'scroll-padding-right': 1, 'scroll-padding-top': 1, 'shape-margin': 1,
    'stroke-dasharray': 1, 'stroke-miterlimit': 1, 'stroke-width': 1,
    'tab-size': 1, width: 1,
  };
  // 刀 OM-11：整数族 computed 面声明文本源集 + 取整/钳 1 合同。orphans/
  // widows/column-count 值域 <integer> [1,∞]（css-break-3 §orphans/§widows、
  // css-multicol-1 §column-count），order 全值域 <integer>（css-flexbox-1
  // §order）。declared 面非值域/非整数包 Sum（testsuite specified 臂
  // expected CSSMathSum——b1 失败消息实证）；computed 面四舍五入取整
  // （css-values-4 §<integer> 计算值 ties +∞，Math.round 同判）+ 下限钳 1
  // （orphans/widows/column-count computed 臂 expected Unit 形；探针
  // zz-om11probe G2/G3/G4 实证引擎计算串分数 order→0、column-count→'auto'
  // 有损，须从声明文本 reify）。
  var _zwOmIntProps = {
    orphans: 1, widows: 1, 'column-count': 1, order: 1,
  };
  var _zwOmIntMin1Props = { orphans: 1, widows: 1, 'column-count': 1 };
  // 刀 OM-15：sizing 槽 calc-size() computed 串化回显槽集（css-values-5 §8
  // `calc-size()` 计算值保形——清单镜像写面文法门 calc_size_decl_gate 尺寸槽）。
  var _zwOmCalcSizeSlots = {
    width: 1, height: 1, 'min-width': 1, 'max-width': 1,
    'min-height': 1, 'max-height': 1, 'block-size': 1, 'inline-size': 1,
    'min-block-size': 1, 'min-inline-size': 1, 'max-block-size': 1,
    'max-inline-size': 1, 'flex-basis': 1,
  };
  // 刀 OM-15：sizing 槽 calc-size() computed 串化（css-values-5 §8 `calc-size()`
  // 计算值＝calc-size 结构保形：basis 关键词/嵌套递归保形、% 与 `size` 词元
  // 保留、长度绝对化 px（em/rem 按字号）、纯 px 叶 math 求值、除法源积项
  // 括号化）。declared 通道（CALCSIZE-CANON）已产出 canonical 骨架（积项序/
  // 括号内项/2in→192px），本函数补 computed 形差异三面：
  //   a) 相对长度绝对化（declared 保 em 形——计算值按字号折 px，探针
  //      zz-om15probe 25em→400px）；
  //   b) 纯 px 叶 min/max 求值（declared 保函数形——计算值取值序，探针
  //      min(30px, 2em)→30px）；
  //   c) 除法源积项首位括号（declared 首位 `0.5 * size` 裸——computed 恒
  //      括号，语料 calc-size-typed-om 期望 `(0.5 * size)`；整数因子积项
  //      `2 * size` 不括）。
  // FIXME: 除法源识别用小数因子启发（declared 串化已失 `size / 2` 结构源），
  // 字面 `0.5 * size` 会被误括——结构源旗标待 declared 串化补齐后换真判。
  // 结构不锚（非单一顶层 calc-size 形/var() 引用）或 em 无字号基时 fail-closed
  // null（调用方回落引擎计算串——零漂移）。
  function _zwOmCalcSizeAbsolutize(tok, fontPx) {
    var m = /^([+-]?[0-9.]+(?:[eE][+-]?[0-9]+)?)([a-z%]+)$/i.exec(tok);
    if (!m) return null;
    var u = m[2].toLowerCase();
    if (u === '%') return m[1] + '%';
    var v = parseFloat(m[1]);
    if (u === 'px') return _zwOmNumStr(v) + 'px';
    var px = null;
    if (u === 'em' || u === 'rem') {
      if (!(fontPx > 0)) return null;
      px = v * fontPx;
    } else if (u === 'in') px = v * 96;
    else if (u === 'cm') px = (v * 96) / 2.54;
    else if (u === 'mm') px = (v * 96) / 25.4;
    else if (u === 'q') px = (v * 96) / 101.6;
    else if (u === 'pt') px = (v * 96) / 72;
    else if (u === 'pc') px = v * 12;
    else return null; // ex/ch 字体度量依存 fail-closed（保形面外溢出不入）
    return _zwOmNumStr(parseFloat(px.toPrecision(12))) + 'px';
  }
  function _zwOmCalcSizeCanonicalTerm(term, fontPx) {
    var t = term.trim();
    if (t.charAt(0) === '(' && t.charAt(t.length - 1) === ')') return t;
    // 纯 px 叶 min/max/abs 求值（css-values-4 §9 计算值 math 求值）；size
    // 依存/% 混叶不求保形（sign(size) 语料形）。
    var fm = /^(min|max|abs)\(([^()]*)\)$/i.exec(t);
    if (fm) {
      var args = fm[2].split(',');
      var pxs = [];
      for (var k = 0; k < args.length; k++) {
        var ap = _zwOmCalcSizeAbsolutize(args[k].trim(), fontPx);
        if (ap === null || ap.indexOf('%') >= 0) { pxs = null; break; }
        pxs.push(parseFloat(ap));
      }
      if (pxs !== null && pxs.length > 0) {
        var fn = fm[1].toLowerCase();
        if (fn === 'min') return _zwOmNumStr(Math.min.apply(null, pxs)) + 'px';
        if (fn === 'max') return _zwOmNumStr(Math.max.apply(null, pxs)) + 'px';
        return _zwOmNumStr(Math.abs(pxs[0])) + 'px';
      }
      return t;
    }
    // 除法源积项启发括号（头注 FIXME）；`sign(size) * size` 前导非数字不入。
    var pm = /^([0-9.]+)\s*\*\s*(.+)$/.exec(t);
    if (pm && pm[1].indexOf('.') >= 0) return '(' + t + ')';
    var ab = _zwOmCalcSizeAbsolutize(t, fontPx);
    return ab !== null ? ab : t;
  }
  function _zwOmCalcSizeCanonicalSum(sum, fontPx) {
    // 顶层 term 拆分：深度 0 且两侧空白的 +/-（css-values-4 §calc-sum term 界）
    var terms = [];
    var buf = '';
    var depth = 0;
    for (var i = 0; i < sum.length; i++) {
      var ch = sum.charAt(i);
      if (ch === '(') depth++;
      else if (ch === ')') depth--;
      if (depth === 0 && (ch === '+' || ch === '-') && i > 0
        && /\s/.test(sum.charAt(i - 1)) && /\s/.test(sum.charAt(i + 1))) {
        if (buf.trim() === '') return null;
        terms.push(buf.trim());
        buf = '';
        continue;
      }
      buf += ch;
    }
    if (buf.trim() === '') return null;
    terms.push(buf.trim());
    for (var j = 0; j < terms.length; j++) terms[j] = _zwOmCalcSizeCanonicalTerm(terms[j], fontPx);
    // 稳定排序：% 项前置（canonical 序；declared 已序则恒等）
    var idx = [];
    for (var q = 0; q < terms.length; q++) idx.push(q);
    idx.sort(function (a, b) {
      var pa = terms[a].indexOf('%') >= 0 ? 0 : 1;
      var pb = terms[b].indexOf('%') >= 0 ? 0 : 1;
      return pa - pb;
    });
    var out = [];
    for (var w = 0; w < idx.length; w++) out.push(terms[idx[w]]);
    return out.join(' + ');
  }
  function _zwOmCalcSizeCanonicalBasis(basis, fontPx) {
    var lb = basis.toLowerCase();
    if (lb === 'auto' || lb === 'min-content' || lb === 'max-content'
      || lb === 'fit-content' || lb === 'stretch' || lb === 'any' || lb === 'content') {
      return lb;
    }
    // 嵌套 calc-size（唯一合法函数形 basis）递归
    if (/^calc-size\(/i.test(basis)) return _zwOmCalcSizeComputedForm(basis, fontPx);
    return _zwOmCalcSizeAbsolutize(basis, fontPx);
  }
  function _zwOmCalcSizeComputedForm(text, fontPx) {
    var t = text.trim();
    if (!/^calc-size\(/i.test(t) || t.charAt(t.length - 1) !== ')') return null;
    var inner = t.slice('calc-size('.length, t.length - 1);
    var ci = _zwOmTopLevelComma(inner);
    if (ci < 0) return null;
    var basis = _zwOmCalcSizeCanonicalBasis(inner.slice(0, ci).trim(), fontPx);
    var sum = _zwOmCalcSizeCanonicalSum(inner.slice(ci + 1).trim(), fontPx);
    if (basis === null || sum === null) return null;
    return 'calc-size(' + basis + ', ' + sum + ')';
  }

  // list-valued 属性集（css-typed-om-1 §append/set：仅 list-valued 属性可
  // append 且 set 多值——文法顶层逗号重复族；清单取 common comma-repeated
  // families，语料 set/append 面以 transition-duration 为代表靶）。简写属性
  //（margin/background/transition 等）不入集——append 恒 TypeError。
  var _zwOmListValuedProps = {
    'animation-composition': 1, 'animation-delay': 1, 'animation-direction': 1,
    'animation-duration': 1, 'animation-fill-mode': 1,
    'animation-iteration-count': 1, 'animation-name': 1,
    'animation-play-state': 1, 'animation-range-end': 1,
    'animation-range-start': 1, 'animation-timeline': 1,
    'animation-timing-function': 1,
    'background-attachment': 1, 'background-blend-mode': 1,
    'background-clip': 1, 'background-image': 1, 'background-origin': 1,
    'background-position': 1, 'background-repeat': 1, 'background-size': 1,
    'box-shadow': 1, 'font-family': 1, 'grid-auto-columns': 1, 'grid-auto-rows': 1,
    'mask-composite': 1, 'mask-image': 1,
    'mask-mode': 1, 'mask-origin': 1, 'mask-position': 1, 'mask-repeat': 1,
    'mask-size': 1, 'scroll-margin-block': 1, 'scroll-margin-inline': 1,
    'scroll-padding-block': 1, 'scroll-padding-inline': 1,
    'text-shadow': 1, 'transition-behavior': 1, 'transition-delay': 1,
    'transition-duration': 1, 'transition-property': 1,
    'transition-timing-function': 1, 'will-change': 1,
  };
  function _zwOmIsListValued(p) {
    return p.slice(0, 2) !== '--' && _zwOmListValuedProps[p] === 1;
  }
  // 注册表 syntax 查询（刀 TD1：host `__zw_registered_property_syntax`；未注册
  // 返空串。engine 侧 thread_local 注册表，CSS.registerProperty 与 @property
  // 规则同表）。
  function _zwOmRegisteredSyntaxOf(p) {
    if (typeof globalThis.__zw_registered_property_syntax !== 'function') return '';
    try { return String(globalThis.__zw_registered_property_syntax(p) || ''); } catch (_eRS) { return ''; }
  }
  // CSSUnparsedValue/var() 引用形（css-typed-om-1 §set 值裁决步：var 引用值
  // 不得与其他值并置——append 恒拒，set 仅单值）。
  function _zwOmIsUnparsedLike(v) {
    if (v instanceof CSSUnparsedValue || v instanceof CSSVariableReferenceValue) return true;
    return typeof v === 'string' && /(^|[^\w-])var\(/i.test(v);
  }
  // 数值 token 客观族裁决（invalid String 语料面 '10s'@width / '10px'@
  // transition-duration）——仅数值形单 token 拒绝；keyword/函数/多 token 段
  // 保持放行（语法表词汇覆盖面外，避免 min-content 等合法 keyword 误拒）。
  // 无单位数字：number/integer 族可；length/percentage/angle 族收 unitless
  // zero（css-values-4 §5 unitless-zero 面宽限）。
  function _zwOmValidateNumericText(p, text) {
    var table = _zwOmPropSyntax[p];
    if (!table || table.length === 0) return; // 简写/无表：既有 token 门自理
    var fams = {};
    for (var i = 0; i < table.length; i++) if (table[i].charAt(0) === '<') fams[table[i]] = 1;
    var um = /^([-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?)([a-zA-Z%]+)$/.exec(text);
    if (um) {
      var u = um[2].toLowerCase();
      var inf = u === '%' ? ['percent'] : _zwOmUnitInfo(u);
      var famName = inf ? inf[0] : null;
      var want = {
        length: ['<length>', '<length-percentage>'],
        percent: ['<percentage>', '<length-percentage>'],
        angle: ['<angle>'], time: ['<time>'], frequency: ['<frequency>'],
        resolution: ['<resolution>'], flex: ['<flex>'],
      }[famName] || [];
      for (var j = 0; j < want.length; j++) if (fams[want[j]]) return;
      throw new TypeError("Invalid unit for property");
    }
    if (/^[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?$/.exec(text)) {
      if (fams['<number>'] || fams['<integer>']) return;
      if (fams['<length>'] || fams['<length-percentage>'] || fams['<percentage>'] || fams['<angle>']) return;
      throw new TypeError("Invalid number for property");
    }
  }
  // 刀 TD1：registered custom property computed 文本 → 分型值列表（css-
  // properties-values-api-1 §reification）。syntax 乘子分段（`#` 顶层逗号 /
  // `+` 空白），逐段过分型 arm；任一段无 arm 受理返 null（通用 unparsed 面）。
  function _zwOmReifyRegisteredComputed(syntax, text, prop) {
    var comps = String(syntax).split('|');
    var bases = [];
    var anyHash = false, anyPlus = false;
    for (var i = 0; i < comps.length; i++) {
      var c = comps[i].trim();
      if (c === '') continue;
      if (c.charAt(c.length - 1) === '#') { anyHash = true; c = c.slice(0, -1); }
      else if (c.charAt(c.length - 1) === '+') { anyPlus = true; c = c.slice(0, -1); }
      bases.push(c);
    }
    if (!bases.length) return null;
    var t = String(text).trim();
    if (t === '' || _zwOmCssWide[t.toLowerCase()]) return null;
    var segs = anyHash ? _zwOmTopSplit(t, ',') : (anyPlus ? t.split(/\s+/) : [t]);
    var out = [];
    for (var s = 0; s < segs.length; s++) {
      var seg = segs[s].trim();
      if (seg === '') continue;
      var v = _zwOmReifyRegisteredSegment(bases, seg, prop);
      if (v === null) return null;
      out.push(v);
    }
    return out.length > 0 ? out : null;
  }
  function _zwOmReifyRegisteredSegment(bases, seg, prop) {
    // var() 引用残留段：unparsed 归面（registered computed 不保形）。
    if (/(^|[^\w-])var\(/i.test(seg)) return null;
    for (var i = 0; i < bases.length; i++) {
      var b = bases[i];
      if (b.charAt(0) === '<') {
        var name = b.slice(1, -1).toLowerCase();
        var mkOpaque = function () {
          var o = _zwOmOpaque();
          o._zwOmText = seg;
          o.toString = function () { return this._zwOmText; };
          o._zwOmAssociatedProp = prop;
          return o;
        };
        // <color>/<url>/<string>/<transform-*>：无分型子类面（css-properties-
        // values-api-1 reification 表——<url> 基类 opaque 非 CSSImageValue，
        // 语料 'Computed <url> is reified as CSSStyleValue'）。
        if (name === 'color' || name === 'url' || name === 'string'
          || name === 'transform-function' || name === 'transform-list') return mkOpaque();
        // <custom-ident>：任意 ident 关键字形（语料 'Computed <custom-ident> is
        // reified as CSSKeywordValue'——'none'/'NULL'/'thing' 皆 CSSKeywordValue）。
        if (name === 'custom-ident') {
          var kv2 = new CSSKeywordValue(seg);
          kv2._zwOmAssociatedProp = prop;
          return kv2;
        }
        if (name === 'image') {
          // 相对 url 按文档 baseURI 解析（<image> computed 绝对形——语料
          // 'Computed <image> is reified as CSSImageValue by iterator' 期望
          // `url("<abs>")`）。
          var imgText = seg;
          var um = /^\s*url\(\s*(['"]?)([^'")]*)\1\s*\)\s*$/i.exec(seg);
          if (um && um[2] !== '' && typeof globalThis.URL === 'function') {
            try {
              var base = globalThis.document && globalThis.document.baseURI;
              imgText = 'url("' + new globalThis.URL(um[2], base).href + '")';
            } catch (_eU) { imgText = seg; }
          }
          var iv = new CSSImageValue(imgText);
          iv._zwOmAssociatedProp = prop;
          return iv;
        }
        var uv = _zwOmReifyRegisteredUnit(name, seg);
        if (uv !== null) return uv;
        continue;
      }
      // ident 字面量：case-sensitive 匹配（spec 字面量敏感——语料
      // 'thing1 | THING2' THING2 形）。
      if (seg === b) {
        var kv = new CSSKeywordValue(seg);
        kv._zwOmAssociatedProp = prop;
        return kv;
      }
    }
    return null;
  }
  // 数值族分段分型：单位族按 syntax 名裁决；calc/min/max/clamp 数学形走既有
  // calc 树解析（<length-percentage> 混合形 CSSMathSum——语料 'Computed
  // <length-percentage> [px + %] is reified as CSSMathSum'）。不匹配返 null。
  function _zwOmReifyRegisteredUnit(name, seg) {
    var famWant = {
      'angle': ['angle'], 'length': ['length'], 'percentage': ['percent'],
      'length-percentage': ['length', 'percent'], 'number': ['number'],
      'integer': ['number'], 'time': ['time'], 'frequency': ['frequency'],
      'resolution': ['resolution'], 'flex': ['flex'],
    }[name];
    if (!famWant) return null;
    var m = /^([-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?)([a-zA-Z%]*)$/.exec(seg);
    if (m) {
      var u = (m[2] || 'number').toLowerCase();
      var cu = _zwOmCanonUnit(u);
      if (cu) u = cu;
      var inf = u === 'number' ? ['number'] : _zwOmUnitInfo(u);
      var fam = inf ? inf[0] : null;
      if (fam !== null && famWant.indexOf(fam) >= 0) return new CSSUnitValue(parseFloat(m[1]), u);
      // unitless zero 宽限（css-values-4 §5：length/percentage/angle 族 0 合法
      // ——generate_property('<length>') initial '0' computed 形）。
      if (fam === null && u === 'number' && parseFloat(m[1]) === 0) {
        if (famWant.indexOf('length') >= 0) return new CSSUnitValue(0, 'px');
        if (famWant.indexOf('angle') >= 0) return new CSSUnitValue(0, 'deg');
        if (famWant.indexOf('percent') >= 0) return new CSSUnitValue(0, 'percent');
      }
      return null;
    }
    var low = seg.toLowerCase();
    if (/^(calc|min|max|clamp|hypot|abs|sign|round|mod|rem)\(/.test(low)) {
      try {
        var p = new _zwOmCalcParser(seg);
        var node = p.parseSum();
        p.ws();
        if (p.i < p.s.length) return null;
        return node;
      } catch (_eM) { return null; }
    }
    return null;
  }
  // computedStyleMap 枚举名单（刀 TD1）：inline declared 名单在前 + 注册表名单
  //（去重；registered 名单由 host `__zw_registered_property_names` JSON 提供）。
  // registered 属性恒有 computed 值（initial 兜底），逐名文本预检省略——页级
  // 380+ 注册名 × 逐名 gCS 查询是页超时主因。缓存按 (el, inline 长度, 注册表
  // JSON) 失效——item(i) 每下标重算枚举是二次方放大源，缓存后名单面 O(1)。
  var _zwOmComputedEnumCache = { el: null, len: -1, json: '', names: [] };
  function _zwOmComputedEnumNames(el) {
    var slen = -1, json = '';
    try { slen = el.style.length | 0; } catch (_eC0) {}
    if (typeof globalThis.__zw_registered_property_names === 'function') {
      try { json = String(globalThis.__zw_registered_property_names() || ''); } catch (_eC1) { json = ''; }
    }
    var c = _zwOmComputedEnumCache;
    if (c.el === el && c.len === slen && c.json === json) return c.names;
    var ks = [], seen = {};
    try {
      var st = el.style;
      for (var i = 0; i < slen; i++) { var n = st.item(i); if (n && !seen[n]) { seen[n] = 1; ks.push(n); } }
    } catch (_eE1) {}
    if (json !== '') {
      try {
        var arr = JSON.parse(json);
        for (var j = 0; j < arr.length; j++) {
          var nm = String(arr[j]);
          if (nm && !seen[nm]) { seen[nm] = 1; ks.push(nm); }
        }
      } catch (_eE3) {}
    }
    c.el = el; c.len = slen; c.json = json; c.names = ks;
    return ks;
  }
  // OM-10（css-typed-om-1 §computed-stylepropertymapreadonly-objects 迭代序：
  // 普通属性在前、custom properties 码位升序在后）：computed 枚举源名单。语料
  // computed/iterable test 1 expectedKeys 由 `[...getComputedStyle(body)]` 运行时
  // 派生——gCS 迭代面（part01 Symbol.iterator 臂）与 computedStyleMap 枚举面
  // 共用本名单即构造性自洽（名单本身决定两侧等集）。名单取 typed-om 属性语法
  // 表键集（引擎计算通道无全属性宇宙枚举 FIXME：宿主 cascade 属性枚举接入后
  // 换真值名单）。cmpPN = 语料 comparePropertyNames 同判（非前缀组先、组内
  // 升序 <；返回值不取 0——名单键互异，稳定排序无影响）。
  function _zwOmCmpPropNames(a, b) {
    if ((a.charAt(0) === '-') === (b.charAt(0) === '-')) return a < b ? -1 : 1;
    return b.charAt(0) === '-' ? -1 : 1;
  }
  var _zwOmEnumSourceCache = null;
  globalThis.__zwOmComputedEnumSource = function () {
    if (_zwOmEnumSourceCache) return _zwOmEnumSourceCache;
    var ks = Object.keys(_zwOmPropSyntax);
    ks.sort(_zwOmCmpPropNames);
    _zwOmEnumSourceCache = ks;
    return ks;
  };
  function _zwOmMakeStyleMap(readable, getStyleObj) {
    // getStyleObj(): 底层 CSSStyleDeclaration 面（el.style / rule.style）。
    function mapGetAll(prop) {
      var p = _zwOmPropName(prop);
      var text = '';
      try { text = String(getStyleObj().getPropertyValue(p) || ''); } catch (_eG) { return []; }
      text = text.trim();
      var computedFace = false;
      try { computedFace = !!getStyleObj().__zwOmComputedFace; } catch (_eCF) {}
      // 刀 OM-1 marker 族 computed 兜底：引擎计算样式未产出 marker-* 文本
      // （探针 zz-om11probe2 B2 实证 gCS 返回空串、inline 声明可持久 B1/B3）
      // ——css-masking-1 §marker-* initial none 且 inherited，计算值恒
      // 'none' keyword 形（语料 marker.html computed non-null + 'none'
      // 断言面）。
      if (computedFace && text === '' && _zwOmMarkerProps[p]) {
        return [new CSSKeywordValue('none')];
      }
      if (text === '') return [];
      // 刀 OM-15：computed 面 sizing 槽 calc-size() 形——opaque 基类实例携
      // canonical 串（getPropertyValue OM-15 臂已产出 computed 形；map 层不做
      // typed 分型——css-typed-om-1 §get 不可分型值回落基类同型，本族语料仅
      // 断言 toString；declared 面不受影响照常解析）。
      if (computedFace && _zwOmCalcSizeSlots[p] && /^calc-size\(/i.test(text)) {
        var svCS = _zwOmOpaque();
        svCS._zwOmText = text;
        svCS.toString = function () { return this._zwOmText; };
        svCS._zwOmAssociatedProp = p;
        return [svCS];
      }
      var table = _zwOmPropSyntax[p];
      // 简写属性：无 typed 形态返基类实例（css-typed-om-1 §get 回落形态；
      // 语料 get-shorthand 面 assert_class_string CSSStyleValue）——实例携
      // 关联属性供 set 值裁决步错配门；css-wide keyword 保持 keyword 形。
      // 含 var() 引用文本恒回落 CSSUnparsedValue（css-typed-om-1 §get 值
      // 归一面：var() 引用值不分属性一律 unparsed 形——语料 properties/
      // margin.html 等 'Can set ... to var() references' 面类断言）。
      if (table && table.length === 0 && !_zwOmCssWide[text.toLowerCase()]) {
        if (/(^|[^\w-])var\(/i.test(text)) {
          return [new CSSUnparsedValue(_zwOmTokenizeVar(text))];
        }
        var sv = _zwOmOpaque();
        sv._zwOmText = text;
        sv.toString = function () { return this._zwOmText; };
        sv._zwOmAssociatedProp = p;
        return [sv];
      }
      // computed 面不支持属性：基类 opaque 回落（computedStyleMap 闭包携
      // __zwOmComputedFace 置位；declared 面不受影响照常解析——border-radius
      // 页四角 computed assert_is_unsupported 期望）。
      if (computedFace && _zwOmComputedUnsupportedProps[p] && text !== '' && !_zwOmCssWide[text.toLowerCase()]) {
        var cv = _zwOmOpaque();
        cv._zwOmText = text;
        cv.toString = function () { return this._zwOmText; };
        cv._zwOmAssociatedProp = p;
        return [cv];
      }
      // 刀 TD1：registered custom property computed typed 重ify（css-properties-
      // values-api-1 §reification：registered 属性 computed 值按注册 syntax 分型
      // ——<angle> 族→CSSUnitValue/<custom-ident>/ident 字面量→CSSKeywordValue/
      // <color>/<url>→opaque/<image>→CSSImageValue/calc 混合形→CSSMathSum 树；
      // '*' 与解析失败保持 unparsed 回落。specified 面恒 unparsed——语料
      // 'Specified <angle> is reified as CSSUnparsedValue from get/getAll'）。
      if (computedFace && p.slice(0, 2) === '--') {
        var syntaxR = _zwOmRegisteredSyntaxOf(p);
        if (syntaxR !== '' && syntaxR !== '*') {
          var valsR = _zwOmReifyRegisteredComputed(syntaxR, text, p);
          if (valsR !== null) return valsR;
        }
      }
      // list-valued 属性：顶层逗号逐段解析（§get 取首值/§getAll 全量——
      // 语料 get/getAll 面 'transition-duration: 1s, 2s' 语义）。
      var segs = _zwOmIsListValued(p) ? _zwOmTopSplit(text, ',') : [text];
      var out = [];
      for (var i = 0; i < segs.length; i++) {
        var seg = segs[i].trim();
        if (seg === '') continue;
        var isCalcSeg = /^calc\(/i.test(seg);
        var v = _zwOmParseWithText(p, seg);
        if (v == null) continue;
        // 刀 T3D：list-valued 任一段不可分型（opaque）→ 整值回落单基类实例携
        // 全文本（css-typed-om-1 §normalize an unsupported value：unsupported
        // 值归一不逐段截断——transition-property 'width, height' 经 get→set
        // 转移到另一元素语料钉；段级实例只携段文本，set 侧恒丢其余段）。
        if (segs.length > 1 && v._zwOmKind && v._zwOmKind() === 'opaque') {
          var ov = _zwOmOpaque();
          ov._zwOmText = text;
          ov.toString = function () { return this._zwOmText; };
          ov._zwOmAssociatedProp = p;
          return [ov];
        }
        // 刀 face C part 1：computed 面数值规范化（长度绝对化 px/同族 Sum
        // 折叠/grid 负百分钳零——见 _zwOmComputedCanonicalize）。
        if (computedFace) v = _zwOmComputedCanonicalize(p, v, getStyleObj);
        // 非负属性负数值：association 语义包 Sum（testsuite range_handling
        // 面）——仅 declared 面（computed 面断言期望裸 Unit 形）。
        if (!computedFace && v instanceof CSSUnitValue && v.value < 0 && _zwOmNonNegativeProps[p]) v = new CSSMathSum([v]);
        // 刀 OM-11：整数族 association 语义包 Sum（css-typed-om-1 §normalize
        // specified 面：非整数或低于下限（orphans/widows/column-count 下限 1；
        // order 全值域仅非整数）包 Sum([v])，值域内整数裸 Unit 回显——语料
        // orphans/widows/column-count/order 页 specified 臂 assert_equals
        // expected CSSMathSum 实证）。与上非负臂同型；computed 面走取整/钳 1
        // 臂不包壳。
        if (!computedFace && v instanceof CSSUnitValue && _zwOmIntProps[p]
          && (!Number.isInteger(v.value) || (_zwOmIntMin1Props[p] && v.value < 1))) v = new CSSMathSum([v]);
        // 刀 face C part 1：declared 面单 term calc 保 Sum 壳（css-typed-om-1
        // §parse a calc()：解析结果恒 math 节点，单值仍 Sum 包裹——与
        // CSSNumericValue.parse 'calc(10px)'→Sum 同形；margin 简写页
        // assert_is_calc_sum 面。引擎 declared 文本折叠 'calc(0% + 0%)'→
        // 'calc(0%)' 后类断言期望 Sum 非裸 Unit）。
        if (!computedFace && v instanceof CSSUnitValue && isCalcSeg) v = new CSSMathSum([v]);
        out.push(v);
      }
      return out;
    }
    function mapGet(prop) {
      // css-typed-om-1 §get：未设置属性返回 undefined（语料 get.html 面
      // assert_equals(result, undefined)；has/getAll 走列表全量面）。
      var vs = mapGetAll(prop);
      return vs.length > 0 ? vs[0] : undefined;
    }
    var ReadOnly = {};
    Object.defineProperty(ReadOnly, 'get', { value: function (prop) { return mapGet(prop); }, writable: true, configurable: true });
    Object.defineProperty(ReadOnly, 'getAll', {
      value: function (prop) {
        return mapGetAll(prop);
      }, writable: true, configurable: true,
    });
    Object.defineProperty(ReadOnly, 'has', { value: function (prop) { return mapGet(prop) != null; }, writable: true, configurable: true });
    Object.defineProperty(ReadOnly, 'size', {
      get: function () {
        try { return getStyleObj().length | 0; } catch (_eL) { return 0; }
      }, enumerable: true, configurable: true,
    });
    Object.defineProperty(ReadOnly, 'keys', { value: function () {
      // OM-10：名单改走 _zwOmMapNames（computed 面枚举分流；inline/declared
      // 面同逻辑零漂移）——iterator-om10：迭代器对象挂 Symbol.iterator 自涉
      //（WebIDL maplike 迭代器自身可迭代——css-typed-om-1 §5.3 maplike
      // iterable 展开面；探针 zz-om10probe G1 裸 {next} 展开 TypeError 实证）。
      var names = _zwOmMapNames();
      var idx = 0;
      var itK = { next: function () { return idx < names.length ? { value: names[idx++], done: false } : { value: undefined, done: true }; } };
      itK[Symbol.iterator] = function () { return itK; };
      return itK;
    }, writable: true, configurable: true });
    // 刀 TD1（css-typed-om-1 §5.3 StylePropertyMapReadOnly
    // iterable<DOMString, FrozenArray<CSSStyleValue>>）：values/entries 值形 =
    // CSSStyleValue 列表（单值属性亦单元素列表——语料 stylepropertymap/inline/
    // iterable `values[0]` = `[CSS.px(10)]` 形）；Symbol.iterator 挂 entries
    //（Array.from 产 [name, list] 对——css-properties-values-api-1 typedom.html
    // verify_map_iteration_unparsed `iter_value.length == 1` 面）。
    function _zwOmMapNames() {
      var ks = [];
      try { var st = getStyleObj(); for (var i = 0; i < st.length; i++) ks.push(st.item(i)); } catch (_eN) {}
      // OM-10：computed 面枚举（face 无 length/item，上循环空转——探针
      // zz-om10probe G5 键空实证）——css-typed-om-1 §computed-stylepropertymap-
      // readonly-objects：普通属性 cmpPN 序（__zwOmComputedEnumSource 与 gCS
      // 迭代面同源，expectedKeys 运行时派生即构造性自洽）＋元素 custom
      // properties 码位升序在后（TD1 枚举复用：inline + registered 名单，
      // registered 属性恒 '--' 起始，滤 '--' 前缀）。size 未同步（仍读
      // st.length → 0，语料无 computed size 断言面）—— FIXME: face
      // length/item 接线后取枚举长度。
      var cfN = false;
      try { cfN = !!getStyleObj().__zwOmComputedFace; } catch (_eN4) {}
      if (cfN) {
        ks.length = 0;
        var srcN = (typeof globalThis.__zwOmComputedEnumSource === 'function') ? globalThis.__zwOmComputedEnumSource() : [];
        for (var k2 = 0; k2 < srcN.length; k2++) ks.push(srcN[k2]);
        var elC = null;
        try { elC = getStyleObj().__zwOmComputedEl || null; } catch (_eN5) {}
        if (elC) {
          var allC = _zwOmComputedEnumNames(elC);
          var cusC = [];
          for (var k3 = 0; k3 < allC.length; k3++) {
            var nmC = String(allC[k3]);
            if (nmC.slice(0, 2) === '--') cusC.push(nmC);
          }
          cusC.sort(_zwOmCmpPropNames);
          for (var k4 = 0; k4 < cusC.length; k4++) ks.push(cusC[k4]);
        }
        return ks;
      }
      return ks;
    }
    Object.defineProperty(ReadOnly, 'values', { value: function () {
      // iterator-om10：迭代器对象挂 Symbol.iterator 自涉（同 keys 臂注记）。
      var self = this; var ks = _zwOmMapNames();
      var idx = 0;
      var itV = { next: function () { return idx < ks.length ? { value: self.getAll(ks[idx++]), done: false } : { value: undefined, done: true }; } };
      itV[Symbol.iterator] = function () { return itV; };
      return itV;
    }, writable: true, configurable: true });
    Object.defineProperty(ReadOnly, 'entries', { value: function () {
      // iterator-om10：迭代器对象挂 Symbol.iterator 自涉（同 keys 臂注记）。
      var self = this; var ks = _zwOmMapNames();
      var idx = 0;
      var itE = { next: function () { return idx < ks.length ? { value: [ks[idx], self.getAll(ks[idx++])], done: false } : { value: undefined, done: true }; } };
      itE[Symbol.iterator] = function () { return itE; };
      return itE;
    }, writable: true, configurable: true });
    Object.defineProperty(ReadOnly, 'forEach', { value: function (cb) {
      var ks = _zwOmMapNames();
      for (var i = 0; i < ks.length; i++) cb(this.getAll(ks[i]), ks[i], this);
    }, writable: true, configurable: true });
    ReadOnly[Symbol.iterator] = ReadOnly.entries;
    if (readable) return ReadOnly;
    // Mutable 面
    Object.defineProperty(ReadOnly, 'set', {
      value: function (prop, value) {
        var p = _zwOmPropName(prop);
        var vals = Array.prototype.slice.call(arguments, 1);
        if (vals.length === 0) throw new TypeError("set requires a value");
        var custom = p.slice(0, 2) === '--';
        var listValued = _zwOmIsListValued(p);
        // css-typed-om-1 §set 值裁决步：非 list-valued 属性仅接受单值
        //（语料 'Setting a non list-valued property with multiple arguments'
        // 面）；var 引用/CSSUnparsedValue 不得与其他值并置（语料 'with a
        // CSSUnparsedValue and other values'/'with a var ref() and other
        // values' 面）。自定义属性豁免（spec custom properties 面）。
        if (!listValued && !custom && vals.length > 1) throw new TypeError("Cannot set multiple values on a non list-valued property");
        for (var i = 0; i < vals.length; i++) _zwOmValidateSet(p, vals[i]);
        if (!custom && vals.length > 1) {
          for (var iu = 0; iu < vals.length; iu++) {
            if (_zwOmIsUnparsedLike(vals[iu])) throw new TypeError("Cannot set a CSSUnparsedValue or var reference with other values");
          }
        }
        var text = _zwOmDeclTextFromValue(p, vals, listValued && !custom ? ', ' : ' ');
        try {
          getStyleObj().setProperty(p, text);
          _zwOmExpandShorthand(getStyleObj(), p, text);
        } catch (_eS) { throw new TypeError("set failed"); }
      }, writable: true, configurable: true,
    });
    Object.defineProperty(ReadOnly, 'append', {
      value: function (prop, value) {
        var p = _zwOmPropName(prop);
        var vals = Array.prototype.slice.call(arguments, 1);
        if (vals.length === 0) throw new TypeError("append requires a value");
        var custom = p.slice(0, 2) === '--';
        if (custom && _zwOmRegisteredSyntaxOf(p) !== '') {
          // css-properties-values-api-1 §reification：registered custom property
          // append 恒 TypeError（单值/`+`/`#` syntax 皆然——语料 'Appending a
          // string to * is not allowed' 族，刀 TD1；未注册自定义属性保持既有
          // 逗号并置面）。
          throw new TypeError("Cannot append to a registered custom property");
        }
        if (!custom) {
          // css-typed-om-1 §append 步 1-2：仅 list-valued 属性可 append
          //（非 list/简写属性 TypeError——语料 'a property that is not list
          // valued'/'a shorthand property' 面）；值逐个过 set 值裁决（语料
          // unsupported/invalid value 面面）；CSSUnparsedValue/var 引用恒拒
          //（语料 'with a CSSUnparsedValue'/'with a var ref' 面）。
          if (!_zwOmIsListValued(p)) throw new TypeError("Cannot append to a non list-valued property");
          for (var i = 0; i < vals.length; i++) _zwOmValidateSet(p, vals[i]);
          for (var iu = 0; iu < vals.length; iu++) {
            if (_zwOmIsUnparsedLike(vals[iu])) throw new TypeError("Cannot append a CSSUnparsedValue or var reference");
          }
        }
        var prev = '';
        try { prev = String(getStyleObj().getPropertyValue(p) || ''); } catch (_eA) { prev = ''; }
        prev = prev.trim();
        if (!custom) {
          // CSS-wide keyword 恒值不得 append（crbug.com/1417176 面）；既有
          // 列表含 var() 引用同理（declared append 语料 variable-reference 面）。
          if (_zwOmCssWide[prev.toLowerCase()]) throw new TypeError("Cannot append to a CSS-wide keyword value");
          if (/(^|[^\w-])var\(/i.test(prev)) throw new TypeError("Cannot append to a list containing a var reference");
        }
        var parts = [];
        for (var iv = 0; iv < vals.length; iv++) parts.push(_zwOmValueToText(vals[iv]));
        var text = parts.join(', ');
        var next = prev !== '' ? prev + ', ' + text : text;
        try { getStyleObj().setProperty(p, next); } catch (_eA2) { throw new TypeError("append failed"); }
      }, writable: true, configurable: true,
    });
    Object.defineProperty(ReadOnly, 'delete', {
      value: function (prop) {
        try { getStyleObj().removeProperty(_zwOmPropName(prop)); } catch (_eD) {}
      }, writable: true, configurable: true,
    });
    Object.defineProperty(ReadOnly, 'clear', {
      value: function () {
        try { getStyleObj().cssText = ''; } catch (_eC) {}
      }, writable: true, configurable: true,
    });
    return ReadOnly;
  }

  // ===== 13. Element.prototype 挂接 =====
  // 相对色 computed 面骨架（css-color-5 §relative-colors，WPT
  // relative-color-with-system-color 钉面）：inline 声明为
  // `fn(from <origin> <channels>)` 且 origin 折叠为 currentcolor 时，Typed-OM
  // computed 值保形骨架 `fn(from currentcolor <channels>)`——getComputedStyle
  // 返回 resolved 色，故骨架检测须按 inline 声明原文先行。origin 为绝对色
  //（system color 等）时返 ''，保持宿主 resolved 通道（computed 整体解析）。
  function _zwOmRelativeSkeletonText(decl, el) {
    var m = /^([a-zA-Z][-\w]*)\(\s*from\s+([\s\S]+)\)$/.exec(String(decl).trim());
    if (!m) return '';
    var fnL = m[1].toLowerCase();
    if (fnL !== 'hsl' && fnL !== 'hwb' && fnL !== 'lab' && fnL !== 'lch'
      && fnL !== 'oklab' && fnL !== 'oklch' && fnL !== 'rgb' && fnL !== 'color') return '';
    // origin：函数块（括号配平）或单个 ident；其余取 channels 原文。
    var rest = m[2].trim();
    var origin = null, channels = '';
    var fo = /^([a-zA-Z][-\w]*)\s*\(/.exec(rest);
    if (fo) {
      var depth = 0;
      for (var i = fo[0].length - 1; i < rest.length; i++) {
        var ch = rest.charAt(i);
        if (ch === '(') depth++;
        else if (ch === ')') { depth--; if (!depth) { origin = rest.slice(0, i + 1); channels = rest.slice(i + 1).trim(); break; } }
      }
    } else {
      var io = /^([a-zA-Z][-\w]*)(?:\s+([\s\S]+))?$/.exec(rest);
      if (io) { origin = io[1]; channels = io[2] || ''; }
    }
    if (!origin) return '';
    if (globalThis.__zwCssRelativeOrigin(origin, el) !== 'currentcolor') return '';
    if (channels) {
      // sibling-index() → 元素在父级子列表的 1-based 索引；找不到（detached）
      // 保留原文不编造。sibling-count() 未建模（FIXME）。
      var p = el && el.parentNode;
      if (p && p.children) {
        for (var k = 0; k < p.children.length; k++) {
          if (p.children[k] === el) {
            channels = channels.replace(/sibling-index\s*\(\s*\)/gi, String(k + 1));
            break;
          }
        }
      }
    }
    return fnL + '(from currentcolor' + (channels ? ' ' + channels : '') + ')';
  }
  // 逻辑属性 → 物理槽位翻译（css-logical-1 §logical-to-physical；假设
  // horizontal-tb + ltr——语料页无 writing-mode 声明，FIXME: 纵排/rtl
  // 需按元素 writing-mode/direction 重映射）。轴简写（margin-block 等）
  // 与 border 简写不入表——host 无展开的形（R5 探针）物理槽位无值，
  // 保持 declared 回落。
  var _zwOmLogicalPhysical = {
    'margin-block-start': 'margin-top', 'margin-block-end': 'margin-bottom',
    'margin-inline-start': 'margin-left', 'margin-inline-end': 'margin-right',
    'padding-block-start': 'padding-top', 'padding-block-end': 'padding-bottom',
    'padding-inline-start': 'padding-left', 'padding-inline-end': 'padding-right',
    'inset-block-start': 'top', 'inset-block-end': 'bottom',
    'inset-inline-start': 'left', 'inset-inline-end': 'right',
    'border-block-start-width': 'border-top-width', 'border-block-end-width': 'border-bottom-width',
    'border-inline-start-width': 'border-left-width', 'border-inline-end-width': 'border-right-width',
    'border-block-start-color': 'border-top-color', 'border-block-end-color': 'border-bottom-color',
    'border-inline-start-color': 'border-left-color', 'border-inline-end-color': 'border-right-color',
    'border-block-start-style': 'border-top-style', 'border-block-end-style': 'border-bottom-style',
    'border-inline-start-style': 'border-left-style', 'border-inline-end-style': 'border-right-style',
    'border-start-start-radius': 'border-top-left-radius', 'border-start-end-radius': 'border-top-right-radius',
    'border-end-start-radius': 'border-bottom-left-radius', 'border-end-end-radius': 'border-bottom-right-radius'
  };
  // 刀 P：computed 值 Typed OM L1 不支持面（css-backgrounds-3 §border-*-radius：
  // computed 为双值对，testsuite assert_is_unsupported 期望基类 opaque——
  // declared 面仍逐语法解析 equal；仅物理四角，logical.html 逻辑角 computed
  // 走默认 equal 期望，不在此列）。刀 G2-b：border-image-outset/slice/width/
  // repeat computed 形（outset/width 数值缩放多值、slice '1 2 fill' 关键字
  // 混排、repeat 关键字——css-backgrounds-3/border-image computed 语义
  // Typed OM 无分型，页面 computed 臂 assert_is_unsupported 期望基类 opaque；
  // 同页 declared 面仍 typed 解析 equal，未在本列）。刀 OM-5：
  // column-rule-style/width 入列（css-multicol-1 §column-rule-*：testsuite
  // computed 臂显式 assert_class_string 基类 CSSStyleValue 期望——Typed OM L1
  // 无 per-property 分型，关键词 thin/medium/thick 与 <length> 均不升级子类；
  // declared 面仍逐语法解析 equal 不受影响）。刀 OM-11：scroll-snap-align
  // 同合同入列（css-scroll-snap-1 §scroll-snap-align：testsuite computed 臂
  // assert_is_unsupported 期望基类——语料四关键词臂 assert_class_string
  // expected [object CSSStyleValue] got CSSKeywordValue，b1 失败消息实证）。
  var _zwOmComputedUnsupportedProps = {
    'column-rule-style': 1, 'column-rule-width': 1, 'scroll-snap-align': 1,
    'border-top-left-radius': 1, 'border-top-right-radius': 1,
    'border-bottom-left-radius': 1, 'border-bottom-right-radius': 1,
    'border-image-outset': 1, 'border-image-repeat': 1,
    'border-image-slice': 1, 'border-image-width': 1
  };
  function _zwOmComputedProp(el, prop) {
    // 复用 part01 getComputedStyle 代理回调链（注册元素 sel/handle 路由由
    // 既有面完成），零新 Rust 回调。颜色属性的相对色骨架面（上）先于宿主
    // resolved 读数。
    var text = '';
    var decl = '';
    try {
      decl = (el && el.style && String(el.style.getPropertyValue(prop) || '')) || '';
      if (decl && (prop === 'color' || /-color$/.test(prop) || prop === 'fill' || prop === 'stroke')
        && typeof globalThis.__zwCssRelativeOrigin === 'function') {
        text = _zwOmRelativeSkeletonText(decl, el);
      }
    } catch (_eRS) { text = ''; decl = ''; }
    if (text !== '') return text;
    // 刀 P computed 面三臂（宿主 gCS 读数不达标形，按 declared 原文先行）：
    // ① 宽度关键字（css-backgrounds-3 §border-width：thin/medium/thick 计算值
    //    1/3/5px 与 border-style 无关——testsuite assert_is_keyword_width 期望，
    //    宿主 style:none 形返 0px 不合）；
    // ② padding 负 % 钳零保单位（css-box-3 computed 钳形，testsuite
    //    assert_is_equal_with_clamping_percentage 期望 0%，宿主给 0px）；
    // ③ currentcolor 关键字保形（css-color-4：typed-om computed 关键字面
    //    CSSKeywordValue 期望，宿主 resolved rgb 不合）。
    var declL = decl.toLowerCase();
    if (decl) {
      // ① 族门正例（css-backgrounds-3/css-ui-4/css-multicol 绝对 px 计算值）：
      //    仅 border-*-width / outline-width / column-rule-width——scrollbar-width
      //    等 -width 尾缀属性 computed 为 as-specified 关键字（css-scrollbars-1），
      //    不得劫持（b0 通过面，刀 P 首跑 P2F 定谳后收窄）。
      if (prop === 'outline-width' || prop === 'column-rule-width'
        || (prop.slice(0, 7) === 'border-' && prop.slice(-6) === '-width')) {
        if (declL === 'thin') return '1px';
        if (declL === 'medium') return '3px';
        if (declL === 'thick') return '5px';
      }
      if (prop.slice(0, 8) === 'padding-' && /^-\S*%$/.test(decl)) return '0%';
      if (/-color$/.test(prop) && declL === 'currentcolor') return 'currentcolor';
    }
    if (globalThis.getComputedStyle) {
      // 逻辑名翻译物理槽位咨询（declared 无值或非关键字形——宿主 canonical
      // 链 em/cm→px、% 保形、calc→px 由物理面提供）。
      var q = _zwOmLogicalPhysical[prop] || prop;
      try { text = String(globalThis.getComputedStyle(el).getPropertyValue(q) || ''); } catch (_eG) { text = ''; }
    }
    return text;
  }
  // CSSStyleRule.styleMap（css-typed-om-1 § the-stylepropertymap declared 面）——
  // part06 规则对象为 plain object（无 CSSStyleRule 原型可挂，FIXME: CSSOM 规则
  // 接口族原型化后改原型 getter），经 __zwAttachRuleStyleMap 钩子逐规则挂
  // SameObject styleMap（styleMap 底座 = rule.style 声明）。
  globalThis.__zwAttachRuleStyleMap = function (rule) {
    if (!rule || rule.__zwStyleMapAttached || rule.type !== 1 || !rule.style) return;
    rule.__zwStyleMapAttached = true;
    var self = rule;
    Object.defineProperty(rule, 'styleMap', {
      configurable: true,
      enumerable: false,
      get: function () {
        var cached = self.__zwRuleStyleMap;
        if (cached) return cached;
        var map = _zwOmMakeStyleMap(false, function () { return self.style; });
        Object.defineProperty(self, '__zwRuleStyleMap', { value: map, enumerable: false, configurable: true, writable: true });
        return map;
      },
    });
  };
  if (globalThis.Element && globalThis.Element.prototype) {
    Object.defineProperty(globalThis.Element.prototype, 'attributeStyleMap', {
      configurable: true,
      enumerable: false,
      get: function () {
        if (typeof this.style === 'undefined' && typeof this.getAttribute !== 'function') return undefined;
        var cached = this.__zwAttrStyleMap;
        if (cached) return cached;
        var self = this;
        var map = _zwOmMakeStyleMap(false, function () { return self.style; });
        try {
          Object.defineProperty(self, '__zwAttrStyleMap', { value: map, enumerable: false, configurable: true, writable: true });
        } catch (_eC1) {}
        return map;
      },
    });
    // WebIDL 属性面非枚举（expando enumeration 语料/r3046 面）。
    Object.defineProperty(globalThis.Element.prototype, 'computedStyleMap', {
      value: function () {
        var self = this;
        return _zwOmMakeStyleMap(true, function () {
          // computed 面：host 计算串为主；host 未接属性（计算串空）回落 declared
          // 文本解析，保证 get 非 null 的 keyword/unit 面。
          var face = {
            __zwOmComputedFace: true,
            // OM-10：枚举面元素回执（_zwOmMapNames computed 分流经
            // _zwOmComputedEnumNames 取元素 custom properties 名单）。
            __zwOmComputedEl: self,
            getPropertyValue: function (p) {
              // 刀 OM-2：scroll-padding 族 computed 恒从 inline 声明文本 reify
              // （引擎计算串对 %/负值/calc 有损——探针 zz-om2probe：物理向
              // '0%'/'3.14%' 回 '0px'、负/calc 回 'auto'；声明文本经
              // _zwOmComputedCanonicalize 兑现 css-scroll-snap-1 §scroll-padding
              // computed 值语义：percent 保形钳零、长度绝对化 px、calc 折叠）。
              if (_zwOmScrollPadProps[p]) {
                try { return String((self.style && self.style.getPropertyValue(p)) || ''); } catch (_eSP) { return ''; }
              }
              // 刀 OM-4：cursor computed 从 inline 声明文本 reify 词表内关键词
              // （引擎计算串词表落后 css-ui-4：探针 zz-om4probe——6 现代词
              // context-menu/cell/vertical-text/alias/copy/no-drop gcs 回 'auto'、
              // 8 legacy 向 gcs 伪归一 e/w→'ew-resize' n/s/ne/se→'ns-resize'、
              // 4 bidi 词 gcs 回 'auto'。css-ui-4 §cursor 计算值＝指定关键词
              // 原样无归一化，声明文本探针实证逐字保留）。词表内关键词原样
              // 返回（map 层 reify CSSKeywordValue）；词表外（inherit/var()/
              // url() 系）回落引擎计算串保继承与替代语义。
              if (p === 'cursor') {
                var dCur = '';
                try { dCur = String((self.style && self.style.getPropertyValue('cursor')) || '').trim(); } catch (_eCU) {}
                if (dCur && _zwOmPropSyntax['cursor'] && _zwOmPropSyntax['cursor'].indexOf(dCur) !== -1) {
                  return dCur;
                }
              }
              // 刀 OM-5：font-stretch computed 从 inline 声明文本 reify 词表内
              // 关键词（引擎计算串把九关键词按 css-fonts-4 §font-stretch 映射表
              // 折算为 <percentage> 输出——语料 font-stretch.html 关键词臂
              // computed 走默认 equal 期望 CSSKeywordValue(关键词) 原样）。
              // 词表内关键词原样返回（map 层 reify CSSKeywordValue）；词表外
              // （<percentage>/calc/var()/inherit 系）回落引擎计算串保既有
              // 百分比钳位臂（assert_is_equal_with_clamping_and_rounding_
              // percentage 面已绿）。
              if (p === 'font-stretch') {
                var dFS = '';
                try { dFS = String((self.style && self.style.getPropertyValue('font-stretch')) || '').trim(); } catch (_eFSx) {}
                if (dFS && _zwOmPropSyntax['font-stretch'] && _zwOmPropSyntax['font-stretch'].indexOf(dFS.toLowerCase()) !== -1) {
                  return dFS;
                }
              }
              // 刀 OM-5：逻辑角 border-*-radius computed 百分比形从 inline 声明
              // 文本 reify（css-backgrounds-3 §border-*-radius 百分比 computed
              // 保形——引擎计算串按物理四角绝对化 px 并钳负值，语料 logical.html
              // 逻辑角百分比臂走默认 equal 期望 CSSUnitValue(n,'percent') 与
              // calc 折叠 percent Unit；<length> 形不入臂——引擎 px 绝对化即
              // defaultComputed assert_is_unit('px') 面）。var()/css-wide 系
              // 回落引擎保继承与替代语义。
              if (_zwOmLogicalRadiusProps[p]) {
                var dRad = '';
                try { dRad = String((self.style && self.style.getPropertyValue(p)) || '').trim(); } catch (_eRA) {}
                if (dRad && dRad.indexOf('%') !== -1 && dRad.indexOf('var(') === -1
                  && !_zwOmCssWide[dRad.toLowerCase()]) {
                  return dRad;
                }
              }
              // 刀 OM-6：touch-action computed 关键词原样回显（css-pointerevents-2
              // §touch-action 计算值＝指定关键词序列，无归一化——引擎 gCS 把
              // pan-left/right/up/down 与 pinch-zoom 归一 'auto'（探针
              // zz-om6probe 挂载态实证），语料 touch-action.html 关键词臂
              // computed 走默认 equal 期望 CSSKeywordValue 原样）。词表内声明
              // 文本原样返回（map 层 reify CSSKeywordValue，OM-4 cursor 同型）；
              // 词表外（var()/css-wide 系）回落引擎计算串保继承语义。
              if (p === 'touch-action') {
                var dTA = '';
                try { dTA = String((self.style && self.style.getPropertyValue('touch-action')) || '').trim(); } catch (_eTA) {}
                if (dTA && _zwOmPropSyntax['touch-action'] && _zwOmPropSyntax['touch-action'].indexOf(dTA) !== -1) {
                  return dTA;
                }
              }
              // 刀 OM-6：column-gap/row-gap computed 从 inline 声明文本 reify
              // （css-align-3 §gap——normal 关键词计算值保形，引擎 gCS 对
              // column-gap 回 'auto'、row-gap 回 '0px'；百分比计算值保 % 形、
              // 负 % 走 _zwOmGridClampPctProps 钳 0（assert_is_equal_with_
              // clamping_percentage 期望 Unit(0,'percent')——引擎串 '0px' 已
              // 丢单位）；<length> 绝对化 px 类断言）。声明文本经既有
              // _zwOmComputedCanonicalize 折叠/钳位；var()/css-wide 系回落引擎。
              if (p === 'column-gap' || p === 'row-gap') {
                var dGap = '';
                try { dGap = String((self.style && self.style.getPropertyValue(p)) || '').trim(); } catch (_eGP) {}
                if (dGap && dGap.indexOf('var(') === -1 && !_zwOmCssWide[dGap.toLowerCase()]
                  && (_zwOmPropSyntax[p] && _zwOmPropSyntax[p].indexOf(dGap) !== -1
                    || /^(calc\(|[-+0-9.])/.test(dGap))) {
                  return dGap;
                }
              }
              // 刀 OM-6：line-height 数字/百分比/calc 形 computed 从 inline 声明
              // 文本 reify（css-inline-3 §line-height 计算值：纯 <number> 保
              // number 不绝对化——引擎 gCS 折算字号 px（探针 '3.14'→'50.24px'，
              // 语料 number 臂 assert_equals unit 'number'）；负值与 calc 形被
              // 引擎声明弃置回 'normal'（关键词类断言期望 Unit 类）；
              // <length> 绝对化 px、<percentage> 走 canonicalize line-height
              // 解析臂绝对化 px——正形与引擎输出逐值恒等零回归）。
              // 'normal'/var()/css-wide 系回落引擎。
              if (p === 'line-height') {
                var dLH = '';
                try { dLH = String((self.style && self.style.getPropertyValue('line-height')) || '').trim(); } catch (_eLH) {}
                if (dLH && dLH.indexOf('var(') === -1 && !_zwOmCssWide[dLH.toLowerCase()]
                  && /^(calc\(|[-+0-9.])/.test(dLH)) {
                  return dLH;
                }
              }
              // 刀 OM-7：stroke-dasharray/stroke-width 数值形 computed 从 inline
              // 声明文本 reify（svg2 §stroke-dasharray/§stroke-width——number
              // 计算值语料 assert_is_unit('px') 用户单位类、负 % 钳 0（
              // _zwOmGridClampPctProps 同合同）；引擎 gCS 对 number 形保形回显
              // unit 'number'（探针 zz-om7probe 挂载态实证）。单值数值形门：
              // calc 前缀或带可选单位数字 token——多值 '5% 1em 2%' 不入回落
              // 引擎（unsupported 臂原期望）；关键词（none）回落引擎。
              if (p === 'stroke-dasharray' || p === 'stroke-width') {
                var dSW = '';
                try { dSW = String((self.style && self.style.getPropertyValue(p)) || '').trim(); } catch (_eSW) {}
                if (dSW && dSW.indexOf('var(') === -1 && !_zwOmCssWide[dSW.toLowerCase()]
                  && (/^calc\(/.test(dSW) || /^[-+]?[0-9.]+[a-z%]*$/.test(dSW))) {
                  return dSW;
                }
              }
              // 刀 OM-7：tab-size 数值形 computed 从 inline 声明文本 reify
              // （css-text-3 §tab-size——number 计算值保 number 形负值钳 0
              // （assert_is_equal_with_clamping_number 期望 Unit(0,'number')），
              // <length> 走通用相对单位折 px（负 em/calc 形引擎声明弃置回默认
              // number 8，探针实证）；单值数值形门同 stroke-* 臂；var()/
              // css-wide/关键词回落引擎。
              if (p === 'tab-size') {
                var dTS = '';
                try { dTS = String((self.style && self.style.getPropertyValue('tab-size')) || '').trim(); } catch (_eTS) {}
                if (dTS && dTS.indexOf('var(') === -1 && !_zwOmCssWide[dTS.toLowerCase()]
                  && (/^calc\(/.test(dTS) || /^[-+]?[0-9.]+[a-z%]*$/.test(dTS))) {
                  return dTS;
                }
              }
              // 刀 OM-11：整数族 computed 从 inline 声明文本 reify（css-values-4
              // §<integer> 计算值取整 + orphans/widows/column-count 下限 1——
              // 引擎计算串对分数形有损：探针 zz-om11probe G2/G3/G4 实证
              // orphans -3.14 回裸 -3.14、order -3.14/3.14 回 0、column-count
              // 分数/calc 回 'auto'。声明文本经 _zwOmComputedCanonicalize
              // 取整/钳 1 臂兑现合同）。单值数值形门（calc 前缀或纯数字
              // token）同 stroke-*/tab-size 臂；var()/css-wide 系回落引擎。
              if (_zwOmIntProps[p]) {
                var dInt = '';
                try { dInt = String((self.style && self.style.getPropertyValue(p)) || '').trim(); } catch (_eIN) {}
                if (dInt && dInt.indexOf('var(') === -1 && !_zwOmCssWide[dInt.toLowerCase()]
                  && (/^calc\(/.test(dInt) || /^[-+]?[0-9.]+$/.test(dInt))) {
                  return dInt;
                }
              }
              // 刀 OM-11：text-decoration-line computed 词表内关键词原样回显
              // （css-text-decor-3 §text-decoration-line 计算值＝指定关键词，
              // 无归一化——引擎计算串对未实现词 blink/spelling-error/
              // grammar-error 回 'none'（探针 zz-om11probe G7 实证），语料三臂
              // assert_equals expected 关键词原文 got 'none'）。词表内单关键词
              // 声明文本原样返回（map 层 reify CSSKeywordValue，OM-4 cursor/
              // OM-6 touch-action 同型）；词表外（多关键词并置/var()/
              // css-wide 系）回落引擎保既有面零漂移。
              if (p === 'text-decoration-line') {
                var dTD = '';
                try { dTD = String((self.style && self.style.getPropertyValue('text-decoration-line')) || '').trim(); } catch (_eTD) {}
                if (dTD && _zwOmPropSyntax['text-decoration-line']
                  && _zwOmPropSyntax['text-decoration-line'].indexOf(dTD.toLowerCase()) !== -1) {
                  return dTD;
                }
              }
              // 刀 OM-15：sizing 槽 calc-size() computed 串化回显（css-values-5
              // §8 `calc-size()` 计算值＝结构保形，非 used px——引擎计算串把
              // 非 definite 形吞为 used/0px/none，declared 原文经
              // _zwOmCalcSizeComputedForm 补 computed 形差异后回显；var() 引用
              // 与结构不锚形回落引擎保替换语义零漂移）。
              if (_zwOmCalcSizeSlots[p]) {
                var dCS = '';
                try { dCS = String((self.style && self.style.getPropertyValue(p)) || '').trim(); } catch (_eCS0) {}
                if (dCS && /^calc-size\(/i.test(dCS) && dCS.indexOf('var(') === -1) {
                  var fpx = null;
                  try {
                    var fsm = /^\s*([0-9.]+)px\s*$/.exec(String(_zwOmComputedProp(self, 'font-size') || ''));
                    if (fsm) fpx = parseFloat(fsm[1]);
                  } catch (_eCS1) {}
                  var cCS = _zwOmCalcSizeComputedForm(dCS, fpx);
                  if (cCS !== null) return cCS;
                }
              }
              var c = _zwOmComputedProp(self, p);
              if (c !== '') return c;
              try { return String((self.style && self.style.getPropertyValue(p)) || ''); } catch (_eD) { return ''; }
            },
          };
          // 刀 TD1 枚举面（css-typed-om-1 §computedStyleMap iteration）：inline
          // declared 名单 + 注册表名单（registered 初始值属性 computed 必现——
          // 语料 'Registered property with initial value show up on iteration
          // of computedStyleMap'）。
          Object.defineProperty(face, 'length', {
            get: function () { return _zwOmComputedEnumNames(self).length; },
            enumerable: true, configurable: true,
          });
          face.item = function (i) {
            var ks = _zwOmComputedEnumNames(self);
            return i >= 0 && i < ks.length ? ks[i] : '';
          };
          return face;
        });
      },
      writable: true,
      configurable: true,
      enumerable: false,
    });
  }
})();
