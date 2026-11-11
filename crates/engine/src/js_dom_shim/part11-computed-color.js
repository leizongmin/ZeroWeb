// part11: 现代色函数 computed value 通路（DC-2 次批，css-color-4/5）。
// 归口：css-color 语料 "color doesn't seem to be supported in the computed
// style" 单簇 7,199+——getComputedStyle().color 对 hsl()/rgb() 现代空格语法、
// none powerless 分量、hwb()/lab()/lch()/oklab()/oklch()/color()、
// color-mix()、相对色 `fn(from ...)` 返回空。
// 管线：声明文本 → 解析 AST（分量 spec：number/%/none/calc/angle/通道关键字）
// → 解析（calc 求值、色空间换算、mix 插值、相对色通道替换）→ 按族序列化：
//   - 可解析 sRGB 值（named/hex/rgb/hsl/hwb 无 none）→ legacy
//     'rgb(r, g, b)' / 'rgba(r, g, b, a)'（整型化；css-color-4 § serializing）
//   - 含 none 的 rgb 现代语法 → 'color(srgb ...)'（`rgb(128 none none)` →
//     `color(srgb 0.50196078 none none)`，语料期望锚定）
//   - hsl/hwb 含 none → 'hsl(120 none none)' 自形保形（powerless 分量）
//   - lab 族含 none → 'lab(20 none none / none)' 自形
//   - color() → 自空间 canonical（数字 8 位有效数字去尾零；往返稳定）
//   - color-mix → 插值空间输出形（hsl/hwb/srgb 族 → color(srgb)；lch → lch()）
//   - 相对色 → 目标空间输出形（rgb/hsl/hwb 目标 → color(srgb)；语料锚定）
// 插值算法（css-color-5 § color-mix）：通道权重按 Σw 归一、alpha 按
// min(Σw,1) 归一、premultiplied（Wp=Σwa，通道=Σv·wa·a/Wp）、hue 四法
// （shorter 默认）、none 分量取对侧值（两侧皆 none 保 none → 自形序列化）。
// 零漂移边界：仅 part01c `_zwPlainColorSpec` 返空后回调（legacy
// named/hex/comma rgb(hsla) 走原路径输出不变）；part01 gCS `has` 陷阱与
// part06 CSS.supports 钩子由本文件登记全局入口。
// https://drafts.csswg.org/css-color-4/#resolving-color-values
// https://drafts.csswg.org/css-color-5/#color-mix
// https://drafts.csswg.org/css-color-5/#relative-colors
(function () {
  // 幂等（重复装配零副作用）——专属标记，不查 __zwComputedColor：装配序为
  // part11-color.js 基座别名在前、本文件真数学管线在后覆盖。
  if (globalThis.__zwCCComputedInstalled) return;
  globalThis.__zwCCComputedInstalled = true;

  // 基座回退引用（part11-color.js 的 declared 折叠计算面）：本管线不识别的形
  //（sign()/abs()/var()/转义及解析失败）回退基座——基座 calc 折叠器对
  // sign(1em - 10px) 等可静态求值形有既有正确解（语料 hsl/lab/rgb sign 案
  // 基座面已过）；基座也无效时返 null 走调用方宿主链（零漂移）。
  var _zwCCPrevComputed = typeof globalThis.__zwComputedColor === 'function'
    ? globalThis.__zwComputedColor : null;

  // currentcolor 求值的属性上下文（__zwComputedColor 第三参入口暂存）：
  // 'color' 走父链（css-color-4 §5.3 自引用=继承）、其余先试自元素。模块态 +
  // 入口 save/restore——嵌套 __zwPlainComputed 再入（currentcolor 链）不 clobber。
  var _zwCCPropCtx = '';

  // ============ 基础工具 ============

  // 去注释（声明文本经 inline style 原样存储，解析前剥离）。
  function _zwCCStripComments(s) {
    return String(s == null ? '' : s).replace(/\/\*[\s\S]*?\*\//g, ' ');
  }

  // CSS ident 转义解码（`r\gb` / `r\67 b` → 'rgb'；css-syntax § consume escaped
  // code point——语料 rgb 转义断言面）。
  function _zwCCDecodeEscapes(s) {
    return String(s).replace(/\\([0-9a-fA-F]{1,6})[ \t\n]?/g, function (_m, h) {
      var cp = parseInt(h, 16);
      return (cp >= 0 && cp <= 0x10ffff) ? String.fromCodePoint(cp) : '�';
    }).replace(/\\([\s\S])/g, '$1');
  }

  // 括号深度感知顶层切分（单字符分隔符集合）。
  function _zwCCTopSplit(s, seps) {
    var out = [], cur = '', depth = 0;
    for (var i = 0; i < s.length; i++) {
      var c = s.charAt(i);
      if (c === '(') depth++;
      else if (c === ')') depth--;
      if (depth === 0 && seps.indexOf(c) >= 0) { out.push(cur); cur = ''; continue; }
      cur += c;
    }
    out.push(cur);
    return out;
  }

  // 指数形展开纯小数（CSS 序列化从不用科学计数；语料比较器按字面剥数字
  // 比文本，'e-' 会破坏文本一致性）。
  function _zwCCExpToPlain(m, e) {
    var neg = m.charAt(0) === '-';
    if (neg || m.charAt(0) === '+') m = m.slice(1);
    var dot = m.indexOf('.');
    var digits = m.replace('.', '');
    var pt = (dot < 0 ? m.length : dot) + parseInt(e, 10);
    var out;
    if (pt <= 0) out = '0.' + new Array(1 - pt).join('0') + digits;
    else if (pt >= digits.length) out = digits + new Array(pt - digits.length + 1).join('0');
    else out = digits.slice(0, pt) + '.' + digits.slice(pt);
    out = out.replace(/(\.\d*?)0+$/, '$1').replace(/\.$/, '');
    return (neg && parseFloat(out) !== 0 ? '-' : '') + out;
  }

  // 数字格式化：8 位有效数字去尾零（color() 通道 canonical——`1.00`→`1`、
  // `50%`→`0.5`、`0.200`→`0.2`；语料 color-computed-color-function 断言面）。
  function _zwCCNum(v) {
    if (v !== v) return 'NaN';
    if (v === 0) return '0';
    var s = v.toPrecision(8);
    if (s.indexOf('e') >= 0 || s.indexOf('E') >= 0) {
      var parts = s.split(/[eE]/);
      return _zwCCExpToPlain(parts[0], parts[1]);
    }
    if (s.indexOf('.') >= 0) s = s.replace(/0+$/, '').replace(/\.$/, '');
    return s;
  }

  // 6 位有效数字（css-values-4 number serialization；lch/oklch hue 分量——
  // corpus 1.28rad → 73.3386 精确断言；8 位形 73.338598 不等）。
  function _zwCCSig6(v) {
    if (v !== v) return 'NaN';
    if (v === 0) return '0';
    var s = v.toPrecision(6);
    if (s.indexOf('e') >= 0 || s.indexOf('E') >= 0) {
      var parts6 = s.split(/[eE]/);
      return _zwCCExpToPlain(parts6[0], parts6[1]);
    }
    return String(Number(s));
  }

  // ============ 命名色表（css-color-4 § named colors）============

  var _zwCCNamed = {
    aliceblue: [240, 248, 255], antiquewhite: [250, 235, 215], aqua: [0, 255, 255],
    aquamarine: [127, 255, 212], azure: [240, 255, 255], beige: [245, 245, 220],
    bisque: [255, 228, 196], black: [0, 0, 0], blanchedalmond: [255, 235, 205],
    blue: [0, 0, 255], blueviolet: [138, 43, 226], brown: [165, 42, 42],
    burlywood: [222, 184, 135], cadetblue: [95, 158, 160], chartreuse: [127, 255, 0],
    chocolate: [210, 105, 30], coral: [255, 127, 80], cornflowerblue: [100, 149, 237],
    cornsilk: [255, 248, 220], crimson: [220, 20, 60], cyan: [0, 255, 255],
    darkblue: [0, 0, 139], darkcyan: [0, 139, 139], darkgoldenrod: [184, 134, 11],
    darkgray: [169, 169, 169], darkgreen: [0, 100, 0], darkgrey: [169, 169, 169],
    darkkhaki: [189, 183, 107], darkmagenta: [139, 0, 139], darkolivegreen: [85, 107, 47],
    darkorange: [255, 140, 0], darkorchid: [153, 50, 204], darkred: [139, 0, 0],
    darksalmon: [233, 150, 122], darkseagreen: [143, 188, 143], darkslateblue: [72, 61, 139],
    darkslategray: [47, 79, 79], darkslategrey: [47, 79, 79], darkturquoise: [0, 206, 209],
    darkviolet: [148, 0, 211], deeppink: [255, 20, 147], deepskyblue: [0, 191, 255],
    dimgray: [105, 105, 105], dimgrey: [105, 105, 105], dodgerblue: [30, 144, 255],
    firebrick: [178, 34, 34], floralwhite: [255, 250, 240], forestgreen: [34, 139, 34],
    fuchsia: [255, 0, 255], gainsboro: [220, 220, 220], ghostwhite: [248, 248, 255],
    gold: [255, 215, 0], goldenrod: [218, 165, 32], gray: [128, 128, 128],
    green: [0, 128, 0], greenyellow: [173, 255, 47], grey: [128, 128, 128],
    honeydew: [240, 255, 240], hotpink: [255, 105, 180], indianred: [205, 92, 92],
    indigo: [75, 0, 130], ivory: [255, 255, 240], khaki: [240, 230, 140],
    lavender: [230, 230, 250], lavenderblush: [255, 240, 245], lawngreen: [124, 252, 0],
    lemonchiffon: [255, 250, 205], lightblue: [173, 216, 230], lightcoral: [240, 128, 128],
    lightcyan: [224, 255, 255], lightgoldenrodyellow: [250, 250, 210], lightgray: [211, 211, 211],
    lightgreen: [144, 238, 144], lightgrey: [211, 211, 211], lightpink: [255, 182, 193],
    lightsalmon: [255, 160, 122], lightseagreen: [32, 178, 170], lightskyblue: [135, 206, 250],
    lightslategray: [119, 136, 153], lightslategrey: [119, 136, 153], lightsteelblue: [176, 196, 222],
    lightyellow: [255, 255, 224], lime: [0, 255, 0], limegreen: [50, 205, 50],
    linen: [250, 240, 230], magenta: [255, 0, 255], maroon: [128, 0, 0],
    mediumaquamarine: [102, 205, 170], mediumblue: [0, 0, 205], mediumorchid: [186, 85, 211],
    mediumpurple: [147, 112, 219], mediumseagreen: [60, 179, 113], mediumslateblue: [123, 104, 238],
    mediumspringgreen: [0, 250, 154], mediumturquoise: [72, 209, 204], mediumvioletred: [199, 21, 133],
    midnightblue: [25, 25, 112], mintcream: [245, 255, 250], mistyrose: [255, 228, 225],
    moccasin: [255, 228, 181], navajowhite: [255, 222, 173], navy: [0, 0, 128],
    oldlace: [253, 245, 230], olive: [128, 128, 0], olivedrab: [107, 142, 35],
    orange: [255, 165, 0], orangered: [255, 69, 0], orchid: [218, 112, 214],
    palegoldenrod: [238, 232, 170], palegreen: [152, 251, 152], paleturquoise: [175, 238, 238],
    palevioletred: [219, 112, 147], papayawhip: [255, 239, 213], peachpuff: [255, 218, 185],
    peru: [205, 133, 63], pink: [255, 192, 203], plum: [221, 160, 221],
    powderblue: [176, 224, 230], purple: [128, 0, 128], rebeccapurple: [102, 51, 153],
    red: [255, 0, 0], rosybrown: [188, 143, 143], royalblue: [65, 105, 225],
    saddlebrown: [139, 69, 19], salmon: [250, 128, 114], sandybrown: [244, 164, 96],
    seagreen: [46, 139, 87], seashell: [255, 245, 238], sienna: [160, 82, 45],
    silver: [192, 192, 192], skyblue: [135, 206, 235], slateblue: [106, 90, 205],
    slategray: [112, 128, 144], slategrey: [112, 128, 144], snow: [255, 250, 250],
    springgreen: [0, 255, 127], steelblue: [70, 130, 180], tan: [210, 180, 140],
    teal: [0, 128, 128], thistle: [216, 191, 216], tomato: [255, 99, 71],
    turquoise: [64, 224, 208], violet: [238, 130, 238], wheat: [245, 222, 179],
    white: [255, 255, 255], whitesmoke: [245, 245, 245], yellow: [255, 255, 0],
    yellowgreen: [154, 205, 50]
  };

  // ============ 系统色（css-color-4 § system colors）============
  // computed 值策略：一致性页（system-color-consistency）断言「系统色 ==
  // 同元素对应属性读数」——按钮/表单/mark 的 UA 默认声明在我方 plain 走查
  // 缺位（读数 ''），故多数系统色解析 ''（期望与读数同为 ''，断言恒成立）。
  // not-canvas 子测（LinkText/ActiveText/VisitedText 不得等于 Canvas 底色）
  // 需非空且互异——映射具体值（LinkText 对齐 part01c a:link UA 默认
  // rgb(0, 0, 255)；ActiveText 同标准链接蓝；VisitedText 经典紫）。
  var _zwCCSystem = {
    accentcolor: '', accentcolortext: '', activetext: 'rgb(0, 0, 255)',
    buttonborder: '', buttonface: '', buttontext: '', canvas: '', canvastext: '',
    field: '', fieldtext: '', graytext: '', highlight: '', highlighttext: '',
    infobackground: '', infotext: '', linktext: 'rgb(0, 0, 255)', mark: '',
    marktext: '', selecteditem: '', selecteditemtext: '',
    visitedtext: 'rgb(85, 26, 139)',
    // deprecated（css-color-4 § deprecated system colors——保留解析面）。
    activeborder: '', activecaption: '', appworkspace: '', background: '',
    buttonhighlight: '', buttonshadow: '', captiontext: '',
    inactiveborder: '', inactivecaption: '', inactivecaptiontext: '',
    menu: '', menutext: '', scrollbar: '', threeddarkshadow: '', threedface: '',
    threedhighlight: '', threedlightshadow: '', window: '', windowframe: '',
    windowtext: ''
  };

  // dark used-scheme 面板（css-color-4 § system colors：系统色按 used
  // color-scheme 取 light/dark 面板值；值无 UA 钉面——对齐 css-parser dark 臂
  // 同款近似：canvas 族 rgb(32, 33, 36)、文本族白）。仅 dark 臂：light 面板
  // 保持既有静态值/空串（空串=「解析为空」历史口径，system-color-consistency
  // 页 ''=='' 等值钉面依赖，填 light 实值会成批翻红——FIXME: UA 表单样式建模
  // 后统一填实）。消费面：system-color-support 页 scheme-aware 12 键
  // assert_not_equals(light, dark) 差分（plain 臂走查）。
  var _zwCCSystemDark = {
    canvas: 'rgb(32, 33, 36)', canvastext: 'rgb(255, 255, 255)',
    linktext: 'rgb(138, 180, 255)', visitedtext: 'rgb(191, 145, 229)',
    buttonface: 'rgb(43, 43, 43)', buttontext: 'rgb(255, 255, 255)',
    field: 'rgb(32, 33, 36)', fieldtext: 'rgb(255, 255, 255)',
    highlight: 'rgb(13, 71, 161)', selecteditem: 'rgb(13, 71, 161)',
    appworkspace: 'rgb(32, 33, 36)', scrollbar: 'rgb(60, 64, 67)'
  };

  // used color-scheme 合成（css-color-adjust-1 §color-scheme；语义对齐
  // style-system parse_color_scheme_dark）。__zwPlainComputed 第三参 qvMode
  // 开作者臂，沿父链返回首个 color-scheme 声明原文（part01c color-scheme
  // 分支）；token 判定须空格分词（'light-dark' 内 \b 会误断词）；light dark
  // 双 token → 用户偏好决定，runner 环境 prefers_dark 恒 false → 保守 light
  // （与 Rust 臂 (true,true)→prefers_dark=false 同口径）；无声明/normal/only
  // → 文档 scheme（corpus color 页无 meta color-scheme 依赖，保守 light）。
  function _zwCCUsedSchemeDark(el) {
    if (!el || el.nodeType !== 1) return false;
    var decl = '';
    if (typeof globalThis.__zwPlainComputed === 'function') {
      try {
        decl = String(globalThis.__zwPlainComputed(el, 'color-scheme', true) || '');
      } catch (_eUD) { decl = ''; }
    }
    if (!decl) return false;
    var toks = decl.replace(/\/\*[\s\S]*?\*\//g, ' ').toLowerCase().split(/\s+/);
    var hasLight = toks.indexOf('light') >= 0;
    var hasDark = toks.indexOf('dark') >= 0;
    return hasDark && !hasLight;
  }

  // 相对色 origin 骨架判定（css-color-5 §relative-colors，Typed-OM computed
  // 面钉面 relative-color-with-system-color）：origin 折叠到 currentcolor 时
  // computed 值骨架保形 'hsl(from currentcolor ...)'（currentcolor 依赖 used
  // value 上下文，computed 时间不解析）；origin 为绝对色（system color/具名
  // 色/light-dark 绝对支）时 computed 整体解析，返 '' 由调用方走宿主 resolved
  // 通道。var() 支未展开（FIXME：保守走宿主）。
  function _zwCCOriginSkeleton(originText, el) {
    var s = _zwCCStripComments(String(originText == null ? '' : originText)).trim();
    if (!s) return '';
    if (s.toLowerCase() === 'currentcolor') return 'currentcolor';
    var fnHead = /^([a-zA-Z][-\w]*)\s*\(([\s\S]*)\)$/.exec(s);
    if (fnHead && fnHead[1].toLowerCase() === 'light-dark') {
      var groups = _zwCCTopSplit(fnHead[2], ',');
      if (groups.length !== 2) return '';
      return _zwCCOriginSkeleton(_zwCCUsedSchemeDark(el) ? groups[1] : groups[0], el);
    }
    return '';
  }

  // ============ calc 求值器（数字/% 两型；opacity 与色分量共用）============

  // 返回 {v, pct} 或 NaN（类型错/非法/除零 → NaN；消费面 NaN→0 对齐
  // css-color-4 `lab(calc(NaN) 0 0)` → `lab(0 0 0)` 断言）。
  function _zwCCEval(expr) {
    var s = _zwCCStripComments(expr);
    var pos = 0;
    function ws() { while (pos < s.length && /\s/.test(s.charAt(pos))) pos++; }
    function eat(ch) { ws(); if (s.charAt(pos) === ch) { pos++; return true; } return false; }
    function parseSum() {
      var left = parseProd();
      if (left !== left) return NaN;
      for (;;) {
        ws();
        var c = s.charAt(pos);
        if (c === '+' || c === '-') {
          pos++;
          var right = parseProd();
          if (right !== right) return NaN;
          if (left.pct !== right.pct || left.ang !== right.ang) return NaN; // +− 同型（css-values § calc type checking）
          left = { v: left.v + (c === '+' ? right.v : -right.v), pct: left.pct, ang: left.ang };
        } else return left;
      }
    }
    function parseProd() {
      var left = parseUnit();
      if (left !== left) return NaN;
      for (;;) {
        ws();
        var c = s.charAt(pos);
        if (c === '*' || c === '/') {
          pos++;
          var right = parseUnit();
          if (right !== right) return NaN;
          if (c === '*') {
            if (left.pct && right.pct) return NaN; // %×% 非法
            if (left.ang && right.ang) return NaN; // 角×角非法
            left = { v: left.v * right.v, pct: left.pct || right.pct, ang: left.ang || right.ang };
          } else {
            if (right.v === 0) return NaN; // 除零 → NaN
            if (right.pct) return NaN;
            if (right.ang) {
              // 角/角 → 数（'50rad / 50deg'，css-values-4 § trig）
              if (!left.ang) return NaN;
              left = { v: left.v / right.v, pct: false, ang: false };
            } else left = { v: left.v / right.v, pct: left.pct, ang: left.ang };
          }
        } else return left;
      }
    }
    function parseUnit() {
      ws();
      if (s.charAt(pos) === '(') {
        pos++;
        var inner = parseSum();
        if (!eat(')')) return NaN;
        return inner;
      }
      var m = /^([-+]?(\d+\.?\d*|\.\d+)([eE][-+]?\d+)?)\s*(%|deg|grad|rad|turn)?/.exec(s.slice(pos));
      if (m) {
        pos += m[0].length;
        var nv = parseFloat(m[1]), u = (m[4] || '').toLowerCase();
        if (u === '%') return { v: nv, pct: true, ang: false };
        if (u === 'deg') return { v: nv, pct: false, ang: true };
        if (u === 'grad') return { v: nv * 0.9, pct: false, ang: true };
        if (u === 'rad') return { v: nv * 180 / Math.PI, pct: false, ang: true };
        if (u === 'turn') return { v: nv * 360, pct: false, ang: true };
        return { v: nv, pct: false, ang: false };
      }
      // css-values-4 calc 常量 pi（'50deg * (180 / pi)' 语料形）。
      var pm = /^pi\b/i.exec(s.slice(pos));
      if (pm) {
        pos += 2;
        return { v: Math.PI, pct: false, ang: false };
      }
      // css-values-4 calc 常量：infinity/-infinity/NaN（'rgb(calc(infinity), 0, 0)'
      // → 255 面）。
      var cm = /^([-+]?)(infinity|nan)\b/i.exec(s.slice(pos));
      if (cm) {
        pos += cm[0].length;
        if (cm[2].toLowerCase() === 'nan') return { v: NaN, pct: false };
        var iv = cm[1] === '-' ? -Infinity : Infinity;
        return { v: iv, pct: false };
      }
      var fm = /^([a-zA-Z][a-zA-Z0-9-]*)\(/.exec(s.slice(pos));
      if (fm) {
        var fname = fm[1].toLowerCase();
        // calc(...) 单参透明求值（嵌套 calc 形 `calc(25% * 2)`——opacity 臂
        // 整值直入 _zwCCEval；色分量臂已剥壳不在此路径）。
        if (fname === 'calc') {
          pos += fm[0].length;
          var calcInner = parseSum();
          if (calcInner !== calcInner) return NaN;
          if (!eat(')')) return NaN;
          return calcInner;
        }
        // 三角函数（css-values-4 § trig：number 参 = 弧度，angle 参先转弧度；
        // 反三角返回 angle——内部以 deg 表示）。
        if (fname === 'sin' || fname === 'cos' || fname === 'tan') {
          pos += fm[0].length;
          var trArg = parseSum();
          if (trArg !== trArg || trArg.pct) return NaN;
          if (!eat(')')) return NaN;
          var trRad = trArg.ang ? trArg.v * Math.PI / 180 : trArg.v;
          var trRes = fname === 'sin' ? Math.sin(trRad) : fname === 'cos' ? Math.cos(trRad) : Math.tan(trRad);
          return { v: trRes, pct: false, ang: false };
        }
        if (fname === 'asin' || fname === 'acos' || fname === 'atan') {
          pos += fm[0].length;
          var atArg = parseSum();
          if (atArg !== atArg || atArg.pct || atArg.ang) return NaN;
          if (!eat(')')) return NaN;
          var atRes = fname === 'asin' ? Math.asin(atArg.v) : fname === 'acos' ? Math.acos(atArg.v) : Math.atan(atArg.v);
          return { v: atRes * 180 / Math.PI, pct: false, ang: true };
        }
        // pow（css-values-4 § exponent functions：负底非整指数 → NaN，Math.pow 同）。
        if (fname === 'pow') {
          pos += fm[0].length;
          var pw1 = parseSum();
          if (pw1 !== pw1 || pw1.pct || pw1.ang) return NaN;
          if (!eat(',')) return NaN;
          var pw2 = parseSum();
          if (pw2 !== pw2 || pw2.pct || pw2.ang) return NaN;
          if (!eat(')')) return NaN;
          return { v: Math.pow(pw1.v, pw2.v), pct: false, ang: false };
        }
        if (fname !== 'min' && fname !== 'max' && fname !== 'clamp') return NaN;
        pos += fm[0].length;
        var depth = 1, start = pos;
        while (pos < s.length && depth > 0) {
          if (s.charAt(pos) === '(') depth++;
          else if (s.charAt(pos) === ')') depth--;
          if (depth > 0) pos++;
        }
        if (depth !== 0) return NaN;
        var argstr = s.slice(start, pos);
        pos++; // 收尾 ')'
        var args = [], d2 = 0, cur = '';
        for (var i = 0; i < argstr.length; i++) {
          var cc = argstr.charAt(i);
          if (cc === '(') d2++;
          if (cc === ')') d2--;
          if (cc === ',' && d2 === 0) { args.push(cur); cur = ''; continue; }
          cur += cc;
        }
        args.push(cur);
        var vals = [];
        for (var j = 0; j < args.length; j++) {
          var ev = _zwCCEval(args[j]);
          if (ev !== ev) return NaN;
          vals.push(ev);
        }
        var pct = vals[0].pct;
        for (var k = 1; k < vals.length; k++) if (vals[k].pct !== pct || vals[k].ang !== vals[0].ang) return NaN;
        var pick = vals[0];
        if (fname === 'min') {
          for (var k2 = 1; k2 < vals.length; k2++) if (vals[k2].v < pick.v) pick = vals[k2];
        } else if (fname === 'max') {
          for (var k3 = 1; k3 < vals.length; k3++) if (vals[k3].v > pick.v) pick = vals[k3];
        } else {
          // clamp(min, val, max)
          pick = { v: Math.min(Math.max(vals[1].v, vals[0].v), vals[2].v), pct: pct };
        }
        return pick;
      }
      return NaN;
    }
    var r = parseSum();
    ws();
    if (pos !== s.length) return NaN;
    return r;
  }

  // ============ 色空间换算（XYZ(D65/D50) 枢纽；css-color-4 § conversion）============

  function _zwCCSrgbToLin(c) {
    var a = c < 0 ? -c : c;
    var l = a <= 0.04045 ? a / 12.92 : Math.pow((a + 0.055) / 1.055, 2.4);
    return c < 0 ? -l : l;
  }
  function _zwCCLinToSrgb(c) {
    var a = c < 0 ? -c : c;
    var s = a <= 0.0031308 ? a * 12.92 : 1.055 * Math.pow(a, 1 / 2.4) - 0.055;
    return c < 0 ? -s : s;
  }
  function _zwCCA98ToLin(c) {
    var a = c < 0 ? -c : c;
    var l = Math.pow(a, 563 / 256);
    return c < 0 ? -l : l;
  }
  function _zwCCA98ToEnc(c) {
    var a = c < 0 ? -c : c;
    var e = Math.pow(a, 256 / 563);
    return c < 0 ? -e : e;
  }
  function _zwCCProphotoToLin(c) {
    var a = c < 0 ? -c : c;
    var l = a < 16 / 512 ? a / 16 : Math.pow(a, 1.8);
    return c < 0 ? -l : l;
  }
  function _zwCCProphotoToEnc(c) {
    var a = c < 0 ? -c : c;
    var e = a < ET_PPR ? a * 16 : Math.pow(a, 1 / 1.8);
    return c < 0 ? -e : e;
  }
  var ET_PPR = 16 / 512;
  // rec2020 传递函数（css-color-4 §rec2020：α=4.5β，β=0.08145；编解码须
  // 互逆——解码指数 1/0.45≈2.2222，不能用 srgb 的 2.4）。
  function _zwCC2020ToLin(c) {
    var a = c < 0 ? -c : c;
    var l = a < 0.08145 ? a / 4.5 : Math.pow((a + 0.0993) / 1.0993, 1 / 0.45);
    return c < 0 ? -l : l;
  }
  function _zwCC2020ToEnc(c) {
    var a = c < 0 ? -c : c;
    var e = a < 0.08145 ? a * 4.5 : 1.0993 * Math.pow(a, 0.45) - 0.0993;
    return c < 0 ? -e : e;
  }

  var _zwCCSpaces = {
    'srgb': { toLin: _zwCCSrgbToLin, toEnc: _zwCCLinToSrgb,
      toXyz: [[0.4123908, 0.3575843, 0.1804808], [0.2126390, 0.7151687, 0.0721923], [0.0193308, 0.1191948, 0.9505322]],
      fromXyz: [[3.2409699, -1.5373831, -0.4986108], [-0.9692436, 1.8759675, 0.0415551], [0.0556301, -0.2039770, 1.0569715]] },
    'srgb-linear': { isLinear: true,
      toXyz: [[0.4123908, 0.3575843, 0.1804808], [0.2126390, 0.7151687, 0.0721923], [0.0193308, 0.1191948, 0.9505322]],
      fromXyz: [[3.2409699, -1.5373831, -0.4986108], [-0.9692436, 1.8759675, 0.0415551], [0.0556301, -0.2039770, 1.0569715]] },
    'display-p3': { toLin: _zwCCSrgbToLin, toEnc: _zwCCLinToSrgb,
      toXyz: [[0.4865709, 0.2656677, 0.1982173], [0.2289746, 0.6917385, 0.0792869], [0.0, 0.0451134, 1.0439444]],
      // fromXyz = toXyz 数值求逆（css-color-4 § conversion；原表行 1/2 混入
      // 他空间行，互逆核验 maxdev 2.06 实证后重建）。
      fromXyz: [[2.4934973, -0.9313838, -0.4027109], [-0.8294893, 1.7626642, 0.0236248], [0.0358459, -0.0761724, 0.9568845]] },
    'display-p3-linear': { isLinear: true,
      toXyz: [[0.4865709, 0.2656677, 0.1982173], [0.2289746, 0.6917385, 0.0792869], [0.0, 0.0451134, 1.0439444]],
      // fromXyz = toXyz 数值求逆（原行 1/2 抄入他空间行，往返 NaN 级发散）。
      fromXyz: [[2.4934973, -0.9313838, -0.4027109], [-0.8294893, 1.7626642, 0.0236248], [0.0358459, -0.0761724, 0.9568845]] },
    'a98-rgb': { toLin: _zwCCA98ToLin, toEnc: _zwCCA98ToEnc,
      toXyz: [[0.5766690, 0.1855582, 0.1882287], [0.2973448, 0.6273636, 0.0752915], [0.0270329, 0.0706888, 0.9913375]],
      fromXyz: [[2.0415879, -0.5650072, -0.3447313], [-0.9692436, 1.8759675, 0.0415551], [0.0134443, -0.1183781, 1.0153100]] },
    // prophoto 原生 D50（css-color-4 §prophoto-rgb 矩阵；B 列符号与
    // 0.7118749 等抄写误差已按规范修正，fromXyz = toXyz 数值求逆）。
    'prophoto-rgb': { toLin: _zwCCProphotoToLin, toEnc: _zwCCProphotoToEnc, d50: true,
      toXyz: [[0.7976656072, 0.1351905296, 0.0312093796], [0.2880403108, 0.7118749372, -0.0000827301], [0.0, 0.0, 0.8251904608]],
      fromXyz: [[1.3459592653, -0.2556080238, -0.0509309088], [-0.5446048243, 1.5081657725, 0.0207486032], [0.0, 0.0, 1.2118414445]] },
    // rec2020（css-color-4 §rec2020 全精度矩阵；G 行 0.6779980715 与
    // fromXyz 数值求逆，往返核验 1e-16）。
    'rec2020': { toLin: _zwCC2020ToLin, toEnc: _zwCC2020ToEnc,
      toXyz: [[0.6369580483012914, 0.1446169035862083, 0.1688809751641721], [0.2627002120112671, 0.6779980715188707, 0.05930171646986196], [0.0, 0.0280726930490874, 1.0609850577107909]],
      fromXyz: [[1.7166511880, -0.3556707838, -0.2533662814], [-0.6666843518, 1.6164812366, 0.0157685458], [0.0176398574, -0.0427706133, 0.9421031212]] },
    'xyz-d50': { isXyz: true, d50: true }, 'xyz-d65': { isXyz: true }, 'xyz': { isXyz: true, alias: 'xyz-d65' }
  };
  // Bradford 色适应 D65↔D50（css-color-4 lab 转换；由 Bradford 锥矩阵
  // M·diag(cone50/cone65)·M⁻¹ 数值推导，白点互映核验过）。
  var _zwCCD65toD50 = [[1.0479298, 0.0229469, -0.0501923], [0.0296278, 0.9904345, -0.0170738], [-0.0092430, 0.0150552, 0.7518742]];
  var _zwCCD50toD65 = [[0.9554734, -0.0230984, 0.0632592], [-0.0283697, 1.0099954, 0.0210414], [0.0123140, -0.0205077, 1.3303660]];

  function _zwCCMatVec(m, v) {
    return [m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
      m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
      m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2]];
  }
  function _zwCCToXyz65(space, lin) {
    var sp = _zwCCSpaces[space];
    if (!sp) return null;
    if (sp.isXyz) return (space === 'xyz-d50') ? _zwCCMatVec(_zwCCD50toD65, lin) : lin.slice();
    var xyz = _zwCCMatVec(sp.toXyz, lin);
    return sp.d50 ? _zwCCMatVec(_zwCCD50toD65, xyz) : xyz;
  }
  function _zwCCFromXyz65(space, xyz) {
    var sp = _zwCCSpaces[space];
    if (!sp) return null;
    var x = sp.d50 ? _zwCCMatVec(_zwCCD65toD50, xyz) : xyz.slice();
    if (sp.isXyz) return x;
    return _zwCCMatVec(sp.fromXyz, x);
  }
  function _zwCCToLin(space, comps) {
    var sp = _zwCCSpaces[space];
    if (!sp || sp.isXyz || sp.isLinear) return comps.slice();
    return [sp.toLin(comps[0]), sp.toLin(comps[1]), sp.toLin(comps[2])];
  }
  function _zwCCFromLin(space, lin) {
    var sp = _zwCCSpaces[space];
    if (!sp || sp.isXyz || sp.isLinear) return lin.slice();
    return [sp.toEnc(lin[0]), sp.toEnc(lin[1]), sp.toEnc(lin[2])];
  }

  // ============ lab 族换算（css-color-4 § lab color）============

  var _zwCCEps = 216 / 24389, _zwCCKappa = 24389 / 27;
  // D50 白点（css-color-4 § lab color；xyz 入参/出参为绝对 XYZ，函数内做归一）。
  var _zwCCWhiteD50 = [0.9642956764295677, 1.0, 0.8251046025104602];
  function _zwCCXyzToLab(xyz) { // d50
    function f(t) { return t > _zwCCEps ? Math.cbrt(t) : (_zwCCKappa * t + 16) / 116; }
    var fx = f(xyz[0] / _zwCCWhiteD50[0]), fy = f(xyz[1] / _zwCCWhiteD50[1]), fz = f(xyz[2] / _zwCCWhiteD50[2]);
    return [116 * fy - 16, 500 * (fx - fy), 200 * (fy - fz)];
  }
  function _zwCCLabToXyz(lab) { // d50
    var fy = (lab[0] + 16) / 116, fx = fy + lab[1] / 500, fz = fy - lab[2] / 200;
    var fx3 = fx * fx * fx, fz3 = fz * fz * fz;
    var xr = fx3 > _zwCCEps ? fx3 : (116 * fx - 16) / _zwCCKappa;
    var yr = lab[0] > 8 ? Math.pow(fy, 3) : lab[0] / _zwCCKappa;
    var zr = fz3 > _zwCCEps ? fz3 : (116 * fz - 16) / _zwCCKappa;
    return [xr * _zwCCWhiteD50[0], yr * _zwCCWhiteD50[1], zr * _zwCCWhiteD50[2]];
  }

  // ============ oklab 换算（css-color-4 § oklab）============

  var _zwCCOkM1 = [[0.8190184432116, 0.3619062562801, -0.1288737826116], [0.0329836539704, 0.9292868468966, 0.0361446681696], [0.0481773015662, 0.2642395249444, 0.6335478258137]];
  var _zwCCOkM2 = [[0.2104542553, 0.7936177850, -0.0040720468], [1.9779984951, -2.4285922050, 0.4505937099], [0.0259040371, 0.7827717662, -0.8086757660]];
  var _zwCCOkM2i = [[1, 0.3963377774, 0.2158037573], [1, -0.1055613458, -0.0638541728], [1, -0.0894841775, -1.2914855480]];
  // 官方 XYZ↔LMS 矩阵直接作用在绝对 D65 XYZ 上（css-color-4 § oklab，
  // D65 白点互映核验：M1·white→(1,1,1)，M1i·(1,1,1)→white）。
  var _zwCCOkM1i = [[1.2270138535, -0.5577999757, 0.2812561490], [-0.0405801784, 1.1122568696, -0.0716767866], [-0.0763812845, -0.4214819784, 1.5861632204]];
  // OkM1/OkM2 数值真逆（Gauss-Jordan，13 位字面；f64 往返 ~1e-15）。发表逆
  // 精度实证：OkM1i 松（与真逆偏差 7.6e-4，链内致 e-5 级输出噪声）、OkM2i
  // 本系真逆（偏差 5.5e-8）。仅嵌套 relative-out 原生直转臂消费——发表逆
  // （OkM1i/OkM2i）仍供序列化投影面（8 位语料锚按发表常数判定，全局换
  // 真逆会漂锚）。
  var _zwCCOkM1t = [[1.226885850081, -0.5578177139334, 0.2813924209634], [-0.04057593472673, 1.112286907645, -0.07171110613332], [-0.07637352998276, -0.4214930601520, 1.586923891347]];
  var _zwCCOkM2t = [[0.999999998451, 0.396337792174, 0.215803758061], [1.00000000888, -0.105561342324, -0.0638541747717], [1.00000005467, -0.089484182095, -1.29148553786]];
  function _zwCCXyzToOklab(xyz) {
    var lms = _zwCCMatVec(_zwCCOkM1, xyz);
    var lp = Math.cbrt(lms[0]), mp = Math.cbrt(lms[1]), sp = Math.cbrt(lms[2]);
    return _zwCCMatVec(_zwCCOkM2, [lp, mp, sp]);
  }
  function _zwCCOklabToXyz(lab) {
    var lms = _zwCCMatVec(_zwCCOkM2i, lab);
    var l = lms[0] * lms[0] * lms[0], m = lms[1] * lms[1] * lms[1], s = lms[2] * lms[2] * lms[2];
    return _zwCCMatVec(_zwCCOkM1i, [l, m, s]);
  }

  // ============ hsl/hwb ↔ srgb（css-color-4 § hsl/hwb）============

  function _zwCCHslToRgb(h, s, l) { // h 度数任意域，s/l 0..1
    h = ((h % 360) + 360) % 360;
    var c = (1 - Math.abs(2 * l - 1)) * s;
    var hp = h / 60, x = c * (1 - Math.abs(hp % 2 - 1));
    var r, g, b;
    if (hp < 1) { r = c; g = x; b = 0; }
    else if (hp < 2) { r = x; g = c; b = 0; }
    else if (hp < 3) { r = 0; g = c; b = x; }
    else if (hp < 4) { r = 0; g = x; b = c; }
    else if (hp < 5) { r = x; g = 0; b = c; }
    else { r = c; g = 0; b = x; }
    var m = l - c / 2;
    return [r + m, g + m, b + m];
  }
  function _zwCCRgbToHsl(r, g, b) {
    var mx = Math.max(r, g, b), mn = Math.min(r, g, b), d = mx - mn;
    var l = (mx + mn) / 2;
    var s = d === 0 ? 0 : d / (1 - Math.abs(2 * l - 1));
    var h = 0;
    if (d !== 0) {
      if (mx === r) h = 60 * (((g - b) / d) % 6);
      else if (mx === g) h = 60 * ((b - r) / d + 2);
      else h = 60 * ((r - g) / d + 4);
    }
    if (h < 0) h += 360;
    return [h, s, l];
  }
  function _zwCCHwbToRgb(h, w, b) {
    var prim = _zwCCHslToRgb(h, 1, 0.5);
    if (w + b >= 1) { // w+b≥1 规范化（css-color-4：等比归一，灰同色相）
      var t = w + b;
      w = w / t; b = 1 - w;
    }
    var f = 1 - w - b;
    return [prim[0] * f + w, prim[1] * f + w, prim[2] * f + w];
  }
  function _zwCCRgbToHwb(r, g, b) {
    var hsl = _zwCCRgbToHsl(r, g, b);
    return [hsl[0], Math.min(r, g, b), 1 - Math.max(r, g, b)];
  }

  // ============ 分量 spec ============
  // 形：{none:true} | {num:v} | {pct:v} | {angle:v 度} | {calc:expr}
  //     | {chan:'r'|...}（相对色通道关键字） | {chanexpr:'...'}（相对 calc）

  function _zwCCNumSpec(text) {
    var t = String(text).trim();
    if (t === 'none') return { none: true };
    if (/^([-+]?(\d+\.?\d*|\.\d+)([eE][-+]?\d+)?)$/.test(t)) return { num: parseFloat(t) };
    if (t.charAt(t.length - 1) === '%') {
      var inner = t.slice(0, -1).trim();
      if (/^([-+]?(\d+\.?\d*|\.\d+)([eE][-+]?\d+)?)$/.test(inner)) return { pct: parseFloat(inner) };
    }
    var am = /^([-+]?(\d+\.?\d*|\.\d+)([eE][-+]?\d+)?)(deg|grad|rad|turn)$/i.exec(t);
    if (am) {
      var v = parseFloat(am[1]), u = am[4].toLowerCase();
      var d = u === 'deg' ? v : u === 'grad' ? v * 360 / 400 : u === 'rad' ? v * 180 / Math.PI : v * 360;
      return { angle: d };
    }
    return null;
  }

  var _zwCCChanKw = ['r', 'g', 'b', 'h', 's', 'l', 'w', 'c', 'a', 'alpha', 'x', 'y', 'z'];

  // calc() → {calc}（纯数值）或 {chanexpr}（含通道关键字——相对色延迟求值）。
  // 裸函数表达式含通道关键字同 chanexpr（语料 'pow(l, 1)'、'sin(l)' 直写形）。
  function _zwCCCalcSpec(text) {
    var t = String(text).trim();
    var m = /^calc\(([\s\S]*)\)$/i.exec(t);
    var probe = null, expr = null;
    if (m) { probe = m[1]; expr = m[1]; }
    else if (/^[a-zA-Z][a-zA-Z0-9-]*\(/.test(t) && /\)$/.test(t)) { probe = t; expr = t; }
    if (probe == null) return null;
    for (var i = 0; i < _zwCCChanKw.length; i++) {
      if (new RegExp('(^|[^0-9a-zA-Z.])' + _zwCCChanKw[i] + '($|[^0-9a-zA-Z(])', 'i').test(probe)) {
        return { chanexpr: expr };
      }
    }
    return m ? { calc: m[1] } : null;
  }

  function _zwCCCompSpec(text) {
    var t = String(text).trim();
    var calc = _zwCCCalcSpec(t);
    if (calc) return calc;
    return _zwCCNumSpec(t);
  }

  // 百分比满量数值化：pct → pct/100*scale；num 原值；none → null；NaN 透传。
  function _zwCCSpecNum(sp, scale) {
    if (!sp || sp.none) return null;
    if (sp.num != null) return sp.num;
    if (sp.pct != null) return sp.pct * scale / 100;
    if (sp.angle != null) return sp.angle;
    if (sp.calc != null) {
      var r = _zwCCEval(sp.calc);
      if (r !== r) return NaN;
      return r.pct ? r.v * scale / 100 : r.v;
    }
    return NaN;
  }
  // 角度/数值统一取角度（hue）。
  function _zwCCSpecAngle(sp) {
    return _zwCCSpecNum(sp, 360);
  }

  // ============ 解析 AST ============
  // 形：{kind:'named',rgb} {kind:'transparent'} {kind:'system',name}
  //     {kind:'currentcolor'} {kind:'rgbhex',comps,alpha}
  //     {kind:'rgb'|'hsl'|'hwb'|'lab'|'lch'|'oklab'|'oklch',comps,alpha,legacy}
  //     {kind:'color',space,comps,alpha}
  //     {kind:'mix',space,hueMethod,c1:{color,w},c2:{color,w}}
  //     {kind:'relative',fn,space,origin,comps,alpha}

  function _zwCCAlphaSpec(alphaText) {
    // 缺省 alpha 槽标 omitted（relative 输出 = origin alpha，css-color-5
    // relative syntax；非 relative 路径 _zwCCAlphaNum 仍按 1）。
    if (alphaText == null) return { num: 1, omitted: true };
    var t = String(alphaText).trim();
    var low = t.toLowerCase();
    if (_zwCCChanKw.indexOf(low) >= 0) return { chan: low };
    // sibling-index()/sibling-count() 槽文本存 chanexpr（resolve 期
    // _zwCCEvalRel 尾参 el 求值；裸函数非 calc 文法，_zwCCCalcSpec 不认）。
    if (t.indexOf('sibling-') >= 0) return { chanexpr: t };
    var calc = _zwCCCalcSpec(t);
    if (calc) return calc;
    return _zwCCNumSpec(t) || { chan: low };
  }

  // 相对色检测：rest 以 'from' 起（css-color-5 § relative colors 语法）。
  function _zwCCTryRelative(rest) {
    var t = String(rest).trim();
    if (!/^from(\s|$|\()/i.test(t)) return null;
    var rest2 = t.replace(/^from\s*/i, '');
    rest2 = rest2.trim();
    // origin 可为 ident/hex（token 截取）或函数形（'(' 起括号平衡扫描——
    // 'rgba(255, 0, 0, 0.3)' 起点非 '(' 也要走扫描，token 正则会把函数实参
    // 逗号误作边界）。
    var pm = /^([^\s(]*\()/.exec(rest2);
    if (pm) {
      var depth = 0;
      for (var i = pm[1].length - 1; i < rest2.length; i++) {
        var c = rest2.charAt(i);
        if (c === '(') depth++;
        else if (c === ')') { depth--; if (depth === 0) return { origin: rest2.slice(0, i + 1), rest: rest2.slice(i + 1) }; }
      }
      return null;
    }
    var tm = /^([^\s(]+)\s*([\s\S]*)$/.exec(rest2);
    if (!tm) return null;
    return { origin: tm[1], rest: tm[2] };
  }

  // 函数实参切分：现代语法（空格分量 + '/' alpha）；分量按括号深度 0 的
  // 空白切分——'calc(g * 2)' 等带内嵌函数的通道槽不被拆碎。
  function _zwCCSplitModern(inner) {
    var depth = 0, slashAt = -1;
    for (var i = 0; i < inner.length; i++) {
      var ch = inner.charAt(i);
      if (ch === '(') depth++;
      else if (ch === ')') depth--;
      else if (ch === '/' && depth === 0) { slashAt = i; break; }
    }
    var body = slashAt >= 0 ? inner.slice(0, slashAt) : inner;
    var alpha = slashAt >= 0 ? inner.slice(slashAt + 1).trim() : null;
    var comps = [], cur = '', d2 = 0;
    for (var j = 0; j < body.length; j++) {
      var c2 = body.charAt(j);
      if (c2 === '(') d2++;
      else if (c2 === ')') d2--;
      if (d2 === 0 && /\s/.test(c2)) {
        if (cur) { comps.push(cur); cur = ''; }
        continue;
      }
      cur += c2;
    }
    if (cur) comps.push(cur);
    return { comps: comps, alpha: alpha };
  }

  function _zwCCParseComps(list, count) {
    if (!list || list.length !== count) return null;
    var comps = [];
    for (var i = 0; i < count; i++) {
      var sp = _zwCCCompSpec(list[i]);
      if (!sp) return null;
      comps.push(sp);
    }
    return comps;
  }

  function _zwCCParseFn(fnRaw, inner) {
    var fn = _zwCCDecodeEscapes(fnRaw).toLowerCase();
    var legacy = inner.indexOf(',') >= 0;
    var rel;
    if (fn === 'rgb' || fn === 'rgba') {
      rel = _zwCCTryRelative(inner);
      if (rel) return _zwCCParseRelative(fn, 'srgb', rel);
      if (legacy) {
        var parts = _zwCCTopSplit(inner, ',');
        if (parts.length !== 3 && parts.length !== 4) return null;
        var comps = _zwCCParseComps(parts.slice(0, 3).map(function (s) { return s.trim(); }), 3);
        if (!comps) return null;
        for (var i = 0; i < 3; i++) if (comps[i].none) return null; // legacy 无 none
        var al = parts.length === 4 ? _zwCCAlphaSpec(parts[3]) : { num: 1 };
        if (al.none) return null;
        return { kind: 'rgb', comps: comps, alpha: al, legacy: true };
      }
      var mod = _zwCCSplitModern(inner);
      var comps2 = _zwCCParseComps(mod.comps, 3);
      if (!comps2) return null;
      return { kind: 'rgb', comps: comps2, alpha: _zwCCAlphaSpec(mod.alpha) };
    }
    if (fn === 'hsl' || fn === 'hsla') {
      rel = _zwCCTryRelative(inner);
      if (rel) return _zwCCParseRelative(fn, 'hsl', rel);
      if (legacy) {
        var parts2 = _zwCCTopSplit(inner, ',');
        if (parts2.length !== 3 && parts2.length !== 4) return null;
        var comps3 = _zwCCParseComps(parts2.slice(0, 3).map(function (s) { return s.trim(); }), 3);
        if (!comps3) return null;
        for (var j = 0; j < 3; j++) if (comps3[j].none) return null;
        var al2 = parts2.length === 4 ? _zwCCAlphaSpec(parts2[3]) : { num: 1 };
        if (al2.none) return null;
        return { kind: 'hsl', comps: comps3, alpha: al2, legacy: true };
      }
      var mod2 = _zwCCSplitModern(inner);
      var comps4 = _zwCCParseComps(mod2.comps, 3);
      if (!comps4) return null;
      return { kind: 'hsl', comps: comps4, alpha: _zwCCAlphaSpec(mod2.alpha) };
    }
    if (fn === 'hwb') {
      rel = _zwCCTryRelative(inner);
      if (rel) return _zwCCParseRelative(fn, 'hwb', rel);
      if (legacy) return null; // hwb 无逗号形（css-color-4 语法）
      var mod3 = _zwCCSplitModern(inner);
      var comps5 = _zwCCParseComps(mod3.comps, 3);
      if (!comps5) return null;
      return { kind: 'hwb', comps: comps5, alpha: _zwCCAlphaSpec(mod3.alpha) };
    }
    if (fn === 'lab' || fn === 'lch' || fn === 'oklab' || fn === 'oklch') {
      rel = _zwCCTryRelative(inner);
      if (rel) return _zwCCParseRelative(fn, fn, rel);
      var mod4 = _zwCCSplitModern(inner);
      var comps6 = _zwCCParseComps(mod4.comps, 3);
      if (!comps6) return null;
      return { kind: fn, comps: comps6, alpha: _zwCCAlphaSpec(mod4.alpha) };
    }
    if (fn === 'color') {
      rel = _zwCCTryRelative(inner);
      if (rel) return _zwCCParseRelative('color', null, rel);
      var mod5 = _zwCCSplitModern(inner);
      if (!mod5.comps || mod5.comps.length !== 4) return null;
      var space = _zwCCDecodeEscapes(mod5.comps[0]).toLowerCase();
      if (_zwCCSpaces[space]) space = _zwCCSpaces[space].alias || space;
      else if (space !== 'lab' && space !== 'lch' && space !== 'oklab' && space !== 'oklch') return null;
      var comps7 = _zwCCParseComps(mod5.comps.slice(1), 3);
      if (!comps7) return null;
      return { kind: 'color', space: space, comps: comps7, alpha: _zwCCAlphaSpec(mod5.alpha) };
    }
    if (fn === 'alpha') {
      // css-color-5 § the alpha color function：alpha(from <color> / <alpha>?)——
      // alpha 替换、通道原样；缺省 alpha-value = 原 alpha。
      var relA = _zwCCTryRelative(inner);
      if (!relA) return null;
      var restA = relA.rest.trim();
      var alphaSpecA = null;
      if (restA.charAt(0) === '/') {
        var at = restA.slice(1).trim();
        if (!at) return null;
        alphaSpecA = _zwCCAlphaSpec(at);
      }
      return { kind: 'alpha', origin: relA.origin, alphaSpec: alphaSpecA };
    }
    if (fn === 'color-mix') return _zwCCParseMix(inner);
    if (fn === 'light-dark') {
      // light-dark( <color>#, <color># )（css-color-5 §light-dark）：恰好两组
      // 逗号分隔（组内多值 fallback 链是 tentative 语法，corpus 钉面恒 2 组）。
      // 支值暂存 AST 延迟 resolve（currentColor 支须按属性上下文求值）。
      var ldGroups = _zwCCTopSplit(inner, ',');
      if (ldGroups.length !== 2) return null;
      var ldL = _zwCCParseColorValue(ldGroups[0].trim());
      var ldD = _zwCCParseColorValue(ldGroups[1].trim());
      if (!ldL || !ldD) return null;
      return { kind: 'lightdark', light: ldL, dark: ldD };
    }
    return null;
  }

  // color-mix([in <space> [hueMethod hue]]?, <color> [w%]?, <color> [w%]?)
  //——'in' 空间子句可省（css-color-5：corpus "Absence of colorspace means
  // OKLab"，'color-mix(25% oklab(...), oklab(...))' 前置乘数形同分支）。
  function _zwCCParseMix(inner) {
    var args = _zwCCTopSplit(inner, ',');
    if (args.length < 2) return null;
    var hm = /^in\s+([a-z0-9-]+)(?:\s+(shorter|longer|increasing|decreasing)\s+hue)?$/i.exec(args[0].trim());
    var space, hueMethod, sideArgs;
    if (hm) {
      space = hm[1].toLowerCase();
      hueMethod = hm[2] ? hm[2].toLowerCase() : 'shorter';
      sideArgs = args.slice(1);
    } else {
      space = 'oklab';
      hueMethod = 'shorter';
      sideArgs = args;
    }
    var valid = ['srgb', 'srgb-linear', 'display-p3', 'display-p3-linear', 'a98-rgb',
      'prophoto-rgb', 'rec2020', 'xyz', 'xyz-d50', 'xyz-d65', 'hsl', 'hwb', 'lab', 'lch', 'oklab', 'oklch'];
    if (valid.indexOf(space) < 0) return null;
    // 别名折叠：'in xyz' ≡ 'in xyz-d65'（css-color-4/5 xyz 别名；与 color()
    // 解析侧同规）。side 自空间解析已折叠（'color(xyz …)' → space 'xyz-d65'），
    // _zwCCMixSide 按 space 全等匹配 own 形——不折叠则 'color-mix(in xyz,
    // color(xyz none …), …)' side 匹配失败走 rgb 回退丢 none 旗标（none→0 全
    // 权重归一，WPT color-computed-color-mix-function 13 案 2026-11-11）。
    if (_zwCCSpaces[space]) space = _zwCCSpaces[space].alias || space;
    function calcW(expr) {
      var ev = _zwCCEval(expr);
      if (ev !== ev) return null;
      var v = ev.pct ? ev.v / 100 : ev.v;
      return Math.min(1, Math.max(0, v));
    }
    function parseSide(text) {
      var t = text.trim();
      var lead = /^([-+]?(\d+\.?\d*|\.\d+))%(?![\d.])\s+([\s\S]+)$/.exec(t);
      if (lead) {
        var lw = parseFloat(lead[1]) / 100;
        // 权重域 [0%,100%]，域外整值非法（css-color-5 §color-mix；corpus
        // '-10%'/'150%' 拒面）。
        if (lw < 0 || lw > 1) return null;
        return { w: lw, color: lead[3].trim() };
      }
      // calc() 权重形：'red calc(10%)'（css-color-5 weight syntax）。
      var leadCalc = /^(calc\([^()]*\))\s+([\s\S]+)$/.exec(t);
      if (leadCalc) {
        var cw = calcW(leadCalc[1]);
        if (cw == null) return null;
        return { w: cw, color: leadCalc[2].trim() };
      }
      // 尾部 % 形：尾 % 不得落在函数括号内。
      var trail = /^([\s\S]+?)\s+([-+]?(\d+\.?\d*|\.\d+))%$/.exec(t);
      if (trail) {
        var d = 0;
        for (var i = 0; i < trail[1].length; i++) {
          if (trail[1].charAt(i) === '(') d++;
          else if (trail[1].charAt(i) === ')') d--;
        }
        if (d === 0) {
          var tw = parseFloat(trail[2]) / 100;
          if (tw < 0 || tw > 1) return null; // 域外权重拒（同 lead 臂）
          return { w: tw, color: trail[1].trim() };
        }
      }
      // 尾 calc 形：'blue calc(10%)'（同括号平衡检查）。
      var trailCalc = /^([\s\S]+?)\s+(calc\([^()]*\))$/.exec(t);
      if (trailCalc) {
        var d2 = 0;
        for (var i2 = 0; i2 < trailCalc[1].length; i2++) {
          if (trailCalc[1].charAt(i2) === '(') d2++;
          else if (trailCalc[1].charAt(i2) === ')') d2--;
        }
        if (d2 === 0) {
          var cw2 = calcW(trailCalc[2]);
          if (cw2 == null) return null;
          return { w: cw2, color: trailCalc[1].trim() };
        }
      }
      // 缺省权重 null（css-color-5：两参形缺省侧 = 100% − 对侧；N 参形等权）。
      return { w: null, color: t };
    }
    var sides = [];
    for (var si = 0; si < sideArgs.length; si++) {
      var sd = parseSide(sideArgs[si]);
      if (!sd) return null;
      var cv = _zwCCParseColorValue(sd.color);
      if (!cv) return null;
      sides.push({ color: cv, w: sd.w });
    }
    // 单参形：'color-mix(in srgb, red)' ≡ 权重 100%（alpha 随权重缩放）。
    if (sides.length === 1) {
      if (sides[0].w == null) sides[0].w = 1;
      sides.push({ color: sides[0].color, w: 0 });
    }
    return { kind: 'mix', space: space, hueMethod: hueMethod, sides: sides };
  }

  // 相对色解析。
  function _zwCCParseRelative(fnRaw, space, rel) {
    var fn = fnRaw.toLowerCase();
    if (fn === 'color') {
      var mod = _zwCCSplitModern(rel.rest);
      if (!mod.comps || mod.comps.length !== 4) return null;
      var sp = _zwCCDecodeEscapes(mod.comps[0]).toLowerCase();
      if (_zwCCSpaces[sp]) sp = _zwCCSpaces[sp].alias || sp;
      else if (sp !== 'lab' && sp !== 'lch' && sp !== 'oklab' && sp !== 'oklch') return null;
      var comps = [];
      for (var i = 1; i < 4; i++) {
        var c = _zwCCParseCompRel(mod.comps[i]);
        if (!c) return null;
        comps.push(c);
      }
      return { kind: 'relative', fn: 'color', space: sp, origin: rel.origin, comps: comps, alpha: _zwCCAlphaSpec(mod.alpha) };
    }
    var mod2 = _zwCCSplitModern(rel.rest);
    if (!mod2.comps || mod2.comps.length !== 3) return null;
    var comps2 = [];
    for (var j = 0; j < 3; j++) {
      var c2 = _zwCCParseCompRel(mod2.comps[j]);
      if (!c2) return null;
      comps2.push(c2);
    }
    return { kind: 'relative', fn: fn, space: space, origin: rel.origin, comps: comps2, alpha: _zwCCAlphaSpec(mod2.alpha) };
  }

  function _zwCCParseCompRel(text) {
    var t = String(text).trim();
    var low = t.toLowerCase();
    if (_zwCCChanKw.indexOf(low) >= 0) return { chan: low };
    // sibling-index()/sibling-count() 槽存 chanexpr（裸函数非 calc 文法；
    // resolve 期 _zwCCEvalRel 尾参 el 求值——relative-color-with-system-color
    // test3 钉面）。
    if (t.indexOf('sibling-') >= 0) return { chanexpr: t };
    var calc = _zwCCCalcSpec(t);
    if (calc) return calc;
    var num = _zwCCNumSpec(t);
    return num || null;
  }

  // 顶层值解析入口（函数/命名/hex/关键字）。
  function _zwCCParseColorValue(text) {
    var s = _zwCCStripComments(text).trim();
    if (!s) return null;
    var low = s.toLowerCase();
    if (low === 'currentcolor') return { kind: 'currentcolor' };
    if (low === 'transparent') return { kind: 'transparent' };
    if (low === 'none') return { kind: 'transparent' };
    if (s.charAt(0) === '#') return _zwCCParseHex(s);
    var fm = /^([a-zA-Z][a-zA-Z0-9\\ -]*)\(\s*([\s\S]*)\)\s*$/.exec(s);
    if (fm) {
      // 先解转义再去空白：'r\67 b' 的 '\67 ' 依赖反斜杠+空格成 'g'，反序会变成 'r67b'。
      var fnName = _zwCCDecodeEscapes(fm[1]).replace(/[\s\\]+/g, '');
      return _zwCCParseFn(fnName, fm[2]);
    }
    var ident = _zwCCDecodeEscapes(s).toLowerCase();
    if (_zwCCNamed[ident]) return { kind: 'named', rgb: _zwCCNamed[ident] };
    if (Object.prototype.hasOwnProperty.call(_zwCCSystem, ident)) return { kind: 'system', name: ident };
    return null;
  }

  function _zwCCParseHex(s) {
    var hex = s.slice(1);
    var m3 = /^([0-9a-fA-F])([0-9a-fA-F])([0-9a-fA-F])$/.exec(hex);
    var m4 = /^([0-9a-fA-F])([0-9a-fA-F])([0-9a-fA-F])([0-9a-fA-F])$/.exec(hex);
    var m6 = /^([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})$/.exec(hex);
    var m8 = /^([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})$/.exec(hex);
    function h2(p) { return parseInt(p, 16); }
    if (m3) return { kind: 'rgbhex', comps: [{ num: h2(m3[1]) * 17 / 255 }, { num: h2(m3[2]) * 17 / 255 }, { num: h2(m3[3]) * 17 / 255 }], alpha: { num: 1 } };
    if (m4) return { kind: 'rgbhex', comps: [{ num: h2(m4[1]) * 17 / 255 }, { num: h2(m4[2]) * 17 / 255 }, { num: h2(m4[3]) * 17 / 255 }], alpha: { num: h2(m4[4]) * 17 / 255 } };
    if (m6) return { kind: 'rgbhex', comps: [{ num: h2(m6[1]) / 255 }, { num: h2(m6[2]) / 255 }, { num: h2(m6[3]) / 255 }], alpha: { num: 1 } };
    if (m8) return { kind: 'rgbhex', comps: [{ num: h2(m8[1]) / 255 }, { num: h2(m8[2]) / 255 }, { num: h2(m8[3]) / 255 }], alpha: { num: h2(m8[4]) / 255 } };
    return null;
  }

  // ============ 求值 ============
  // 解析形：{kind:'rgb', rgb:[0..1], alpha, aNone, hasNone, srcComps, srcKind}
  //         {kind:'own', fn|space, comps:[num|'none'], alpha, aNone}
  //           （lab 族/color() 自空间保形——往返稳定与 computed 保形共用）
  //         {kind:'labmix', space, coords, none, alpha}
  //         {kind:'colormix', space, comps, none, alpha}
  //         {kind:'relative-out', space, comps, none, alpha}
  //         {kind:'system', text}

  function _zwCCAlphaNum(sp) {
    if (!sp || sp.none) return 0;
    var v = _zwCCSpecNum(sp, 1);
    if (v == null || v !== v) return 0;
    return Math.min(1, Math.max(0, v));
  }
  function _zwCCIsNone(sp) { return !!(sp && sp.none); }

  // 元素 inline 声明/自定义属性（currentcolor/var() 通道；沿父链）。
  function _zwCCInlineDecl(el, prop) {
    try {
      if (el && el.style && typeof el.style.getPropertyValue === 'function') {
        var v = el.style.getPropertyValue(prop);
        if (v) return String(v).trim();
      }
    } catch (_eIC) {}
    return '';
  }
  function _zwCCVarLookup(el, name) {
    var p = el, guard = 0;
    while (p && p.nodeType === 1 && guard++ < 256) {
      var v = _zwCCInlineDecl(p, name);
      if (v) return v;
      p = p.parentNode;
    }
    // 作者样式表臂（页级 `html { --x: ... }` 声明——inline 链查空后经 part01c
    // 作者快照沿父链首声明；color-computed-relative-color var(--accent/--color/
    // --mycolor/--mygray) 13 案钉面）。
    if (typeof globalThis.__zwPlainAuthorDecl === 'function') {
      var q = el; guard = 0;
      while (q && q.nodeType === 1 && guard++ < 256) {
        var v2 = globalThis.__zwPlainAuthorDecl(q, name);
        if (v2) return String(v2);
        q = q.parentNode;
      }
    }
    return '';
  }

  function _zwCCResolve(ast, el, depth) {
    if (!ast || depth > 16) return null;
    switch (ast.kind) {
      case 'named':
        return { kind: 'rgb', rgb: [ast.rgb[0] / 255, ast.rgb[1] / 255, ast.rgb[2] / 255], alpha: 1, aNone: false, hasNone: false };
      case 'transparent':
        return { kind: 'rgb', rgb: [0, 0, 0], alpha: 0, aNone: false, hasNone: false };
      case 'system': {
        // dark used-scheme 分派（css-color-4 § system colors）：used dark 且
        // dark 面板有值时 dark 优先（linktext/visitedtext 等非空 light 静态值
        // 键若 light 优先则 dark 分派永不生效——v9 记录轮 3 案实证），否则
        // light 面板值（既有静态值/空串口径）。
        var txt = (_zwCCUsedSchemeDark(el) && _zwCCSystemDark[ast.name])
          ? _zwCCSystemDark[ast.name]
          : _zwCCSystem[ast.name];
        return { kind: 'system', text: txt || '' };
      }
      case 'lightdark': {
        // light-dark(L, D) used-scheme 选支（css-color-5 §light-dark）：
        // https://drafts.csswg.org/css-color-5/#light-dark 。支值递归 resolve——
        // currentcolor 支走既有 currentcolor 臂（propCtx 'color' 时父链语义
        // 保持：currentColor 在 color 属性自引用 = 继承值，
        // light-dark-currentcolor-in-color 钉面）。
        var wantDark = _zwCCUsedSchemeDark(el);
        return _zwCCResolve(wantDark ? ast.dark : ast.light, el, depth + 1);
      }
      case 'currentcolor': {
        // currentcolor = 本元素 color 计算值（css-color-4：currentColor 值 =
        // 继承的 color）。委托 __zwPlainComputed(p, 'color', true)——qvMode
        // 臂恒开（WPT computed 页锚：#container{color:rgb(255,0,0)} 样式表
        // 声明，inline-only 爬升读不到）；part11 再解析（legacy 值原形透传）。
        // 非 color 属性上下文先试本元素（css-color-4 §5.3：currentcolor 用于
        // background-color 等属性取本元素 color 计算值——relative-color 语料
        // target inline color 实证，仅爬父链漏自元素恒黑）；color 属性自身
        // （_zwCCPropCtx 'color'）保持父链优先（自引用=继承语义，防自递归）。
        if (_zwCCPropCtx !== 'color' && el && el.nodeType === 1
            && typeof globalThis.__zwPlainComputed === 'function') {
          var cstrSelf = '';
          try { cstrSelf = globalThis.__zwPlainComputed(el, 'color', true); } catch (_eCCSE) { cstrSelf = ''; }
          if (cstrSelf) {
            var subSelf = _zwCCResolveText(cstrSelf, el, depth + 1);
            if (subSelf) return subSelf;
          }
        }
        var p = el && el.parentNode, guard = 0;
        while (p && p.nodeType === 1 && guard++ < 256) {
          var cstr = '';
          if (typeof globalThis.__zwPlainComputed === 'function') {
            try { cstr = globalThis.__zwPlainComputed(p, 'color', true); } catch (_eCCC) { cstr = ''; }
          }
          if (cstr) {
            var sub = _zwCCResolveText(cstr, p, depth + 1);
            if (sub) return sub;
          }
          p = p.parentNode;
        }
        return { kind: 'rgb', rgb: [0, 0, 0], alpha: 1, aNone: false, hasNone: false };
      }
      case 'rgbhex':
        return { kind: 'rgb',
          rgb: [_zwCCSpecNum(ast.comps[0], 1), _zwCCSpecNum(ast.comps[1], 1), _zwCCSpecNum(ast.comps[2], 1)],
          alpha: _zwCCAlphaNum(ast.alpha), aNone: _zwCCIsNone(ast.alpha), hasNone: false };
      case 'rgb': {
        var rs = [], anyNone = false;
        for (var i = 0; i < 3; i++) {
          var v = _zwCCSpecNum(ast.comps[i], 255);
          if (v == null) { rs.push('none'); anyNone = true; }
          else if (v !== v) rs.push(0); // calc NaN → 0（css-color-4，非 none）
          else rs.push(v / 255);
        }
        return { kind: 'rgb',
          rgb: rs.map(function (x) { return x === 'none' ? 0 : x; }),
          alpha: _zwCCAlphaNum(ast.alpha), aNone: _zwCCIsNone(ast.alpha), hasNone: anyNone,
          srcComps: rs, srcKind: 'rgb' };
      }
      case 'hsl': {
        var h = _zwCCSpecAngle(ast.comps[0]);
        var s = _zwCCSpecNum(ast.comps[1], 100), l = _zwCCSpecNum(ast.comps[2], 100);
        // none 标记只认显式 none（calc NaN → 0 值非 none，css-color-4）。
        var hN = h == null, sN = s == null, lN = l == null;
        var anyNone2 = hN || sN || lN;
        var hV = (hN || h !== h) ? 0 : h;
        var sV = (sN || s !== s) ? 0 : Math.min(100, Math.max(0, s));
        var lV = (lN || l !== l) ? 0 : Math.min(100, Math.max(0, l));
        var rgb = _zwCCHslToRgb(hV, sV / 100, lV / 100);
        // srcBare：S/L 槽 % 语法跟随（css-color-4 computed 序列化 echo 输入形
        // ——'hsl(120 80% none)' → '% 形'、RT 'hsl(270 50 none)' → 裸形；
        // % 输入声明面剥 % 后由 part03 raw 影子供原文，裸输入无影子走 store
        // 形——两侧判据一致；数组槽位 = [sBare, lBare]）。
        return { kind: 'rgb', rgb: rgb,
          alpha: _zwCCAlphaNum(ast.alpha), aNone: _zwCCIsNone(ast.alpha), hasNone: anyNone2,
          srcComps: [hN ? 'none' : hV, sN ? 'none' : sV, lN ? 'none' : lV], srcKind: 'hsl',
          srcBare: [!(ast.comps[1] && ast.comps[1].pct != null), !(ast.comps[2] && ast.comps[2].pct != null)] };
      }
      case 'hwb': {
        var h2 = _zwCCSpecAngle(ast.comps[0]);
        var w = _zwCCSpecNum(ast.comps[1], 100), b = _zwCCSpecNum(ast.comps[2], 100);
        var h2N = h2 == null, wN = w == null, bN = b == null;
        var anyNone3 = h2N || wN || bN;
        var h2V = (h2N || h2 !== h2) ? 0 : h2;
        var wV = (wN || w !== w) ? 0 : Math.min(100, Math.max(0, w));
        var bV = (bN || b !== b) ? 0 : Math.min(100, Math.max(0, b));
        var rgb2 = _zwCCHwbToRgb(h2V, wV / 100, bV / 100);
        // srcBare：W/B 槽 % 语法跟随（同 hsl 臂——数组槽位 = [wBare, bBare]）。
        return { kind: 'rgb', rgb: rgb2,
          alpha: _zwCCAlphaNum(ast.alpha), aNone: _zwCCIsNone(ast.alpha), hasNone: anyNone3,
          srcComps: [h2N ? 'none' : h2V, wN ? 'none' : wV, bN ? 'none' : bV], srcKind: 'hwb',
          srcBare: [!(ast.comps[1] && ast.comps[1].pct != null), !(ast.comps[2] && ast.comps[2].pct != null)] };
      }
      case 'lab': case 'lch': case 'oklab': case 'oklch': {
        var isOk = (ast.kind === 'oklab' || ast.kind === 'oklch');
        var isPolar = (ast.kind === 'lch' || ast.kind === 'oklch');
        var Ls = _zwCCSpecNum(ast.comps[0], isOk ? 1 : 100);
        var Cs = isPolar ? _zwCCSpecNum(ast.comps[1], isOk ? 0.4 : 150) : _zwCCSpecNum(ast.comps[1], isOk ? 0.4 : 125);
        var Hs = isPolar ? _zwCCSpecAngle(ast.comps[2]) : null;
        var Bs = isPolar ? null : _zwCCSpecNum(ast.comps[2], isOk ? 0.4 : 125);
        var lN2 = Ls == null, cN2 = Cs == null;
        var hN3 = isPolar ? Hs == null : false;
        var bN4 = isPolar ? false : Bs == null;
        var anyNone4 = lN2 || cN2 || hN3 || bN4;
        var L = (lN2 || Ls !== Ls) ? 0 : (isOk ? Math.min(1, Math.max(0, Ls)) : Math.min(100, Math.max(0, Ls)));
        // Cartesian a/b 可负可超界不 clamp（css-color-4：lab() a/b 无界；仅
        // polar C 非负）——'lab(50 -160 160)' 保负值语料锚。
        var C = (cN2 || Cs !== Cs) ? 0 : (isPolar ? Math.max(0, Cs) : Cs);
        var H = (hN3 || Hs !== Hs) ? 0 : (Hs == null ? 0 : ((Hs % 360) + 360) % 360);
        var B = (bN4 || Bs !== Bs) ? 0 : Bs;
        var labC = isPolar ? [L, C, H] : [L, C, B];
        // rgb 投影（mix/origin 消费面）。
        var labAbs = isPolar ? [L, C * Math.cos(H * Math.PI / 180), C * Math.sin(H * Math.PI / 180)] : [L, C, B];
        var xyz = isOk ? _zwCCOklabToXyz(labAbs) : _zwCCMatVec(_zwCCD50toD65, _zwCCLabToXyz(labAbs));
        var lin = _zwCCFromXyz65('srgb', xyz);
        var rgb3 = [_zwCCLinToSrgb(lin[0]), _zwCCLinToSrgb(lin[1]), _zwCCLinToSrgb(lin[2])];
        // own 形：lab 族 computed 保自形（'lab(50% 20 -30)' → 'lab(50 20 -30)'，
        // 不投 legacy rgb——round-trip 与 Chrome 序列化口径，同 color() 自形）。
        return { kind: 'rgb', rgb: rgb3,
          alpha: _zwCCAlphaNum(ast.alpha), aNone: _zwCCIsNone(ast.alpha), hasNone: anyNone4,
          srcComps: [lN2 ? 'none' : labC[0], cN2 ? 'none' : labC[1], (hN3 || bN4) ? 'none' : labC[2]],
          srcKind: ast.kind,
          own: { fn: ast.kind, comps: [lN2 ? 'none' : labC[0], cN2 ? 'none' : labC[1], (hN3 || bN4) ? 'none' : labC[2]] } };
      }
      case 'color': {
        var cs = [], anyNone5 = false;
        for (var i2 = 0; i2 < 3; i2++) {
          var v2 = _zwCCSpecNum(ast.comps[i2], 1);
          if (v2 == null) { cs.push('none'); anyNone5 = true; }
          else if (v2 !== v2) cs.push(0); // calc NaN → 0 非 none
          else cs.push(v2);
        }
        // 自空间坐标（往返稳定序列化）。
        var nums = cs.map(function (x) { return x === 'none' ? 0 : x; });
        if (ast.space === 'lab' || ast.space === 'lch' || ast.space === 'oklab' || ast.space === 'oklch') {
          return { kind: 'own', fn: ast.space, comps: cs, alpha: _zwCCAlphaNum(ast.alpha), aNone: _zwCCIsNone(ast.alpha),
            rgb: null };
        }
        var xyz3 = _zwCCToXyz65(ast.space, _zwCCToLin(ast.space, nums));
        var lin4 = _zwCCFromXyz65('srgb', xyz3);
        var rgb5 = [_zwCCLinToSrgb(lin4[0]), _zwCCLinToSrgb(lin4[1]), _zwCCLinToSrgb(lin4[2])];
        return { kind: 'own', space: ast.space, comps: cs, alpha: _zwCCAlphaNum(ast.alpha), aNone: _zwCCIsNone(ast.alpha), rgb: rgb5, hasNone: anyNone5 };
      }
      case 'mix':
        return _zwCCResolveMix(ast, el, depth + 1);
      case 'alpha': {
        // origin 全解析（own 形带 rgb 投影、rgb 形带 srcComps none 掩码）。
        var ar = _zwCCResolveText(ast.origin, el, depth + 1);
        if (!ar || !ar.rgb || ar.kind === 'system') return null;
        var nA;
        if (ast.alphaSpec == null) nA = ar.alpha;
        else if (ast.alphaSpec.none) nA = 'none';
        else if (ast.alphaSpec.chan != null) {
          if (ast.alphaSpec.chan !== 'alpha') return null;
          nA = ar.alpha;
        } else if (ast.alphaSpec.chanexpr != null) {
          var evA3 = _zwCCEvalRel(ast.alphaSpec.chanexpr, null, null, 1, ar.alpha, el);
          if (evA3 !== evA3) return null;
          nA = Math.min(1, Math.max(0, evA3));
        } else {
          var vA2 = _zwCCSpecNum(ast.alphaSpec, 1);
          if (vA2 == null || vA2 !== vA2) return null;
          nA = Math.min(1, Math.max(0, vA2));
        }
        if (ar.kind === 'own') {
          // 自空间 origin（color()/lab 族）保自形——'alpha(from color(display-p3
          // 1 0 0) / 0.5)' → 'color(display-p3 1 0 0 / 0.5)'。
          var ownNone = ar.comps.map(function (x) { return x === 'none'; });
          return { kind: 'alpha-out', own: { fn: ar.fn, space: ar.space, comps: ar.comps, none: ownNone }, rgb: ar.rgb, alpha: nA };
        }
        var nmA = [false, false, false];
        if (ar.hasNone && ar.srcComps) {
          for (var qi = 0; qi < 3; qi++) if (ar.srcComps[qi] === 'none') nmA[qi] = true;
        }
        // srgb 族 origin → 函数形输出（'alpha(from red / 0.5)' →
        // 'color(srgb 1 0 0 / 0.5)'，非 legacy rgba——css-color-5 序列化）。
        // chans：origin 原空间坐标（srcKind 对应空间嵌套免投影往返）。
        var chansA = ar.srcComps ? ar.srcComps.slice() : null;
        return { kind: 'alpha-out', rel: { comps: ar.rgb, none: nmA }, rgb: ar.rgb, alpha: nA, chans: chansA, srcKind: ar.srcKind };
      }
      case 'relative':
        return _zwCCResolveRelative(ast, el, depth + 1);
    }
    return null;
  }

  // 文本 → 解析颜色（var() 展开；currentcolor 沿父链；无效返 null）。
  function _zwCCResolveText(text, el, depth) {
    var s = String(text == null ? '' : text).trim();
    s = _zwCCSubSibling(s, el);
    var vm = /^([\s\S]*?)var\(\s*(--[a-zA-Z0-9_-]+)\s*(?:,\s*([\s\S]+?)\s*)?\)([\s\S]*)$/.exec(s);
    if (vm) {
      var val = el ? _zwCCVarLookup(el, vm[2]) : '';
      if (!val && vm[3] != null) val = vm[3];
      // supports 判定通道（el 缺席）：未定义 var 视作可替换（CSS.supports
      // ('color', 'var(--x)') Chrome 返 true——语法面不含值级联）。
      if (!val && !el) val = 'rgb(0, 0, 0)';
      if (!val) return null;
      return _zwCCResolveText(vm[1] + val + vm[4], el, depth + 1);
    }
    var ast = _zwCCParseColorValue(s);
    if (!ast) return null;
    return _zwCCResolve(ast, el, depth);
  }

  // ============ color-mix（css-color-5 § color-mix）============

  function _zwCCHueLerp(h1, h2, t, method) {
    var d = ((h2 - h1) % 360 + 360) % 360;
    if (method === 'shorter') { if (d > 180) d -= 360; }
    else if (method === 'longer') { if (d > 0 && d < 180) d -= 360; else if (d === 0) d = 360; }
    else if (method === 'decreasing') { if (d > 0) d -= 360; }
    return h1 + t * d;
  }

  // 混合侧坐标 + none 掩码：同空间原生坐标直取（免 rgb 往返投影精度损失）；
  // rgb 投影时无彩色 → hue none（css-color-4 powerless hue：hsl s=0 或明度
  // 0/1、hwb w+b≥1、lch/oklch C=0），供 mix 缺通道 carry-over。
  function _zwCCMixSide(rr, space) {
    var none = [false, false, false], chans = [];
    var ownComps = null;
    if (rr.kind === 'relative-out' && rr.space === space) {
      for (var ri = 0; ri < 3; ri++) {
        if (rr.none && rr.none[ri]) { none[ri] = true; chans.push(0); }
        else chans.push(Number(rr.comps[ri]));
      }
      return { chans: chans, none: none };
    }
    if (rr.srcKind === space && rr.srcComps) ownComps = rr.srcComps;
    else if (rr.kind === 'own' && rr.comps && (rr.fn === space || rr.space === space)) ownComps = rr.comps;
    if (ownComps) {
      for (var i = 0; i < 3; i++) {
        if (ownComps[i] === 'none') { none[i] = true; chans.push(0); }
        else chans.push(Number(ownComps[i]));
      }
      return { chans: chans, none: none };
    }
    var rgb = rr.rgb || [0, 0, 0];
    if (space === 'srgb') return { chans: rgb.slice(), none: none };
    if (space === 'hsl') {
      var v = _zwCCRgbToHsl(rgb[0], rgb[1], rgb[2]);
      none[0] = v[1] <= 1e-9 || v[2] <= 0 || v[2] >= 1;
      return { chans: [v[0], v[1] * 100, v[2] * 100], none: none };
    }
    if (space === 'hwb') {
      var v2 = _zwCCRgbToHwb(rgb[0], rgb[1], rgb[2]);
      none[0] = v2[1] + v2[2] >= 1 - 1e-9;
      return { chans: [v2[0], v2[1] * 100, v2[2] * 100], none: none };
    }
    if (space === 'lab' || space === 'lch') {
      var lab = _zwCCXyzToLab(_zwCCMatVec(_zwCCD65toD50, _zwCCToXyz65('srgb', _zwCCToLin('srgb', rgb))));
      // 灰色 snap：白/黑/灰 a=b=0（Chrome lab 口径，roundtrip 噪声会污染
      // C 与 hue carry）。阈值 1e-3：矩阵链灰噪声实测 ≤1.4e-4，语料最小
      // 真彩色通道差 ≥1.3e-2，两侧各留一个数量级。
      if (Math.max(Math.abs(rgb[0] - rgb[1]), Math.abs(rgb[1] - rgb[2])) < 1e-3) lab = [lab[0], 0, 0];
      if (space === 'lch') {
        var C = Math.sqrt(lab[1] * lab[1] + lab[2] * lab[2]);
        var h = Math.atan2(lab[2], lab[1]) * 180 / Math.PI;
        if (h < 0) h += 360;
        none[2] = C <= 1e-9;
        return { chans: [lab[0], C, h], none: none };
      }
      return { chans: lab.slice(), none: none };
    }
    if (space === 'oklab' || space === 'oklch') {
      var ok = _zwCCXyzToOklab(_zwCCToXyz65('srgb', _zwCCToLin('srgb', rgb)));
      var gray2 = Math.max(Math.abs(rgb[0] - rgb[1]), Math.abs(rgb[1] - rgb[2])) < 1e-3;
      if (gray2) ok = [ok[0], 0, 0];
      if (space === 'oklch') {
        var C2 = Math.sqrt(ok[1] * ok[1] + ok[2] * ok[2]);
        var h2 = Math.atan2(ok[2], ok[1]) * 180 / Math.PI;
        if (h2 < 0) h2 += 360;
        none[2] = C2 <= 1e-9;
        return { chans: [ok[0], C2, h2], none: none };
      }
      return { chans: ok.slice(), none: none };
    }
    return { chans: _zwCCFromXyz65(space, _zwCCToXyz65('srgb', _zwCCToLin('srgb', rgb))), none: none };
  }

  // 插值空间坐标 → rgb（alpha()/origin 消费面投影）。
  function _zwCCMixCoordsToRgb(space, coords) {
    if (space === 'hsl') return _zwCCHslToRgb(coords[0], coords[1] / 100, coords[2] / 100);
    if (space === 'hwb') return _zwCCHwbToRgb(coords[0], coords[1] / 100, coords[2] / 100);
    if (space === 'lab' || space === 'lch') {
      var abs = (space === 'lch')
        ? [coords[0], coords[1] * Math.cos(coords[2] * Math.PI / 180), coords[1] * Math.sin(coords[2] * Math.PI / 180)]
        : coords.slice();
      var lin = _zwCCFromXyz65('srgb', _zwCCMatVec(_zwCCD50toD65, _zwCCLabToXyz(abs)));
      return [_zwCCLinToSrgb(lin[0]), _zwCCLinToSrgb(lin[1]), _zwCCLinToSrgb(lin[2])];
    }
    if (space === 'oklab' || space === 'oklch') {
      var abs2 = (space === 'oklch')
        ? [coords[0], coords[1] * Math.cos(coords[2] * Math.PI / 180), coords[1] * Math.sin(coords[2] * Math.PI / 180)]
        : coords.slice();
      var lin2 = _zwCCFromXyz65('srgb', _zwCCOklabToXyz(abs2));
      return [_zwCCLinToSrgb(lin2[0]), _zwCCLinToSrgb(lin2[1]), _zwCCLinToSrgb(lin2[2])];
    }
    var lin3 = _zwCCFromXyz65('srgb', _zwCCToXyz65(space, _zwCCToLin(space, coords)));
    return [_zwCCLinToSrgb(lin3[0]), _zwCCLinToSrgb(lin3[1]), _zwCCLinToSrgb(lin3[2])];
  }

  function _zwCCResolveMix(ast, el, depth) {
    var n = ast.sides.length;
    var i, ci;
    var rr = [], ms = [];
    for (i = 0; i < n; i++) {
      var rz = _zwCCResolve(ast.sides[i].color, el, depth);
      if (!rz) return null;
      rr.push(rz);
      ms.push(_zwCCMixSide(rz, ast.space));
    }
    // 权重：两参形缺省侧 = 100% − 对侧；N 参形缺省等权 1；显式和 ≠ 1 →
    // 通道插值前归一（css-color-5 § color-mix normalization）；和 0 → 等分。
    var wts = [];
    for (i = 0; i < n; i++) wts.push(ast.sides[i].w);
    if (n === 2) {
      if (wts[0] == null && wts[1] == null) { wts[0] = 0.5; wts[1] = 0.5; }
      else if (wts[0] == null) wts[0] = 1 - wts[1];
      else if (wts[1] == null) wts[1] = 1 - wts[0];
    } else {
      for (i = 0; i < n; i++) if (wts[i] == null) wts[i] = 1;
    }
    var W = 0;
    for (i = 0; i < n; i++) W += wts[i];
    var was = [];
    for (i = 0; i < n; i++) was.push(W > 0 ? wts[i] / W : 1 / n);
    // alpha none 环形取后继首个非 none 侧（css-color-5 carry-over）；
    // 全 none → 结果 alpha none。
    var avs = [];
    for (i = 0; i < n; i++) avs.push((rr[i].aNone === true || rr[i].alpha === 'none') ? null : rr[i].alpha);
    var bothANone = true;
    for (i = 0; i < n; i++) if (avs[i] != null) { bothANone = false; break; }
    var als = [];
    for (i = 0; i < n; i++) {
      if (avs[i] != null) { als.push(avs[i]); continue; }
      var carried = 1;
      for (var k = 1; k <= n; k++) {
        var avN = avs[(i + k) % n];
        if (avN != null) { carried = avN; break; }
      }
      als.push(carried);
    }
    // 结果 alpha：权重和 > 1 → 归一（'30% + 90%' → 0.7 非原值和）；≤ 1 →
    // 原值和（'12.5% + 37.5%' → 0.5，css-color-5 normalization 只收缩不放大）。
    var rawSum = 0;
    for (i = 0; i < n; i++) rawSum += wts[i] * als[i];
    var alpha = Math.min(1, Math.max(0, W > 1 ? rawSum / W : rawSum));
    var Wp = 0;
    for (i = 0; i < n; i++) Wp += was[i] * als[i];
    var hueAt = (ast.space === 'hsl' || ast.space === 'hwb') ? 0 : (ast.space === 'lch' || ast.space === 'oklch') ? 2 : -1;
    var out = [], outNone = [false, false, false];
    for (ci = 0; ci < 3; ci++) {
      var allNone = true;
      for (i = 0; i < n; i++) if (!ms[i].none[ci]) { allNone = false; break; }
      if (allNone) { outNone[ci] = true; out.push(0); continue; }
      // 缺通道侧 carry 首个非 none 侧值并保留权重（css-color-5 missing
      // channel carry-over）。
      var v0 = null;
      for (i = 0; i < n; i++) if (!ms[i].none[ci]) { v0 = ms[i].chans[ci]; break; }
      var vsum = 0;
      for (i = 0; i < n; i++) {
        var vv = ms[i].none[ci] ? v0 : ms[i].chans[ci];
        vsum += vv * was[i] * als[i];
      }
      // hue 按归一位置权重插值（不预乘 alpha）；none hue 侧 carry 首个非
      // none 侧 hue（'in lch longer hue, red, white' → 白 hue 继承红后
      // d=0 走 360°）。其余通道按 alpha 预乘权重（css-color-4/5
      // interpolation：premultiplied space，hue 角除外）。
      // FIXME: N(≥3) 参极坐标 hue 顺序折返插值为近似（语料仅覆盖两参极形）。
      if (ci === hueAt) {
        var h0 = null;
        for (i = 0; i < n; i++) if (!ms[i].none[ci]) { h0 = ms[i].chans[ci]; break; }
        var hAcc = null, wAcc = 0;
        for (i = 0; i < n; i++) {
          var hv = ms[i].none[ci] ? h0 : ms[i].chans[ci];
          if (hAcc == null) { hAcc = hv; wAcc = was[i]; continue; }
          hAcc = _zwCCHueLerp(hAcc, hv, was[i] / (wAcc + was[i]), ast.hueMethod);
          wAcc += was[i];
        }
        out.push((((hAcc % 360) + 360) % 360));
      } else {
        out.push(Wp <= 0 ? 0 : vsum / Wp);
      }
    }
    var anyNone = outNone[0] || outNone[1] || outNone[2];
    if (ast.space === 'srgb' || ast.space === 'hsl' || ast.space === 'hwb') {
      // srgb 族插值域 = gamma 编码域（css-color-5 § color-mix：srgb 空间插值
      // 不解 gamma）。坐标→rgb 投影按插值空间（srgb 直用；hsl/hwb 百分坐标换算）。
      var lin;
      if (ast.space === 'hsl') lin = _zwCCHslToRgb(out[0], out[1] / 100, out[2] / 100);
      else if (ast.space === 'hwb') lin = _zwCCHwbToRgb(out[0], out[1] / 100, out[2] / 100);
      else lin = out.slice();
      // hsl/hwb 通道双 none 或结果 alpha none → 自函数保形（数字不带 %，
      // Chrome computed 口径：'hsl(60 40 40 / none)'、'hsl(none none none)'）。
      if ((anyNone || bothANone) && (ast.space === 'hsl' || ast.space === 'hwb')) {
        return { kind: 'rgb', rgb: lin, alpha: alpha, aNone: bothANone, hasNone: true, srcBare: true,
          srcComps: [outNone[0] ? 'none' : out[0], outNone[1] ? 'none' : out[1], outNone[2] ? 'none' : out[2]],
          srcKind: ast.space };
      }
      // 其余（含 srgb anyNone）→ 'color(srgb ...)' 函数形（Chrome computed
      // 口径：mix 输出插值空间函数形，非 legacy rgb；8 位数字 0..1 值）。
      return { kind: 'colormix', space: 'srgb', comps: lin, none: outNone, alpha: alpha, rgb: lin.slice() };
    }
    if (ast.space === 'lab' || ast.space === 'lch' || ast.space === 'oklab' || ast.space === 'oklch') {
      return { kind: 'labmix', space: ast.space, coords: out, none: outNone, alpha: alpha, aNone: bothANone,
        rgb: _zwCCMixCoordsToRgb(ast.space, out) };
    }
    return { kind: 'colormix', space: ast.space, comps: out, none: outNone, alpha: alpha, aNone: bothANone,
      rgb: _zwCCMixCoordsToRgb(ast.space, out) };
  }

  // color() 语法 percent → 通道原生值（css-color-4 各空间满量：srgb 族/xyz
  // 100% = 1；lab L 100% = 100、a/b ±125；lch C 150、H 360；oklab a/b ±0.4）。
  function _zwCCColorPct(space, i, pv) {
    if (space === 'lab') return i === 0 ? pv : pv * 1.25;
    if (space === 'lch') return i === 0 ? pv : (i === 1 ? pv * 1.5 : pv * 3.6);
    if (space === 'oklab') return i === 0 ? pv / 100 : pv * 0.004;
    if (space === 'oklch') return i === 0 ? pv / 100 : (i === 1 ? pv * 0.004 : pv * 3.6);
    return pv / 100;
  }

  // ============ 相对色（css-color-5 § relative colors）============
  function _zwCCOriginInSpace(originText, el, space, depth) {
    var r = _zwCCResolveText(originText, el, depth);
    if (!r || (!r.rgb)) return null;
    var alphaNone = r.aNone === true || r.alpha === 'none';
    var alphaVal = alphaNone ? 0 : r.alpha;
    var isPol = space === 'lch' || space === 'oklch';
    // Chrome lch/oklch origin 口径（语料实证，与 hsl/hwb 相反）：
    // 'lch(from lch(none none none) l c h)' → 'lch(0 0 0)'——none 通道引用归 0
    // 不传播；'/ alpha' 引 none origin → '/ 0'——alphaNone 不传播。
    var origNone = function (flags) {
      return isPol ? [false, false, false] : (flags || [false, false, false]).slice();
    };
    var origAlphaNone = function () { return isPol ? false : alphaNone; };
    // 原生坐标直取（嵌套 relative/alpha 同空间——免 rgb 往返投影精度损失，
    // oklch hue 往返漂移数度实证）；none 通道旗标沿 origin 透传（css-color-5：
    // 关键字引用 none 通道 → 输出 none）。
    if (r.kind === 'relative-out' && r.space === space) {
      return { channels: r.chans.slice(), alpha: alphaVal, alphaNone: origAlphaNone(), none: origNone(r.none) };
    }
    if (r.kind === 'alpha-out') {
      if (r.srcKind === space && r.chans) return { channels: r.chans.slice(), alpha: alphaVal, alphaNone: origAlphaNone(), none: [false, false, false] };
      if (r.own && (r.own.fn === space || r.own.space === space)) {
        var oc = [], on = [false, false, false];
        for (var oi = 0; oi < 3; oi++) {
          if (r.own.comps[oi] === 'none') { on[oi] = true; oc.push(0); }
          else oc.push(Number(r.own.comps[oi]));
        }
        return { channels: oc, alpha: alphaVal, alphaNone: origAlphaNone(), none: origNone(on) };
      }
    }
    // color() 字面 origin 同空间直取（'color(from color(a98-rgb 0.7 none 0.3)
    // a98-rgb r g b)' 等——rgb 往返投影会引入 e-4 级噪声与负号，语料以
    // 精确 0 断言）。通道值按 color() 语法域解释（css-color-4 §color）；
    // none 旗标透传，由关键字消费面按目标形决定传播或消解。kind 'own'
    // 无 fn 字段，空间相等即判别（lab 字面无 space，不会误入）。
    if ((r.kind === 'own' || r.kind === 'color') && r.space === space && r.comps) {
      var dc = [], dn = [false, false, false];
      for (var di = 0; di < 3; di++) {
        var dp = r.comps[di];
        if (dp === 'none' || (dp && dp.none)) { dn[di] = true; dc.push(0); continue; }
        var dv = typeof dp === 'number' ? dp
          : dp && dp.num != null ? dp.num
          : dp && dp.pct != null ? _zwCCColorPct(space, di, dp.pct)
          : NaN;
        if (dv !== dv) { dc.length = 0; break; }
        dc.push(dv);
      }
      if (dc.length === 3) return { channels: dc, alpha: alphaVal, alphaNone: origAlphaNone(), none: origNone(dn) };
    }
    // 同空间原生坐标直取（'lch(from lch(...) …)' 等，免 rgb 往返投影漂移；
    // legacy rgb 与 srgb 同域 0..1）。
    if (r.srcComps && (r.srcKind === space || (space === 'srgb' && r.srcKind === 'rgb'))) {
      var nc = [], nn = [false, false, false];
      for (var si = 0; si < 3; si++) {
        if (r.srcComps[si] === 'none') { nn[si] = true; nc.push(0); }
        else nc.push(Number(r.srcComps[si]));
      }
      return { channels: nc, alpha: alphaVal, alphaNone: origAlphaNone(), none: origNone(nn) };
    }
    // 嵌套 relative-out oklab 族跨空间原生直转（css-color-5 nested relative；
    // 语料 'color(from oklab(from color(xyz-d50 0.99 0.88 0.77) l a b)
    // xyz-d50 x y z)' 精确 0.99 断言）：rgb 投影往返链经 OkM1i 发表松逆
    // （偏差 7.6e-4）引入 e-5 级输出噪声，此处以数值真逆直配——f64 往返
    // ~1e-15。lab/lch native cbrt 对自逆（同形 corpus PASS），不 入此臂；
    // hsl/hwb native 越界往返归 out-of-gamut 桶。
    if (r.kind === 'relative-out' && r.chans
        && (r.space === 'oklab' || r.space === 'oklch')
        && (space === 'xyz' || space === 'xyz-d50' || space === 'xyz-d65')) {
      var o3 = r.space === 'oklch'
        ? [r.chans[0], r.chans[1] * Math.cos(r.chans[2] * Math.PI / 180), r.chans[1] * Math.sin(r.chans[2] * Math.PI / 180)]
        : r.chans.slice();
      var x65 = _zwCCMatVec(_zwCCOkM1t, _zwCCMatVec(_zwCCOkM2t, o3).map(function (q) { return q * q * q; }));
      // 'xyz' 与 d65 同系（css-color-4 color() xyz 别名 d65 口径）。
      var xv = space === 'xyz-d50' ? _zwCCMatVec(_zwCCD65toD50, x65) : x65;
      return { channels: xv, alpha: alphaVal, alphaNone: origAlphaNone(), none: origNone(r.none) };
    }
    var rgb = r.rgb;
    var channels, convNone = [false, false, false];
    if (space === 'srgb') channels = rgb.slice();
    else if (space === 'hsl') { var v = _zwCCRgbToHsl(rgb[0], rgb[1], rgb[2]); channels = [v[0], v[1] * 100, v[2] * 100]; }
    else if (space === 'hwb') { var v2 = _zwCCRgbToHwb(rgb[0], rgb[1], rgb[2]); channels = [v2[0], v2[1] * 100, v2[2] * 100]; }
    else if (space === 'lab' || space === 'lch') {
      var lab = _zwCCXyzToLab(_zwCCMatVec(_zwCCD65toD50, _zwCCToXyz65('srgb', _zwCCToLin('srgb', rgb))));
      if (space === 'lch') {
        // lch hue = lab 极角 atan2(b, a)，非 hsl hue（角度域不同）。
        var hl = Math.atan2(lab[2], lab[1]) * 180 / Math.PI;
        if (hl < 0) hl += 360;
        var cl = Math.sqrt(lab[1] * lab[1] + lab[2] * lab[2]);
        // 转换型消色（C=0，如 display-p3 黑 → lch）→ hue none（语料
        // 'lch(from color(display-p3 0 0 0) l c h)' → 'lch(0 0 none)'）。
        convNone[2] = cl <= 1e-9;
        channels = [lab[0], cl, hl];
      } else channels = lab.slice();
    } else if (space === 'oklab' || space === 'oklch') {
      var ok = _zwCCXyzToOklab(_zwCCToXyz65('srgb', _zwCCToLin('srgb', rgb)));
      if (space === 'oklch') {
        var hh = Math.atan2(ok[2], ok[1]) * 180 / Math.PI;
        if (hh < 0) hh += 360;
        var ck = Math.sqrt(ok[1] * ok[1] + ok[2] * ok[2]);
        convNone[2] = ck <= 1e-9;
        channels = [ok[0], ck, hh];
      } else channels = ok.slice();
    } else {
      // 跨空间（color() 目标）：FromXyz65 出线性，按空间 gamma 编码
      // （xyz 无编码直通；语料 'color(from color(srgb ...) display-p3 r g b)'
      // want 0.313005 = 编码值，线性 0.0793 实证）。
      var xyz2 = _zwCCToXyz65('srgb', _zwCCToLin('srgb', rgb));
      channels = _zwCCFromLin(space, _zwCCFromXyz65(space, xyz2));
    }
    return { channels: channels, alpha: alphaVal, alphaNone: origAlphaNone(), none: isPol ? convNone : [false, false, false] };
  }

  function _zwCCEvalRel(expr, kws, channels, scale, alphaVal, el) {
    var subs = _zwCCSubSibling(String(expr), el);
    var i, re;
    if (kws) {
      for (i = 0; i < kws.length; i++) {
        re = new RegExp('(^|[^0-9a-zA-Z.])' + kws[i] + '($|[^0-9a-zA-Z(])', 'g');
        subs = subs.replace(re, function (_m, pre, post) { return pre + '(' + channels[i] + ')' + post; });
      }
    }
    if (alphaVal != null) {
      re = /(^|[^0-9a-zA-Z.])alpha($|[^0-9a-zA-Z(])/g;
      subs = subs.replace(re, function (_m, pre, post) { return pre + '(' + alphaVal + ')' + post; });
    }
    var r = _zwCCEval(subs);
    if (r !== r) return NaN;
    return r.pct ? r.v * scale / 100 : r.v;
  }

  function _zwCCResolveRelative(ast, el, depth) {
    var space = ast.space;
    var orig = _zwCCOriginInSpace(ast.origin, el, space, depth);
    if (!orig) return null;
    var kws = space === 'hwb' ? ['h', 'w', 'b'] : space === 'hsl' ? ['h', 's', 'l']
      : space === 'lch' || space === 'oklch' ? ['l', 'c', 'h']
      : space === 'lab' || space === 'oklab' ? ['l', 'a', 'b']
      : space === 'xyz' || space === 'xyz-d50' || space === 'xyz-d65' ? ['x', 'y', 'z'] : ['r', 'g', 'b'];
    // legacy rgb 目标判别：number 组件域 0..255 仅属 rgb()/rgba() 目标；
    // color(srgb ...) 目标 number 为 0..1 原生域（语料 'color(from X srgb
    // 2 3 4)' want 2 3 4 vs 'rgb(from X 25 g b)' want 0.098）。
    var legacyRgb = ast.fn === 'rgb' || ast.fn === 'rgba';
    // num/pct/calc 通道值 → 输出域换算（css-color-5 relative syntax：rgb 目标
    // number = 0..255（'rgb(from red 25 g b)' → 25/255）；hsl/hwb s/l number =
    // 0..1；lab 族 number = 原生域；percent 按各空间满量）。
    function toOut(sp, i) {
      if (sp.none) return 'none';
      if (sp.num != null || sp.angle != null) {
        var nv = sp.angle != null ? sp.angle : sp.num;
        if (legacyRgb && space === 'srgb') return nv / 255;
        if (space === 'hsl' || space === 'hwb') return i === 0 ? nv : nv * 100;
        return nv;
      }
      if (sp.pct != null) {
        var pv = sp.pct;
        if (space === 'srgb') return pv / 100;
        if (space === 'hsl' || space === 'hwb') return i === 0 ? pv * 3.6 : pv;
        if (space === 'lab') return i === 0 ? pv : pv * 1.25;
        if (space === 'lch') return i === 0 ? pv : (i === 1 ? pv * 1.5 : pv * 3.6);
        if (space === 'oklab') return i === 0 ? pv / 100 : pv * 0.004;
        if (space === 'oklch') return i === 0 ? pv / 100 : (i === 1 ? pv * 0.004 : pv * 3.6);
        return pv / 100;
      }
      if (sp.calc != null) {
        var r = _zwCCEval(_zwCCSubSibling(sp.calc, el));
        if (r !== r) return NaN;
        if (r.pct) return toOut({ pct: r.v }, i);
        // calc 数值结果 = 原生域直用（语料 'calc(sin(pi / 4) * 100)' → l=70.71；
        // bare number 才按 0..1 域 ×100——hsl/hwb s/l/w/b 槽）。
        if ((space === 'hsl' || space === 'hwb') && i !== 0) return r.v;
        return toOut({ num: r.v }, i);
      }
      return NaN;
    }
    var out = [], outNone = [false, false, false];
    for (var i = 0; i < 3; i++) {
      var sp = ast.comps[i];
      if (!sp) return null;
      if (sp.none) { outNone[i] = true; out.push(0); continue; }
      if (sp.chan != null) {
        // 'alpha' 通道关键字（css-color-5：任意通道槽可引用 origin alpha，
        // 值按槽原生域解释——legacy rgb 组件槽 number 域 0..255，语料
        // 'rgb(from rebeccapurple b alpha r / g)' → alpha=1/255；origin
        // alpha 为 none 时按 missing 消解为零，不传播 none）。
        if (sp.chan === 'alpha') {
          var av = orig.alphaNone ? 0 : orig.alpha;
          out.push(legacyRgb && space === 'srgb' ? av / 255 : av);
          continue;
        }
        var idx = kws.indexOf(sp.chan);
        // color() 目标 x/y/z 通道关键字对任意空间作序数别名（css-color-5
        // 仅定义 xyz 空间，Chrome 实证按首三通道接受——语料嵌套往返面）。
        if (idx < 0 && ast.fn === 'color') {
          var xmap = { x: 0, y: 1, z: 2 };
          if (xmap[sp.chan] != null) idx = xmap[sp.chan];
        }
        if (idx < 0) return null;
        // origin 同通道为 none → 输出 none（css-color-5 none 通道传播）；
        // color() 目标例外：none 一律消解为 0，序列化不携带 none
        // （css-color-4 无 none 通道概念，Chrome 实证——语料 color() 目标面）。
        if (orig.none && orig.none[idx]) {
          if (ast.fn === 'color') { out.push(0); continue; }
          outNone[i] = true; out.push(0); continue;
        }
        out.push(orig.channels[idx]);
        continue;
      }
      if (sp.chanexpr != null) {
        // channel 关键字按 number 语义替换（css-color-5：'calc(r + 1)' 的 r =
        // 原生通道数——origin 通道输出域 → num 域替换，结果再回输出域）。
        var chanVals = orig.channels.map(function (c, ci) {
          // 通道关键字 num 域 = 原生通道域（语料 'calc(l + 1)' → 41%、
          // 'calc((r / 255) * 100%)' → r=102；hsl/hwb s/l 为 0..100 数；
          // color(srgb) 目标 0..1 域——'calc(r + 0.01)' → 0.71）。
          if (legacyRgb && space === 'srgb') return c * 255;
          return c;
        });
        // % 结果按通道关键字 num 域满量程换算（语料实证：'calc((r/255)*100%)'
        // → 40%·255/100 = 102 → /255 = 0.4；lab a 槽满 125；oklab a 满 0.4）。
        var relScale = legacyRgb && space === 'srgb' ? [255, 255, 255]
          : (space === 'hsl' || space === 'hwb') ? [360, 100, 100]
          : space === 'lab' ? [100, 125, 125]
          : space === 'lch' ? [100, 150, 360]
          : space === 'oklab' ? [1, 0.4, 0.4]
          : space === 'oklch' ? [1, 0.4, 360] : [1, 1, 1];
        var ev = _zwCCEvalRel(sp.chanexpr, kws, chanVals, relScale[i], el);
        if (ev !== ev) return null;
        if (legacyRgb && space === 'srgb') ev = ev / 255;
        out.push(ev);
        continue;
      }
      var v = toOut(sp, i);
      if (v === 'none' || v == null || v !== v) return null;
      out.push(v);
    }
    var alphaOut;
    if (ast.alpha.none) alphaOut = 'none';
    else if (ast.alpha.omitted) alphaOut = orig.alphaNone ? 'none' : orig.alpha; // 缺省槽 = origin alpha
    else if (ast.alpha.chan != null) {
      // 'alpha' 关键字取 origin alpha；origin alpha none 的消解随目标形：
      // color() 目标换算消解为零（语料 'color(from color(srgb ... / none)
      // srgb r g b / alpha)' want '/ 0'）；同语法目标（legacy rgb/hsl/hwb/
      // lab 族）none 传播（语料 'rgb(from rgb(20% 40% 60% / none) r g b /
      // alpha)' want '/ none'）。
      if (ast.alpha.chan === 'alpha') alphaOut = orig.alphaNone ? (ast.fn === 'color' ? 0 : 'none') : orig.alpha;
      else {
        // 通道关键字 alpha：取 origin 在目标 fn 数值域的通道值 clamp [0,1]
        // （语料：color() '/ r'=0.7；lab '/ a'=20→1 无后缀；oklab '/ a'=0.2；
        // rgb legacy '/ r' 通道域 0..255 → 51→1，非「回落 origin alpha」）。
        var aIdx = kws.indexOf(ast.alpha.chan);
        if (aIdx < 0) return null;
        var aChan = orig.channels[aIdx];
        if ((ast.fn === 'rgb' || ast.fn === 'rgba') && space === 'srgb') aChan = aChan * 255;
        alphaOut = (orig.none && orig.none[aIdx]) ? 'none' : Math.min(1, Math.max(0, aChan));
      }
    } else if (ast.alpha.chanexpr != null) {
      var evA = _zwCCEvalRel(ast.alpha.chanexpr, null, null, 1, orig.alphaNone ? 0 : orig.alpha, el);
      if (evA !== evA) return null;
      alphaOut = evA;
    } else {
      var va = _zwCCSpecNum(ast.alpha, 1);
      if (va == null || va !== va) return null;
      alphaOut = va;
    }
    if (alphaOut !== 'none') alphaOut = Math.min(1, Math.max(0, alphaOut));
    var clamped = [];
    for (var j = 0; j < 3; j++) {
      var v2 = out[j];
      // srgb 域（color()/rgb()/rgba() 目标）不钳——相对色 computed 值保
      // out-of-gamut 精度（语料 -0.153577 锚；嵌套无损往返经 clamp 后回不到
      // 0.99 实证）；lab L ∈ [0,100]、oklab L ∈ [0,1]（语料 '200'→100、
      // '2'→1）；a/b/C 无界不钳。
      if (space === 'lab') v2 = j === 0 ? Math.min(100, Math.max(0, v2)) : v2;
      else if (space === 'oklab') v2 = j === 0 ? Math.min(1, Math.max(0, v2)) : v2;
      // 极坐标：lch L ∈ [0,100]、oklch L ∈ [0,1]（语料 '200'→100、'2'→1、
      // 'calc(... * infinity)'→100）；C ≥ 0 不设上限；hue 归一 0..360。
      else if (space === 'lch') v2 = j === 0 ? Math.min(100, Math.max(0, v2)) : j === 1 ? Math.max(0, v2) : ((v2 % 360) + 360) % 360;
      else if (space === 'oklch') v2 = j === 0 ? Math.min(1, Math.max(0, v2)) : j === 1 ? Math.max(0, v2) : ((v2 % 360) + 360) % 360;
      else if (space === 'hsl' || space === 'hwb') v2 = j === 0 ? ((v2 % 360) + 360) % 360 : Math.max(0, v2);
      clamped.push(v2);
    }
    // rgb 投影（alpha() origin 消费面）+ chans（目标空间原生坐标——嵌套
    // relative `lab(from lab(from ...) ...)` 免 rgb 往返投影精度损失）。
    var rgbP;
    if (space === 'srgb') rgbP = clamped.slice();
    else if (space === 'hsl') rgbP = _zwCCHslToRgb(clamped[0], clamped[1] / 100, clamped[2] / 100);
    else if (space === 'hwb') rgbP = _zwCCHwbToRgb(clamped[0], clamped[1] / 100, clamped[2] / 100);
    else if (space === 'xyz' || space === 'xyz-d65') rgbP = _zwCCFromLin('srgb', _zwCCFromXyz65('srgb', clamped.slice()));
    else if (space === 'xyz-d50') rgbP = _zwCCFromLin('srgb', _zwCCFromXyz65('srgb', _zwCCMatVec(_zwCCD50toD65, clamped.slice())));
    else {
      // lab 族与 rgb 族（a98/display-p3/prophoto/rec2020/线性族）统一走空间
      // 投影 helper——原实现把 rgb 族坐标误作 lab 处理，投影近黑（嵌套相对
      // 色 origin 全链失真实证）。
      rgbP = _zwCCMixCoordsToRgb(space, clamped);
    }
    return { kind: 'relative-out', space: space, fn: ast.fn, comps: clamped, none: outNone, alpha: alphaOut, rgb: rgbP, chans: clamped.slice() };
  }

  // ============ 序列化 ============

  function _zwCCAlphaStr(alpha, aNone) {
    if (aNone) return 'none';
    if (alpha == null || alpha !== alpha) return 'none';
    if (alpha >= 1) return null;
    if (alpha <= 0) return '0';
    return _zwCCNum(alpha);
  }

  // legacy rgb/rgba（可解析 sRGB；整型化）。+1e-7 浮点噪声门（1/255 半值
  // 边界 127.49999… 沉 127——hwb 0.2=1-0.3-0.5 浮点实证，真实台阶 ≥0.002 安全）。
  function _zwCCSerialLegacyRgb(rgb, alpha, aNone) {
    var r = Math.round(Math.min(1, Math.max(0, rgb[0])) * 255 + 1e-7);
    var g = Math.round(Math.min(1, Math.max(0, rgb[1])) * 255 + 1e-7);
    var b = Math.round(Math.min(1, Math.max(0, rgb[2])) * 255 + 1e-7);
    var a = _zwCCAlphaStr(alpha, aNone);
    if (a == null) return 'rgb(' + r + ', ' + g + ', ' + b + ')';
    return 'rgba(' + r + ', ' + g + ', ' + b + ', ' + a + ')';
  }

  // rgb 现代语法含 none → color(srgb ...)（srcComps 为 0..1 或 'none'）。
  function _zwCCSerialSrgbNone(srcComps, alpha, aNone) {
    var parts = [];
    for (var i = 0; i < 3; i++) parts.push(srcComps[i] === 'none' ? 'none' : _zwCCNum(srcComps[i]));
    var a = _zwCCAlphaStr(alpha, aNone);
    return 'color(srgb ' + parts.join(' ') + (a ? ' / ' + a : '') + ')';
  }

  // hsl/hwb none 保形（powerless；h 度、s/l/w/b 值）。% vs 裸按输入语法 echo：
  // relative 输出恒裸（'hsl(270 50 none)' 语料锚）；direct 声明随原文——% 输入
  // 经 raw 影子取原文 parse（srcBare=false → '%'），裸输入（RT 回设形）无影子
  // 走 store 裸形（srcBare=true）。bare 参数布尔（全裸，relative 臂）或
  // [sBare, lBare]/[wBare, bBare] 数组（direct parse 臂）。declared 面仍在写侧
  // 剥 %（color-valid-* corpus 锚 'hsl(120 80% none)' → 'hsl(120 80 none)'）。
  function _zwCCSerialHslNone(srcComps, alpha, aNone, bare) {
    var bl = Array.isArray(bare) ? bare : [!!bare, !!bare];
    var h = srcComps[0] === 'none' ? 'none' : _zwCCNum(((srcComps[0] % 360) + 360) % 360);
    var s = srcComps[1] === 'none' ? 'none' : _zwCCNum(srcComps[1]) + (bl[0] ? '' : '%');
    var l = srcComps[2] === 'none' ? 'none' : _zwCCNum(srcComps[2]) + (bl[1] ? '' : '%');
    var a = _zwCCAlphaStr(alpha, aNone);
    return 'hsl(' + h + ' ' + s + ' ' + l + (a ? ' / ' + a : '') + ')';
  }
  function _zwCCSerialHwbNone(srcComps, alpha, aNone, bare) {
    var bl = Array.isArray(bare) ? bare : [!!bare, !!bare];
    var h = srcComps[0] === 'none' ? 'none' : _zwCCNum(((srcComps[0] % 360) + 360) % 360);
    var w = srcComps[1] === 'none' ? 'none' : _zwCCNum(srcComps[1]) + (bl[0] ? '' : '%');
    var b = srcComps[2] === 'none' ? 'none' : _zwCCNum(srcComps[2]) + (bl[1] ? '' : '%');
    var a = _zwCCAlphaStr(alpha, aNone);
    return 'hwb(' + h + ' ' + w + ' ' + b + (a ? ' / ' + a : '') + ')';
  }

  function _zwCCSerialLab(fn, srcComps, alpha, aNone) {
    var parts = [];
    for (var i = 0; i < 3; i++) {
      if (srcComps[i] === 'none') { parts.push('none'); continue; }
      // L/C/hue 8 位（round-trip 同精度）：hue 走 echo——store 声明面对 decimal
      // hue 已 echo（part11-color.js lab 族 hue echo），8 位回显幂等保 RT
      //（corpus '33.690066'/'73.3386' 双锚：rad 输入声明面折 6 位 decimal 后
      // 回显同形；computed 面只见 store 串，无需重复单位换算）。
      parts.push(_zwCCNum(srcComps[i]));
    }
    var a = _zwCCAlphaStr(alpha, aNone);
    return fn + '(' + parts.join(' ') + (a ? ' / ' + a : '') + ')';
  }

  function _zwCCSerialColorSpace(space, comps, none, alpha, aNone) {
    var parts = [];
    for (var i = 0; i < 3; i++) parts.push(none && none[i] ? 'none' : _zwCCNum(comps[i]));
    var a = _zwCCAlphaStr(alpha, aNone);
    return 'color(' + space + ' ' + parts.join(' ') + (a ? ' / ' + a : '') + ')';
  }

  // 相对色输出：hsl/hwb 带 none（通道或 alpha）→ 自形裸数字（语料
  // 'hsl(270 50 none)'/'hwb(270 20 40 / none)'；全数值形仍 color(srgb)，
  // 与 mix 序列化规则同构）；srgb 族目标 → color(srgb)；lab 族 → 自形。
  function _zwCCSerialRelative(out) {
    var space = out.space;
    var aNone = out.alpha === 'none';
    if (space === 'hsl' || space === 'hwb') {
      var anyNone = out.none[0] || out.none[1] || out.none[2];
      if (anyNone || aNone) {
        var srcComps = [];
        for (var k = 0; k < 3; k++) srcComps.push(out.none[k] ? 'none' : out.comps[k]);
        return space === 'hsl'
          ? _zwCCSerialHslNone(srcComps, aNone ? null : out.alpha, aNone, true)
          : _zwCCSerialHwbNone(srcComps, aNone ? null : out.alpha, aNone, true);
      }
    }
    if (space === 'srgb' || space === 'hsl' || space === 'hwb') {
      var rgbv;
      if (space === 'hsl') rgbv = _zwCCHslToRgb(out.comps[0], out.comps[1] / 100, out.comps[2] / 100);
      else if (space === 'hwb') rgbv = _zwCCHwbToRgb(out.comps[0], out.comps[1] / 100, out.comps[2] / 100);
      else rgbv = out.comps.slice();
      var a = _zwCCAlphaStr(aNone ? null : out.alpha, aNone);
      // color() 与 legacy rgb 相对输出不钳通道（css-color-5 相对色 computed
      // 值以 color(srgb) 保形、保 out-of-gamut 精度——语料嵌套无损往返
      // 'color(from rgb(from color(xyz-d50 0.99 ...) r g b) xyz-d50 x y z)'
      // 经 clamp 后回不到 0.99 实证；hsl/hwb 目标仍钳）。
      var cl = out.fn === 'color' || out.fn === 'rgb' || out.fn === 'rgba';
      var comps = [];
      for (var i = 0; i < 3; i++) comps.push(out.none[i] ? 'none' : _zwCCNum(cl ? rgbv[i] : Math.min(1, Math.max(0, rgbv[i]))));
      return 'color(srgb ' + comps.join(' ') + (a ? ' / ' + a : '') + ')';
    }
    if (space === 'lab' || space === 'lch' || space === 'oklab' || space === 'oklch') {
      var parts = [];
      for (var j = 0; j < 3; j++) parts.push(out.none[j] ? 'none' : _zwCCNum(out.comps[j]));
      var a2 = _zwCCAlphaStr(out.alpha === 'none' ? null : out.alpha, out.alpha === 'none');
      return space + '(' + parts.join(' ') + (a2 ? ' / ' + a2 : '') + ')';
    }
    return _zwCCSerialColorSpace(space, out.comps, out.none, out.alpha === 'none' ? null : out.alpha, out.alpha === 'none');
  }

  // ============ 入口 ============

  // resolve 结果 → computed 序列化串（__zwComputedColor 与 declared origin
  // 规范化共用分发——lab clamp/hue mod/alpha 数字化语义单一来源）。
  function _zwCCSerialResolve(r) {
    if (r.kind === 'system') return r.text;
    if (r.own && r.kind === 'rgb') {
      // lab 族函数自形保形（round-trip；Chrome 对 lab()/oklch() computed 恒
      // 返回原函数形，不投影 legacy rgb）。
      return _zwCCSerialLab(r.own.fn, r.own.comps, r.alpha, r.aNone);
    }
    if (r.kind === 'own') {
      if (r.fn) return _zwCCSerialLab(r.fn, r.comps, r.alpha, r.aNone);
      return _zwCCSerialColorSpace(r.space, r.comps, r.comps.map(function (x) { return x === 'none'; }), r.alpha, r.aNone);
    }
    if (r.kind === 'rgb') {
      // 通道 none 或 alpha none 同走保形（corpus 'hsl(120 100% 50% / none)'
      // alpha-only none 实证——hasNone 只含通道旗标）。
      var anyN = r.hasNone || r.aNone;
      if (anyN && r.srcKind === 'rgb') return _zwCCSerialSrgbNone(r.srcComps, r.alpha, r.aNone);
      if (anyN && r.srcKind === 'hsl') return _zwCCSerialHslNone(r.srcComps, r.alpha, r.aNone, r.srcBare);
      if (anyN && r.srcKind === 'hwb') return _zwCCSerialHwbNone(r.srcComps, r.alpha, r.aNone, r.srcBare);
      if (anyN && (r.srcKind === 'lab' || r.srcKind === 'lch' || r.srcKind === 'oklab' || r.srcKind === 'oklch')) {
        return _zwCCSerialLab(r.srcKind, r.srcComps, r.alpha, r.aNone);
      }
      return _zwCCSerialLegacyRgb(r.rgb, r.alpha, r.aNone);
    }
    if (r.kind === 'labmix') {
      return _zwCCSerialLab(r.space,
        [r.none[0] ? 'none' : r.coords[0], r.none[1] ? 'none' : r.coords[1], r.none[2] ? 'none' : r.coords[2]],
        r.alpha, r.aNone);
    }
    if (r.kind === 'colormix') return _zwCCSerialColorSpace(r.space, r.comps, r.none, r.alpha, r.aNone);
    if (r.kind === 'alpha-out') {
      if (r.own) {
        if (r.own.fn) return _zwCCSerialLab(r.own.fn, r.own.comps, r.alpha === 'none' ? null : r.alpha, r.alpha === 'none');
        return _zwCCSerialColorSpace(r.own.space, r.own.comps, r.own.none, r.alpha === 'none' ? null : r.alpha, r.alpha === 'none');
      }
      return _zwCCSerialRelative({ space: 'srgb', comps: r.rel.comps, none: r.rel.none, alpha: r.alpha });
    }
    if (r.kind === 'relative-out') return _zwCCSerialRelative(r);
    return '';
  }

  // 相对色 origin 骨架出口（part10 Typed-OM computedStyleMap 消费）：返
  // 'currentcolor'（骨架态）或 ''（绝对色态/不可判定 → 宿主 resolved 通道）。
  globalThis.__zwCssRelativeOrigin = function (originText, el) {
    try { return _zwCCOriginSkeleton(originText, el); } catch (_eOR) { return ''; }
  };

  // sibling-index()/sibling-count() 文本替换（css-values-5 级联依赖函数；
  // 钉面 color-computed-relative-color 'calc(h + 180 * sibling-index())'
  // 期望 hue+180、alpha-color-parsing-valid 'alpha(from green /
  // sibling-count())' valid、relative-color-with-system-color test3）。
  // el 缺席（CSS.supports/decl oracle 支持性通道）取哨兵 1——语法可解析即
  // 支持；el 存在按父级子列表 1-based 索引/总数（lightseagreen lch hue
  // 190.11+180*1=10.11 钉面确证 1-based）；找不到（detached）取 0。
  function _zwCCSubSibling(expr, el) {
    var s = String(expr == null ? '' : expr);
    if (s.indexOf('sibling-') < 0) return s;
    var p = el && el.nodeType === 1 ? el.parentNode : null;
    var idx = 0, cnt = 0;
    if (p && p.children) {
      cnt = p.children.length;
      for (var k = 0; k < cnt; k++) if (p.children[k] === el) { idx = k + 1; break; }
    }
    if (el == null) { idx = 1; cnt = 1; }
    return s.replace(/sibling-index\s*\(\s*\)/gi, String(idx))
            .replace(/sibling-count\s*\(\s*\)/gi, String(cnt));
  }

  // RCS 形判别（css-color-5 §relative-colors）：色函数 'from' 语法，10 函数面。
  var _zwCCRcsFormRe = /(^|[^\w-])(rgb|rgba|hsl|hsla|hwb|lab|lch|oklab|oklch|color)\(\s*from\s/i;

  // computed 颜色：raw 声明文本 → computed 序列化串；无效/不可解析回退基座。
  // propCtx（可选）：触发求值的属性名——'color' 时 currentcolor 走父链，
  // 其余先试本元素 color（css-color-4 §5.3 currentcolor 域语义）。
  globalThis.__zwComputedColor = function (raw, el, propCtx) {
    // 环境依赖形回退基座（part11-color.js declared 折叠面）：sign()/abs() 需
    // 字体/容器/视口单位运行时环境（corpus sign(1em - 10px)/sign(2cqw - 10px)
    // 实证——基座对可静态求值形有正确解）；var() 需自定义属性级联；反斜杠 =
    // CSS 转义（host 词法器处理，如 'r\\67 b(...)'）。基座无效形返 null →
    // 调用方宿主链（覆盖前同路径，零漂移）。
    var prevCtx = _zwCCPropCtx;
    _zwCCPropCtx = String(propCtx == null ? '' : propCtx);
    try {
      var rawS = String(raw == null ? '' : raw);
      if (rawS.indexOf('var(') >= 0) {
        // RCS+var 面（css-color-5 §relative-colors）：宿主替换经 Rust
        // parse_relative_color 恒 legacy 形（calc 通道拒→invalid→黑；WPT 现代
        // 形期望下宿主不可能通过，接管域无 pass→fail 面）。本管线 var 臂
        //（_zwCCVarLookup inline+作者快照级联近似）+ chanexpr calc 通道 +
        // 现代形序列化完备——RCS 形（'from' 语法，或纯 var 引用且替换值含
        // 'from'）先试自解；自解失败/非 RCS 形保基座→宿主零漂移。
        var vrHit = _zwCCRcsFormRe.test(rawS);
        if (!vrHit) {
          var nmV = /^\s*var\(\s*(--[a-zA-Z0-9_-]+)/.exec(rawS);
          var lvV = nmV && el ? _zwCCVarLookup(el, nmV[1]) : '';
          vrHit = !!lvV && _zwCCRcsFormRe.test(lvV);
        }
        if (vrHit) {
          var rVar = null;
          try { rVar = _zwCCResolveText(rawS, el, 0); } catch (_eVR) { rVar = null; }
          if (rVar) return _zwCCSerialResolve(rVar);
        }
        return _zwCCPrevComputed ? _zwCCPrevComputed(raw, el) : '';
      }
      if (/(^|[^\w-])(sign|abs)\(/i.test(rawS) || rawS.indexOf('\\') >= 0) {
        return _zwCCPrevComputed ? _zwCCPrevComputed(raw, el) : '';
      }
      var r = null;
      try { r = _zwCCResolveText(raw, el, 0); } catch (_eCR) { r = null; }
      if (!r) return _zwCCPrevComputed ? _zwCCPrevComputed(raw, el) : '';
      return _zwCCSerialResolve(r);
    } finally {
      _zwCCPropCtx = prevCtx;
    }
  };

  // CSS.supports 钩子：颜色值可解析性（系统色同为合法值）。
  globalThis.__zwCssColorValid = function (raw) {
    try {
      var r = _zwCCResolveText(raw, null, 0);
      return !!r;
    } catch (_eCV) { return false; }
  };

  // opacity computed：number/%/calc()/min()/max()/clamp() → clamp [0,1]。
  globalThis.__zwOpacityComputed = function (raw) {
    var t = _zwCCStripComments(raw).trim();
    if (!t) return '';
    var r = _zwCCEval(t);
    if (r !== r) return /^calc\(/i.test(t) ? '0' : '';
    var v = r.pct ? r.v / 100 : r.v;
    v = Math.min(1, Math.max(0, v));
    return _zwCCNum(v);
  };

  // 系统色表暴露（后续批次 valid/invalid 声明面复用）。
  globalThis.__zwSystemColors = _zwCCSystem;

  // gCS 通道 plain 覆盖判定：inline 原文命中「宿主 cascade 产出错误计算值」
  // 的形 → 宿主读数不可信，落 plain 面重读。三类实证（named-color/hsl 页）：
  // ① 注释形（'/**/transparent'——宿主不剥注释，解析失败回退样式表红）；
  // ② currentcolor（宿主读值缺作者样式表臂，恒黑）；
  // ③ 现代形（宿主 partial-parse：'hsl(none none none)' 当 0 → 黑；完全
  //   不认的形宿主返空，gCS 臂自然落 plain，无需覆盖）。
  // legacy 形（named/hex/transparent/comma rgb·hsla/系统色）宿主可正确产出
  // → false（零漂移：editing 语料 legacy 值通路不变）。
  globalThis.__zwCssColorNeedsPlain = function (raw) {
    var t = String(raw == null ? '' : raw).trim();
    if (!t) return false;
    if (t.indexOf('/*') >= 0) return true;
    var low = t.toLowerCase();
    if (low === 'currentcolor') return true;
    // css-wide 关键字宿主 cascade 正确产出（css-cascade-3 §7）——不抢答：
    // 抢答落 plain 爬升臂，爬升链不建模关键字（'initial' 读空继续爬升误取
    // 父值——inheritance.html 'Property color inherits' 回归实证）。
    if (low === 'initial' || low === 'inherit' || low === 'unset'
      || low === 'revert' || low === 'revert-layer') return false;
    if (low === 'transparent' || low === 'none' || low.charAt(0) === '#') return false;
    if (_zwCCNamed[low]) return false;
    if (Object.prototype.hasOwnProperty.call(_zwCCSystem, low)) return false;
    if (/^(rgba?|hsla?)\([^)]*,[^)]*\)$/i.test(t)) return false; // comma legacy
    return true;
  };

  // ============ declared 规范化覆盖（DC-2 再批：css-color corpus 声明面）============
  // 基座 part11-color.js 对 mix/relative/contrast 形保原串透传（{form:'raw'}）。
  // 本段覆盖 __zwColorNormalizeDecl：
  //   - relative 色函数（css-color-5 §relative colors 声明序列化）：rgba/hsla
  //     别名折返 rgb/hsl、origin 规范化（keyword 保形小写、var/嵌套 mix·relative
  //     保原串、其余走 computed 管线序列化——lab clamp/hue mod 360/alpha 数字化
  //     与 computed 面同一出口）、通道表+alpha-part 原文保形（深度感知空白归一）；
  //     整值有效性以 computed resolve 为 oracle（通道关键字/类型表已被 computed
  //     relative 语料 1,100 断言钉）→ 失败即拒。
  //   - color-mix（css-color-5 §color-mix 声明序列化）：权重后置、未指定侧 =
  //     100% − 对侧、双 50% 双省略、内嵌色同 origin 规范化；域外权重/method
  //     子句/两侧语法无效即拒（corpus invalid-mix 141 面）；非两侧形保透传。
  //   - contrast-color（css-color-6）：恰一 <color> 参，否则拒；合法形保原串。
  //   - color-layers（css-color-6 语法面）/light-dark：保透传（后续批次建模）。
  // var() 引用/含注释/含转义值整值不拦截（cssText 注释保形 T49fix 锚；var 需
  // 运行时替换面；转义 host 词法器处理）。
  // https://drafts.csswg.org/css-color-5/#relative-colors
  // https://drafts.csswg.org/css-color-6/#color-layers

  // 深度感知 rest 归一：空白串压单空格、顶层 '/' 前后单空格；calc() 内空白与
  // 除号不动（通道表原文保形，仅排版归一——corpus 期望单空格规范排布）。
  function _zwCCDeclRestNorm(s) {
    var t = String(s == null ? '' : s).trim();
    var out = '', depth = 0, i = 0;
    while (i < t.length) {
      var c = t.charAt(i);
      if (c === '(') depth++;
      else if (c === ')') depth = depth > 0 ? depth - 1 : 0;
      if (depth === 0 && c === '/') {
        out = out.replace(/\s+$/, '') + ' / ';
        i++;
        while (i < t.length && /\s/.test(t.charAt(i))) i++;
        continue;
      }
      if (depth === 0 && /\s/.test(c)) {
        if (out !== '' && !/\s$/.test(out)) out += ' ';
        i++;
        continue;
      }
      out += c;
      i++;
    }
    return out;
  }

  // rest 通道槽 calc() 声明面规范形（css-color-5 相对色通道 calc：Chrome 序列
  // 化为乘式规范——数×通道、常数前置、除常数折叠为 1/n 乘（6 位有效，corpus
  // 'calc(a / 3)'→'calc(0.333333 * a)'）、减常数折负常数前置（'calc(l - 20)'→
  // 'calc(-20 + l)'）、加减中的乘式项括号化（'calc(g * .5 + g * .5)'→
  // 'calc((0.5 * g) + (0.5 * g))'）。仅支持纯数×关键字形状；其余（单位、多
  // 关键字积、嵌套函数）保原形不求值。
  function _zwCCDeclCalcCanon(s) {
    var toks = [];
    var i = 0;
    while (i < s.length) {
      var c = s.charAt(i);
      if (/\s/.test(c)) { i++; continue; }
      var m = /^(\d+\.?\d*|\.\d+)/.exec(s.slice(i));
      if (m) { toks.push({ t: 'n', v: parseFloat(m[1]) }); i += m[1].length; continue; }
      var m2 = /^[a-zA-Z][a-zA-Z0-9-]*/.exec(s.slice(i));
      if (m2) { toks.push({ t: 'k', v: m2[0] }); i += m2[0].length; continue; }
      if ('+-*/()'.indexOf(c) >= 0) { toks.push({ t: c }); i++; continue; }
      return null; // 单位/未知字符 → 不规范
    }
    var p = 0;
    function peek() { return p < toks.length ? toks[p] : null; }
    function parseFactor() {
      var tk = peek();
      if (!tk) return null;
      if (tk.t === 'n') { p++; return { num: tk.v, kws: [] }; }
      if (tk.t === 'k') { p++; return { num: 1, kws: [tk.v] }; }
      if (tk.t === '(') {
        p++;
        var inner = parseExpr();
        if (!inner || !peek() || peek().t !== ')') return null;
        p++;
        return { num: 1, kws: ['(' + inner + ')'] };
      }
      return null;
    }
    function parseTerm() {
      var acc = parseFactor();
      if (!acc) return null;
      while (peek() && (peek().t === '*' || peek().t === '/')) {
        var op = toks[p++].t;
        var f = parseFactor();
        if (!f) return null;
        if (op === '*') {
          if (f.kws.length === 0) acc.num *= f.num;
          else acc = { num: acc.num, kws: acc.kws.concat(f.kws) };
        } else {
          if (f.kws.length !== 0) return null; // 除关键字 → 不规范
          acc.num /= f.num;
        }
      }
      return acc;
    }
    function parseExpr() {
      var t0 = parseTerm();
      if (!t0) return null;
      var sum = t0.kws.length === 0 ? t0.num : 0;
      var hasNum = t0.kws.length === 0;
      var kws = [];
      if (t0.kws.length > 0) kws.push({ text: termText(t0), compound: t0.kws.length > 1 || t0.num !== 1 });
      while (peek() && (peek().t === '+' || peek().t === '-')) {
        var op = toks[p++].t;
        var tr = parseTerm();
        if (!tr) return null;
        if (tr.kws.length === 0) {
          sum += op === '+' ? tr.num : -tr.num; // 常数项折叠（'l - 20' → -20）
          hasNum = true;
          continue;
        }
        var txt = termText(tr);
        var compound = tr.kws.length > 1 || tr.num !== 1;
        if (op === '+') kws.push({ text: txt, compound: compound });
        else kws.push({ text: txt, compound: compound, neg: true });
      }
      if (peek()) return null; // 尾部残token（未闭括号等）
      var hasNumPart = hasNum && (sum !== 0 || kws.length === 0);
      var multi = kws.length + (hasNumPart ? 1 : 0) > 1; // 多项时乘式项括号化
      var outParts = [];
      if (hasNumPart) outParts.push(_zwCCSig6(sum));
      for (var q = 0; q < kws.length; q++) {
        var seg = kws[q];
        var body = multi && seg.compound ? '(' + seg.text + ')' : seg.text;
        if (outParts.length === 0) outParts.push(seg.neg ? '-' + body : body);
        else outParts.push(seg.neg ? ' - ' + body : ' + ' + body);
      }
      if (outParts.length === 0) return null; // 全常数折叠为 0 且无通道项
      return outParts.join('');
    }
    function termText(tr) {
      var head = tr.num !== 1 ? _zwCCSig6(tr.num) + ' * ' : '';
      return head + tr.kws.join(' * ');
    }
    var res = parseExpr();
    return res;
  }

  // rest 通道表/alpha-part 声明面规范化：calc() 槽规范形（上函数）、color()
  // 通道关键字 'xyz' 别名展开 'xyz-d65'（css-color-5 通道名用规范空间名——
  // corpus 'color(from color(xyz ...) xyz x y z)' → 'xyz-d65 x y z' 锚）、
  // alpha-part 前导点数字补 0（'.25'→'0.25'、'.20'→'0.20' 保原位数字——
  // Chrome 声明面仅补前导零不重排尾零）；其余槽保形。
  function _zwCCDeclRestCanon(fn, rest, originKept) {
    var norm = _zwCCDeclRestNorm(rest);
    // 顶层 '/' 拆通道区与 alpha 区。
    var depth = 0, slashAt = -1;
    for (var i = 0; i < norm.length; i++) {
      var ch = norm.charAt(i);
      if (ch === '(') depth++;
      else if (ch === ')') depth--;
      else if (ch === '/' && depth === 0) { slashAt = i; break; }
    }
    var chan = slashAt >= 0 ? norm.slice(0, slashAt) : norm;
    var alpha = slashAt >= 0 ? norm.slice(slashAt + 1).trim() : null;
    // 通道区深度感知切槽（calc( 内空白不切）。
    var slots = [], cur = '', d2 = 0;
    for (var j = 0; j < chan.length; j++) {
      var c2 = chan.charAt(j);
      if (c2 === '(') d2++;
      else if (c2 === ')') d2--;
      if (d2 === 0 && /\s/.test(c2)) {
        if (cur) { slots.push(cur); cur = ''; }
        continue;
      }
      cur += c2;
    }
    if (cur) slots.push(cur);
    var out = [];
    for (var k = 0; k < slots.length; k++) {
      var slot = slots[k];
      var cm = /^calc\(([\s\S]*)\)$/i.exec(slot);
      if (cm) {
        var cc = _zwCCDeclCalcCanon(cm[1]);
        out.push(cc !== null ? 'calc(' + cc + ')' : slot); // 不规范形保原槽
        continue;
      }
      if (fn === 'color' && slot === 'xyz') { out.push('xyz-d65'); continue; }
      out.push(slot);
    }
    var chanOut = out.join(' ');
    if (alpha == null) return chanOut;
    // alpha-part 前导点补 0（'.25'→'0.25' 保原位数字）——语料行为表：rgb
    // 恒补（keyword 与 srgb origin 双锚）；hwb/lab 仅 origin 恰保原样时补
    //（origin 变形则保）；hsl/lch/oklab 恒保；oklch/color 无语料锚随族保形。
    // 其余 token（关键字 alpha/none/百分比/calc）一律保形。
    var dot = fn === 'rgb' || ((fn === 'hwb' || fn === 'lab') && originKept);
    var am = dot ? /^([-+]?)\.(\d+)$/.exec(alpha) : null;
    return chanOut + ' / ' + (am ? am[1] + '0.' + am[2] : alpha);
  }

  // 内嵌色/origin/mix 侧规范化。返回 { s } 规范形 | { raw } 保原形（var()/
  // alpha(from)）| null 无效。嵌套 mix/relative 递归规范化（corpus 锚
  // 'color(from color(from color(xyz ...) xyz x y z) xyz x y z)' 整链别名
  // 展开）；失败（null）→ 整值拒。
  function _zwCCDeclColorCanonic(text) {
    var t = _zwCCStripComments(text).trim();
    if (!t) return null;
    if (/^var\(/i.test(t)) return { raw: t };
    // contrast-color（css-color-6）未建模面保原串（relative origin
    // 'rgb(from contrast-color(blue) r g b)' cc4 透传锚）。
    if (/^contrast-color\(/i.test(t)) return { raw: t };
    // color-layers（css-color-6 语法面）：递归校验 + normal 前缀省略（嵌套
    // 色层校验用；'color-layers(red, blue)' 作 mix 侧/相对 origin）。
    if (/^color-layers\(/i.test(t)) {
      var lo = _zwCCDeclLayers(t);
      if (lo === undefined || lo === null) return lo === undefined ? { raw: t } : null;
      return { s: lo };
    }
    var ast = null;
    try { ast = _zwCCParseColorValue(t); } catch (_ePD) { return null; }
    if (!ast) return null;
    if (ast.kind === 'mix') {
      var mo = _zwCCDeclMix(t);
      if (mo === undefined || mo === null) return mo === undefined ? { raw: t } : null;
      return { s: mo };
    }
    if (ast.kind === 'relative') {
      var ro = _zwCCDeclRelative(t);
      if (ro === undefined || ro === null) return ro === undefined ? { raw: t } : null;
      return { s: ro };
    }
    if (ast.kind === 'alpha') return { raw: t };
    // light-dark decl 面保形（css-color-5：声明回读不折叠选支——relative
    // origin 经 decl canon 折叠会丢失 currentcolor 语义，Typed-OM 相对色骨架
    // 面 relative-color-with-system-color 钉面依赖 origin 原文）。
    if (ast.kind === 'lightdark') return { raw: t };
    if (ast.kind === 'named' || ast.kind === 'system') return { s: _zwCCDecodeEscapes(t).toLowerCase() };
    if (ast.kind === 'currentcolor') return { s: 'currentcolor' };
    if (ast.kind === 'transparent') return { s: 'transparent' };
    var r = null;
    try { r = _zwCCResolveText(t, null, 0); } catch (_eRD) { return null; }
    if (!r) {
      // 环境依赖形（sign/abs 容器/字体单位）不求值不拒——保原串交运行时面
      //（与 __zwComputedColor 环境回退同一哲学）。
      if (/(^|[^\w-])(sign|abs)\(/i.test(t)) return { raw: t };
      return null;
    }
    // srgb 族声明面折叠（css-color-5 relative origin/mix 侧序列化：legacy
    // rgba 数字形，通道 none→0 消解（'rgb(none none none)'→'rgb(0, 0, 0)'、
    // 'hsl(120deg 40% 40% / none)'→'rgba(61, 143, 61, 0)' corpus 锚）——
    // 不走 computed none 保形分支（那是 computed 面口径）。
    if (r.kind === 'rgb' && (r.srcKind === 'rgb' || r.srcKind === 'hsl' || r.srcKind === 'hwb')) {
      // alpha none 同消解为 0（'rgb(20% 40% 60% / none)'→'rgba(51, 102, 153, 0)'）。
      var lg = _zwCCSerialLegacyRgb(r.rgb, r.aNone ? 0 : r.alpha, false);
      return { s: lg };
    }
    var s2 = _zwCCSerialResolve(r);
    return s2 ? { s: s2 } : null;
  }

  // 百分比权重文本（分数 → 6 位舍入百分值最短十进制 + '%'）。
  function _zwCCDeclPctTxt(w) {
    return _zwCCNum(Math.round(w * 1e6) / 1e4) + '%';
  }

  // 浅面（无 in 子句 mix 侧）色规范化。color() 形保原 token（Chrome declared
  // color() 不重写数字——'color(srgb .1 .2 .3)' corpus 锚），仅 'xyz' 空间别
  // 名展开（'color(xyz .1 ...)'→'color(xyz-d65 .1 ...)' 锚）；其余委托基座
  // declared 序列化（named 保 keyword、hsl→rgb 数字折、oklab 数值自形——
  // corpus 无 in 两参锚）。null（非法）→ 整值拒。函数声明提升，基座引用在
  // 段尾绑定；返回 { s } 与完整面 canonic 同形。
  function _zwCCShallowColor(c) {
    var t = _zwCCStripComments(c).trim();
    if (!t) return null;
    if (/^color\(/i.test(t)) {
      var ali = t.replace(/(^|[\s(/])xyz(?=$|[\s,)/])/g, '$1xyz-d65');
      return { s: ali };
    }
    var o = _zwCCPrevNormalizeDecl('color', t);
    return o == null ? null : { s: o };
  }

  // relative 通道表槽 token 校验。kw=该 fn 通道关键字白名单；hueSlot=角度
  // 可入且百分比禁入的槽（hsl/hwb 首槽、lch/oklch 第三槽）；allowAngle 同
  // hueSlot。返 true=该槽合法。
  function _zwCCDeclRelSlotOk(kw, hueSlot, tok) {
    var t = String(tok).trim();
    if (!t) return false;
    var low = t.toLowerCase();
    if (low === 'none') return true;
    // sibling-index()/sibling-count() 函数形文法合法（css-values-5；求值归
    // resolve 期 sibling 替换——decl 面只校验槽文法）。
    if (t.indexOf('sibling-') >= 0) return true;
    if (/^calc\(/i.test(t)) {
      // calc 内通道关键字按无单位数代换（Chrome corpus 行为）。加减法混型即
      // 拒：关键字项与带维数项（%/角度）直接相加减——'calc(h + 1deg)'/
      // 'calc(r + 1%)' invalid 锚，'calc(-20 + l)' 纯数合法锚。乘除维数标量
      // 合法（类型标量换算：'calc(r * 100%)'/'calc((h / 360) * 360deg)'/
      // 'calc(50% * l / 100)' 与 sin 复合形 corpus valid 锚）。
      var inner = t.replace(/^calc\(/i, '').replace(/\)$/, '');
      var kwRe = '';
      for (var ki = 0; ki < kw.length; ki++) {
        kwRe += (ki ? '|' : '') + kw[ki];
      }
      var kwTest = new RegExp('(^|[^\\w-])(' + kwRe + ')(?=$|[^\\w-])', 'i');
      var dimTest = /(?:\d)\s*(?:%|deg|turn|rad|grad)(?![\w-])/i;
      var hasKw = kwTest.test(inner);
      if (hasKw) {
        // 深度 0 加减项切分；相邻项一含关键字一含维数字面 → 拒。
        var terms = [], curT = '', d3 = 0;
        for (var ti = 0; ti < inner.length; ti++) {
          var c3 = inner.charAt(ti);
          if (c3 === '(') d3++;
          else if (c3 === ')') d3--;
          if (d3 === 0 && (c3 === '+' || c3 === '-')) {
            if (curT.trim() === '') { curT += c3; continue; } // 前导符号
            terms.push(curT); curT = '';
            continue;
          }
          curT += c3;
        }
        terms.push(curT);
        for (var pi2 = 0; pi2 + 1 < terms.length; pi2++) {
          var kwA = kwTest.test(terms[pi2]), kwB = kwTest.test(terms[pi2 + 1]);
          var dmA = dimTest.test(terms[pi2]), dmB = dimTest.test(terms[pi2 + 1]);
          if ((kwA && dmB) || (kwB && dmA)) return false;
        }
        return true;
      }
      // 纯字面 calc：结果类型槽检查——角度仅 hue 槽、hue 槽禁百分比。
      if (!hueSlot && /(?:\d)\s*(?:deg|turn|rad|grad)(?![\w-])/i.test(inner)) return false;
      if (hueSlot && /(?:\d)\s*%(?![\w-])/.test(inner)) return false;
      return true;
    }
    // 裸数学函数形（css-values-4 § math functions，corpus 'pow(l, 1)' 直写
    // 形钉面）：decl 面只校验函数名白名单与括号平衡——求值归 resolve 期
    // _zwCCEvalRel（_zwCCCalcSpec 裸函数形同路由 chanexpr）。calc( 已由上臂
    // 先行；色函数名不在白名单（嵌套相对色非通道文法）。
    if (/^\s*-?(atan2|pow|hypot|round|mod|rem|sin|cos|tan|asin|acos|atan|sqrt|log|exp|sign|abs|clamp|min|max)\(/i.test(t) && /\)$/.test(t)) {
      return true;
    }
    var nm = /^[-+]?(\d+\.?\d*|\.\d+)(%|deg|turn|rad|grad)?$/i.exec(t);
    if (nm) {
      var u = (nm[2] || '').toLowerCase();
      if (u === '%') return !hueSlot; // 百分比禁入 hue 槽（'hsl 10% s l'/'lch l c 10%' 锚）
      if (u !== '') return hueSlot; // 角度仅 hue 槽（'rgb r 10deg 10' 锚）
      return true;
    }
    var km = /^[a-zA-Z][a-zA-Z0-9-]*$/.exec(t);
    if (km) {
      for (var i = 0; i < kw.length; i++) if (kw[i] === low) return true;
      return false; // 关键字不在白名单（'red'/'l g b'/'x g b'/'hue' 锚）
    }
    return false;
  }

  // relative 通道表槽类型校验（css-color-5 相对色通道文法 + corpus invalid 88
  // 面的 Chrome 行为）：恰好 3 通道槽 + 可选 / alpha 槽；color() 表首 token 为
  // 空间名（rgb 族关键字 r/g/b、xyz 族 x/y/z）；关键字在全部槽位自由排列且含
  // 'alpha'（'b alpha r / g'/'h alpha l / s'/'x x x / x' valid 锚）；角度仅
  // hue 槽、百分比禁入 hue 槽（'r g b / 10deg' 锚）。返 true=表合法。
  function _zwCCDeclRelTableOk(fn, rest) {
    var kw = fn === 'rgb' ? ['r', 'g', 'b']
      : fn === 'hsl' ? ['h', 's', 'l']
      : fn === 'hwb' ? ['h', 'w', 'b']
      : (fn === 'lab' || fn === 'oklab') ? ['l', 'a', 'b']
      : (fn === 'lch' || fn === 'oklch') ? ['l', 'c', 'h']
      : null;
    var body = String(rest).trim();
    if (fn === 'color') {
      var sm = /^(srgb|srgb-linear|a98-rgb|rec2020|prophoto-rgb|display-p3|display-p3-linear|xyz|xyz-d50|xyz-d65)(\s+([\s\S]*))?$/i.exec(body);
      if (!sm) return false;
      var sp = sm[1].toLowerCase();
      kw = (sp === 'xyz' || sp === 'xyz-d50' || sp === 'xyz-d65') ? ['x', 'y', 'z'] : ['r', 'g', 'b'];
      body = (sm[3] || '').trim();
    }
    if (!kw) return true; // 非通道表 fn——不拦（保上层口径）
    kw = kw.concat(['alpha']); // 关键字全槽并集（含 alpha 槽）
    var parts = _zwCCSplitModern(body);
    if (parts.comps.length !== 3) return false;
    for (var ci = 0; ci < 3; ci++) {
      var hueSlot = (fn === 'hsl' || fn === 'hwb') ? ci === 0 : (fn === 'lch' || fn === 'oklch') ? ci === 2 : false;
      if (!_zwCCDeclRelSlotOk(kw, hueSlot, parts.comps[ci])) return false;
    }
    if (parts.alpha != null && !_zwCCDeclRelSlotOk(kw, false, parts.alpha)) return false;
    return true;
  }

  // relative 色函数声明面。返规范串 | null（语法无效，拒）| undefined（非
  // 相对形，交基座）。
  function _zwCCDeclRelative(core) {
    var m = /^(rgb|rgba|hsl|hsla|hwb|lab|lch|oklab|oklch|color)\(([\s\S]*)\)$/i.exec(core);
    if (!m) return undefined;
    var rel = _zwCCTryRelative(m[2]);
    if (!rel) return undefined; // 非 relative 形（普通 rgb() 等）走基座
    var ast = null;
    try { ast = _zwCCParseColorValue(core); } catch (_ePR) { return null; }
    if (!ast || ast.kind !== 'relative') return null; // relative 语法无效（legacy
    // 逗号形 'rgb(from red, r, g, b)' 解析即败——css-color-5 relative 现代语法）
    var fnFold = m[1].toLowerCase();
    if (fnFold === 'rgba') fnFold = 'rgb';
    else if (fnFold === 'hsla') fnFold = 'hsl';
    // 通道表槽类型校验（css-color-5 通道文法；corpus invalid 88 面）。
    if (!_zwCCDeclRelTableOk(fnFold, rel.rest)) return null;
    var oast = null;
    try { oast = _zwCCParseColorValue(_zwCCStripComments(rel.origin).trim()); } catch (_ePO) { return null; }
    if (!oast) {
      // contrast-color origin（css-color-6 未建模）交基座透传——有效性
      // oracle 不覆盖未建模面（'rgb(from contrast-color(blue) r g b)'
      // 'contrast-color(contrast-color(pink))' cc4 透传锚）。
      if (/^contrast-color\(/i.test(_zwCCStripComments(rel.origin).trim())) return undefined;
      return null; // origin 非法 → 整值拒
    }
    if (oast.kind !== 'system') {
      // 有效性 oracle：整 ast 走 computed resolve（origin 可解析形 + 通道
      // 关键字/类型全查；system origin 不入转换链——声面合法，computed 面放弃）。
      var rv = null;
      try { rv = _zwCCResolve(ast, null, 0); } catch (_eRR) { rv = null; }
      if (!rv) return null;
    }
    var oc = _zwCCDeclColorCanonic(rel.origin);
    if (oc === null) return null;
    // origin 规范化是否变形（≠原文）：hwb/lab 的 rest alpha 前导点补 0 仅在
    // origin 恰保原样时发生（'hwb(from rebeccapurple / .2)'→0.2 补、
    // 'hwb(from rgb(...) / .2)' 保、'lab(from lab(25 20 50) / .35)' 补、
    // 'lab(from lab(... / 40%) / .35)' 保——语料行为表，无规范统一规则）。
    var originKept = oc.s != null && oc.s === _zwCCStripComments(rel.origin).trim();
    return fnFold + '(from ' + (oc.raw ? oc.raw : oc.s) + ' ' + _zwCCDeclRestCanon(fnFold, rel.rest, originKept) + ')';
  }

  // color-mix 声明面（css-color-5：可选 in 子句 + n≥1 个 [色 [权重]] 参）。
  // 返规范串 | null（语法无效，拒）| undefined（无语料锚形状，交基座透传）。
  // 权重序列化：全未指定全省（均匀缺省重建一致——含 n=2 双 50% 与 n≥3 多色
  // 'red, green, blue' 锚）、单参 100% 省略（缺省重建一致）/0% 保留、部分未
  // 指定补均分剩余、全显式 n≥3 全保留；任一侧 calc 权重则两侧逐侧保形。
  // 色规范化双口径：带 in 子句（解析面）→ 本管线完整规范化（corpus 'in xyz'
  // → 'xyz-d65 0.1' 数字化锚）；无 in 子句（浅面）→ 基座 declared 色序列化
  //（'hsl(...)'→rgb 折、'color(srgb .1 .2 .3)' 保原数字 token、oklab 数值
  // 自形——corpus 无 in 两参锚）。
  function _zwCCDeclMix(core) {
    var m = /^color-mix\(([\s\S]*)\)$/i.exec(core);
    if (!m) return undefined;
    var args = _zwCCTopSplit(m[1], ',');
    if (args.length < 1 || args.length > 8) return undefined;
    var hm = /^in\s+([a-z0-9-]+)(\s+(?:shorter|longer|increasing|decreasing)\s+hue)?$/i.exec(args[0].trim());
    var mi = 0;
    if (hm) {
      if (args.length === 1) return undefined; // 仅 in 子句（缺色参）——无语料锚，保透传
      mi = 1;
    }
    // method 子句规范化（css-color-5 序列化：插值空间缺省 oklab、hue 方法缺省
    // shorter——整体缺省时省略子句；'in xyz' 别名 'in xyz-d65' corpus 锚；
    // 非 'shorter hue' 的 hue 方法保形——'in hsl longer hue' 实证）。
    var methodTxt = '';
    if (hm) {
      var sp = hm[1].toLowerCase();
      var spInfo = _zwCCSpaces[sp];
      if (spInfo && spInfo.alias) sp = spInfo.alias;
      var hueM = hm[2] ? hm[2].toLowerCase().replace(/\s+/g, ' ').trim() : '';
      if (hueM) {
        // hue 方法仅限 hue 承载空间（css-color-5 §color-mix 插值：hsl/hwb/lch/
        // oklch；其余空间带 hue 方法整值非法——corpus invalid 'in srgb longer
        // hue' 等 12 面）。
        if (sp !== 'hsl' && sp !== 'hwb' && sp !== 'lch' && sp !== 'oklch') return null;
      }
      if (sp !== 'oklab' || (hueM !== '' && hueM !== 'shorter hue')) {
        methodTxt = 'in ' + sp + (hueM && hueM !== 'shorter hue' ? ' ' + hueM : '');
      }
    }
    function side(text) {
      var t = text.trim();
      var w = null, wraw = null;
      var c = t;
      var lead = /^([-+]?(\d+\.?\d*|\.\d+))%(?![\d.])\s+([\s\S]+)$/.exec(t);
      if (lead) {
        w = parseFloat(lead[1]) / 100;
        if (w < 0 || w > 1) return null;
        c = lead[3].trim();
      } else {
        var trail = /^([\s\S]+?)\s+([-+]?(\d+\.?\d*|\.\d+))%$/.exec(t);
        if (trail) {
          var d = 0;
          for (var i = 0; i < trail[1].length; i++) {
            if (trail[1].charAt(i) === '(') d++;
            else if (trail[1].charAt(i) === ')') d--;
          }
          if (d === 0) {
            w = parseFloat(trail[2]) / 100;
            if (w < 0 || w > 1) return null;
            c = trail[1].trim();
          }
        }
        if (w == null) {
          // calc(<num>%) 权重不可求值——保形输出（'red calc(20%)'/'calc(50% *
          // sign(100em - 1px))' corpus 锚），域检查跳过。
          var ctrail = /^([\s\S]+?)\s+(calc\((?:(?!calc\()[\s\S])*)\)$/.exec(t);
          if (ctrail) {
            var d2 = 0;
            for (var j = 0; j < ctrail[1].length; j++) {
              if (ctrail[1].charAt(j) === '(') d2++;
              else if (ctrail[1].charAt(j) === ')') d2--;
            }
            if (d2 === 0) { w = 'calc'; wraw = ctrail[2] + ')'; c = ctrail[1].trim(); }
          }
        }
      }
      // 无 in 子句（浅面）→ 基座 declared 色序列化（含 mix/relative 透传原串
      // 形；'color-mix(hsl(...), hsl(...))'→rgb 折锚）；带 in 子句 → 完整解析
      // 面规范化。null（非法）→ 整值拒。
      var out = hm ? _zwCCDeclColorCanonic(c) : _zwCCShallowColor(c);
      if (out === null) return null;
      return { color: out, w: w, wraw: wraw };
    }
    var sides = [];
    for (var si = mi; si < args.length; si++) {
      var sd = side(args[si]);
      if (sd === null) return null;
      sides.push(sd);
    }
    function colTxt(sd) { return sd.color.raw ? sd.color.raw : sd.color.s; }
    var body;
    var anyCalc = false;
    var wsum = 0, nMissing = 0;
    for (var qi = 0; qi < sides.length; qi++) {
      if (sides[qi].w === 'calc') anyCalc = true;
      else if (sides[qi].w == null) nMissing++;
      else wsum += sides[qi].w;
    }
    if (anyCalc) {
      // 任一侧 calc 权重 → 各侧权重文本逐侧保形（calc 侧保形、数值侧照常输出
      //——'red calc(10%), blue 50%' 锚；缺失侧裸色名——'red calc(50% *
      // sign(...)), blue' 锚），不做数值补全/省略。
      var parts = [];
      for (var ri = 0; ri < sides.length; ri++) {
        parts.push(colTxt(sides[ri]) + (sides[ri].wraw ? ' ' + sides[ri].wraw
          : sides[ri].w != null ? ' ' + _zwCCDeclPctTxt(sides[ri].w) : ''));
      }
      body = parts.join(', ');
    } else if (nMissing === sides.length) {
      // 全未指定 = 均匀缺省，重建一致 → 全省略（n=2 双 50%、n≥3 'red, green,
      // blue' corpus 锚）。
      var allParts = [];
      for (var ui = 0; ui < sides.length; ui++) allParts.push(colTxt(sides[ui]));
      body = allParts.join(', ');
    } else {
      // 部分未指定 → 未指定侧均分剩余（'red 50%, green, blue'→25%/25% 锚）。
      // 全体权重序列（补全后）= 均匀缺省（各 1/n）→ 全省略（重建一致——
      // 'red 50%, blue' 双省、'red 100%' 单参省、'red, green, blue' 全省锚）；
      // 否则显式侧逐侧保留输出（'70% red, 50% blue' 全保留、'red 0%' 单参
      // 保留、'red 75%, blue 75%' 双保留锚）。
      var wmiss = nMissing > 0 ? (1 - wsum) / nMissing : 0;
      var uniform = true;
      for (var vi = 0; vi < sides.length; vi++) {
        var wv = sides[vi].w == null ? wmiss : sides[vi].w;
        if (Math.round(wv * 1e6) !== Math.round(1e6 / sides.length)) { uniform = false; break; }
      }
      var outParts = [];
      for (var ti = 0; ti < sides.length; ti++) {
        var sdw = sides[ti].w;
        outParts.push(colTxt(sides[ti])
          + (uniform ? '' : sdw == null ? ' ' + _zwCCDeclPctTxt(wmiss) : ' ' + _zwCCDeclPctTxt(sdw)));
      }
      body = outParts.join(', ');
    }
    return 'color-mix(' + (methodTxt ? methodTxt + ', ' : '') + body + ')';
  }

  // contrast-color 声明面（css-color-6：恰一 <color> 参；corpus invalid 面
  // 空参/数值/keyword/双参全拒）。返 null（拒）| undefined（不拦，交基座）。
  function _zwCCDeclContrast(core) {
    var m = /^contrast-color\(([\s\S]*)\)$/i.exec(core);
    if (!m) return undefined;
    var t = m[1].trim();
    if (!t) return null;
    if (_zwCCTopSplit(t, ',').length !== 1) return null;
    var ast = null;
    try { ast = _zwCCParseColorValue(t); } catch (_ePC) { return null; }
    if (!ast) {
      // 嵌套 contrast-color 参（'contrast-color(contrast-color(pink))' cc4
      // 透传锚）——css-color-6 未建模面交基座保原串。
      if (/^contrast-color\(/i.test(t)) return undefined;
      return null;
    }
    return undefined; // 合法形无 declared 重排锚——保原串
  }

  // alpha() 声明面校验（css-color-5 §relative-alpha；corpus invalid 23 面）：
  // 形态恰为 alpha(from <color> / <alpha>)——缺 from、缺 origin、缺 alpha、
  // 额外 token（'0.5 0.5'）、origin 非色（'42'/'none'/'red, blue'）、alpha 槽
  // 通道关键字（'calc(r * 0.5)'）均拒；合法形委托基座归一（origin 规范化 +
  // calc 折形——alpha-color-parsing-valid 42 锚零漂移）。返规范串 | null（拒）。
  function _zwCCDeclAlpha(core) {
    var m = /^alpha\(\s*from\s+([\s\S]+)\)$/i.exec(core);
    if (!m) return null; // 缺 from（'alpha(red / 0.5)'）/'alpha()'
    var body = m[1];
    var depth = 0, si = -1;
    for (var i = 0; i < body.length; i++) {
      var ch = body.charAt(i);
      if (ch === '(') depth++;
      else if (ch === ')') depth--;
      else if (ch === '/' && depth === 0) { si = i; break; }
    }
    if (si < 0) return null; // 缺 alpha（'alpha(from red)'）
    var origin = body.slice(0, si).trim();
    var apart = body.slice(si + 1).trim();
    if (!origin || !apart) return null; // 缺 origin / 缺 alpha 值
    if (/^alpha\(\s*from\s+/i.test(origin)) {
      // 嵌套 alpha 递归校验（'alpha(from alpha(from currentcolor) / 0.5)' 内
      // 层缺 alpha-part 拒）。
      if (_zwCCDeclAlpha(origin) == null) return null;
    } else {
      if (/^none$/i.test(origin)) return null;
      try { if (_zwCCParseColorValue(origin) == null) return null; } catch (_eAO) { return null; }
    }
    // alpha 槽：深度 0 恰一 token（'0.5 0.5'/'alpha alpha'/'/ 0.5' 多 token 拒）。
    var toks = [], cur = '', d2 = 0;
    for (var j = 0; j < apart.length; j++) {
      var c2 = apart.charAt(j);
      if (c2 === '(') d2++;
      else if (c2 === ')') d2--;
      if (d2 === 0 && /\s/.test(c2)) { if (cur) { toks.push(cur); cur = ''; } continue; }
      cur += c2;
    }
    if (cur) toks.push(cur);
    if (toks.length !== 1) return null;
    if (/^calc\(/i.test(toks[0])) {
      // calc 内 ident 白名单（alpha-part 作用域仅 alpha/树函数——'calc(r * 0.5)'
      // 通道关键字拒，corpus 锚）。
      var ids = toks[0].match(/[A-Za-z][A-Za-z0-9-]*/g) || [];
      for (var wi = 0; wi < ids.length; wi++) {
        if (ids[wi] !== 'alpha' && ids[wi] !== 'calc' && ids[wi] !== 'min' && ids[wi] !== 'max' &&
          ids[wi] !== 'clamp' && ids[wi] !== 'sibling-index' && ids[wi] !== 'sibling-count') return null;
      }
    } else {
      var a0 = toks[0].toLowerCase();
      // 裸树作用域函数（'sibling-index()'/'sibling-count()' valid 锚）。
      if (/^(sibling-index|sibling-count)\(\s*\)$/.test(a0)) {
        return _zwCCPrevNormalizeDecl == null ? undefined : _zwCCPrevNormalizeDecl('color', core);
      }
      if (!(/^[-+]?(\d+\.?\d*|\.\d+)%?$/.test(a0) || a0 === 'none' || a0 === 'alpha')) return null;
    }
    return _zwCCPrevNormalizeDecl == null ? undefined : _zwCCPrevNormalizeDecl('color', core);
  }

  // color-layers 声明面（css-color-6 #color-layers：可选单一 <blend-mode> +
  // n≥1 逗号分隔色层）。序列化：'normal,' 前缀省略（缺省重建一致——corpus
  // valid 替换锚），其余 blend 保形小写、色层逐字保形（valid 160 逐字锚）。
  // 校验：blend 白名单 16 关键字（plus-lighter/plus-darker 及任意其他 token
  // 非法）、色层必为可解析色、无空段。返规范串 | null（拒）| undefined（不拦）。
  function _zwCCDeclLayers(core) {
    var m = /^color-layers\(([\s\S]*)\)$/i.exec(core);
    if (!m) return undefined;
    var args = _zwCCTopSplit(m[1], ',');
    if (args.length < 1) return null;
    var BLEND = ['normal', 'multiply', 'screen', 'overlay', 'darken', 'lighten',
      'color-dodge', 'color-burn', 'hard-light', 'soft-light', 'difference',
      'exclusion', 'hue', 'saturation', 'color', 'luminosity'];
    var blend = '';
    var ci = 0;
    var a0 = args[0].trim().toLowerCase();
    if (BLEND.indexOf(a0) >= 0) { blend = a0; ci = 1; }
    if (ci === 1 && args.length === 1) return null; // 'color-layers(normal)' 零色层
    var colors = [];
    for (var i = ci; i < args.length; i++) {
      var c = args[i].trim();
      if (!c) return null; // 'red, blue, ' 尾空段
      if (_zwCCDeclColorCanonic(c) == null) return null; // 'red blue' 空格分隔等
      colors.push(c);
    }
    if (!colors.length) return null;
    return 'color-layers(' + (blend && blend !== 'normal' ? blend + ', ' : '') + colors.join(', ') + ')';
  }

  // declared 归一覆盖（part11-color.js 基座之后装配——别名链尾挂）。
  var _zwCCPrevNormalizeDecl = typeof globalThis.__zwColorNormalizeDecl === 'function'
    ? globalThis.__zwColorNormalizeDecl : null;
  globalThis.__zwColorNormalizeDecl = function (prop, value) {
    if (prop !== 'color' || _zwCCPrevNormalizeDecl == null) {
      return _zwCCPrevNormalizeDecl ? _zwCCPrevNormalizeDecl(prop, value) : String(value == null ? '' : value);
    }
    var s = String(value == null ? '' : value);
    var t = s.trim();
    if (t === '') return _zwCCPrevNormalizeDecl(prop, value);
    var pri = '';
    var pm = /\s*!\s*important\s*$/i.exec(t);
    if (pm) { pri = ' !important'; t = t.slice(0, pm.index).trim(); if (!t) return null; }
    var low0 = t.toLowerCase();
    if (low0 === 'initial' || low0 === 'inherit' || low0 === 'unset'
      || low0 === 'revert' || low0 === 'revert-layer') {
      return _zwCCPrevNormalizeDecl(prop, value); // csswide 交基座
    }
    // 注释/var()/转义整值不拦（基座透传面——cssText 注释保形 T49fix、var 运行
    // 时替换、转义 host 词法器）。
    if (t.indexOf('/*') >= 0 || t.indexOf('var(') >= 0 || t.indexOf('\\') >= 0) {
      return _zwCCPrevNormalizeDecl(prop, value);
    }
    var out;
    var fnHead = /^([a-zA-Z][a-zA-Z0-9-]*)\s*\(/.exec(t);
    if (fnHead) {
      var fnL = fnHead[1].toLowerCase();
      if (fnL === 'rgb' || fnL === 'rgba' || fnL === 'hsl' || fnL === 'hsla' || fnL === 'hwb'
        || fnL === 'lab' || fnL === 'lch' || fnL === 'oklab' || fnL === 'oklch' || fnL === 'color') {
        out = _zwCCDeclRelative(t);
      } else if (fnL === 'color-mix') {
        out = _zwCCDeclMix(t);
      } else if (fnL === 'contrast-color') {
        out = _zwCCDeclContrast(t);
      } else if (fnL === 'alpha') {
        out = _zwCCDeclAlpha(t);
      } else if (fnL === 'color-layers') {
        out = _zwCCDeclLayers(t);
      }
    }
    if (out === undefined) return _zwCCPrevNormalizeDecl(prop, value);
    return out === null ? null : out + pri;
  };
})();
