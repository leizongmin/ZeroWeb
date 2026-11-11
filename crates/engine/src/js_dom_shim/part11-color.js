// part11-color.js — CSS Color 4/5 值模型（<color> 声明归一 + 计算值序列化）。
//
// DC-2 次批（wptfyi-css color 簇）：css-color/parsing 页族分两通道消费本模块——
// ① declared：`element.style.color` 写入归一（test_valid_value 期望 Chrome/Blink
//    规范序列化形）+ 非法值拒绝（test_invalid_value 期望赋值忽略）；② computed：
//    `getComputedStyle().color`（test_computed_value）——完整指定 legacy 形
//（named/hex/rgb/hsl/hwb）解析为 8 位 rgb()/rgba() 逗号形；带 `none` 分量的
// hsl/hwb 保留原函数形（css-color-4 §missing：缺失分量色不可序列化为 legacy 形，
// Blink 实测 `hsl(none none none)` 计算值保形）；rgb 恒消解（valid corpus 声明
// 规范化期望 `rgb(128 none none)` → `rgb(128, 0, 0)`；computed 面 none 传播由
// part03 raw 影子承担）；lab/lch/oklab/oklch/color() 恒保留各自空间形并归一分量
//（8 位有效——round-trip 同精度；hue 6 位——css-values-4 序列化，corpus
// 73.3386 精确面）。
// color-mix()/relative color/contrast-color()/light-dark() 本批**原文透传**
//（declared 保原串、computed 返 null 走既有 host 链——次批后续补，无回归）。
//
// 装配面：globalThis 双钩子（钩子缺席时调用方回落 T46 既有路径，零漂移）——
//   __zwColorNormalizeDecl(prop, value) → 规范串 | null（拒）
//   __zwComputedColorValue(elt, prop, declaredValue) → 计算串 | null（回落 host）
// 钩子由 part06 _zwNormalizeDeclValue（全部声明写面共享）与 part01
// getComputedStyle（has trap + color/opacity 内联臂）消费。
//
// 口径依据（Chrome 154 corpus oracle + Blink 154 活体）：
//   - named 保关键词小写形；hex → rgb8；rgb/rgba（legacy 逗号 + 现代空白）恒
//     8 位逗号形（含 none 消解——valid corpus 声明规范化面）；hsl/hwb 全指定 →
//     rgb8，带 none → 保留形（% 落数去 %）；
//   - alpha：=1 省略、<1 数值（legacy 形 3 位小数封顶——#FFFA → 0.667 钉面）、
//     ≤0 → 0、`none` 保留 ` / none`；
//   - calc() 声明面只折常量 + 乘法操作数排序（数→量→函数原子；
//     `sign(1em-10px) * 10%` → `10% * sign(1em - 10px)`，css-values-4 §calc
//     serialization），计算面全数值 calc 解包为裸数；NaN → 0；color() 通道
//     ±∞ 计算面保留（语料 `color(srgb 0 calc(infinity) 0)` 实证）。
(function () {
  'use strict';
  if (globalThis.__zwColorNormalizeDecl) return; // 幂等（重复装配零副作用）

  // ── CSS 命名色（css-color-4 §4，148 项；来源 color-computed-named-color 语料）──
  var _zwCNamed = {
    aliceblue: '240, 248, 255', antiquewhite: '250, 235, 215', aqua: '0, 255, 255',
    aquamarine: '127, 255, 212', azure: '240, 255, 255', beige: '245, 245, 220',
    bisque: '255, 228, 196', black: '0, 0, 0', blanchedalmond: '255, 235, 205',
    blue: '0, 0, 255', blueviolet: '138, 43, 226', brown: '165, 42, 42',
    burlywood: '222, 184, 135', cadetblue: '95, 158, 160', chartreuse: '127, 255, 0',
    chocolate: '210, 105, 30', coral: '255, 127, 80', cornflowerblue: '100, 149, 237',
    cornsilk: '255, 248, 220', crimson: '220, 20, 60', cyan: '0, 255, 255',
    darkblue: '0, 0, 139', darkcyan: '0, 139, 139', darkgoldenrod: '184, 134, 11',
    darkgray: '169, 169, 169', darkgreen: '0, 100, 0', darkgrey: '169, 169, 169',
    darkkhaki: '189, 183, 107', darkmagenta: '139, 0, 139',
    darkolivegreen: '85, 107, 47', darkorange: '255, 140, 0', darkorchid: '153, 50, 204',
    darkred: '139, 0, 0', darksalmon: '233, 150, 122', darkseagreen: '143, 188, 143',
    darkslateblue: '72, 61, 139', darkslategray: '47, 79, 79', darkslategrey: '47, 79, 79',
    darkturquoise: '0, 206, 209', darkviolet: '148, 0, 211', deeppink: '255, 20, 147',
    deepskyblue: '0, 191, 255', dimgray: '105, 105, 105', dimgrey: '105, 105, 105',
    dodgerblue: '30, 144, 255', firebrick: '178, 34, 34', floralwhite: '255, 250, 240',
    forestgreen: '34, 139, 34', fuchsia: '255, 0, 255', gainsboro: '220, 220, 220',
    ghostwhite: '248, 248, 255', gold: '255, 215, 0', goldenrod: '218, 165, 32',
    gray: '128, 128, 128', green: '0, 128, 0', greenyellow: '173, 255, 47',
    grey: '128, 128, 128', honeydew: '240, 255, 240', hotpink: '255, 105, 180',
    indianred: '205, 92, 92', indigo: '75, 0, 130', ivory: '255, 255, 240',
    khaki: '240, 230, 140', lavender: '230, 230, 250', lavenderblush: '255, 240, 245',
    lawngreen: '124, 252, 0', lemonchiffon: '255, 250, 205', lightblue: '173, 216, 230',
    lightcoral: '240, 128, 128', lightcyan: '224, 255, 255',
    lightgoldenrodyellow: '250, 250, 210', lightgray: '211, 211, 211',
    lightgreen: '144, 238, 144', lightgrey: '211, 211, 211', lightpink: '255, 182, 193',
    lightsalmon: '255, 160, 122', lightseagreen: '32, 178, 170',
    lightskyblue: '135, 206, 250', lightslategray: '119, 136, 153',
    lightslategrey: '119, 136, 153', lightsteelblue: '176, 196, 222',
    lightyellow: '255, 255, 224', lime: '0, 255, 0', limegreen: '50, 205, 50',
    linen: '250, 240, 230', magenta: '255, 0, 255', maroon: '128, 0, 0',
    mediumaquamarine: '102, 205, 170', mediumblue: '0, 0, 205',
    mediumorchid: '186, 85, 211', mediumpurple: '147, 112, 219',
    mediumseagreen: '60, 179, 113', mediumslateblue: '123, 104, 238',
    mediumspringgreen: '0, 250, 154', mediumturquoise: '72, 209, 204',
    mediumvioletred: '199, 21, 133', midnightblue: '25, 25, 112',
    mintcream: '245, 255, 250', mistyrose: '255, 228, 225', moccasin: '255, 228, 181',
    navajowhite: '255, 222, 173', navy: '0, 0, 128', oldlace: '253, 245, 230',
    olive: '128, 128, 0', olivedrab: '107, 142, 35', orange: '255, 165, 0',
    orangered: '255, 69, 0', orchid: '218, 112, 214', palegoldenrod: '238, 232, 170',
    palegreen: '152, 251, 152', paleturquoise: '175, 238, 238',
    palevioletred: '219, 112, 147', papayawhip: '255, 239, 213', peachpuff: '255, 218, 185',
    peru: '205, 133, 63', pink: '255, 192, 203', plum: '221, 160, 221',
    powderblue: '176, 224, 230', purple: '128, 0, 128', rebeccapurple: '102, 51, 153',
    red: '255, 0, 0', rosybrown: '188, 143, 143', royalblue: '65, 105, 225',
    saddlebrown: '139, 69, 19', salmon: '250, 128, 114', sandybrown: '244, 164, 96',
    seagreen: '46, 139, 87', seashell: '255, 245, 238', sienna: '160, 82, 45',
    silver: '192, 192, 192', skyblue: '135, 206, 235', slateblue: '106, 90, 205',
    slategray: '112, 128, 144', slategrey: '112, 128, 144', snow: '255, 250, 250',
    springgreen: '0, 255, 127', steelblue: '70, 130, 180', tan: '210, 180, 140',
    teal: '0, 128, 128', thistle: '216, 191, 216', tomato: '255, 99, 71',
    turquoise: '64, 224, 208', violet: '238, 130, 238', wheat: '245, 222, 179',
    white: '255, 255, 255', whitesmoke: '245, 245, 245', yellow: '255, 255, 0',
    yellowgreen: '154, 205, 50'
  };
  // 系统色（css-color-4 §6.3）：declared 保关键词小写；computed 返 null 走 host
  //（Chrome 映射调色板随 theme 变，未建表前不误答）。
  var _zwCSystem = {
    canvas: 1, canvastext: 1, linktext: 1, visitedtext: 1, activetext: 1,
    buttonface: 1, buttontext: 1, buttonborder: 1, field: 1, fieldtext: 1,
    highlight: 1, highlighttext: 1, selecteditem: 1, selecteditemtext: 1,
    mark: 1, marktext: 1, graytext: 1, accentcolor: 1, accentcolortext: 1,
    activeborder: 1, activecaption: 1, appworkspace: 1, background: 1,
    buttonhighlight: 1, buttonshadow: 1, captiontext: 1, inactiveborder: 1,
    inactivecaption: 1, inactivecaptiontext: 1, infobackground: 1, infotext: 1,
    menu: 1, menutext: 1, scrollbar: 1, threeddarkshadow: 1, threedface: 1,
    threedhighlight: 1, threedlightshadow: 1, threedshadow: 1, window: 1,
    windowframe: 1, windowtext: 1
  };
  var _zwCCssWide = { inherit: 1, initial: 1, unset: 1, revert: 1, 'revert-layer': 1 };

  // 计算面容器查询单位基座（css-conditional-5 §container-lengths）：cq 单位按
  // 最近 container-type ≠ normal 祖先的 content 尺寸求值。__zwComputedColorValue
  // 入口按元素探明并置入（null=无元素/未探明 → 调用方保既有视口派生兜底）。
  // FIXME: 容器宽 auto（无显式 px 声明）需布局 used 值，现探不明即兜底；
  // cqh 高度轴与具名容器（container-name 选择性查询）未建模——corpus 桶 1 均
  // cqw+显式 px 容器，随实证面扩。
  var _zwCCqwBase = null;
  function _zwCContainerInlineSize(elt) {
    try {
      var a = elt && elt.nodeType === 1 ? elt.parentNode : null;
      var gcs = globalThis.getComputedStyle;
      while (a && a.nodeType === 1) {
        var ct = '';
        try { ct = String(gcs ? (gcs(a).getPropertyValue('container-type') || '') : ''); } catch (_eCT) { ct = ''; }
        if (ct && ct !== 'normal') {
          var w = '';
          try { w = String(gcs ? (gcs(a).getPropertyValue('width') || '') : ''); } catch (_eCW) { w = ''; }
          var m = /^\s*([+-]?\d*\.?\d+)px\s*$/i.exec(w);
          if (m) {
            var fv = parseFloat(m[1]);
            if (isFinite(fv)) return fv;
          }
          return null; // 容器命中但内联尺寸探不明：fail-closed，不虚构
        }
        a = a.parentNode;
      }
    } catch (_eCI) { /* 原型链/分离节点异常 → 兜底 */ }
    return null;
  }

  // CSS 标识符大小写折叠仅 ASCII（css-syntax §3.3.10；Unicode 同形字不折叠——
  // corpus invalid-named 钉 U+212A KELVIN SIGN 形 blacK/darKblue 必须拒识别）。
  function _zwCAsciiLower(s) {
    return String(s).replace(/[A-Z]/g, function (c) { return String.fromCharCode(c.charCodeAt(0) + 32); });
  }
  // 括号感知的顶层扫描：calc(min(..) / 3) 内的 '/'、空格、逗号不参与分割
  //（css-color-4 函数参数在括号深度 0 按语法切分）。
  function _zwCSplitTopSlash(s) {
    var depth = 0, out = [''];
    for (var i = 0; i < s.length; i++) {
      var ch = s.charAt(i);
      if (ch === '(') depth++;
      else if (ch === ')') depth--;
      if (depth === 0 && ch === '/') { out.push(''); continue; }
      out[out.length - 1] += ch;
    }
    return out;
  }
  function _zwCSplitTopWs(s) {
    var depth = 0, out = [], cur = '';
    for (var i = 0; i < s.length; i++) {
      var ch = s.charAt(i);
      if (ch === '(') depth++;
      else if (ch === ')') depth--;
      if (depth === 0 && /\s/.test(ch)) { if (cur) { out.push(cur); cur = ''; } continue; }
      cur += ch;
    }
    if (cur) out.push(cur);
    return out;
  }
  function _zwCHasTopComma(s) {
    var depth = 0;
    for (var i = 0; i < s.length; i++) {
      var ch = s.charAt(i);
      if (ch === '(') depth++;
      else if (ch === ')') depth--;
      else if (depth === 0 && ch === ',') return true;
    }
    return false;
  }

  // ── 数值格式化 ──
  // 通用：整数直书；小数 6 位封顶去尾零（corpus 0.1/0.25/62.5 形）。
  function _zwCNum(n) {
    if (isNaN(n)) return 'NaN';
    if (!isFinite(n)) return n > 0 ? 'infinity' : '-infinity'; // css-values-4 规范形（corpus calc(infinity)）
    var r = Math.round(n * 1e6) / 1e6;
    return String(r);
  }
  // hue 分量：6 位有效数字（css-values-4 number serialization；corpus 1.28rad → 73.3386）。
  function _zwCSig6(n) {
    if (isNaN(n)) return 'NaN';
    if (!isFinite(n)) return n > 0 ? 'infinity' : '-infinity';
    return String(Number(n.toPrecision(6)));
  }
  // color() 通道：8 位有效数字（corpus rgb(none…) computed → color(srgb 0.50196078 …)）。
  function _zwCSig8(n) {
    if (isNaN(n)) return 'NaN';
    if (!isFinite(n)) return n > 0 ? 'infinity' : '-infinity';
    return String(Number(n.toPrecision(8)));
  }
  // legacy rgba alpha：8-bit round-trip 最短十进制（css-color-4 §16.1.1——序列化值
  // ×255 取整须回到同一 8-bit alpha 通道；#FFFA→0.667、#FF000080→0.5、
  // #FF000001→0.004，Blink T46 钉面 + corpus rgba(…, 0.52) 计算面同规）。
  function _zwCAlpha8(n) {
    var a8 = Math.round(n * 255);
    for (var d = 1; d <= 6; d++) {
      var txt = String(Number(n.toFixed(d)));
      if (Math.round(parseFloat(txt) * 255) === a8) return txt;
    }
    return String(Number(n.toPrecision(6)));
  }

  // ── 角度归一：deg 数值（grad/rad/turn 换算，css-color-4 §7 hue）──
  function _zwCAngleToDeg(n, unit) {
    if (unit === 'grad') return n * 0.9;
    if (unit === 'rad') return n * 180 / Math.PI;
    if (unit === 'turn') return n * 360;
    return n; // deg 或无单位
  }

  // lab 族百分比标度（css-color-4 §13/§14：lab L%×1 a/b%×1.25；lch C%×1.5；
  // oklab/oklch L%×0.01 C/a/b%×0.004；hue 槽已换算 deg 无标度）。
  function _zwCLabScale(form, oi) {
    if (form === 'lab') return oi === 0 ? 1 : 1.25;
    if (form === 'lch') return oi === 1 ? 1.5 : 1;
    return oi === 0 ? 0.01 : (oi === 2 && form === 'oklch' ? 1 : 0.004);
  }

  // ── HSL/HWB → sRGB（0-255，css-color-4 §9/§10；Math.round 8 位取整）──
  // hue NaN/±∞ → 0（css-color-4：NaN hue 按 0 处理——corpus hsl(calc(infinity)…) → 红）。
  function _zwCHslToRgb(h, s, l) {
    if (!isFinite(h) || isNaN(h)) h = 0;
    if (isNaN(s)) s = 0;
    if (isNaN(l)) l = 0;
    h = ((h % 360) + 360) % 360;
    s = Math.max(0, Math.min(100, s)) / 100;
    l = Math.max(0, Math.min(100, l)) / 100;
    var c = (1 - Math.abs(2 * l - 1)) * s;
    var x = c * (1 - Math.abs((h / 60) % 2 - 1));
    var m = l - c / 2;
    var r = 0, g = 0, b = 0;
    if (h < 60) { r = c; g = x; } else if (h < 120) { r = x; g = c; }
    else if (h < 180) { g = c; b = x; } else if (h < 240) { g = x; b = c; }
    else if (h < 300) { r = x; b = c; } else { r = c; b = x; }
    return [Math.round((r + m) * 255), Math.round((g + m) * 255), Math.round((b + m) * 255)];
  }
  // HWB：纯 hue 色按 60° 扇区直算（css-color-4 §10；不再借道 hsl——corpus
  // hwb(90 20% 10%) → rgb(140, 230, 51) 钉 140.25 取整面）。
  // 全程 Math.fround 模拟 Chrome/Skia float32 内部精度：半整通道（0.3f+0.2f
  // 相消出精确 0.5f → 127.5 → 128）在 double 链路是 127.49…→ 127，必须对齐
  //（corpus hwb(120 30% 50%) → rgb(77, 128, 77) 钉）。
  function _zwCHwbToRgb(h, w, b) {
    var fr = Math.fround;
    if (!isFinite(h) || isNaN(h)) h = 0;
    if (isNaN(w)) w = 0;
    if (isNaN(b)) b = 0;
    h = ((h % 360) + 360) % 360;
    w = fr(Math.max(0, Math.min(100, w)) / 100);
    b = fr(Math.max(0, Math.min(100, b)) / 100);
    if (w + b >= 1) {
      var g8 = Math.round(w / (w + b) * 255);
      return [g8, g8, g8];
    }
    var hp = h / 60, i = Math.floor(hp) % 6, f = fr(hp - Math.floor(hp));
    var pr = 0, pg = 0, pb = 0;
    if (i === 0) { pr = 1; pg = f; }
    else if (i === 1) { pr = fr(1 - f); pg = 1; }
    else if (i === 2) { pg = 1; pb = f; }
    else if (i === 3) { pg = fr(1 - f); pb = 1; }
    else if (i === 4) { pr = f; pb = 1; }
    else { pr = 1; pb = fr(1 - f); }
    var k = fr(fr(1 - w) - b);
    // *255 后再 fround：float32 里 229.49999… 舍入到精确 229.5f，取整半值
    // 进位才得 230（corpus hwb(90 20% 10%) → rgb(140, 230, 51) 钉）。
    return [Math.round(fr(fr(fr(pr * k) + w) * 255)), Math.round(fr(fr(fr(pg * k) + w) * 255)), Math.round(fr(fr(fr(pb * k) + w) * 255))];
  }

  // 纯数值四则表达式求值（sign() 计算面专用；输入已验证仅 [0-9+-*/().\s]）。
  // 返回数值或 null（文法不合）。禁 Function/eval（沙箱 CSP 面）。
  function _zwCArith(str) {
    var s = String(str), i = 0, n = s.length;
    function ws() { while (i < n && /\s/.test(s.charAt(i))) i++; }
    function num() {
      ws();
      var st = i;
      while (i < n && /[0-9.]/.test(s.charAt(i))) i++;
      if (i === st) return null;
      var v = parseFloat(s.slice(st, i));
      return isNaN(v) ? null : v;
    }
    function atom() {
      ws();
      if (s.charAt(i) === '(') {
        i++;
        var v = expr();
        ws();
        if (s.charAt(i) !== ')') return null;
        i++;
        return v;
      }
      if (s.charAt(i) === '-') { i++; var a = atom(); return a == null ? null : -a; }
      if (s.charAt(i) === '+') { i++; return atom(); }
      return num();
    }
    function prod() {
      var a = atom();
      if (a == null) return null;
      for (;;) {
        ws();
        var c = s.charAt(i);
        if (c === '*' || c === '/') {
          i++;
          var b = atom();
          if (b == null) return null;
          a = c === '*' ? a * b : a / b;
        } else return a;
      }
    }
    function expr() {
      var a = prod();
      if (a == null) return null;
      for (;;) {
        ws();
        var c = s.charAt(i);
        if (c === '+' || c === '-') {
          i++;
          var b = prod();
          if (b == null) return null;
          a = c === '+' ? a + b : a - b;
        } else return a;
      }
    }
    var v = expr();
    ws();
    return i >= n ? v : null;
  }
  // 只处理完全数值/百分比的子树（常量折叠）；含函数原子（sign/var/…) 的项
  // 保形并做操作数排序（数 → 量(%/deg) → 原子）。calc/min/max/clamp 前缀统一
  // 入口；min/max/clamp 全数值参数折叠为该函数的解（declared 面仍保 calc() 包裹）。
  // corpus 形：
  //   calc(0.5 + 1) → calc(1.5)        calc(50% * 3) → calc(150%)
  //   calc(0 / 0)   → calc(NaN)        calc(NaN)     → calc(NaN)
  //   clamp(50%, 0%, 70%) → calc(50%)  min(0%, 50%) → calc(0%)
  //   calc(50% + (sign(1em - 10px) * 10%)) → calc(50% + (10% * sign(1em - 10px)))
  // 返回 { kind: 'num'|'raw'|'calc'|'invalid', v, text }：num=全数值（v 已折），
  // invalid=文法/单位非法（整条声明拒），raw=不可解析保原串。
  function _zwCCalcFold(src, evalAtoms) {
    var toks = [];
    var re = /(\d*\.\d+|\d+\.?\d*)(%|deg|grad|rad|turn|em|px|rem|vw|vh|s|ms|cqw|cqh|cqi)?|([Nn][Aa][Nn]|[Ii]nfinity)|([+\-*\/(),])|([A-Za-z][A-Za-z0-9-]*)/g;
    var m;
    var s = String(src).trim();
    var pm = /^(calc|min|max|clamp)\(/.exec(_zwCAsciiLower(s));
    if (!pm) return null;
    var fname = pm[1];
    var body = s.slice(pm[0].length, s.lastIndexOf(')') > 0 ? s.lastIndexOf(')') : s.length);
    re.lastIndex = 0;
    while ((m = re.exec(body)) !== null) {
      if (m[1] !== undefined) toks.push({ t: 'q', v: parseFloat(m[1]), u: m[2] || '' });
      else if (m[3] !== undefined) toks.push({ t: 'atom', v: m[3].toLowerCase() === 'nan' ? 'NaN' : 'Infinity' });
      else if (m[4] !== undefined) toks.push({ t: m[4] });
      else if (m[5] !== undefined) toks.push({ t: 'id', v: m[5] });
    }
    // 完全 token 化检查（无法解析的记号 → 原样保留）。
    var probe = body.replace(/(\d*\.\d+|\d+\.?\d*)(%|deg|grad|rad|turn|em|px|rem|vw|vh|s|ms|cqw|cqh|cqi)?/g, '@')
      .replace(/[Nn][Aa][Nn]|[Ii]nfinity/g, '@').replace(/[+\-*\/(),]/g, '@').replace(/[A-Za-z][A-Za-z0-9-]*/g, '@')
      .replace(/[\s@]/g, '');
    if (probe !== '') return { kind: 'raw', text: s };
    var pos = 0;
    function peek() { return toks[pos]; }
    function next() { return toks[pos++]; }
    // atom：数/量/NaN/Infinity/函数原子（id + 括号平衡子串原样）。
    function parseAtom() {
      var tk = peek();
      if (!tk) return null;
      if (tk.t === 'q') { next(); return { kind: 'num', v: tk.v, u: tk.u }; }
      if (tk.t === 'atom') { next(); return tk.v === 'NaN' ? { kind: 'num', v: NaN, u: '' } : { kind: 'num', v: tk.v === 'Infinity' ? Infinity : -Infinity, u: '' }; }
      if (tk.t === 'id') {
        // 函数/关键字原子：抓括号平衡原文（sign(...) / var(...) / alpha 等）。
        var name = tk.v; next();
        if (peek() && peek().t === '(') {
          var depth = 0, start = pos;
          do {
            var t2 = next();
            if (!t2) return null;
            if (t2.t === '(') depth++;
            else if (t2.t === ')') depth--;
          } while (depth > 0);
          // 原子内部（如 sign(1em - 10px)）整体保形——不含首尾括号
          //（start 指向 '('，pos-1 为配对 ')'）。
          var innerToks = toks.slice(start + 1, pos - 1);
          var atomText = name + '(' + _zwCJoinRaw(innerToks) + ')';
          // 计算面（evalAtoms）：sign(<数值差>) 求值——目标 font-size 缺省 16px
          //（corpus sign(1em - 10px) 形；cqw 等容器单位不在此列，保形留给后续）。
          if (evalAtoms && name === 'sign') {
            var inner = _zwCJoinRaw(innerToks)
              .replace(/(\d*\.?\d+)em/g, function (_, n) { return String(parseFloat(n) * 16); })
              .replace(/(\d*\.?\d+)px/g, function (_, n) { return n; })
              .replace(/(\d*\.?\d+)%/g, function (_, n) { return n; })
              // 容器查询单位：计算面容器基座在手（__zwComputedColorValue 探明
              // 最近 container-type ≠ normal 祖先内联尺寸）按 cqw = N·W/100 求值
              //（css-conditional-5 §container-lengths；cqi 水平书写 ≡ cqw）；
              // 无基座（declared 面/未探明）保视口派生大正值兜底
              //（corpus sign(2cqw - 10px) → +1）。cqh 高度轴未建模同兜底。
              .replace(/(\d*\.?\d+)(cqw|cqh|cqi)/g, function (_, n, u) {
                if (_zwCCqwBase != null && (u === 'cqw' || u === 'cqi')) {
                  return String(parseFloat(n) * _zwCCqwBase / 100);
                }
                return String(parseFloat(n) * 1000);
              });
            if (!/[^0-9+\-*\/().\s]/.test(inner) && !/%/.test(inner)) {
              var sv = _zwCArith(inner);
              if (sv != null && isFinite(sv)) return { kind: 'num', v: sv > 0 ? 1 : sv < 0 ? -1 : 0, u: '' };
            }
          }
          return { kind: 'atom', text: atomText };
        }
        return { kind: 'atom', text: name };
      }
      return null;
    }
    function _zwCJoinRaw(ts) {
      var out = '';
      for (var i = 0; i < ts.length; i++) {
        var t = ts[i];
        if (t.t === 'q') out += _zwCNum(t.v) + t.u;
        else if (t.t === 'atom') out += t.v;
        else out += t.t;
        if (i + 1 < ts.length) {
          var a = t, b = ts[i + 1];
          var needSp = !(a.t === '(' || b.t === ')' || b.t === ',' ||
            (b.t === 'q' && (a.t === '+' || a.t === '-' || a.t === '*' || a.t === '/')) ||
            (a.t === 'q' && (b.t === 'q')));
          if (needSp && !(b.t === 'q' && (a.t === 'q' || a.t === 'atom')) ) {
            // 二元操作符两侧保空格；单目量不补。
            if (!(a.t === '+' || a.t === '-' || a.t === '*' || a.t === '/' || b.t === '+' || b.t === '-' || b.t === '*' || b.t === '/')) out += '';
          }
          if ((a.t === '+' || a.t === '-' || a.t === '*' || a.t === '/' || b.t === '+' || b.t === '-' || b.t === '*' || b.t === '/' || b.t === '(')) out += ' ';
        }
      }
      return out.replace(/\(\s+/g, '(').replace(/\s+\)/g, ')');
    }
    function parsePow() { // 一元 +/- 与括号子表达式
      var tk = peek();
      var sign = 1;
      while (tk && (tk.t === '+' || tk.t === '-')) { if (tk.t === '-') sign = -sign; next(); tk = peek(); }
      if (tk && tk.t === '(') {
        next();
        var inner = parseSum();
        if (!inner) return null;
        if (!peek() || peek().t !== ')') return null;
        next();
        if (inner.kind === 'num') inner.v = inner.v * sign;
        else if (sign < 0) return { kind: 'paren', neg: true, t: inner };
        return inner;
      }
      var a = parseAtom();
      if (!a) return null;
      if (a.kind === 'num') a.v = a.v * sign;
      else if (sign < 0) a.text = '-' + a.text;
      return a;
    }
    function parseProd() {
      var first = parsePow();
      if (!first) return null;
      var factors = [{ f: first, op: null }];
      while (peek() && (peek().t === '*' || peek().t === '/')) {
        var op = next().t;
        var fct = parsePow();
        if (!fct) return null;
        factors.push({ f: fct, op: op });
      }
      return { kind: 'prod', factors: factors };
    }
    function parseSum() {
      var first = parseProd();
      if (!first) return null;
      var terms = [{ t: first, op: null }];
      while (peek() && (peek().t === '+' || peek().t === '-')) {
        var op = next().t;
        var trm = parseProd();
        if (!trm) return null;
        terms.push({ t: trm, op: op });
      }
      return { kind: 'sum', terms: terms };
    }
    function foldProd(p) {
      // 先折叠括号组因子（parsePow 括号支返回 sum AST）：全数值则归约，
      // 部分折叠保括号组裸形（无 calc 包裹，corpus `calc(50% + (10% * sign(...)))`）。
      for (var pi = 0; pi < p.factors.length; pi++) {
        var pf = p.factors[pi].f;
        if (pf && pf.kind === 'sum') {
          var pfs = foldSum(pf);
          if (pfs.kind === 'invalid') return { kind: 'invalid' };
          if (pfs.kind === 'num') p.factors[pi].f = pfs;
          else if (pfs.kind === 'calc') p.factors[pi].f = { kind: 'paren', t: { kind: 'sumtext', terms: pfs.terms } };
          else p.factors[pi].f = { kind: 'atom', text: pfs.text };
        }
      }
      // 全数值：折叠；单位×单位/单位÷单位非法（css-values-4；corpus turn*turn 拒）。
      // 含原子/括号节点：排序（数 → 量(%/deg…) → 原子/括号；除号保持左位）。
      var allNum = true;
      for (var i = 0; i < p.factors.length; i++) {
        if (p.factors[i].f.kind !== 'num') { allNum = false; break; }
      }
      if (allNum) {
        var v = p.factors[0].f.v, u = p.factors[0].f.u;
        for (var j = 1; j < p.factors.length; j++) {
          var ft = p.factors[j];
          if (u && ft.f.u) return { kind: 'invalid' };
          if (ft.op === '*') { v *= ft.f.v; if (!u) u = ft.f.u; }
          else { v /= ft.f.v; if (ft.f.u) u = ''; } // 除以带单位量 → 无单位；除以纯数保单位
        }
        return { kind: 'num', v: v, u: u };
      }
      var idx = [], k;
      for (k = 0; k < p.factors.length; k++) idx.push(k);
      // 含除号不排序（除法左结合，换位变语义）；纯乘积按数 → 量 → 原子排。
      var hasDiv = false;
      for (k = 0; k < p.factors.length; k++) if (p.factors[k].op === '/') { hasDiv = true; break; }
      if (!hasDiv) {
        idx.sort(function (a, b) {
          var fa = p.factors[a], fb = p.factors[b];
          var ra = fa.f.kind === 'num' ? (fa.f.u ? 1 : 0) : 2;
          var rb = fb.f.kind === 'num' ? (fb.f.u ? 1 : 0) : 2;
          return ra - rb;
        });
      }
      var sorted = [];
      for (k = 0; k < idx.length; k++) sorted.push(p.factors[idx[k]]);
      return { kind: 'prodtext', factors: sorted };
    }
    function _zwCNodeText(nd) {
      if (!nd) return '';
      switch (nd.kind) {
        case 'num': return _zwCNum(nd.v) + (nd.u || '');
        case 'atom': return nd.text;
        case 'paren': return (nd.neg ? '-' : '') + '(' + _zwCNodeText(nd.t) + ')';
        case 'prod': case 'prodtext': return prodText(nd);
        case 'sum': case 'sumtext': return _zwCRenderSum(nd.terms);
        default: return '';
      }
    }
    function prodText(p) {
      var out = '';
      for (var i = 0; i < p.factors.length; i++) {
        var fct = p.factors[i];
        var txt = _zwCNodeText(fct.f);
        if (fct.f.kind === 'num' && fct.f.v < 0 && i > 0) txt = '(' + txt + ')';
        // 排序后 op 跟随原因子（首因子 op=null）：取右因子，null 回退左因子。
        if (i > 0) out += ' ' + (fct.op != null ? fct.op : p.factors[i - 1].op) + ' ';
        out += txt;
      }
      return out;
    }
    function foldSum(s) {
      var allNum = true, i;
      for (i = 0; i < s.terms.length; i++) {
        if (s.terms[i].t.kind === 'prod') {
          var folded = foldProd(s.terms[i].t);
          if (folded.kind === 'invalid') return folded;
          s.terms[i].t = folded;
          if (folded.kind !== 'num') allNum = false;
        } else if (s.terms[i].t.kind !== 'num') allNum = false;
      }
      // sum 项排序（css-values-4：数 → 量 → 原子，稳定）——仅全 '+' 连接
      //（减号换位变语义，corpus 未证重排，保持原序）。
      if (s.terms.length > 1) {
        var okSort = true;
        for (i = 1; i < s.terms.length; i++) if (s.terms[i].op !== '+') { okSort = false; break; }
        if (okSort) {
          function rank(nd) { return nd.kind === 'num' ? (nd.u ? 1 : 0) : 2; }
          s.terms.sort(function (a, b) { return rank(a.t) - rank(b.t); });
        }
      }
      if (allNum) {
        var v = s.terms[0].t.v, u = s.terms[0].t.u;
        for (var j = 1; j < s.terms.length; j++) {
          var tm = s.terms[j];
          var tv = tm.t.v, tu = tm.t.u;
          if (u !== tu) return { kind: 'calc', text: 'calc(' + _zwCRenderSum(s.terms) + ')', terms: s.terms }; // 单位混算不折
          v = tm.op === '+' ? v + tv : v - tv;
        }
        return { kind: 'num', v: v, u: u };
      }
      return { kind: 'calc', text: 'calc(' + _zwCRenderSum(s.terms) + ')', terms: s.terms };
    }
    function _zwCRenderSum(terms) {
      var out = '';
      for (var i = 0; i < terms.length; i++) {
        var tm = terms[i];
        var txt = _zwCNodeText(tm.t);
        // 多 term 的 sum 内多因子积加括号（css-values-4 serialization；corpus
        // `50 + (10 * sign(...))` 形）；单 term 恒等（corpus `calc(0.5 * alpha)` 无括号）。
        if (terms.length > 1 && (tm.t.kind === 'prodtext' || tm.t.kind === 'prod') && tm.t.factors.length > 1) txt = '(' + txt + ')';
        // 排序后 op 跟随原因子（首项 op=null）：null 回退左项 op。
        if (i > 0) out += ' ' + (tm.op != null ? tm.op : terms[i - 1].op) + ' ';
        out += txt;
      }
      return out;
    }
    var root = parseSum();
    if (!root) return { kind: 'raw', text: s };
    if (fname === 'calc') {
      if (pos !== toks.length) return { kind: 'raw', text: s };
      var foldedSum = foldSum(root);
      if (foldedSum.kind === 'invalid') return null;
      if (foldedSum.kind === 'num') return foldedSum;
      if (foldedSum.kind === 'raw') return foldedSum;
      return foldedSum; // foldSum 已产出渲染好的 {kind:'calc', text:'calc(...)'}
    }
    // min/max/clamp：逗号分隔参数表，全数值解（任一未解 → 保原串）。
    var args = [foldSum(root)];
    while (peek() && peek().t === ',') {
      next();
      var a2 = parseSum();
      if (!a2) return { kind: 'raw', text: s };
      args.push(foldSum(a2));
    }
    if (pos !== toks.length) return { kind: 'raw', text: s };
    var vals = [];
    for (var ai = 0; ai < args.length; ai++) {
      if (args[ai].kind === 'invalid') return null;
      if (args[ai].kind !== 'num') return { kind: 'raw', text: s };
      vals.push(args[ai]);
    }
    if (fname === 'clamp') {
      // clamp(min, val, max) = max(min, min(val, max))；corpus 期望保 calc 包裹。
      if (vals.length !== 3 || vals[0].u !== vals[1].u || vals[1].u !== vals[2].u) return { kind: 'raw', text: s };
      return { kind: 'num', v: Math.max(vals[0].v, Math.min(vals[1].v, vals[2].v)), u: vals[0].u };
    }
    if (vals.length < 1) return { kind: 'raw', text: s };
    var bv = vals[0];
    for (var bi = 1; bi < vals.length; bi++) {
      if (vals[bi].u !== bv.u) return { kind: 'raw', text: s };
      bv = { kind: 'num', v: fname === 'min' ? Math.min(bv.v, vals[bi].v) : Math.max(bv.v, vals[bi].v), u: bv.u };
    }
    return bv;
  }

  // ── 分量解析 ──
  // 返回 { t: 'num'|'calc'|'none', v, u, calcText }；u ∈ ''|'%'|'deg'|'grad'|'rad'|'turn'。
  // fromCalc：calc/min/max/clamp 折出的纯数值——retained 序列化面保 calc() 包裹
  //（corpus lab(calc(150)…) / opacity calc(0.25 * 2) → calc(0.5)），resolve 面用数值。
  function _zwCComp(tok, evalAtoms) {
    var s = String(tok).trim();
    if (s === '' ) return null;
    if (_zwCAsciiLower(s) === 'none') return { t: 'none' };
    var low = _zwCAsciiLower(s);
    if (/^(calc|min|max|clamp)\(/.test(low)) {
      var f = _zwCCalcFold(low, evalAtoms); // 计算面求值 sign()/容器单位原子
      if (f === null || f.kind === 'invalid') return null;
      if (f.kind === 'num') return { t: 'num', v: f.v, u: f.u, fromCalc: true };
      return { t: 'calc', text: f.text };
    }
    var m = /^([+-]?(?:\d*\.\d+|\d+\.?\d*)(?:[eE][-+]?\d+)?)(%|deg|grad|rad|turn)?$/.exec(s);
    if (m) {
      var u = m[2] || '';
      var v = parseFloat(m[1]);
      return { t: 'num', v: v, u: u };
    }
    if (low === 'nan' || low === 'infinity' || low === '-infinity') {
      return { t: 'num', v: low === 'nan' ? NaN : (low === 'infinity' ? Infinity : -Infinity), u: '' };
    }
    return null;
  }
  function _zwCIsNone(c) { return c && c.t === 'none'; }

  // alpha 规范：返回 { omit: bool, none: bool, v }（clamp [0,1]）；calc 分量保形
  //（clamp 交由使用面：declared 原样、computed 求值后再 clamp）。
  function _zwCAlphaCanon(c) {
    if (_zwCIsNone(c)) return { omit: false, none: true, v: 0 };
    if (c.t === 'calc') return { omit: false, none: false, v: NaN, calc: true, text: c.text };
    var v = c.v;
    if (c.u === '%') v = v / 100;
    if (isNaN(v)) v = 0;
    if (v <= 0) v = 0;
    if (v >= 1) return { omit: true, none: false, v: 1 };
    return { omit: false, none: false, v: v };
  }

  // relative alpha 声明面规范化（css-color-5 §relative colors；corpus
  // alpha-color-parsing-valid 钉）：origin 色走完整色模型规范序列化（hsl→rgb8、
  // lch 去 %、关键词小写、嵌套 alpha(from…) 递归）；alpha-part calc 折形排序。
  // 输入/输出均小写规范化文本；不可解析返 null（调用方保透传）。
  function _zwCDeclAlphaFrom(low) {
    var m = /^alpha\(\s*from\s+([\s\S]*)\)$/.exec(low);
    if (!m) return null;
    var body = m[1];
    // 顶层 '/'（括号深度 0）分割 origin 与 alpha-part。
    var depth = 0, si = -1;
    for (var i = 0; i < body.length; i++) {
      var ch = body.charAt(i);
      if (ch === '(') depth++;
      else if (ch === ')') depth--;
      else if (ch === '/' && depth === 0) { si = i; break; }
    }
    if (si < 0) return null;
    var origin = body.slice(0, si).trim();
    var apart = body.slice(si + 1).trim();
    if (!origin || !apart) return null;
    var o2 = origin;
    if (/^alpha\(\s*from\s+/.test(o2)) {
      var r2 = _zwCDeclAlphaFrom(o2);
      o2 = r2 == null ? o2 : r2;
    } else {
      var oc = _zwCParse(o2);
      if (oc && oc.form !== 'raw' && oc.form !== 'alphaFrom') {
        var os = _zwCSerializeDeclared(oc);
        if (os != null) o2 = os;
      }
    }
    var a2 = apart;
    if (/^(calc|min|max|clamp)\(/.test(apart)) {
      var f = _zwCCalcFold(apart);
      if (f && f.kind === 'num') a2 = 'calc(' + _zwCNum(f.v) + (f.u || '') + ')';
      else if (f && f.kind === 'calc') a2 = f.text;
      // alpha-part 计算式仅 alpha 与树作用域函数可用（css-color-5 relative
      // color：通道关键字 r/g/b… 不在 alpha-part 作用域，corpus `calc(r * 0.5)` 拒）。
      var ids = String(a2).match(/[A-Za-z][A-Za-z0-9-]*/g) || [];
      for (var wi = 0; wi < ids.length; wi++) {
        if (ids[wi] !== 'alpha' && ids[wi] !== 'calc' && ids[wi] !== 'min' && ids[wi] !== 'max' && ids[wi] !== 'clamp' &&
            ids[wi] !== 'sibling-index' && ids[wi] !== 'sibling-count') return null;
      }
    }
    return 'alpha(from ' + o2 + ' / ' + a2 + ')';
  }

  // ── 函数形解析 ──
  // 返回 { form, legacy, comps:[c,c,c], alpha, space, name } 或 null（非法）或
  // { form: 'raw' }（本批透传形：color-mix/relative/light-dark/contrast-color）。
  function _zwCParse(text, evalAtoms) {
    var s = String(text == null ? '' : text).replace(/\/\*[^]*?\*\//g, ' ').trim();
    if (!s || /\s/.test(s.replace(/\(.*?\)/g, '()').replace(/[-A-Za-z0-9.%,#]/g, ''))) {
      // 含裸非法记号（如 ';'）——快速否定（括号内已剥离）。
    }
    var low = _zwCAsciiLower(s);
    // CSS-wide keyword。
    if (_zwCCssWide[low]) return { form: 'csswide', name: low };
    // 关键词。
    if (low === 'transparent') return { form: 'keyword', name: 'transparent' };
    if (low === 'currentcolor') return { form: 'keyword', name: 'currentcolor' };
    if (_zwCSystem[low]) return { form: 'keyword', name: low, sys: true };
    if (_zwCNamed[low]) return { form: 'keyword', name: low };
    // 十六进制（#RGB/#RGBA/#RRGGBB/#RRGGBBAA）。
    if (s.charAt(0) === '#') {
      var hx = s.slice(1);
      if (!/^[0-9a-fA-F]+$/.test(hx)) return null;
      if (hx.length === 3 || hx.length === 4) {
        var e = '';
        for (var i = 0; i < hx.length; i++) e += hx.charAt(i) + hx.charAt(i);
        hx = e;
      }
      if (hx.length !== 6 && hx.length !== 8) return null;
      var r = parseInt(hx.slice(0, 2), 16), g = parseInt(hx.slice(2, 4), 16), b = parseInt(hx.slice(4, 6), 16);
      var a8 = hx.length === 8 ? parseInt(hx.slice(6, 8), 16) : 255;
      return { form: 'hex', r: r, g: g, b: b, a: a8 / 255 };
    }
    // 透传形（本批不建模；保原串/null 判定交后续批次）。
    var fm = /^([a-z][a-z0-9-]*)\(/.exec(low);
    if (!fm) return null;
    var fn = fm[1];
    if (fn === 'color-mix' || fn === 'light-dark' || fn === 'contrast-color' || fn === 'color-mix-in-predefined' || fn === 'color-layers') {
      return { form: 'raw' };
    }
    if (fn === 'var') return { form: 'raw' }; // var() 引用：declared 保形，宿主替换后计算面重入
    if (fn === 'alpha') {
      // relative alpha（css-color-5）：alpha(from <color> / <alpha>) declared
      // 规范化（origin 规范序列化 + alpha-part calc 折形）；其余形保透传。
      var raf = _zwCDeclAlphaFrom(low);
      return raf ? { form: 'alphaFrom', text: raf } : { form: 'raw' };
    }
    if (low.indexOf('from ') >= 0 || low.indexOf('from\n') >= 0) return { form: 'raw' }; // relative（次批建模）
    if (!/\)$/.test(low)) return null;
    var inner = low.slice(fn.length + 1, low.length - 1);
    if (fn === 'color') {
      // color(<space> <c> <c> <c> [/ <alpha>])——space 归一 xyz→xyz-d65。
      // 禁逗号形（css-color-4 §9.1 modern-only；corpus `color(srgb 0, 0, 0)` 拒）。
      if (_zwCHasTopComma(inner)) return null;
      var slashColor = _zwCSplitTopSlash(inner);
      if (slashColor.length > 2) return null;
      var mainC = _zwCSplitTopWs(slashColor[0].trim());
      if (mainC.length !== 4) return null;
      var space = mainC[0];
      var SPACE_OK = { srgb: 1, 'srgb-linear': 1, 'a98-rgb': 1, rec2020: 1, 'prophoto-rgb': 1, 'display-p3': 1, 'display-p3-linear': 1, xyz: 1, 'xyz-d50': 1, 'xyz-d65': 1 };
      if (!SPACE_OK[space]) return null;
      if (space === 'xyz') space = 'xyz-d65';
      var ccs = [];
      for (var ci = 1; ci < 4; ci++) {
        var cc = _zwCComp(mainC[ci], evalAtoms);
        if (!cc) return null;
        if (cc.t === 'num' && cc.u !== '' && cc.u !== '%') return null; // 通道禁角度
        ccs.push(cc);
      }
      var alphaC = null;
      if (slashColor.length === 2) {
        alphaC = _zwCComp(slashColor[1].trim(), evalAtoms);
        if (!alphaC) return null;
        if (alphaC.t === 'num' && alphaC.u !== '' && alphaC.u !== '%') return null; // 通道 alpha 禁角度
      } else alphaC = { t: 'num', v: 1, u: '' };
      return { form: 'color', space: space, comps: ccs, alpha: alphaC };
    }
    if (fn === 'rgb' || fn === 'rgba' || fn === 'hsl' || fn === 'hsla' || fn === 'hwb') {
      var legacy = _zwCHasTopComma(inner); // 括号感知：calc(min(1px,2px)) 内逗号不算 legacy
      if (legacy && fn === 'hwb') return null; // hwb 无 legacy 逗号形（css-color-4 §10 modern-only）
      var slashRR = _zwCSplitTopSlash(inner);
      if (slashRR.length > 2) return null; // 多顶层 slash 拒
      var hasSlashR = slashRR.length === 2;
      var mainTxt = slashRR[0];
      var alphaTxt = hasSlashR ? slashRR[1] : null;
      var compsTxt;
      if (legacy) {
        if (hasSlashR) return null; // legacy 禁 slash 形（css-color-4 §5/§8/§9）
        compsTxt = mainTxt.split(',').map(function (x) { return x.trim(); });
      } else {
        compsTxt = _zwCSplitTopWs(mainTxt.trim());
      }
      var isRgb = fn === 'rgb' || fn === 'rgba';
      if (legacy) {
        // legacy：4 参（alpha 第 4 逗号段）或 3 参。
        if (compsTxt.length !== 3 && compsTxt.length !== 4) return null;
        if (compsTxt.length === 4) {
          if (alphaTxt != null) return null;
          alphaTxt = compsTxt[3];
          compsTxt = compsTxt.slice(0, 3);
        }
      } else {
        if (compsTxt.length !== 3) return null;
      }
      var comps = [];
      for (var mi = 0; mi < compsTxt.length; mi++) {
        var mc = _zwCComp(compsTxt[mi], evalAtoms);
        if (!mc) return null;
        if (legacy && mc.t === 'none') return null; // none 仅 modern 形（css-color-4 §missing）
        if (mc.t === 'num') {
          if (isRgb && mc.u !== '' && mc.u !== '%') return null; // rgb 通道禁角度
          if (!isRgb && mi > 0 && mc.u !== '' && mc.u !== '%') return null; // S/L/W/B 槽禁角度
        }
        comps.push(mc);
      }
      // legacy rgb 通道单位同质（全数或全 %）——calc 分量不参与（corpus
      // rgba(calc(50% + (sign…)*10%), 400%, -400%, 0.5) 合法，输出 modern 形）。
      if (isRgb && legacy) {
        var anyP = false, anyN = false;
        for (var li = 0; li < comps.length; li++) {
          if (comps[li].t === 'calc') continue;
          if (comps[li].u === '%') anyP = true; else anyN = true;
        }
        if (anyP && anyN) return null;
      }
      // hsl legacy 槽位单位（corpus invalid-hsl：H 必数/角、S/L 必 %）。
      if ((fn === 'hsl' || fn === 'hsla') && legacy) {
        var h0 = comps[0];
        if (h0.t === 'num' && h0.u === '%') return null;
        for (var hi = 1; hi < 3; hi++) {
          var hc = comps[hi];
          if (hc.t === 'num' && hc.u !== '%') return null;
        }
      }
      var alphaF = null;
      if (alphaTxt != null) {
        var at = alphaTxt.trim();
        if (at === '') return null;
        alphaF = _zwCComp(at, evalAtoms);
        if (!alphaF) return null;
        if (alphaF.t === 'num' && alphaF.u !== '' && alphaF.u !== '%') return null;
        if (legacy && alphaF.t === 'none') return null; // legacy 文法无 none（corpus rgb(255,255,255,none) 拒）
      } else alphaF = { t: 'num', v: 1, u: '' };
      if (isRgb) return { form: 'rgb', legacy: legacy, comps: comps, alpha: alphaF };
      if (fn === 'hwb') return { form: 'hwb', legacy: legacy, comps: comps, alpha: alphaF };
      return { form: 'hsl', legacy: legacy, comps: comps, alpha: alphaF };
    }
    if (fn === 'lab' || fn === 'lch' || fn === 'oklab' || fn === 'oklch') {
      var slashL = _zwCSplitTopSlash(inner);
      if (slashL.length > 2) return null;
      var mL = _zwCSplitTopWs(slashL[0].trim());
      if (mL.length !== 3) return null;
      var lcs = [];
      for (var li2 = 0; li2 < 3; li2++) {
        var lc = _zwCComp(mL[li2], evalAtoms);
        if (!lc) return null;
        if (li2 === 2 && (fn === 'lch' || fn === 'oklch')) {
          // hue：角度或数或 none；fromCalc 保折出单位与符号（corpus lch(calc(20deg*2)…) → calc(40deg)）。
          if (lc.t === 'num' && !lc.fromCalc) {
            if (lc.u !== '' && !(lc.u === 'deg' || lc.u === 'grad' || lc.u === 'rad' || lc.u === 'turn')) return null;
            if (lc.u === 'grad' || lc.u === 'rad' || lc.u === 'turn') {
              lc.v = _zwCAngleToDeg(lc.v, lc.u);
              lc.fromAngleConv = true; // 单位换算旗标——声明面 hue 6 位序列化判据（corpus 1.28rad → 73.3386；decimal echo 面 RT '33.690066'）
            }
            lc.u = '';
          }
        } else if (lc.t === 'num' && lc.u !== '' && lc.u !== '%') return null; // L/C 槽禁角度
        lcs.push(lc);
      }
      var alphaL = null;
      if (slashL.length === 2) {
        alphaL = _zwCComp(slashL[1].trim(), evalAtoms);
        if (!alphaL) return null;
        if (alphaL.t === 'num' && alphaL.u !== '' && alphaL.u !== '%') return null;
      } else alphaL = { t: 'num', v: 1, u: '' };
      return { form: fn, comps: lcs, alpha: alphaL };
    }
    return null;
  }

  // ── declared 规范序列化 ──
  // calc 折出的纯数值在 retained 形仍保 calc() 包裹（corpus lab(calc(150)…)、
  // opacity calc(0.25 * 2) → calc(0.5)）；resolve 形直接用数值。
  function _zwCCompDeclText(c) {
    if (c.t === 'none') return 'none';
    if (c.t === 'calc') return c.text;
    if (c.fromCalc) return 'calc(' + _zwCNum(c.v) + (c.u || '') + ')';
    return _zwCNum(c.v) + c.u;
  }
  function _zwCAlphaDeclText(a, legacy3dp) {
    if (a && a.fromCalc) return ' / calc(' + _zwCNum(a.v) + (a.u || '') + ')';
    var ac = _zwCAlphaCanon(a);
    if (ac.none) return ' / none';
    if (ac.omit) return '';
    if (ac.calc) return ' / ' + ac.text; // calc 保形（declared 不求值）
    return ' / ' + (legacy3dp ? _zwCAlpha8(ac.v) : _zwCNum(ac.v));
  }
  // 计算面 alpha 文本：calc 求值（sign(em/px/cqw) 面打通）→ clamp [0,1]；求不出保 calc 形。
  function _zwCAlphaComputedText(a) {
    var ac = _zwCAlphaCanon(a);
    if (ac.none) return ' / none';
    if (ac.calc) {
      var f = _zwCCalcFold(ac.text, true);
      if (f && f.kind === 'num') {
        var v = f.u === '%' ? f.v / 100 : f.v;
        if (isNaN(v)) v = 0;
        if (v <= 0) return ' / 0';
        if (v >= 1) return '';
        return ' / ' + _zwCNum(v);
      }
      return ' / ' + ac.text;
    }
    if (ac.omit) return '';
    return ' / ' + _zwCNum(ac.v);
  }
  // rgb 消解：全分量无未解 calc/alpha 可解 → rgba8 逗号形；否则返 null。
  // none≈0（rgb 无 powerless 分量）、NaN→0、±∞ clamp。
  function _zwCRgbResolve(comps, alpha) {
    var rv = [];
    for (var i = 0; i < 3; i++) {
      var ci = comps[i];
      if (ci.t === 'calc') return null;
      var v = _zwCIsNone(ci) ? 0 : (ci.u === '%' ? ci.v * 255 / 100 : ci.v);
      if (isNaN(v)) v = 0;
      if (!isFinite(v)) v = v > 0 ? 255 : 0;
      rv.push(Math.max(0, Math.min(255, Math.round(v))));
    }
    var ac = _zwCAlphaCanon(alpha);
    if (ac.calc) return null;
    if (ac.omit) return 'rgb(' + rv.join(', ') + ')';
    return 'rgba(' + rv.join(', ') + ', ' + _zwCAlpha8(ac.v) + ')';
  }
  // rgb 保留 modern 形：任一通道为未解 calc 时声明序列化为空格形
  //（corpus `rgb(calc(50% + (sign…)) 0 0 / 0.5)` 钉——legacy 逗号输入亦输出 modern 形）。
  function _zwCRgbRetained(comps, alpha) {
    var parts = [];
    for (var i = 0; i < 3; i++) {
      var ci = comps[i];
      if (ci.t === 'none') { parts.push('none'); continue; }
      if (ci.t === 'calc') { parts.push(ci.text); continue; }
      var v = ci.u === '%' ? ci.v * 255 / 100 : ci.v;
      if (isNaN(v)) v = 0;
      if (!isFinite(v)) v = v > 0 ? 255 : 0;
      parts.push(_zwCNum(Math.max(0, Math.min(255, Math.round(v)))));
    }
    return 'rgb(' + parts.join(' ') + _zwCAlphaDeclText(alpha, false) + ')';
  }
  function _zwCSerializeDeclared(c) {
    switch (c.form) {
      case 'csswide': return c.name;
      case 'keyword': return c.name;
      case 'alphaFrom': return c.text;
      case 'hex': {
        var a = c.a;
        if (a >= 1) return 'rgb(' + c.r + ', ' + c.g + ', ' + c.b + ')';
        return 'rgba(' + c.r + ', ' + c.g + ', ' + c.b + ', ' + _zwCAlpha8(a) + ')';
      }
      case 'rgb': {
        // declared 恒消解（css-color-4 §serializing：valid corpus
        // 'rgb(128 none none)' 声明规范化期望 'rgb(128, 0, 0)'）；computed 面
        // none 传播不走此路——part03 inline 写存 raw 影子（_zwInlineRawStyle）
        // + part01c computed 读取点取 raw（__zwInlineRawDecl）。
        var rr2 = _zwCRgbResolve(c.comps, c.alpha);
        if (rr2 != null) return rr2;
        return _zwCRgbRetained(c.comps, c.alpha);
      }
      case 'hsl': case 'hwb': {
        // 消解判定：任一 none/未解 calc → 保留 modern 形（css-color-4 §missing；
        // corpus `hsl(120 80% none)` → `hsl(120 80 none)`）。
        var retained = _zwCIsNone(c.alpha) || (c.alpha.t === 'calc');
        for (var j = 0; j < 3; j++) if (_zwCIsNone(c.comps[j]) || c.comps[j].t === 'calc') retained = true;
        var ac2 = _zwCAlphaCanon(c.alpha);
        if (!retained) {
          var h = c.comps[0].u ? _zwCAngleToDeg(c.comps[0].v, c.comps[0].u) : c.comps[0].v;
          var rgb = c.form === 'hsl' ? _zwCHslToRgb(h, c.comps[1].v, c.comps[2].v)
            : _zwCHwbToRgb(h, c.comps[1].v, c.comps[2].v);
          if (ac2.omit) return 'rgb(' + rgb.join(', ') + ')';
          return 'rgba(' + rgb.join(', ') + ', ' + _zwCAlpha8(ac2.v) + ')';
        }
        // 保留形：deg 剥单位 + hue mod 360（css-color-4 §7）；S/L/W/B 去 %
        // 且低位 clamp（corpus S -100 → 0；L 300 保形）；fromCalc 保 calc 包裹。
        var parts = [];
        for (var pi = 0; pi < 3; pi++) {
          var pc = c.comps[pi];
          if (pi === 0 && pc.t === 'num' && pc.u) {
            var hd = (( _zwCAngleToDeg(pc.v, pc.u) % 360) + 360) % 360;
            parts.push(_zwCSig6(hd));
          } else {
            var pt = _zwCCompDeclText(pc);
            if (pi > 0 && pc.t === 'num') pt = _zwCNum(pc.v < 0 ? 0 : pc.v); // S/L/W/B 剥 % 且低位 clamp
            parts.push(pt);
          }
        }
        return c.form + '(' + parts.join(' ') + _zwCAlphaDeclText(c.alpha, false) + ')';
      }
      case 'lab': case 'lch': case 'oklab': case 'oklch': {
        var out = [];
        for (var oi = 0; oi < 3; oi++) {
          var oc = c.comps[oi];
          if (oc.t === 'none') { out.push('none'); continue; }
          if (oc.t === 'calc' || oc.fromCalc) { out.push(_zwCCompDeclText(oc)); continue; }
          var ov = oc.u === '%' ? oc.v * _zwCLabScale(c.form, oi) : oc.v;
          if (oi === 0) {
            // clamp 仅 plain 数/百分比（calc 保形——corpus lab(calc(-50 * 3)…) → calc(-150)）。
            var lim = (c.form === 'lab' || c.form === 'lch') ? 100 : 1;
            ov = Math.max(0, Math.min(lim, ov));
          } else if (oi === 1 && (c.form === 'lch' || c.form === 'oklch')) {
            ov = Math.max(0, ov); // chroma clamp [0,∞)（corpus lch(20 -20 0) → 0）
          } else if (oi === 2 && (c.form === 'lch' || c.form === 'oklch')) {
            ov = ((ov % 360) + 360) % 360; // hue mod 360（corpus 380deg → 20）
            // decimal hue echo（RT 幂等面——computed 串回设时 6 位舍入毁等值，
            // corpus RT 'lch(0.70000031 54.083273 33.690066)' 实证）；角度单位
            // 换算（rad/grad/turn→deg，parse 期 fromAngleConv 旗标）仍 6 位
            //（corpus 1.28rad → 73.3386 精确面）。
            out.push(oc.fromAngleConv ? _zwCSig6(ov) : _zwCSig8(ov));
            continue;
          }
          // lab 族分量 8 位有效（与 computed 面序列化同精度——round-trip 步把
          // computed 串回设 declared 时 6 位舍入会毁 RT 等值，corpus
          // 'oklab(0.36666664 0.46666664 0.56666664)' 实证）。
          out.push(_zwCSig8(ov));
        }
        return c.form + '(' + out.join(' ') + _zwCAlphaDeclText(c.alpha, false) + ')';
      }
      case 'color': {
        var oc2 = [];
        for (var qi = 0; qi < 3; qi++) {
          var qc = c.comps[qi];
          if (qc.t === 'none') { oc2.push('none'); continue; }
          if (qc.t === 'calc' || qc.fromCalc) { oc2.push(_zwCCompDeclText(qc)); continue; }
          // 通道 8 位有效（round-trip：computed 串回设 declared 后再求值须
          // 等值——6 位舍入曾致 corpus mix RT '0.66666667'→'0.666667' 翻红）。
          oc2.push(_zwCSig8(qc.u === '%' ? qc.v / 100 : qc.v));
        }
        return 'color(' + c.space + ' ' + oc2.join(' ') + _zwCAlphaDeclText(c.alpha, false) + ')';
      }
      default: return null;
    }
  }

  // ── 计算值序列化 ──
  // currentcolor 沿父链解析（inline → host 级联 → plain 链；深度 64 上限）。
  function _zwCResolveParentColor(elt) {
    var p = elt && elt.parentNode;
    var depth = 0;
    while (p && p.nodeType === 1 && depth++ < 64) {
      var iv = '';
      try { iv = p.style && p.style.color ? String(p.style.color) : ''; } catch (_eS1) { iv = ''; }
      if (iv && _zwCAsciiLower(iv) !== 'currentcolor') {
        var pc = _zwCParse(iv);
        if (pc && pc.form !== 'csswide') {
          var txt = _zwCSerializeComputed(pc, p, true);
          if (txt) return txt;
        }
      }
      var sel = p.__zwSelector;
      if (sel && typeof __zw_get_computed_style === 'function') {
        try {
          var hv = __zw_get_computed_style(sel, 'color');
          if (hv) return String(hv);
        } catch (_eH1) {}
      }
      if (typeof globalThis.__zwPlainComputed === 'function') {
        try {
          var pv = globalThis.__zwPlainComputed(p, 'color');
          if (pv) return String(pv);
        } catch (_eP1) {}
      }
      p = p.parentNode;
    }
    return 'rgb(0, 0, 0)';
  }
  // 计算面分量文本：calc 全数值解包（NaN→0），plain 量保形（clamp 由通道规则定）。
  // evalAtoms：sign()/容器单位原子按缺省环境求值（em=16px、cqw 等大正值）。
  function _zwCCompComputed(c, evalAtoms) {
    if (c.t === 'calc') {
      var f = _zwCCalcFold(c.text, evalAtoms);
      if (f && f.kind === 'num') return { v: isNaN(f.v) ? 0 : f.v, u: f.u, inf: !isFinite(f.v) && !isNaN(f.v) };
      return { raw: c.text, inf: false };
    }
    return { v: c.v, u: c.u, inf: false };
  }
  function _zwCSerializeComputed(c, elt, isParentWalk) {
    switch (c.form) {
      case 'keyword': {
        if (c.name === 'currentcolor') return _zwCResolveParentColor(elt);
        if (c.sys) return null; // 系统色走 host 链（Chrome 调色板映射未建表）
        if (c.name === 'transparent') return 'rgba(0, 0, 0, 0)';
        var nv = _zwCNamed[c.name].split(', ');
        return 'rgb(' + nv[0] + ', ' + nv[1] + ', ' + nv[2] + ')';
      }
      case 'hex': {
        if (c.a >= 1) return 'rgb(' + c.r + ', ' + c.g + ', ' + c.b + ')';
        return 'rgba(' + c.r + ', ' + c.g + ', ' + c.b + ', ' + _zwCAlpha8(c.a) + ')';
      }
      case 'rgb': {
        // 缺失分量（none）计算面转 color(srgb …) 形（css-color-4 §missing computed；
        // corpus `rgb(128 none none)` → `color(srgb 0.50196078 none none)` 钉 8 位
        // 有效数字）。全指定 → 消解 rgba8 / 保留 modern 形（同 declared）。
        var anyNone = false;
        for (var iN = 0; iN < 3; iN++) if (_zwCIsNone(c.comps[iN])) anyNone = true;
        if (!anyNone) {
          var rr2 = _zwCRgbResolve(c.comps, c.alpha);
          if (rr2 != null) return rr2;
          return _zwCRgbRetained(c.comps, c.alpha);
        }
        var oc3 = [];
        for (var i5 = 0; i5 < 3; i5++) {
          var cc5 = c.comps[i5];
          if (cc5.t === 'none') { oc3.push('none'); continue; }
          if (cc5.t === 'calc') { oc3.push(cc5.text); continue; }
          var v5 = cc5.u === '%' ? cc5.v * 2.55 : cc5.v;
          if (isNaN(v5)) v5 = 0;
          oc3.push(_zwCSig8(v5 / 255));
        }
        var ac3 = _zwCAlphaCanon(c.alpha);
        var a5;
        if (ac3.none) a5 = ' / none';
        else if (ac3.omit) a5 = '';
        else if (ac3.calc) a5 = ' / ' + ac3.text;
        else a5 = ' / ' + _zwCNum(ac3.v);
        return 'color(srgb ' + oc3.join(' ') + a5 + ')';
      }
      case 'hsl': case 'hwb': {
        // 先试全解析（计算面求值 sign/cqw 等原子——corpus hsl(calc(50deg + (sign…)) …)
        // → rgba(191, 191, 0, 0.5)）；none/未解 calc → 保留形回显（% 保形，
        // corpus computed-hsl `hsl(120 80% none)` → 恒等钉）。
        var resolvable = !_zwCIsNone(c.alpha) && c.alpha.t !== 'calc';
        var rc = [];
        if (resolvable) {
          for (var i6 = 0; i6 < 3; i6++) {
            var cc6 = c.comps[i6];
            if (_zwCIsNone(cc6) || cc6.t === 'calc') { resolvable = false; break; }
            var cv6 = _zwCCompComputed(cc6, true);
            if (cv6.raw) { resolvable = false; break; }
            rc.push(cv6);
          }
        }
        var ac4 = _zwCAlphaCanon(c.alpha);
        if (resolvable && !ac4.calc) {
          var h6 = rc[0].u ? _zwCAngleToDeg(rc[0].v, rc[0].u) : rc[0].v;
          var rgb6 = c.form === 'hsl' ? _zwCHslToRgb(h6, rc[1].v, rc[2].v)
            : _zwCHwbToRgb(h6, rc[1].v, rc[2].v);
          var alphaTxt6 = _zwCAlphaComputedText(c.alpha);
          return (alphaTxt6 === '' ? 'rgb(' : 'rgba(') + rgb6.join(', ') + (alphaTxt6 === '' ? ')' : ',' + alphaTxt6.slice(2) + ')');
        }
        var parts6 = [];
        for (var p6 = 0; p6 < 3; p6++) {
          var pc6 = c.comps[p6];
          parts6.push(pc6.t === 'none' ? 'none' : (pc6.t === 'calc' ? pc6.text : _zwCNum(pc6.v) + pc6.u));
        }
        return c.form + '(' + parts6.join(' ') + (c.alpha.t === 'none' ? ' / none' : _zwCAlphaDeclText(c.alpha, false)) + ')';
      }
      case 'lab': case 'lch': case 'oklab': case 'oklch': {
        var out = [];
        for (var oi = 0; oi < 3; oi++) {
          var oc = c.comps[oi];
          if (oc.t === 'none') { out.push('none'); continue; }
          var cv = _zwCCompComputed(oc, true);
          if (cv.raw) { out.push(cv.raw); continue; }
          var val = cv.v * (cv.u === '%' ? _zwCLabScale(c.form, oi) : 1);
          if (isNaN(val)) val = 0;
          if (oi === 0) {
            var lim2 = (c.form === 'lab' || c.form === 'lch') ? 100 : 1;
            if (cv.inf) val = val > 0 ? lim2 : 0;
            else val = Math.max(0, Math.min(lim2, val));
          } else if (oi === 1 && (c.form === 'lch' || c.form === 'oklch')) {
            if (cv.inf) val = val > 0 ? (cv.u === '%' ? _zwCLabScale(c.form, oi) * 100 : 1e6) : 0;
            else val = Math.max(0, val); // chroma clamp [0,∞)（corpus lch 计算 -0.5 → 0）
          } else if (oi === 2 && (c.form === 'lch' || c.form === 'oklch')) {
            if (isNaN(val)) val = 0; // NaN hue → 0（css-color-4）
            val = ((val % 360) + 360) % 360;
            out.push(_zwCSig6(val));
            continue;
          }
          out.push(_zwCNum(val));
        }
        var alphaPart = _zwCAlphaComputedText(c.alpha);
        return c.form + '(' + out.join(' ') + alphaPart + ')';
      }
      case 'color': {
        var oc2 = [];
        for (var qi = 0; qi < 3; qi++) {
          var qc = c.comps[qi];
          if (qc.t === 'none') { oc2.push('none'); continue; }
          var qv = _zwCCompComputed(qc, true);
          if (qv.raw) { oc2.push(qv.raw); continue; }
          var val2 = qv.v * (qv.u === '%' ? 1 / 100 : 1);
          if (isNaN(val2)) val2 = 0; // NaN → 0；±∞ 保留（corpus color(srgb 0 calc(infinity) 0)）
          oc2.push(_zwCNum(val2));
        }
        var alphaPart2 = _zwCAlphaComputedText(c.alpha);
        return 'color(' + c.space + ' ' + oc2.join(' ') + alphaPart2 + ')';
      }
      default: return null; // raw/alphaFrom/透传形走 host 链
    }
  }

  // ── opacity 模型（css-color-3/css-values-4：<number>|<percentage>[0,1]）──
  // declared：% ÷100 落数（corpus 50% → 0.5、-100% → -1）、不 clamp；
  // calc/min/max/clamp 保 calc() 包裹（corpus calc(25% * 2) → calc(50%)、
  // clamp(50%, 0%, 70%) → calc(50%)）。
  function _zwCOpacNormalize(value) {
    var s = String(value == null ? '' : value).trim();
    var low = _zwCAsciiLower(s);
    if (_zwCCssWide[low]) return low;
    var c = _zwCComp(s);
    if (!c || (c.t === 'num' && c.u !== '' && c.u !== '%')) return null;
    if (c.t === 'calc') return c.text;
    if (c.t === 'none') return null;
    if (c.fromCalc) return 'calc(' + _zwCNum(c.v) + (c.u || '') + ')';
    return _zwCNum(c.u === '%' ? c.v / 100 : c.v); // declared 不 clamp（corpus -2/-100% 保形）
  }
  function _zwCOpacComputed(value) {
    var c = _zwCComp(String(value).trim());
    if (!c || c.t === 'none') return null;
    var v = c.u === '%' ? c.v / 100 : c.v;
    if (c.t === 'calc') {
      var f = _zwCCalcFold(c.text, true);
      if (!f || f.kind !== 'num') return null;
      v = f.u === '%' ? f.v / 100 : f.v;
    }
    if (isNaN(v)) v = 0;
    return _zwCNum(Math.max(0, Math.min(1, v)));
  }

  // ── globalThis 钩子 ──
  // declared 归一：prop 'color' 全模型 / 'opacity' 数值面；csswide 全属性 lowercase；
  // 未建模形（color-mix/relative/light-dark/contrast/color-layers/var）保原串；
  // 非法 → null（拒）。
  globalThis.__zwColorNormalizeDecl = function (prop, value) {
    var s = String(value == null ? '' : value);
    var t = s.trim();
    if (prop !== 'color' && prop !== 'opacity') return s;
    if (t === '') return s; // 空值 = 调用方 remove 语义（echo，不拒）
    // 尾缀 !important 先剥离、归一 core 后回拼（cssText 写路径优先级随值串进来；
    // T48 cwwInheritImportant / T49 cssTextImportant 钉面——R3193 优先级后缀形）。
    var pri = '';
    var pm = /\s*!\s*important\s*$/i.exec(t);
    if (pm) {
      pri = ' !important';
      t = t.slice(0, pm.index).trim();
      if (!t) return null;
    }
    var low = _zwCAsciiLower(t);
    if (_zwCCssWide[low]) return low + pri;
    if (t === '') return null; // 仅注释/仅优先级空核（corpus ' /* hey */\n' 拒）
    if (prop === 'opacity') {
      var o2 = _zwCOpacNormalize(t);
      return o2 === null ? null : o2 + pri;
    }
    var hasCmt = t.indexOf('/*') >= 0;
    var c = _zwCParse(t);
    if (!c) return null;
    if (c.form === 'raw') return t + pri; // 透传形保原串（次批建模）
    if (hasCmt) return t + pri; // 值内注释：剥离核合法 → 保原串含注释（T49fix commentSemi 钉）
    var out = _zwCSerializeDeclared(c);
    return out === null ? t + pri : out + pri;
  };
  // ── image 声明面 light-dark() 浅校验（DC-2 light-dark image 批）──
  // 仅 image 四属性且整值顶层为 light-dark( 前缀时介入（css-color-5
  // §funcdef-light-dark 图像上下文）：恰两参顶层逗号 + 支形态校验——none /
  // 函数形态（url()/*-gradient()/嵌套 light-dark() 递归）合法，无括号单词
  //（named 色/垃圾 ident）与色值函数（rgb()/hsl() 族）拒——light-dark-image
  // 语料 invalid 组（arity≠2/空参段/色值支）锚驱动。浅校验与 Rust cascade
  // 全解析（parse_background_image 家族）分面：白名单外函数形态
  //（image-set()/cross-fade() 等不支持面）与多图层混合（顶层逗号>1 段）放行
  // passthrough 零回归。返回：undefined=不介入（passthrough）、null=拒、
  // 字符串=放行原串。
  var _zwCImageColorFnRe = /^(rgb|rgba|hsl|hsla|hwb|lab|lch|oklab|oklch|color|color-mix|color-contrast)\(/;
  var _zwCTopSplitComma = function (s) {
    // 括号感知顶层逗号分割（part11-computed-color _zwCCTopSplit 同构独立实现，
    // 不跨文件引用——本文件先于其装配）：段数即顶层逗号数+1；url() 内引号串
    // 含逗号由括号深度保护（url('a,b.png') 深度 1 不分割）。
    var segs = [];
    var depth = 0;
    var start = 0;
    for (var i = 0; i < s.length; i++) {
      var ch = s.charAt(i);
      if (ch === '(') depth++;
      else if (ch === ')') { if (depth > 0) depth--; }
      else if (ch === ',' && depth === 0) { segs.push(s.slice(start, i)); start = i + 1; }
    }
    segs.push(s.slice(start));
    return segs;
  };
  var _zwCImageLDCheck = function (inner) {
    // light-dark( 已剥前缀的参串：恰两参且支形态合法 → true。
    var segs = _zwCTopSplitComma(inner);
    if (segs.length !== 2) return false;
    for (var i = 0; i < 2; i++) {
      var br = segs[i].trim();
      if (!br) return false; // 空参段（`url(a),,url(b)` / 尾逗号）
      var bl = _zwCAsciiLower(br);
      if (bl === 'none') continue;
      if (_zwCImageColorFnRe.test(bl)) return false; // 色值不是 image
      if (bl.lastIndexOf('light-dark(', 0) === 0) {
        if (!br.charAt(br.length - 1) || br.charAt(br.length - 1) !== ')') return false;
        if (!_zwCImageLDCheck(br.slice(11, -1))) return false;
        continue;
      }
      if (br.indexOf('(') < 0) return false; // 无括号单词：仅 none 合法（named 色/垃圾 ident 拒）
      // 其余函数形态（url()/*-gradient()/image()/image-set() 等）放行。
    }
    return true;
  };
  globalThis.__zwImageNormalizeDecl = function (prop, value) {
    var p = String(prop).toLowerCase();
    if (p !== 'background-image' && p !== 'mask-image' && p !== 'list-style-image'
        && p !== 'border-image-source') return undefined;
    var t = String(value == null ? '' : value).trim();
    if (_zwCAsciiLower(t).lastIndexOf('light-dark(', 0) !== 0) return undefined;
    if (t.charAt(t.length - 1) !== ')') return null; // 括号不平衡
    // 多图层混合（`light-dark(...), url(c)` 整值顶层逗号在函数外）不介入——宿主
    // parse_background_image_layers 链处理，浅校验保持 passthrough 零回归。
    if (_zwCTopSplitComma(t).length !== 1) return undefined;
    var inner = t.slice(11, -1);
    return _zwCImageLDCheck(inner) ? t : null;
  };
  // computed：prop 'color'/'opacity'；未建模/系统色 → null（调用方回落既有链）。
  globalThis.__zwComputedColorValue = function (elt, prop, declaredValue) {
    if (prop === 'opacity') return _zwCOpacComputed(declaredValue);
    if (prop !== 'color') return null;
    _zwCCqwBase = _zwCContainerInlineSize(elt); // cq 单位容器基座（css-conditional-5 §container-lengths）
    var c;
    try {
      c = _zwCParse(String(declaredValue == null ? '' : declaredValue).trim(), true); // 计算面：sign()/容器单位原子求值
    } finally {
      _zwCCqwBase = null;
    }
    if (!c || c.form === 'raw' || c.form === 'csswide') return null;
    try {
      return _zwCSerializeComputed(c, elt, false);
    } catch (_eCC) {
      return null;
    }
  };

  // ── 消费者兼容别名（part01 gCS 抢答臂 / part01c plain 臂 / part06 supports
  // 前置钩子沿用 DC-2 次批接线入口名，委托到本文件实现）──
  // supports 前置判定：null=拒；合法/透传（color-mix 等）→ true（css-color-5
  // 语法合法但本批不建模序列化——chrome supports('color', 'color-mix(...)') 恒 true）。
  globalThis.__zwCssColorValid = function (value) {
    return __zwColorNormalizeDecl('color', value) !== null;
  };
  globalThis.__zwOpacityComputed = function (value) {
    var r = __zwComputedColorValue(null, 'opacity', value);
    return r == null ? '' : r;
  };
  globalThis.__zwComputedColor = function (raw, node) {
    var r = __zwComputedColorValue(node, 'color', raw);
    return r == null ? '' : r;
  };
  // 宿主 partial-parse 抢答判定：现代函数形宿主产出错误计算值（decl_supported
  // 不含现代语法）→ plain 重读；currentcolor 需作者臂级联（宿主读数恒黑）；
  // legacy 逗号形宿主 cascade 可正确产出（零漂移不接管）。
  globalThis.__zwCssColorNeedsPlain = function (raw) {
    var s2 = String(raw == null ? '' : raw).trim();
    if (!s2) return false;
    var low2 = _zwCAsciiLower(s2);
    if (low2 === 'currentcolor') return true;
    if (/^(rgb|rgba|hsl|hsla)\(/.test(low2) && low2.indexOf(',') >= 0) return false;
    return /^[a-z][a-z0-9-]*\(/.test(low2);
  };
})();
