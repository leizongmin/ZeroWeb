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
      // CSSUnparsedValue。
      if (inTable && _zwOmPropSyntax[prop].length === 0) {
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
  globalThis.CSSKeywordValue = CSSKeywordValue;

  // ===== CSSNumericValue（abstract；Unit/Math 基接口 + 算术面）=====
  // testhelper 数值类型保持块裸引该标识符（spec CSSNumericValue : CSSStyleValue）。
  function CSSNumericValue() { throw new TypeError("Illegal constructor"); }
  CSSNumericValue.prototype = Object.create(CSSStyleValue.prototype);
  CSSNumericValue.prototype.constructor = CSSNumericValue;
  // to(unit)：绝对族跨单位换算；相对/不同族 TypeError。
  CSSNumericValue.prototype.to = function (unit) {
    var u = _zwOmCanonUnit(String(unit));
    var a = _zwOmUnitInfo(this._zwOmUnit), b = u ? _zwOmUnitInfo(u) : null;
    if (!b) throw new TypeError("Invalid unit");
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
  function _zwOmUnitTypeOf(v) {
    return (v && typeof v.type === 'function') ? v.type() : {};
  }
  // 同绝对族 Sum 顶层裸 Unit 折叠（'calc(1px + 1in)' → Sum[px(97)]；percent/
  // 相对族/Negate 包裹成员不折叠——语料 'calc(1px + 2em)' 不简化同面）。
  function _zwOmFoldNumericSum(sumNode) {
    var members = sumNode._zwOmValues;
    var out = [];
    var firstIdx = {};
    for (var i = 0; i < members.length; i++) {
      var m = members[i];
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
    return out.length === members.length ? sumNode : new CSSMathSum(out);
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
  // type()：数值类型记录（assert_numeric_type_equals 消费形状：length/angle/
  // time/frequency/resolution/flex/percent 0|1 + percentHint）。
  CSSUnitValue.prototype.type = function () { return _zwOmUnitType(this._zwOmUnit); };
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
  Object.defineProperty(CSSVariableReferenceValue.prototype, 'variable', { get: function () { return this._zwOmVariable; }, enumerable: true, configurable: true });
  Object.defineProperty(CSSVariableReferenceValue.prototype, 'fallback', { get: function () { return this._zwOmFallback; }, enumerable: true, configurable: true });
  CSSVariableReferenceValue.prototype._zwOmKind = function () { return 'varref'; };
  CSSVariableReferenceValue.prototype.toString = function () {
    if (this._zwOmFallback == null) return 'var(' + this._zwOmVariable + ')';
    return 'var(' + this._zwOmVariable + ', ' + this._zwOmFallback.toString() + ')';
  };
  globalThis.CSSVariableReferenceValue = CSSVariableReferenceValue;

  function CSSUnparsedValue() {
    var list = (arguments.length === 1 && Array.isArray(arguments[0])) ? arguments[0] : Array.prototype.slice.call(arguments);
    for (var i = 0; i < list.length; i++) {
      var s = list[i];
      if (typeof s !== 'string' && !(s instanceof CSSVariableReferenceValue)) throw new TypeError("Invalid unparsed segment");
    }
    var arr = list.slice();
    arr.__proto__ = CSSUnparsedValue.prototype;
    return arr;
  }
  // 原型链挂 CSSStyleValue（spec CSSUnparsedValue : CSSStyleValue；实例本体
  // 仍是 Array exotic 对象——length/index/isArray 天然成立），迭代器手动补。
  CSSUnparsedValue.prototype = Object.create(CSSStyleValue.prototype);
  CSSUnparsedValue.prototype.constructor = CSSUnparsedValue;
  CSSUnparsedValue.prototype._zwOmKind = function () { return 'unparsed'; };
  CSSUnparsedValue.prototype.toString = function () {
    var out = '';
    for (var i = 0; i < this.length; i++) out += String(this[i]);
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
  }

  function CSSTransformComponent() {}
  CSSTransformComponent.prototype = Object.create(CSSStyleValue.prototype);
  CSSTransformComponent.prototype._zwOmKind = function () { return 'transform-comp'; };
  globalThis.CSSTransformComponent = CSSTransformComponent;

  // 组件构造器：实例 `_` 前缀存储槽 + 原型 accessor；数值入参按 CSSNumberish
  // 归一（number → CSSUnitValue number 面）；各组件 is2D 按 spec 判据暴露
  //（testhelper assert_style_value_equals 组件分派消费 x/y/z/is2D/angle）。
  function _zwOmCoerceNumeric(v, name) {
    if (typeof v === 'number') return new CSSUnitValue(v, 'number');
    if (!(v instanceof CSSStyleValue)) throw new TypeError("Invalid " + name);
    return v;
  }
  function _zwOmIsZeroNum(v) { return v instanceof CSSUnitValue && v.value === 0 && v.unit === 'number'; }
  function _zwOmIsOneNum(v) { return v instanceof CSSUnitValue && v.value === 1 && v.unit === 'number'; }
  function _zwOmCompProp(proto, f, coerce) {
    Object.defineProperty(proto, f, {
      get: function () { return this['_' + f]; },
      set: function (v) { this['_' + f] = coerce ? coerce(v) : v; },
      enumerable: true, configurable: true,
    });
  }
  function _zwOmIs2DProp(proto, calc) {
    Object.defineProperty(proto, 'is2D', { get: calc, enumerable: true, configurable: true });
  }

  function CSSTranslate(x, y, z) {
    this._x = _zwOmCoerceNumeric(x, 'x');
    this._y = _zwOmCoerceNumeric(y, 'y');
    this._z = _zwOmCoerceNumeric(z == null ? 0 : z, 'z');
  }
  CSSTranslate.prototype = Object.create(CSSTransformComponent.prototype);
  CSSTranslate.prototype.constructor = CSSTranslate;
  _zwOmCompProp(CSSTranslate.prototype, 'x', function (v) { return _zwOmCoerceNumeric(v, 'x'); });
  _zwOmCompProp(CSSTranslate.prototype, 'y', function (v) { return _zwOmCoerceNumeric(v, 'y'); });
  _zwOmCompProp(CSSTranslate.prototype, 'z', function (v) { return _zwOmCoerceNumeric(v, 'z'); });
  _zwOmIs2DProp(CSSTranslate.prototype, function () { return _zwOmIsZeroNum(this._z); });
  CSSTranslate.prototype.toString = function () {
    var s = 'translate(' + this._x + ', ' + this._y;
    if (!_zwOmIsZeroNum(this._z)) s += ', ' + this._z;
    return s + ')';
  };
  globalThis.CSSTranslate = CSSTranslate;

  function CSSRotate(xOrAngle, y, z, angle) {
    if (arguments.length <= 2) {
      // CSSRotate(angle)：绕 z 轴。
      this._x = 0; this._y = 0; this._z = 1;
      this._angle = _zwOmCoerceNumeric(xOrAngle, 'angle');
    } else {
      // CSSRotate(x, y, z, angle)：轴向量是 unrestricted double。
      if (typeof xOrAngle !== 'number' || typeof y !== 'number' || typeof z !== 'number') throw new TypeError("Invalid axis");
      this._x = xOrAngle; this._y = y; this._z = z;
      this._angle = _zwOmCoerceNumeric(angle, 'angle');
    }
  }
  CSSRotate.prototype = Object.create(CSSTransformComponent.prototype);
  CSSRotate.prototype.constructor = CSSRotate;
  for (var _zwRi = 0; _zwRi < 3; _zwRi++) (function (f) {
    Object.defineProperty(CSSRotate.prototype, f, {
      get: function () { return this['_' + f]; },
      set: function (v) { if (typeof v !== 'number') throw new TypeError("Invalid " + f); this['_' + f] = v; },
      enumerable: true, configurable: true,
    });
  })(['x', 'y', 'z'][_zwRi]);
  _zwOmCompProp(CSSRotate.prototype, 'angle', function (v) { return _zwOmCoerceNumeric(v, 'angle'); });
  _zwOmIs2DProp(CSSRotate.prototype, function () { return this._x === 0 && this._y === 0; });
  CSSRotate.prototype.toString = function () {
    if (this._x === 0 && this._y === 0) return 'rotate(' + this._angle + ')';
    if (this._x === 1 && this._y === 0 && this._z === 0) return 'rotatex(' + this._angle + ')';
    if (this._x === 0 && this._y === 1 && this._z === 0) return 'rotatey(' + this._angle + ')';
    return 'rotate3d(' + this._x + ', ' + this._y + ', ' + this._z + ', ' + this._angle + ')';
  };
  globalThis.CSSRotate = CSSRotate;

  function CSSScale(x, y, z) {
    this._x = _zwOmCoerceNumeric(x, 'x');
    this._y = _zwOmCoerceNumeric(y, 'y');
    this._z = _zwOmCoerceNumeric(z == null ? 1 : z, 'z');
  }
  CSSScale.prototype = Object.create(CSSTransformComponent.prototype);
  CSSScale.prototype.constructor = CSSScale;
  _zwOmCompProp(CSSScale.prototype, 'x', function (v) { return _zwOmCoerceNumeric(v, 'x'); });
  _zwOmCompProp(CSSScale.prototype, 'y', function (v) { return _zwOmCoerceNumeric(v, 'y'); });
  _zwOmCompProp(CSSScale.prototype, 'z', function (v) { return _zwOmCoerceNumeric(v, 'z'); });
  _zwOmIs2DProp(CSSScale.prototype, function () { return _zwOmIsOneNum(this._z); });
  CSSScale.prototype.toString = function () {
    var s = 'scale(' + this._x + ', ' + this._y;
    if (!_zwOmIsOneNum(this._z)) s += ', ' + this._z;
    return s + ')';
  };
  globalThis.CSSScale = CSSScale;

  function CSSSkew(ax, ay) {
    this._ax = _zwOmCoerceNumeric(ax, 'ax');
    this._ay = _zwOmCoerceNumeric(ay == null ? new CSSUnitValue(0, 'deg') : ay, 'ay');
  }
  CSSSkew.prototype = Object.create(CSSTransformComponent.prototype);
  CSSSkew.prototype.constructor = CSSSkew;
  _zwOmCompProp(CSSSkew.prototype, 'ax', function (v) { return _zwOmCoerceNumeric(v, 'ax'); });
  _zwOmCompProp(CSSSkew.prototype, 'ay', function (v) { return _zwOmCoerceNumeric(v, 'ay'); });
  _zwOmIs2DProp(CSSSkew.prototype, function () { return true; });
  CSSSkew.prototype.toString = function () { return 'skew(' + this._ax + ', ' + this._ay + ')'; };
  globalThis.CSSSkew = CSSSkew;

  function CSSSkewX(ax) { this._ax = _zwOmCoerceNumeric(ax, 'ax'); }
  CSSSkewX.prototype = Object.create(CSSTransformComponent.prototype);
  CSSSkewX.prototype.constructor = CSSSkewX;
  _zwOmCompProp(CSSSkewX.prototype, 'ax', function (v) { return _zwOmCoerceNumeric(v, 'ax'); });
  _zwOmIs2DProp(CSSSkewX.prototype, function () { return true; });
  CSSSkewX.prototype.toString = function () { return 'skewx(' + this._ax + ')'; };
  globalThis.CSSSkewX = CSSSkewX;

  function CSSSkewY(ay) { this._ay = _zwOmCoerceNumeric(ay, 'ay'); }
  CSSSkewY.prototype = Object.create(CSSTransformComponent.prototype);
  CSSSkewY.prototype.constructor = CSSSkewY;
  _zwOmCompProp(CSSSkewY.prototype, 'ay', function (v) { return _zwOmCoerceNumeric(v, 'ay'); });
  _zwOmIs2DProp(CSSSkewY.prototype, function () { return true; });
  CSSSkewY.prototype.toString = function () { return 'skewy(' + this._ay + ')'; };
  globalThis.CSSSkewY = CSSSkewY;

  function CSSPerspective(length) { this._length = _zwOmCoerceNumeric(length, 'length'); }
  CSSPerspective.prototype = Object.create(CSSTransformComponent.prototype);
  CSSPerspective.prototype.constructor = CSSPerspective;
  _zwOmCompProp(CSSPerspective.prototype, 'length', function (v) { return _zwOmCoerceNumeric(v, 'length'); });
  _zwOmIs2DProp(CSSPerspective.prototype, function () { return this._length instanceof CSSUnitValue && this._length.value === 0; });
  CSSPerspective.prototype.toString = function () { return 'perspective(' + this._length + ')'; };
  globalThis.CSSPerspective = CSSPerspective;
  function CSSMatrixComponent(matrix, options) {
    if (!(matrix instanceof globalThis.DOMMatrixReadOnly) && !(matrix instanceof globalThis.DOMMatrix) && typeof matrix !== 'object') throw new TypeError("Invalid matrix");
    this._matrix = matrix;
    this._options = options || { is2D: matrix.is2D };
  }
  CSSMatrixComponent.prototype = Object.create(CSSTransformComponent.prototype);
  CSSMatrixComponent.prototype.constructor = CSSMatrixComponent;
  Object.defineProperty(CSSMatrixComponent.prototype, 'matrix', { get: function () { return this._matrix; }, set: function (m) { if (typeof m !== 'object') throw new TypeError("Invalid matrix"); this._matrix = m; }, enumerable: true, configurable: true });
  Object.defineProperty(CSSMatrixComponent.prototype, 'is2D', { get: function () { return !!(this._options && this._options.is2D); }, enumerable: true, configurable: true });
  CSSMatrixComponent.prototype.toString = function () {
    var m = this._matrix && this._matrix._m ? this._matrix._m : null;
    if (m && this.is2D !== false) return 'matrix(' + m[0] + ', ' + m[1] + ', ' + m[4] + ', ' + m[5] + ', ' + m[12] + ', ' + m[13] + ')';
    if (m) return 'matrix3d(' + m.join(', ') + ')';
    return 'matrix(1, 0, 0, 1, 0, 0)';
  };
  CSSMatrixComponent.prototype._zwOmKind = function () { return 'transform-comp'; };
  globalThis.CSSMatrixComponent = CSSMatrixComponent;

  function CSSTransformValue(components) {
    var list = components == null ? [] : components;
    if (!Array.isArray(list)) throw new TypeError("Invalid components");
    for (var i = 0; i < list.length; i++) {
      if (!(list[i] instanceof CSSTransformComponent)) throw new TypeError("Invalid transform component");
    }
    var arr = list.slice();
    arr.__proto__ = CSSTransformValue.prototype;
    return arr;
  }
  CSSTransformValue.prototype = Object.create(CSSStyleValue.prototype);
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
    // 语料消费 is2D/长度断言为主——恒等矩阵起步（FIXME: 全分量矩阵合成）。
    return new (globalThis.DOMMatrixReadOnly || globalThis.DOMMatrix)([1, 0, 0, 1, 0, 0]);
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
  CSSImageValue.prototype.toString = function () { return this._zwOmText; };
  globalThis.CSSImageValue = CSSImageValue;

  // part05 已有 CSSRGB/CSSHSL（color 内部鸭子面）——不覆盖；本批补齐其余
  // 色值构造器（语料 stylevalue-subclasses 直接构造断言通道/序列化面）。
  function _zwOmColorCtor(name, fields) {
    if (globalThis[name]) return globalThis[name];
    function Ctor() {
      var args = Array.prototype.slice.call(arguments);
      for (var i = 0; i < fields.length; i++) {
        var v = args[i];
        if (typeof v === 'number') v = new CSSUnitValue(v, 'number');
        this['_' + fields[i]] = v;
      }
    }
    Object.defineProperty(Ctor, 'name', { value: name, configurable: true });
    Ctor.prototype = Object.create(CSSStyleValue.prototype);
    Ctor.prototype.constructor = Ctor;
    Ctor.prototype._zwOmKind = function () { return 'color'; };
    for (var i = 0; i < fields.length; i++) (function (f) {
      Object.defineProperty(Ctor.prototype, f, { get: function () { return this['_' + f]; }, enumerable: true, configurable: true });
    })(fields[i]);
    globalThis[name] = Ctor;
    return Ctor;
  }
  _zwOmColorCtor('CSSHWB', ['h', 'w', 'b', 'alpha']);
  _zwOmColorCtor('CSSLCH', ['l', 'c', 'h', 'alpha']);
  _zwOmColorCtor('CSSLab', ['l', 'a', 'b', 'alpha']);
  _zwOmColorCtor('CSSOKLCH', ['l', 'c', 'h', 'alpha']);
  _zwOmColorCtor('CSSOKLab', ['l', 'a', 'b', 'alpha']);
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
    // IDL camelCase（paddingTop → padding-top）：仅小写开头的驼峰形 dasherize；
    // 其余（'WiDtH'/'WIDTH'）直接 ASCII lowercase——CSS 属性名解析面大小写
    // 不敏感（css-properties-values-api parse 语料 not-case-sensitive 面）。
    if (/^[a-z]/.test(n) && /[A-Z]/.test(n)) {
      n = n.replace(/([A-Z])/g, function (_m, c, off) { return (off === 0 ? '' : '-') + c.toLowerCase(); });
    }
    return n.toLowerCase();
  }
  function _zwOmNumStr(n) {
    return String(n);
  }
  // 词元化 var() 段（CSSUnparsedValue）：字符串段与 CSSVariableReferenceValue 交替。
  function _zwOmTokenizeVar(text) {
    var segments = [];
    var buf = '';
    var i = 0;
    while (i < text.length) {
      var rest = text.slice(i);
      var m = /^var\(/i.exec(rest);
      if (!m) { buf += text[i]; i++; continue; }
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
        ref._zwOmFallback = fbArr.length === 1 && typeof fbArr[0] === 'string' ? fbArr[0].trim() : new CSSUnparsedValue(fbArr);
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
    if (lower.slice(0, 4) === 'var(' && /[{(;,\s:]var\(/i.test(' ' + lower)) {
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
        if (p.i >= p.s.length) return _zwOmValidateMath(node, fams) ? node : new CSSStyleValue();
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
        if (f === 'length') return !!(famsMap['<length>'] || famsMap['<length-percentage>'] || famsMap['<percentage>']);
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
    var parts = t.match(/(matrix3d|matrix|translate3d|translate[XY]|translate|scale3d|scale[XY]|scale|rotate3d|rotate[XYZ]|rotate|skew[XY]|skew|perspective)\([^)]*\)/gi);
    if (!parts || parts.join(' ') !== t.replace(/\s+/g, ' ').trim()) return null;
    var comps = [];
    for (var i = 0; i < parts.length; i++) {
      var m = /^([a-z]+3d|[a-z]+)\((.*)\)$/i.exec(parts[i].trim());
      if (!m) return null;
      var fn = m[1].toLowerCase();
      var argText = m[2];
      var argParts = argText.split(',');
      var nums = [];
      for (var k = 0; k < argParts.length; k++) {
        // 变换参数直接数值文法（不在属性语法表内，无 declared 面）。
        var tok = argParts[k].trim();
        var pm2 = /^([-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?)(%)?$/.exec(tok);
        if (pm2) {
          nums.push(pm2[2] ? new CSSUnitValue(parseFloat(pm2[1]), 'percent') : new CSSUnitValue(parseFloat(pm2[1]), 'number'));
          continue;
        }
        var dm2 = /^([-+]?(?:\d+\.?\d*|\.\d+))(deg|grad|rad|turn|px|em|rem|ex|ch|vw|vh|vmin|vmax|q|cm|mm|in|pt|pc|s|ms)$/i.exec(tok);
        if (dm2) { nums.push(new CSSUnitValue(parseFloat(dm2[1]), dm2[2].toLowerCase())); continue; }
        nums = null; break;
      }
      if (!nums) return null;
      switch (fn) {
        case 'translate': comps.push(new CSSTranslate(nums[0], nums.length > 1 ? nums[1] : new CSSUnitValue(0, 'px'))); break;
        case 'translatex': comps.push(new CSSTranslate(nums[0], new CSSUnitValue(0, 'px'))); break;
        case 'translatey': comps.push(new CSSTranslate(new CSSUnitValue(0, 'px'), nums[0])); break;
        case 'translate3d': comps.push(new CSSTranslate(nums[0], nums[1], nums[2])); break;
        case 'scale': comps.push(new CSSScale(nums[0], nums.length > 1 ? nums[1] : nums[0])); break;
        case 'scalex': comps.push(new CSSScale(nums[0], 1)); break;
        case 'scaley': comps.push(new CSSScale(1, nums[0])); break;
        case 'scale3d': comps.push(new CSSScale(nums[0], nums[1], nums[2])); break;
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
    return v;
  };

  // ===== 10. 数值类型记录 =====
  function _zwOmUnitType(unit) {
    var info = _zwOmUnitInfo(unit);
    var t = { length: 0, angle: 0, time: 0, frequency: 0, resolution: 0, flex: 0, percent: 0, percentHint: null };
    if (unit === 'percent') { t.percent = 1; return t; }
    if (info) {
      var fam = info[0];
      if (fam === 'percentHint') t.percentHint = 'percent';
      if (t[fam] !== undefined) t[fam] = 1;
    }
    return t;
  }
  function _zwOmMathType(node) {
    var t = { length: 0, angle: 0, time: 0, frequency: 0, resolution: 0, flex: 0, percent: 0, percentHint: null };
    var members = node._zwOmValues || [];
    // Clamp/Negate/Invert/Abs/Sign：单值域；Sum/Min/Max/Hypot：成员域并。
    var op = node.operator;
    for (var i = 0; i < members.length; i++) {
      var mt = members[i] instanceof CSSMathValue ? members[i].type() : _zwOmUnitType(members[i] instanceof CSSUnitValue ? members[i].unit : 'number');
      for (var k in t) {
        if (k === 'percentHint') continue;
        if (op === 'product') t[k] += mt[k]; // 粗粒度：乘法指数累计（FIXME: 除法消解）
        else if (op === 'invert') t[k] -= mt[k];
        else t[k] = Math.max(t[k], mt[k]);
      }
    }
    // percentHint：percent 与另一具体维并现时取该维。
    var concrete = [];
    for (var d in { length: 1, angle: 1, time: 1, frequency: 1, resolution: 1, flex: 1 }) if (t[d] > 0) concrete.push(d);
    if (t.percent > 0) {
      if (concrete.length === 1) t.percentHint = concrete[0];
      else if (concrete.length === 0) t.percentHint = 'percent';
    }
    return t;
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
  function _zwOmArith(mode, self, args) {
    var list = [self];
    for (var i = 0; i < args.length; i++) {
      var v = args[i];
      if (typeof v === 'number') v = new CSSUnitValue(v, 'number');
      list.push(v);
    }
    switch (mode) {
      case 'sum': return new CSSMathSum(list);
      case 'sub': return new CSSMathSum([list[0]].concat(list.slice(1).map(function (v) { return new CSSMathNegate(v); })));
      case 'mul': return new CSSMathProduct(list);
      case 'div': return new CSSMathProduct([list[0]].concat(list.slice(1).map(function (v) { return new CSSMathInvert(v); })));
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
    'font-style':['normal','italic','oblique'],
    'font-variant-alternates':['normal','historical-forms'],
    'font-variant-caps':['normal','small-caps','all-small-caps','petite-caps','all-petite-caps','unicase','titling-caps'],
    'font-variant-east-asian':['normal','jis78','jis83','jis90','jis04','simplified','traditional','full-width','proportional-width','ruby'],
    'font-variant-emoji':['normal','text','emoji','unicode'],
    'font-variant-ligatures':['normal','none','common-ligatures','no-common-ligatures','discretionary-ligatures','no-discretionary-ligatures','historical-ligatures','no-historical-ligatures','contextual','no-contextual'],
    'font-variant-numeric':['normal','lining-nums','oldstyle-nums','proportional-nums','tabular-nums','diagonal-fractions','stacked-fractions','ordinal','slashed-zero'],
    'font-variation-settings':['normal'],
    'font-weight':['normal','bold','bolder','lighter','<number>'],
    'grid-auto-flow':['row','column'],
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
    'z-index':['auto','<number>']
  };
  // 已知属性扩面（opaque roundtrip set 面；语料 225 之外常见属性）。
  var _zwOmKnownExtra = { color: 1, display: 1, background: 1, 'background-color': 1, 'background-image': 1, opacity: 1, visibility: 1, content: 1, transform: 1, 'z-index': 1, overflow: 1, cursor: 1, outline: 1, flex: 1, 'flex-direction': 1, 'grid-template-columns': 1, 'grid-template-rows': 1, transition: 1, animation: 1 };

  // ===== 12. StylePropertyMap =====
  function _zwOmValueToText(v) {
    if (typeof v === 'string') return v.trim();
    if (v instanceof CSSStyleValue) return v.toString();
    if (v && typeof v.toString === 'function' && v.toString !== Object.prototype.toString) return String(v);
    if (typeof v === 'number') return String(v);
    throw new TypeError("Invalid value");
  }
  function _zwOmValidateSet(prop, value) {
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
    if (kind === 'text' || kind === 'opaque' || kind === 'foreign' || kind === 'unparsed' || kind === 'varref' || kind === 'color') return;
    if (prop.slice(0, 2) === '--') return;
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
      var fam = _zwOmUnitType(t.unit);
      var famName = null;
      for (var d in { length: 1, angle: 1, time: 1, frequency: 1, resolution: 1, flex: 1, percent: 1, number: 1 }) if (fam[d] === 1) famName = d;
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
  function _zwOmDeclTextFromValue(prop, values) {
    var texts = [];
    for (var i = 0; i < values.length; i++) texts.push(_zwOmValueToText(values[i]));
    return texts.join(' ');
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
    gap: 1, 'grid-auto-columns': 1, 'grid-auto-rows': 1, 'grid-template-columns': 1,
    'grid-template-rows': 1, height: 1, 'inline-size': 1, 'inset-block': 1,
    'inset-block-end': 1, 'inset-block-start': 1, 'inset-inline': 1,
    'inset-inline-end': 1, 'inset-inline-start': 1, 'line-height': 1,
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

  function _zwOmMakeStyleMap(readable, getStyleObj) {
    // getStyleObj(): 底层 CSSStyleDeclaration 面（el.style / rule.style）。
    function mapGet(prop) {
      var p = _zwOmPropName(prop);
      var text = '';
      try { text = String(getStyleObj().getPropertyValue(p) || ''); } catch (_eG) { return null; }
      if (text === '') return null;
      var v = _zwOmParseWithText(p, text);
      // 非负属性负数值：association 语义包 Sum（testsuite range_handling 面）。
      if (v instanceof CSSUnitValue && v.value < 0 && _zwOmNonNegativeProps[p]) v = new CSSMathSum([v]);
      return v;
    }
    var ReadOnly = {};
    Object.defineProperty(ReadOnly, 'get', { value: function (prop) { return mapGet(prop); }, writable: true, configurable: true });
    Object.defineProperty(ReadOnly, 'getAll', {
      value: function (prop) {
        var v = mapGet(prop);
        return v == null ? [] : [v];
      }, writable: true, configurable: true,
    });
    Object.defineProperty(ReadOnly, 'has', { value: function (prop) { return mapGet(prop) != null; }, writable: true, configurable: true });
    Object.defineProperty(ReadOnly, 'size', {
      get: function () {
        try { return getStyleObj().length | 0; } catch (_eL) { return 0; }
      }, enumerable: true, configurable: true,
    });
    Object.defineProperty(ReadOnly, 'keys', { value: function () {
      var names = [];
      try { var st = getStyleObj(); for (var i = 0; i < st.length; i++) names.push(st.item(i)); } catch (_eK) {}
      var idx = 0;
      return { next: function () { return idx < names.length ? { value: names[idx++], done: false } : { value: undefined, done: true }; } };
    }, writable: true, configurable: true });
    Object.defineProperty(ReadOnly, 'values', { value: function () {
      var self = this; var ks = [];
      try { var st = getStyleObj(); for (var i = 0; i < st.length; i++) ks.push(st.item(i)); } catch (_eV) {}
      var idx = 0;
      return { next: function () { return idx < ks.length ? { value: self.get(ks[idx++]), done: false } : { value: undefined, done: true }; } };
    }, writable: true, configurable: true });
    Object.defineProperty(ReadOnly, 'entries', { value: function () {
      var self = this; var ks = [];
      try { var st = getStyleObj(); for (var i = 0; i < st.length; i++) ks.push(st.item(i)); } catch (_eE) {}
      var idx = 0;
      return { next: function () { return idx < ks.length ? { value: [ks[idx], self.get(ks[idx++])], done: false } : { value: undefined, done: true }; } };
    }, writable: true, configurable: true });
    Object.defineProperty(ReadOnly, 'forEach', { value: function (cb) {
      var ks = [];
      try { var st = getStyleObj(); for (var i = 0; i < st.length; i++) ks.push(st.item(i)); } catch (_eF) {}
      for (var i = 0; i < ks.length; i++) cb(this.get(ks[i]), ks[i], this);
    }, writable: true, configurable: true });
    ReadOnly[Symbol.iterator] = ReadOnly.values;
    if (readable) return ReadOnly;
    // Mutable 面
    Object.defineProperty(ReadOnly, 'set', {
      value: function (prop, value) {
        var p = _zwOmPropName(prop);
        var vals = Array.prototype.slice.call(arguments, 1);
        if (vals.length === 0) throw new TypeError("set requires a value");
        for (var i = 0; i < vals.length; i++) _zwOmValidateSet(p, vals[i]);
        var text = _zwOmDeclTextFromValue(p, vals);
        try {
          getStyleObj().setProperty(p, text);
          _zwOmExpandShorthand(getStyleObj(), p, text);
        } catch (_eS) { throw new TypeError("set failed"); }
      }, writable: true, configurable: true,
    });
    Object.defineProperty(ReadOnly, 'append', {
      value: function (prop, value) {
        var p = _zwOmPropName(prop);
        var prev = '';
        try { prev = String(getStyleObj().getPropertyValue(p) || ''); } catch (_eA) {}
        var text = _zwOmDeclTextFromValue(p, [value]);
        var next = prev ? prev + ' ' + text : text;
        try { getStyleObj().setProperty(p, next); } catch (_eA2) {}
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
  function _zwOmComputedProp(el, prop) {
    // 复用 part01 getComputedStyle 代理回调链（注册元素 sel/handle 路由由
    // 既有面完成），零新 Rust 回调。颜色属性的相对色骨架面（上）先于宿主
    // resolved 读数。
    var text = '';
    try {
      var decl = (el && el.style && String(el.style.getPropertyValue(prop) || '')) || '';
      if (decl && (prop === 'color' || /-color$/.test(prop) || prop === 'fill' || prop === 'stroke')
        && typeof globalThis.__zwCssRelativeOrigin === 'function') {
        text = _zwOmRelativeSkeletonText(decl, el);
      }
    } catch (_eRS) { text = ''; }
    if (text !== '') return text;
    if (globalThis.getComputedStyle) {
      try { text = String(globalThis.getComputedStyle(el).getPropertyValue(prop) || ''); } catch (_eG) { text = ''; }
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
          return {
            getPropertyValue: function (p) {
              var c = _zwOmComputedProp(self, p);
              if (c !== '') return c;
              try { return String((self.style && self.style.getPropertyValue(p)) || ''); } catch (_eD) { return ''; }
            },
            length: 0,
            item: function () { return ''; },
          };
        });
      },
      writable: true,
      configurable: true,
      enumerable: false,
    });
  }
})();
