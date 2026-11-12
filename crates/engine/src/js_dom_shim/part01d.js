  // ============================================================
  // editing-compat style-with-value 簇（2026-11-01）：execCommand 五标准行内值
  // 命令（fontsize/fontname/forecolor/backcolor/hilitecolor）实应用引擎。
  //
  // 归因（gate7 语料普查 + zz-probe-fs6 探针实证）：run 语料 compare innerHTML
  // 567F 全部走 document.execCommand（tests.js:5613——compare 列读 testDiv，
  // vendored myExecCommand 只服务 spec 表格列），而引擎 execCommand 此前只有
  // _zwExecCmdInputType 事件映射（beforeinput/input 照派）+ 值查询面
  // （part01c __zw_ce_value_query，2026-11-01 接通），零变更臂 → 一切期待
  // 包裹的 apply 行 no-op 失败（探针实证 size 2/4/6 全返原始 HTML）。
  //
  // 本文件按 editing spec 参考实现（tests/wpt-runner/wpt-data/editing/include/
  // implementation.js）移植 setSelectionValue 全流程：
  //   ① 无 contained formattable（collapsed）→ state/value override 记账（值
  //     override 经 part01c __zw_ce_value_query 回读——Chrome qV-after 期望源）；
  //   ② 边界 splitText（引擎安全拆分：data 写 + createTextNode + insertBefore
  //     ——引擎 splitText 仅 handle-based wrapper 可用，plain 文本节点缺法，
  //     zz-probe-fs4 实证）；
  //   ③ contained element clearValue pass + node pushDownValues/forceValue
  //     pass（forceValue：CSS flag false 时 font color/face/size attr 臂 +
  //     span style 写入尾；wrap/reorder/isSimpleModifiableElement 全链）；
  //   ④ areEquivalentValues（色三命令 normalizeColor 等值）+
  //     areLooselyEquivalentValues（fontsize sizeMap——font[size] 1-7 级联
  //     px 读数静态表）。
  // 共享机具复用 part01c 闭包：_zwJus*（move/range/tree/visible/editable/cs）
  // + _zwVal*（containment/formattable/effective）+ _zwPlainDecl/_zwPlainColorSpec
  // （specified style 读数与颜色解析）。选区单 range 面经 sel._ranges[0]
  // （_zwJusMovePreservingRanges 边界补偿回写——对象身份不变）。
  // 挂载位：js_dom_bridge.rs 在 part01c 后、part07（闭合外层 IIFE）前拼接，
  // 同闭包可见全部 _zwJus*/_zwVal* 机具；part06 经 globalThis.__zw_ce_value_apply
  // 单点调用。
  // https://w3c.github.io/editing/execCommand.html#set-the-selection's-value
  // https://w3c.github.io/editing/execCommand.html#forcing-the-value-of-a-node
  // FIXME: 值等值/解析面窄口——rgb() 空格语法与 calc()/系统色未覆盖（语料
  //   范围外）；isExtraneousLineBreak 沿用 justify 簇近似（非布局探测）。
  // FIXME: bold/italic/underline/strikethrough 等其余 format 命令仍走
  //   part06 既有 flat 臂（本文件不接管）。
  // ============================================================

  // —— 值命令表（五标准行内值命令）——
  var _zwVaFive = { fontsize: 1, fontname: 1, forecolor: 1, backcolor: 1, hilitecolor: 1 };

  function _zwVaCssFlag() {
    return !!globalThis.__zwStyleWithCSS;
  }

  function _zwVaGetActiveRange() {
    var sel = globalThis.getSelection ? globalThis.getSelection() : null;
    if (!sel || !sel.rangeCount) return null;
    var r = sel._ranges && sel._ranges.length ? sel._ranges[0] : null;
    if (!r || !r.startContainer || !r.endContainer) return null;
    if (r.startContainer.nodeType !== 1 && r.startContainer.nodeType !== 3) return null;
    if (r.endContainer.nodeType !== 1 && r.endContainer.nodeType !== 3) return null;
    return r;
  }

  // —— CSSOM 口径颜色归一（参考实现 :415 normalizeColor 的本仓等价面）——
  // vendored 借真浏览器 gCS 通道归一（钳位/命名/hex/hsl/rgba α=1 合并全在此
  // 发生）；本仓复用 part01c _zwPlainColorSpec（named 表 + rgb/hsl 序列化）
  // 后补 rgb 分量 clamp——plain 解析未钳而 gCS 原生钳位（语料 rgb(15, -10,
  // 375) → rgb(15, 0, 255) 实证）。不可解析返 ''（调用方按无效色分支）。
  function _zwVaNormalizeCssColor(v) {
    var out;
    try {
      out = _zwPlainColorSpec(String(v == null ? '' : v), null);
    } catch (_eNcc) {
      return '';
    }
    if (!out) return '';
    var m = /^(rgba?)\(([^)]*)\)$/.exec(out);
    if (!m) return out;
    var nums = m[2].split(', ');
    for (var i = 0; i < 3; i++) {
      var n = Math.round(Number(nums[i]));
      nums[i] = String(n < 0 ? 0 : (n > 255 ? 255 : n));
    }
    return m[1] + '(' + nums.join(', ') + ')';
  }

  // parseSimpleColor（参考实现 :471）：normalizeColor → 'rgb(N, N, N)' 形 →
  // '#xxxxxx'（HTML 简单色序列化形态，语料期望 <font color="#0000ff"> 实证）。
  // rgba(α<1)/transparent/currentcolor 归一后非 rgb 形 → null（同参考实现——
  // CSS false 旗由此落 span+style 臂而非 font 臂）。
  function _zwVaParseSimpleColor(v) {
    var c = _zwVaNormalizeCssColor(v);
    if (!c) return null;
    if (!/^rgb\([0-9]+, [0-9]+, [0-9]+\)$/.test(c)) return null;
    var nums = c.slice(4, -1).split(', ');
    var hex = function (n) {
      var h = Number(n).toString(16);
      return h.length < 2 ? '0' + h : h;
    };
    return '#' + hex(nums[0]) + hex(nums[1]) + hex(nums[2]);
  }

  // normalizeColor 等值键：可解析 → 规整 rgb/rgba 串（含钳位）；不可解析 →
  // ''（两侧皆不可解析 = 等值，参考实现「neither string is a valid CSS color」臂）。
  function _zwVaNormalizeColorKey(v) {
    return _zwVaNormalizeCssColor(v);
  }

  // Chrome 指定形等值键（forecolor 专用）：named keyword 色以 keyword 名为键
  // （CSSOM 指定形保 keyword，本仓 gPV=blue 实测同形），余形（hex/rgb/hsl）
  // 归一 rgb 串。keyword ≢ 数值同色的 rgb 新值——Chrome 录形 forecolor.js
  // :382（keyword 外壳切分）vs :387（hex 外壳合并）实证；backcolor/hilitecolor
  // 录形 keyword 外壳合并（backcolor.js :193），比较仍走 _zwVaNormalizeColorKey。
  function _zwVaChromeColorKey(v) {
    var s = String(v == null ? '' : v).trim();
    var low = s.toLowerCase();
    if (low && _zwPlainNamedColor[low] !== undefined) return low;
    return _zwVaNormalizeCssColor(s);
  }

  // peel 克隆 style 属性 CSSOM 归一：Chrome splitTreeNode 经 CSSOM 克隆，attr
  // 文本走声明块序列化，keyword 色落 rgb 形（语料 kept-half `color:brown` 期望
  // `color:rgb(165, 42, 42)`、`background-color:aqua/tan` 期望 rgb 形实证）。只
  // 归一 color/background-color 两属性（语料录面），余声明原文保留；`prop:value`
  // 形 `; ` 连接（harness `: /; ` 正则同一化面）；!important 色归一失败原文保留。
  function _zwVaNormalizeStyleAttrCss(raw, preserveColorKeywords) {
    var parts = String(raw == null ? '' : raw).split(';');
    var out = [];
    for (var i = 0; i < parts.length; i++) {
      var d = parts[i];
      var colon = d.indexOf(':');
      if (colon <= 0) continue;
      var pn = d.slice(0, colon).trim().toLowerCase();
      var pv = d.slice(colon + 1).trim();
      if (!pn || !pv) continue;
      if (pn === 'color' || pn === 'background-color') {
        // ③'-a（Chrome 指定形等值前置）：祖先链规范形直出保留 keyword 原文——
        // Chrome CSSOM 对 authored keyword 色保 keyword 形（zz-probe-cssom
        // gPV=blue 实证），forecolor 等值比较按指定形（keyword 壳 ≢ rgb 新值，
        // forecolor.js :382 keyword 外壳切分 vs :387 hex 外壳合并实证）；
        // keyword 保留对 bg 命令无碍（bg 比较仍关键词盲，:193 keyword 外壳
        // 合并录形）。克隆路径（缺省 falsy）仍归一 rgb 形（kept-half 录形）。
        if (preserveColorKeywords
            && _zwVaChromeColorKey(pv) === pv.trim().toLowerCase()) {
          out.push(pn + ':' + pv);
          continue;
        }
        var nv = _zwVaNormalizeCssColor(pv);
        if (nv) pv = nv;
      }
      out.push(pn + ':' + pv);
    }
    return out.join('; ');
  }

  // areEquivalentValues（参考实现 :2008）——五命令子集：fontsize/fontname 无
  // equivalentValues 定义（纯串等值），色三命令 normalizeColor 等值。
  function _zwVaEquivalentValues(cmd, v1, v2, numericFace) {
    if (v1 === null && v2 === null) return true;
    if (typeof v1 !== 'string' || typeof v2 !== 'string') return false;
    if (v1 === v2) return true;
    if (cmd === 'bold') {
      // DC-2 P2 apply 面（2026-11-11）：vendored equivalentValues :3133 原文——
      // bold≡700、normal≡400（UA 表数值读数与 keyword 新值同值判定）。
      return (v1 === 'bold' && v2 === '700') || (v1 === '700' && v2 === 'bold')
        || (v1 === 'normal' && v2 === '400') || (v1 === '400' && v2 === 'normal');
    }
    if (cmd === 'forecolor' || cmd === 'backcolor' || cmd === 'hilitecolor') {
      // ③'-a：forecolor tag 模式指定形比较（keyword 壳 ≢ rgb 新值——:382
      // keyword 外壳切分 vs :387 hex 外壳合并录形）；css 模式保持归一键
      // （:376 css keyword 行录形接受合并形，指定形切分仅 tag 模式实证面）。
      // backcolor/hilitecolor 录形 keyword 壳合并（backcolor.js :193）恒归一键。
      // numericFace：effective 面读数走数值键——attr 来源值经
      // _zwVaNormalizeCssColor 预归一 rgb 形，与 keyword 新值（裸 fixture
      // forecolor blue → <font color=#0000ff>）在 Chrome 语义下恒同色
      // （:114/:269 录形 attr-only 无冗余 style 声明）；keyword 感知仅
      // 属指定形（CSSOM 保留 authored 形），不适用于有效值/attr 面。
      if (cmd === 'forecolor' && !_zwVaCssFlag() && !numericFace) {
        return _zwVaChromeColorKey(v1) === _zwVaChromeColorKey(v2);
      }
      return _zwVaNormalizeColorKey(v1) === _zwVaNormalizeColorKey(v2);
    }
    if (cmd === 'fontsize') {
      // legacy 档位等值面：关键字/数字串统一经 getLegacyFontSize 映射比较
      // （"large" ≡ "4"；相对 size attr 已在 specifiedValue 归一关键字）。
      var l1 = /^[1-7]$/.test(v1) ? v1 : _zwVaLegacyValueOf(v1);
      var l2 = /^[1-7]$/.test(v2) ? v2 : _zwVaLegacyValueOf(v2);
      return l1 != null && l2 != null && l1 === l2;
    }
    return false;
  }

  // areLooselyEquivalentValues 的 sizeMap（参考实现 :2049-2071）——font[size]
  // 1-7 的级联 font-size 读数静态表。**host 引擎 UA 计算口径**（10/13/16/18/
  // 24/32/48px 整数表，与 part01c _zwValLegacyBounds 中点标度同构）——不复刻
  // vendored 的动态 gCS 读法：createElement 自建 wrapper 无 host 映射，gCS 走
  // plain 回落表（10.08/13.12/18.08px 小数），与 effectiveValue 的 host 通道
  // （语料元素 handle-proxy → 18px 整数）异链——动态读 sizeMap['large']=
  // '18.08px' ≠ effective '18px'，looselyEquivalent 恒 false 致同值选区误包
  // （fontsize 5 例 P→F 实证）。Chrome 两侧同走原生 gCS 恒同值，我方等价面 =
  // 与 effectiveValue 链一致。
  var _zwVaSizeMapData = {
    'x-small': '10px', small: '13px', medium: '16px', large: '18px',
    'x-large': '24px', 'xx-large': '32px', 'xxx-large': '48px'
  };
  function _zwVaSizeMap() {
    return _zwVaSizeMapData;
  }

  function _zwVaLooselyEquivalent(cmd, v1, v2, numericFace) {
    if (_zwVaEquivalentValues(cmd, v1, v2, numericFace)) return true;
    if (cmd !== 'fontsize' || typeof v1 !== 'string' || typeof v2 !== 'string') return false;
    var map = _zwVaSizeMap();
    return v1 === map[v2] || v2 === map[v1];
  }

  // getSpecifiedCommandValue（参考实现 :2201）——五命令子集：色三命令 display
  // 非 inline → null；style attr 相关属性读数（_zwPlainDecl：style IDL 优先、
  // attr 扫描兜底）；font color/face/size 表现提示臂（size → 关键字表）。
  function _zwVaSpecifiedValue(el, cmd) {
    if (!el || el.nodeType !== 1) return null;
    if ((cmd === 'backcolor' || cmd === 'hilitecolor')
        && _zwJusCs(el, 'display') !== 'inline') {
      return null;
    }
    // boolean 命令族读面（vendored getSpecifiedCommandValue :2224-2337 同构；
    // 2026-11-10 boolean 读面刀）：此前 boolean 命令落入下方 prop 映射 else 臂
    // 读 background-color——背景块形上五条 boolean 记录/恢复三元组全得幻值
    // 'yellow'，fb 溶解后 restore 五连幻载体（背景块 + <b>[foo]</b> + fb 探针
    // 五嵌套背景 span 实证；纯背景形更走 wrap-in-carrier→fixDisallowed 链内容
    // 全失）；且 b/i/u/s/sub/sup 标签形从不刹车，爬升穿壳到根误记 null。
    // sub/sup 只读标签（vendored 无 vertical-align 面）。
    if (cmd === 'subscript' || cmd === 'superscript') {
      var tVaS = _zwJusTag(el);
      if (tVaS === 'sup') return 'superscript';
      if (tVaS === 'sub') return 'subscript';
      return null;
    }
    // strike/underline 的 style 声明面先于标签面：短手 text-decoration 与
    // 长手 text-decoration-line 在 Chrome 是同一逻辑属性（声明层长手写入
    // 在 getSpecifiedCommandValue 读回同一值）。短手缺位时长手回退读——
    // 本引擎 Site A 溶解换形产物即长手形（strikethrough.js:492/:532 录形
    // `<span style="text-decoration-line:…">`），指定值链读不到会令
    // forceValue 等值门误判重裹（T14c/T15c 探针实证）；长手存在但不含
    // 该线值同样返 null 掩蔽标签面。
    if (cmd === 'strikethrough') {
      var tdVaS = _zwPlainDecl(el, 'text-decoration');
      if (tdVaS) {
        return tdVaS.indexOf('line-through') !== -1 ? 'line-through' : null;
      }
      var tdLs = _zwPlainDecl(el, 'text-decoration-line');
      if (tdLs) {
        return tdLs.indexOf('line-through') !== -1 ? 'line-through' : null;
      }
      var tVaS2 = _zwJusTag(el);
      return (tVaS2 === 's' || tVaS2 === 'strike') ? 'line-through' : null;
    }
    if (cmd === 'underline') {
      var tdVaU = _zwPlainDecl(el, 'text-decoration');
      if (tdVaU) {
        return tdVaU.indexOf('underline') !== -1 ? 'underline' : null;
      }
      var tdLu = _zwPlainDecl(el, 'text-decoration-line');
      if (tdLu) {
        return tdLu.indexOf('underline') !== -1 ? 'underline' : null;
      }
      return _zwJusTag(el) === 'u' ? 'underline' : null;
    }
    if (cmd === 'bold') {
      var fwVaB = _zwPlainDecl(el, 'font-weight');
      if (fwVaB) return String(fwVaB);
      var tVaB = _zwJusTag(el);
      return (tVaB === 'b' || tVaB === 'strong') ? 'bold' : null;
    }
    if (cmd === 'italic') {
      var fsVaI = _zwPlainDecl(el, 'font-style');
      if (fsVaI) return String(fsVaI);
      var tVaI = _zwJusTag(el);
      return (tVaI === 'i' || tVaI === 'em') ? 'italic' : null;
    }
    var prop = cmd === 'fontsize' ? 'font-size'
      : (cmd === 'fontname' ? 'font-family'
        : (cmd === 'forecolor' ? 'color' : 'background-color'));
    var sv = _zwPlainDecl(el, prop);
    if (sv) return String(sv);
    if (_zwJusTag(el) === 'font') {
      if (cmd === 'forecolor' && el.hasAttribute && el.hasAttribute('color')) {
        // ③'-a：font color 属性按解析数值形出（legacy attr 解析语义——
        // forecolor.js :425 `<font color=blue>` + forecolor #0000FF 录形合并，
        // keyword 原文不参与指定形比较）。归一失败（junk 值）回落原文。
        var attrColor = el.getAttribute('color');
        return _zwVaNormalizeCssColor(attrColor) || String(attrColor);
      }
      if (cmd === 'fontname' && el.hasAttribute && el.hasAttribute('face')) {
        return el.getAttribute('face');
      }
      if (cmd === 'fontsize' && el.hasAttribute && el.hasAttribute('size')) {
        var sizeRaw = String(el.getAttribute('size'));
        var kwMap = { 1: 'x-small', 2: 'small', 3: 'medium', 4: 'large',
          5: 'x-large', 6: 'xx-large', 7: 'xxx-large' };
        // 相对字号臂（HTML legacy 相对映射）："+n"/"-n" 以父级计算字号档为基
        // ±n（16px 基档 3 → "+1" 档 4 = large；语料 `<font size=+1>` +
        // fontsize 4 期望零变更——parseInt 直读把 "+1" 折成档 1 误冲突，
        // fix4 首版 peel 误切分实证）。档基 = 父链 plain 计算字号 px 中点标度。
        var relM = /^([+-])([0-9]+)$/.exec(sizeRaw);
        if (relM) {
          var basePx = 16;
          try {
            var pxR = _zwPlainFontSizePx(el.parentNode);
            if (!isNaN(pxR)) basePx = pxR;
          } catch (_eRf) {}
          var baseRank = 3;
          for (var bi = 0; bi < 6; bi++) {
            var next = bi < 5 ? _zwValLegacyBounds[bi + 1] : 48;
            if (basePx < (_zwValLegacyBounds[bi] + next) / 2) {
              baseRank = bi + 1;
              break;
            }
            baseRank = 7;
          }
          var relRank = baseRank
            + (relM[1] === '+' ? 1 : -1) * parseInt(relM[2], 10);
          if (relRank < 1) relRank = 1;
          if (relRank > 7) relRank = 7;
          return kwMap[relRank];
        }
        var size = parseInt(sizeRaw, 10);
        if (isNaN(size)) return null;
        if (size < 1) size = 1;
        if (size > 7) size = 7;
        return kwMap[size];
      }
    }
    return null;
  }

  // cssSizeToLegacy（参考实现 :138）——关键字 → 1-7。
  function _zwVaCssSizeToLegacy(cssVal) {
    return { 'x-small': '1', small: '2', medium: '3', large: '4',
      'x-large': '5', 'xx-large': '6', 'xxx-large': '7' }[cssVal];
  }

  // 绝对长度 → px（CSS 绝对单位 96dpi 口径；'2em' 等相对单位 → null）。
  function _zwVaPxOf(s) {
    var m = /^([0-9]+(\.[0-9]+)?)(cm|mm|in|pt|pc|px)$/.exec(s);
    if (!m) return null;
    var n = parseFloat(m[1]);
    switch (m[3]) {
      case 'cm': return n * 96 / 2.54;
      case 'mm': return n * 96 / 25.4;
      case 'in': return n * 96;
      case 'pt': return n * 96 / 72;
      case 'pc': return n * 16;
      default: return n;
    }
  }

  // getLegacyFontSize 关键字/数字臂（参考实现 :3320 头段）——override 读数
  // 归一（qV 面需 legacy 数字串；'2em' 等不可映射 → null）。
  function _zwVaLegacyValueOf(size) {
    var s = String(size == null ? '' : size);
    var kw = _zwVaCssSizeToLegacy(s);
    if (kw) return kw;
    if (_zwVaNormalizeFontSize(s) !== null) return _zwVaCssSizeToLegacy(_zwVaNormalizeFontSize(s));
    if (/^[0-9]+(\.[0-9]+)?(cm|mm|in|pt|pc|px)$/.test(s)) {
      var px = _zwVaPxOf(s);
      if (px !== null) {
        // 中点映射（同 _zwValLegacyFontSize 标度）。
        var bounds = _zwValLegacyBounds;
        for (var i = 0; i < 6; i++) {
          var next = i < 5 ? bounds[i + 1] : 48;
          if (px < (bounds[i] + next) / 2) return String(i + 1);
        }
        return '7';
      }
    }
    return null;
  }

  // —— style 写入（CSS 化尾：span style 臂）——engine style IDL setProperty
  // 优先；**读核验以 style attr 为权威面**（innerHTML 序列化与 plain gCS 回落
  // 都读 attr——host setProperty 通道对未注册 handle 的自建 wrapper 只写内部
  // 样式面、attr 未落，序列化即丢声明：fontsize wrap span 丢 style 实证）。
  // 核验不过回落 style attr 手工合并（plain/M 形工厂兼容）；清空值从 attr 面
  // 移除该声明。
  function _zwVaSetStyleProp(el, prop, val) {
    var valS = String(val);
    // CSSOM 口径：color 系写入即归一（浏览器 style.setProperty 原生序列化
    // rgb() 形，语料 color:rgb(0, 0, 255) 实证——裸 attr 合并丢归一即 compare
    // 面 color:#0000FF 错形）。归一失败（非色值）保持原串。
    if (prop === 'color' || prop === 'background-color') {
      var norm = _zwVaNormalizeCssColor(valS);
      if (norm) valS = norm;
    }
    try {
      if (el.style && typeof el.style.setProperty === 'function') {
        el.style.setProperty(prop, valS);
      }
    } catch (_eSp) {}
    try {
      // cssom 写回刀（2026-11-11）：读核验基底换 **attr 权威面**。getAttribute
      // 的 style 分支接 CSSOM replay（part04 cssom-1 注，2026-11-11）后，经
      // getAttribute 读基底会看到本函数刚 setProperty 的 pending 值 → cur==valS
      // 跳过 attr 合并 → 声明只剩队列内 SetStyleOnHandle，innerHTML 视图在
      // flush 前读不到 → 序列化丢声明（nested styled-ol outdent 语料 forceValue
      // 载体 span 丢 color 实证，odt :628/:658 P2F）。本函数头注契约不变：attr
      // 为权威——handle 元素走 attr 粒度回调（不索引 SetStyleOnHandle，恒回
      // attr 真值），plain 元素保持原 getAttribute 面。
      var decls = (el.__zwHandle && typeof __zw_get_attr_handle === 'function')
        ? _zwPlainStyleDeclsText(String(__zw_get_attr_handle(el.__zwHandle, 'style') || ''))
        : _zwPlainStyleDecls(el);
      var cur = String(decls[prop] == null ? '' : decls[prop]);
      if (cur !== valS) {
        if (valS === '') delete decls[prop];
        else decls[prop] = valS;
        var parts = [];
        for (var k in decls) {
          if (Object.prototype.hasOwnProperty.call(decls, k)) parts.push(k + ':' + decls[k]);
        }
        // Chrome delta（⑦ 残余，probe-vawrap 实证）：手工合并序列化走
        // `k:v; k2:v2` Chrome CSSOM 形——单声明串 join 无分隔（`font-size:large`
        // 等既有行不变）；多声明形 ③ 并值载体需要 `; ` 分隔（hilitecolor
        // xx-large span 期望 `font-size:xx-large; background-color:…` 语料
        // :367 实证——Chrome style attr 序列化恒 `; ` 分隔，旧 `;` 无空格形
        // 仅单声明行被观测过）。
        var serial = parts.join('; ');
        if (serial) el.setAttribute('style', serial);
        else el.removeAttribute('style');
      }
    } catch (_eSp2) {}
  }

  // DC-2 ①（2026-11-12）：text-decoration 换形壳 attr 直写——丢继承 shorthand
  //（SetTagName 拷贝的原 `text-decoration:` 声明）后直写目标声明（prop 空
  // 串 = 仅丢弃），空串出 attr 整删。Chrome 换形录形 attr 恒单声明
  //（strikethrough.js:492/:537 `style="text-decoration-line:…"`、:497 换 u
  // 剥 decl）——SetStyleProp 叠加路径会并出 `text-decoration:…;
  // text-decoration-line:…` 双声明错形（computed 读面亦被旧 shorthand 短路
  // 致后续等值门误判重裹）。直写使 CSSOM store 按 attr 重放（规范形语义同
  // _zwVaCanonicalizeAncestorStyleAttrs）。
  function _zwVaWriteTdDeclOnly(el, prop, val) {
    try {
      var decls = _zwPlainStyleDecls(el);
      delete decls['text-decoration'];
      if (prop && val) decls[prop] = val;
      else if (prop) delete decls[prop];
      var parts = [];
      for (var k in decls) {
        if (Object.prototype.hasOwnProperty.call(decls, k)) {
          parts.push(k + ':' + decls[k]);
        }
      }
      var serial = parts.join('; ');
      if (serial) el.setAttribute('style', serial);
      else el.removeAttribute('style');
    } catch (_eTdW) {}
  }

  // —— isAllowedChild（参考实现 :1572）——inline contents 白名单 +
  // prohibited paragraph child 名单（spec 照录，含 table 全族——tbody/td/tr 等
  // 缺席使 span 字面父臂对表格元素误 true，forceValue 走 s754 包裹臂产出
  // font 包 tbody 非法形态；canvas/dialog 非 spec 名，移除）。
  // https://w3c.github.io/editing/execCommand.html#prohibited-paragraph-child-names
  var _zwVaInlineContents = ['a', 'abbr', 'b', 'bdi', 'bdo', 'cite', 'code', 'dfn',
    'em', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'i', 'kbd', 'mark', 'p', 'pre', 'q',
    'rp', 'rt', 'ruby', 's', 'samp', 'small', 'span', 'strong', 'sub', 'sup', 'u',
    'var', 'acronym', 'listing', 'strike', 'xmp', 'big', 'blink', 'font', 'marquee',
    'nobr', 'tt'];
  var _zwVaProhibitedParagraphChildren = ['address', 'article', 'aside', 'blockquote',
    'caption', 'center', 'col', 'colgroup', 'dd', 'details', 'dir', 'div', 'dl',
    'dt', 'fieldset', 'figcaption', 'figure', 'footer', 'form', 'h1', 'h2', 'h3',
    'h4', 'h5', 'h6', 'header', 'hgroup', 'hr', 'li', 'listing', 'main', 'menu',
    'nav', 'ol', 'p', 'plaintext', 'pre', 'section', 'summary', 'table', 'tbody',
    'td', 'tfoot', 'th', 'thead', 'tr', 'ul', 'xmp'];

  function _zwVaIsElementWithInlineContents(el) {
    return el && el.nodeType === 1
      && _zwVaInlineContents.indexOf(_zwJusTag(el)) !== -1;
  }

  function _zwVaIsAllowedChild(child, parent_) {
    var blockTab = ['colgroup', 'table', 'tbody', 'tfoot', 'thead', 'tr'];
    if ((blockTab.indexOf(parent_) !== -1
         || _zwJusIsHtmlElement(parent_, blockTab))
        && child && child.nodeType === 3
        && !/^[ \t\n\f\r]*$/.test(String(child.data))) {
      return false;
    }
    var noNonText = ['script', 'style', 'plaintext', 'xmp'];
    if ((noNonText.indexOf(parent_) !== -1
         || _zwJusIsHtmlElement(parent_, noNonText))
        && (!child || child.nodeType !== 3)) {
      return false;
    }
    if (child && (child.nodeType === 9 || child.nodeType === 11 || child.nodeType === 10)) {
      return false;
    }
    var childName = null;
    if (child && child.nodeType === 1 && _zwJusIsHtmlElement(child)) {
      childName = _zwJusTag(child);
    }
    if (childName === null) return true;
    // spec 终检：table 族 + 文档骨架名对任意 parent 恒 false（prohibited∩inline
    // 面只覆盖 inline 父；span 字面父臂经此收口）。参考实现 :1682-1689。
    // https://w3c.github.io/editing/execCommand.html#isallowedchildchild-parent
    if (['body', 'caption', 'col', 'colgroup', 'frame', 'frameset', 'head',
      'html', 'tbody', 'td', 'tfoot', 'th', 'thead', 'tr'].indexOf(childName) !== -1) {
      return false;
    }
    // 字符串 parent（forceValue 'span' 字面调用）无 DOM 祖先链——vendored
    // 尾段 table 的字符串参与面照录（'span'×prohibited paragraph child →
    // false；此前直接进 DOM 爬升遇字符串 nodeType!==1 即 break 恒 true，p 元
    // 素被误 wrap——fontsize 跨 p 案实证）。
    if (typeof parent_ === 'string') {
      if (childName === 'a' && parent_ === 'a') return false;
      if (_zwVaInlineContents.indexOf(parent_) !== -1
          && _zwVaProhibitedParagraphChildren.indexOf(childName) !== -1) {
        return false;
      }
      if (/^h[1-6]$/.test(childName) && /^h[1-6]$/.test(parent_)) return false;
      return true;
    }
    var ancestor = parent_;
    var guard = 0;
    while (ancestor && guard++ < 256) {
      if (ancestor.nodeType !== 1) break;
      if (childName === 'a' && _zwJusIsHtmlElement(ancestor, 'a')) return false;
      if (_zwVaProhibitedParagraphChildren.indexOf(childName) !== -1
          && _zwVaIsElementWithInlineContents(ancestor)) {
        return false;
      }
      if (/^h[1-6]$/.test(childName)
          && _zwJusIsHtmlElement(ancestor)
          && /^h[1-6]$/.test(_zwJusTag(ancestor))) {
        return false;
      }
      ancestor = ancestor.parentNode;
    }
    return true;
  }

  // —— isModifiableElement / isSimpleModifiableElement（参考实现 :1833/:1883）
  // ——style 单属性形态读数经 _zwPlainDecl（style attr 朴素扫描键数）。
  function _zwVaIsModifiable(node) {
    if (!node || node.nodeType !== 1 || !_zwJusIsHtmlElement(node)) return false;
    var tag = _zwJusTag(node);
    var nAttrs = node.attributes && node.attributes.length ? node.attributes.length : 0;
    var hasStyle = node.hasAttribute && node.hasAttribute('style');
    if (['b', 'em', 'i', 's', 'span', 'strike', 'strong', 'sub', 'sup', 'u']
        .indexOf(tag) !== -1) {
      if (nAttrs === 0) return true;
      if (nAttrs === 1 && hasStyle) return true;
    }
    if (tag === 'font' || tag === 'a') {
      var num = nAttrs;
      if (hasStyle) num--;
      if (tag === 'font') {
        if (node.hasAttribute && node.hasAttribute('color')) num--;
        if (node.hasAttribute && node.hasAttribute('face')) num--;
        if (node.hasAttribute && node.hasAttribute('size')) num--;
      }
      if (tag === 'a' && node.hasAttribute && node.hasAttribute('href')) num--;
      if (num === 0) return true;
    }
    return false;
  }

  function _zwVaIsSimpleModifiable(node) {
    if (!node || node.nodeType !== 1 || !_zwJusIsHtmlElement(node)) return false;
    var tag = _zwJusTag(node);
    if (['a', 'b', 'em', 'font', 'i', 's', 'span', 'strike', 'strong', 'sub',
         'sup', 'u'].indexOf(tag) === -1) return false;
    var nAttrs = node.attributes && node.attributes.length ? node.attributes.length : 0;
    if (nAttrs === 0) return true;
    if (nAttrs > 1) return false;
    var hasStyle = node.hasAttribute && node.hasAttribute('style');
    // 恰一属性 = style 且零声明（语料形态；单属性 prop 臂逐属性核对）。
    if (hasStyle) {
      var decls = _zwPlainStyleDecls(node);
      var keys = [];
      for (var k in decls) {
        if (Object.prototype.hasOwnProperty.call(decls, k)) keys.push(k);
      }
      if (keys.length === 0) return true;
      if (keys.length === 1) {
        var p0 = keys[0];
        if ((tag === 'b' || tag === 'strong') && p0 === 'font-weight') return true;
        if ((tag === 'i' || tag === 'em') && p0 === 'font-style') return true;
        if ((tag === 'a' || tag === 'font' || tag === 'span') && p0 !== 'text-decoration') {
          return true;
        }
        if (['a', 'font', 's', 'span', 'strike', 'u'].indexOf(tag) !== -1
            && p0 === 'text-decoration'
            && ['line-through', 'underline', 'overline', 'none']
              .indexOf(String(decls[p0])) !== -1) {
          return true;
        }
      }
      return false;
    }
    if (tag === 'a' && node.hasAttribute && node.hasAttribute('href')) return true;
    if (tag === 'font'
        && (node.hasAttribute('color') || node.hasAttribute('face')
            || node.hasAttribute('size'))) {
      return true;
    }
    return false;
  }

  // —— remove extraneous line breaks（参考实现 :1305-1377）——复用 justify 簇
  // 近似 extraneous 判定（非布局探测；FIXME 同 justify 簇挂账）。
  function _zwVaRemoveExtraneousLineBreaksBefore(node) {
    var ref = node.previousSibling;
    if (!ref) return;
    while (_zwJusHasKids(ref)) ref = ref.lastChild;
    var guard = 0;
    while (_zwJusIsInvisible(ref) && !_zwJusIsExtraneousLineBreak(ref)
           && ref !== node.parentNode && guard++ < 4096) {
      ref = _zwJusPreviousNode(ref);
    }
    if (_zwJusIsEditable(ref) && _zwJusIsExtraneousLineBreak(ref)) {
      if (ref.parentNode) ref.parentNode.removeChild(ref);
    }
  }

  function _zwVaRemoveExtraneousLineBreaksAtTheEndOf(node) {
    var ref = node;
    while (_zwJusHasKids(ref)) ref = ref.lastChild;
    var guard = 0;
    while (_zwJusIsInvisible(ref) && !_zwJusIsExtraneousLineBreak(ref)
           && ref !== node && guard++ < 4096) {
      ref = _zwJusPreviousNode(ref);
    }
    if (_zwJusIsEditable(ref) && _zwJusIsExtraneousLineBreak(ref)) {
      while (_zwJusIsEditable(ref.parentNode) && _zwJusIsInvisible(ref.parentNode)) {
        ref = ref.parentNode;
      }
      if (ref.parentNode) ref.parentNode.removeChild(ref);
    }
  }

  function _zwVaRemoveExtraneousLineBreaksFrom(node) {
    _zwVaRemoveExtraneousLineBreaksBefore(node);
    _zwVaRemoveExtraneousLineBreaksAtTheEndOf(node);
  }

  // —— wrap（参考实现 :1377）——globalRange 簿记换 sel._ranges[0] 单 range
  // 补偿（新 parent 直接插入臂的边界 offset 右移；移动臂经
  // _zwJusMovePreservingRanges 自带补偿）。
  function _zwVaWrap(nodeList, siblingCriteria, newParentInstructions, wrapCommand) {
    if (typeof siblingCriteria === 'undefined') {
      siblingCriteria = function () { return false; };
    }
    if (typeof newParentInstructions === 'undefined') {
      newParentInstructions = function () { return null; };
    }
    var allInvisible = true;
    var i;
    for (i = 0; i < nodeList.length; i++) {
      if (!_zwJusIsInvisible(nodeList[i])) { allInvisible = false; break; }
    }
    var hasBr = false;
    for (i = 0; i < nodeList.length; i++) {
      if (_zwJusIsHtmlElement(nodeList[i], 'br')) { hasBr = true; break; }
    }
    if (allInvisible && !hasBr) return null;
    if (!nodeList[0].parentNode) return null;

    var last = nodeList[nodeList.length - 1];
    if (!_zwJusIsBlockNode(last) && !_zwJusIsHtmlElement(last, 'br')
        && _zwJusIsHtmlElement(last.nextSibling, 'br')) {
      nodeList.push(last.nextSibling);
    }
    // Chrome delta：拉邻条件放宽——vendored 只拉 invisible 兄弟，空白-only 文本
    // 兄弟同样随携带（`<span>foo</span> <span>bar</span>` 合并形语料期望空白并入
    // 单一外层样式化 wrapper：空白先入列 sibCrit 才能沿 nodeList[0].previousSibling
    // 命中已样式化邻接 wrapper 合并；仅随携带不独获值——forceValue 入口空白门）。
    while (nodeList[0].previousSibling
        && (_zwJusIsInvisible(nodeList[0].previousSibling) || _zwVaIsWsText(nodeList[0].previousSibling))) {
      nodeList.unshift(nodeList[0].previousSibling);
    }
    while (nodeList[nodeList.length - 1].nextSibling
        && (_zwJusIsInvisible(nodeList[nodeList.length - 1].nextSibling) || _zwVaIsWsText(nodeList[nodeList.length - 1].nextSibling))) {
      nodeList.push(nodeList[nodeList.length - 1].nextSibling);
    }

    var newParent;
    // ③'-d（tag 模式跨标签 reuse 门，两臂同门）：tag 模式 forecolor 的 font
    // 包裹成员不得灌入相邻 span 指定形壳——next 侧 forecolor.js :742 录形
    // `<font color="#0000ff">[ob]</font><span …>ar</span>`、prev 侧 :382 录形
    // `<span>foo</span><font>[bar]</font><span>baz</span>` 三段独立（灌入形
    // `foobar` 两段并拢为实错）；css 模式不受限（:737 录形并入蓝壳）。
    var crossTagBlock = wrapCommand === 'forecolor' && !_zwVaCssFlag();
    if (_zwJusIsEditable(nodeList[0].previousSibling)
        && siblingCriteria(nodeList[0].previousSibling)
        && !(crossTagBlock
            && _zwJusTag(nodeList[0].previousSibling) !== 'font')) {
      newParent = nodeList[0].previousSibling;
    } else if (_zwJusIsEditable(nodeList[nodeList.length - 1].nextSibling)
        && siblingCriteria(nodeList[nodeList.length - 1].nextSibling)
        && !(crossTagBlock
            && _zwJusTag(nodeList[nodeList.length - 1].nextSibling) !== 'font')) {
      newParent = nodeList[nodeList.length - 1].nextSibling;
    } else {
      newParent = newParentInstructions();
    }
    if (!newParent) return null;

    if (!newParent.parentNode) {
      var insParent = nodeList[0].parentNode;
      var insIdx = _zwJusNodeIndex(newParent);
      // 边界点补偿：宿主选区 boundary 落插入点及之后 → offset +1。
      var selW = globalThis.getSelection ? globalThis.getSelection() : null;
      var rngW = selW && selW._ranges && selW._ranges.length ? selW._ranges[0] : null;
      if (rngW) {
        if (rngW.startContainer === insParent && rngW.startOffset >= insIdx) {
          try { rngW.setStart(rngW.startContainer, rngW.startOffset + 1); } catch (_eW1) {}
        }
        if (rngW.endContainer === insParent && rngW.endOffset >= insIdx) {
          try { rngW.setEnd(rngW.endContainer, rngW.endOffset + 1); } catch (_eW2) {}
        }
      }
      try {
        insParent.insertBefore(newParent, nodeList[0]);
      } catch (_eW3) {
        try { insParent.appendChild(newParent); } catch (_eW4) { return null; }
      }
    }

    var originalParent = nodeList[0].parentNode;
    var inlineKids = function (n) {
      return n && !_zwJusIsBlockNode(n);
    };

    if (_zwJusIsBefore(newParent, nodeList[0])) {
      var visKids = [];
      for (i = 0; i < newParent.childNodes.length; i++) {
        if (_zwJusIsVisible(newParent.childNodes[i])) visKids.push(newParent.childNodes[i]);
      }
      var visMembers = [];
      for (i = 0; i < nodeList.length; i++) {
        if (_zwJusIsVisible(nodeList[i])) visMembers.push(nodeList[i]);
      }
      if (!_zwJusIsBlockNode(newParent)
          && inlineKids(visKids[visKids.length - 1])
          && inlineKids(visMembers[0])
          && !_zwJusIsHtmlElement(newParent.lastChild, 'br')
          // Chrome delta：连接点被空白-only 成员隔断时不插 br（`<span>foo</span>
          // <span>bar</span>` 合并形语料期望无 br——空白已防 run 连字，vendored
          // br 反成多余可见断行）。
          && !_zwVaIsWsText(nodeList[0])
          // Chrome delta（⑦ peel 切分面，2026-11-05）：成员紧邻 newParent（本就
          // 原文连排，peel 切分后中段内容与既有 wrap 直接相接）不插 br——语料
          // `fo[o<font size=2>b]ar</font>` 切分形期望 `o`、`b` 连排单 wrap 无 br。
          && nodeList[0].previousSibling !== newParent) {
        newParent.appendChild(document.createElement('br'));
      }
      for (i = 0; i < nodeList.length; i++) {
        _zwJusMovePreservingRanges(nodeList[i], newParent, -1);
      }
    } else {
      var firstVis = null;
      for (i = 0; i < newParent.childNodes.length; i++) {
        if (_zwJusIsVisible(newParent.childNodes[i])) { firstVis = newParent.childNodes[i]; break; }
      }
      var lastVisMember = null;
      for (i = nodeList.length - 1; i >= 0; i--) {
        if (_zwJusIsVisible(nodeList[i])) { lastVisMember = nodeList[i]; break; }
      }
      if (!_zwJusIsBlockNode(newParent)
          && inlineKids(firstVis)
          && inlineKids(lastVisMember)
          && !_zwJusIsHtmlElement(nodeList[nodeList.length - 1], 'br')
          // Chrome delta：同上——nodeList 侧端点是空白-only 成员时不插 br。
          && !_zwVaIsWsText(nodeList[nodeList.length - 1])
          // Chrome delta（③'-d 接点面）：连接点两侧皆裸文本时不插 br——
          // 文本-文本连排是连续 run（forecolor.js :737/:742 录形 `obar` 无
          // 断行；`o<br>bar` ghost br 为 ⑦ 遗留实错面）。
          && !(firstVis.nodeType === 3 && lastVisMember.nodeType === 3)) {
        newParent.insertBefore(document.createElement('br'), newParent.firstChild);
      }
      for (i = nodeList.length - 1; i >= 0; i--) {
        _zwJusMovePreservingRanges(nodeList[i], newParent, 0);
      }
    }

    if (_zwJusIsEditable(originalParent) && !_zwJusHasKids(originalParent)) {
      if (originalParent.parentNode) {
        originalParent.parentNode.removeChild(originalParent);
      }
    }

    if (_zwJusIsEditable(newParent.nextSibling)
        && siblingCriteria(newParent.nextSibling)) {
      var nextEl = newParent.nextSibling;
      // ③'-d（tag 模式跨标签接续臂）：font 包裹 + 指定形 span 壳只拢 range 侧
      // 首个文本接点，壳本体不动不摘——forecolor.js :742 录形
      // `<font color="#0000ff">[ob]</font><span …>ar</span>`（ar 留蓝壳）。
      // 单文本位移：无 br、无整树吸收。css 模式跨标签维持整树吸收面。
      var crossTagFont = wrapCommand === 'forecolor' && !_zwVaCssFlag()
        && _zwJusTag(nextEl) !== 'font';
      if (crossTagFont) {
        if (newParent.lastChild && newParent.lastChild.nodeType === 3
            && nextEl.firstChild && nextEl.firstChild.nodeType === 3) {
          _zwJusMovePreservingRanges(nextEl.firstChild, newParent, -1);
        }
      } else {
        if (!_zwJusIsBlockNode(newParent)
            && inlineKids(newParent.lastChild)
            && inlineKids(nextEl.firstChild)
            && !_zwJusIsHtmlElement(newParent.lastChild, 'br')
            // Chrome delta：同上——连接点 newParent 侧端点是空白-only 文本时不插 br。
            && !_zwVaIsWsText(newParent.lastChild)
            // Chrome delta（③'-d 接点面）：两侧皆裸文本不插 br（连续 run）。
            && !(newParent.lastChild.nodeType === 3 && nextEl.firstChild.nodeType === 3)) {
          newParent.appendChild(document.createElement('br'));
        }
        var guardM = 0;
        while (_zwJusHasKids(newParent.nextSibling) && guardM++ < 4096) {
          _zwJusMovePreservingRanges(newParent.nextSibling.firstChild, newParent, -1);
        }
        if (newParent.nextSibling.parentNode) {
          newParent.parentNode.removeChild(newParent.nextSibling);
        }
      }
    }

    _zwVaRemoveExtraneousLineBreaksFrom(newParent);
    return newParent;
  }

  // —— clearValue（参考实现 :2476）——五命令子集。
  function _zwVaClearValue(element, command) {
    if (!_zwJusIsEditable(element)) return [];
    if (_zwVaSpecifiedValue(element, command) === null) return [];
    if (_zwVaIsSimpleModifiable(element)) {
      var kids = [];
      var i;
      for (i = 0; i < element.childNodes.length; i++) kids.push(element.childNodes[i]);
      var idx = _zwJusNodeIndex(element);
      for (i = 0; i < kids.length; i++) {
        _zwJusMovePreservingRanges(kids[i], element.parentNode, idx + i);
      }
      if (element.parentNode) element.parentNode.removeChild(element);
      return kids;
    }
    var prop = command === 'fontsize' ? 'font-size'
      : (command === 'fontname' ? 'font-family'
        : (command === 'forecolor' ? 'color'
          // DC-2 P2 apply 面（2026-11-11）：六 format 命令声明清值臂
          //（vendored relevantCssProperty :2476 同构；text-decoration 偏序
          // 删除已于 2026-11-12 接入，见下方 decl 重写臂）。
          : (command === 'bold' ? 'font-weight'
            : (command === 'italic' ? 'font-style'
              : ((command === 'underline' || command === 'strikethrough')
                ? 'text-decoration'
                : (command === 'subscript' || command === 'superscript')
                  ? 'vertical-align' : 'background-color')))));
    // DC-2 ①（2026-11-12）：text-decoration 偏序删除——decl 含命令 token 时
    // 仅剥该 token 重写声明（junk 丢弃、canon 位序 [underline, overline,
    // line-through]；Chrome 录形 underline.js:725-761 junk 族
    // `line-through overline underline` − underline → `overline line-through`、
    // :479-483 `underline line-through` − underline → `line-through` 实证）。
    // 剥空照旧整删（s-underline 全选 `foo[bar]baz` 录形链 :831 null → :2114
    // attributeless 解壳既有绿面；u-lineThru + strike 全选同链）。他 prop
    // 维持整删。
    if (prop === 'text-decoration'
        && (command === 'underline' || command === 'strikethrough')) {
      var remCv = command === 'underline' ? 'underline' : 'line-through';
      var canonCv = ['underline', 'overline', 'line-through'];
      var declCv = '';
      try { declCv = _zwPlainDecl(element, 'text-decoration') || ''; } catch (_eCv1) { declCv = ''; }
      if (declCv) {
        var toksCv = String(declCv).toLowerCase().split(/\s+/);
        var survCv = [];
        for (var tCv = 0; tCv < toksCv.length; tCv++) {
          if (toksCv[tCv] && canonCv.indexOf(toksCv[tCv]) !== -1
              && toksCv[tCv] !== remCv && survCv.indexOf(toksCv[tCv]) === -1) {
            survCv.push(toksCv[tCv]);
          }
        }
        survCv.sort(function (a, b) { return canonCv.indexOf(a) - canonCv.indexOf(b); });
        // 直写 attr 同切刻臂（CSSOM `k: v` 空格形泄漏规避，见该处注）；
        // 剥空走整删（delete + 空 attr 移除），:831 recheck null → :2114
        // attributeless 解壳链不变。
        var declsCv = _zwPlainStyleDecls(element);
        if (survCv.length) declsCv['text-decoration'] = survCv.join(' ');
        else delete declsCv['text-decoration'];
        var partsCv = [];
        for (var kCv in declsCv) {
          if (Object.prototype.hasOwnProperty.call(declsCv, kCv)) {
            partsCv.push(kCv + ':' + declsCv[kCv]);
          }
        }
        var serialCv = partsCv.join('; ');
        if (serialCv) element.setAttribute('style', serialCv);
        else element.removeAttribute('style');
      }
    } else if (_zwPlainDecl(element, prop)) {
      _zwVaSetStyleProp(element, prop, '');
      try {
        var attr = element.getAttribute('style');
        if (attr !== null && String(attr).trim() === '') element.removeAttribute('style');
      } catch (_eCv) {}
    }
    if (_zwJusTag(element) === 'font') {
      if (command === 'forecolor' && element.hasAttribute('color')) {
        element.removeAttribute('color');
      }
      if (command === 'fontname' && element.hasAttribute('face')) {
        element.removeAttribute('face');
      }
      if (command === 'fontsize' && element.hasAttribute('size')) {
        element.removeAttribute('size');
      }
    }
    if (_zwVaSpecifiedValue(element, command) === null) return [];
    return [_zwJusSetTagName(element, 'span')];
  }

  // —— pushDownValues（参考实现 :2586）。
  function _zwVaPushDownValues(node, command, newValue) {
    if (!node.parentNode || node.parentNode.nodeType !== 1) return;
    if (_zwVaLooselyEquivalent(command, _zwValEffectiveValue(node, command), newValue, true)) {
      return;
    }
    var currentAncestor = node.parentNode;
    var ancestorList = [];
    var guard = 0;
    while (_zwJusIsEditable(currentAncestor)
           && currentAncestor.nodeType === 1
           && !_zwVaLooselyEquivalent(command,
                _zwValEffectiveValue(currentAncestor, command), newValue, true)
           && guard++ < 256) {
      ancestorList.push(currentAncestor);
      currentAncestor = currentAncestor.parentNode;
    }
    if (!ancestorList.length) return;
    var propagatedValue = _zwVaSpecifiedValue(
      ancestorList[ancestorList.length - 1], command);
    if (propagatedValue === null && propagatedValue !== newValue) return;
    if (newValue !== null
        && !_zwVaLooselyEquivalent(command,
             _zwValEffectiveValue(ancestorList[ancestorList.length - 1].parentNode, command),
             newValue, true)) {
      return;
    }
    while (ancestorList.length) {
      var currentAncestor2 = ancestorList.pop();
      var spec2 = _zwVaSpecifiedValue(currentAncestor2, command);
      if (spec2 !== null) {
        propagatedValue = spec2;
      }
      // children 快照先于 clearValue（vendored :2652 原文时序）——简单可改性
      // 祖先（纯属性 font 等）被 clearValue 摘壳（子级上移、元素移除）后
      // childNodes 恒空，快照后置 = 携带臂零执行（fs4a/font6 push-down 载体
      // 全丢实证；fix7 前实现时序颠倒）。
      var children = [];
      var i;
      for (i = 0; i < currentAncestor2.childNodes.length; i++) {
        children.push(currentAncestor2.childNodes[i]);
      }
      if (spec2 !== null) {
        _zwVaClearValue(currentAncestor2, command);
      }
      for (i = 0; i < children.length; i++) {
        var child = children[i];
        if (child === node) continue;
        if (child.nodeType === 1
            && _zwVaSpecifiedValue(child, command) !== null
            && !_zwVaEquivalentValues(command, propagatedValue,
                 _zwVaSpecifiedValue(child, command))) {
          continue;
        }
        if (child === ancestorList[ancestorList.length - 1]) continue;
        // Chrome delta（⑨ extraneous br，2026-11-11）：push-down 重裹臂跳过
        // br——vendored forceValue 会把壳外 br 独立重裹（`foo<sup>[bar]<br>
        // </sup>` 解除录形 vendored = `foobar<sup><br></sup>`），Chrome 录形
        // br 不携壳（= `foobar<br>`，superscript 语料 2 变体实证）。直接选中
        // 的 br 仍走主循环 forceValue（multitest `[<br>]` 包裹形不受此门）。
        if (child.nodeType === 1 && _zwJusTag(child) === 'br') continue;
        // Chrome delta（⑧ push-down 载体形，2026-11-05）：flag=false +
        // fontsize + font 属性载体（specified）→ wrap 臂带 raw 源（size attr
        // 原串保留，font6 size="6" ≠ 归一 "7" 实证）；其余走 vendored 归一。
        var carrySrc2 = (!_zwVaCssFlag() && command === 'fontsize'
                         && _zwJusTag(currentAncestor2) === 'font'
                         && spec2 !== null) ? currentAncestor2 : null;
        _zwVaForceValue(child, command, propagatedValue, carrySrc2);
      }
    }
  }

  // —— forceValue（参考实现 :2696）——五命令子集：bold/italic/strike/underline/
  // createlink 新 parent 臂不移植（part06 flat 臂域）。CSS flag false →
  // font color/face/size attr；fontsize xxx-large 恒 font attr（spec 特例）。
  // carrySrc（⑧ 可选）：push-down 携带路径传外段载体元素，font 载体 raw attr
  // 保留形（见 wrap 臂内注释）；主循环/children 递归不传。
  function _zwVaForceValue(node, command, newValue, carrySrc) {
    if (!node.parentNode) return;
    if (newValue === null) return;
    // Chrome delta：空白-only 文本节点不强制赋值（不获独立包裹、不进 children
    // 递归臂）——`<p>foo</p> <p>bar</p>` 跨块、`<p>…<span>bar</span>…</p>` 块内
    // 伴形、`<span>foo</span> <span>bar</span>` 行内伴形语料期望空白恒裸或仅随
    // 邻近结构 wrap 携带（五语料 3 形 × 2 变体实证）。vendored 经 wrap 臂独立
    // 包值；本门置 forceValue 入口统一覆盖主循环/pushDownValues/children 递归
    // 各调用点，pushDownValues 本体维持 vendored 全量。_zwJusIsWhitespaceNode
    // 感知 white-space 属性，pre 格式化空白不受此门。
    if (node.nodeType === 3 && _zwJusIsWhitespaceNode(node)) {
      return;
    }
    // DC-2 ①（2026-11-12）：s/strike decl 壳内 strike ADD 部分选区切分抽裹
    // ——spec(s, strike) 因 inline decl 掩 tag face 恒 null（split 冲突判定
    // 不 peel → 既有面恒内裹不切分），Chrome 录形却切分（strikethrough.js
    // :500-507 `foo<s style="text-decoration: underline">b[a]r</s>baz` +
    // strike = `b</s><span …>a</span><s>r</s>` 分解三段形）。门：strike ADD
    // 且命令 token = 壳 tag face（line-through）、壳带不含 line-through 的
    // inline decl；u 壳对位形（:564-571）恒内裹不入本门。中段 css-true 落
    // longhand span(text-decoration-line: canon(decl)∪line-through)（:502/
    // :542）；css-false 落 [span(text-decoration-line: 非 u 映射 token)] >
    // [u] > strike 嵌套（:507 u>strike、:547 span(ld:overline)>strike）。
    if (command === 'strikethrough' && newValue === 'line-through'
        && node.nodeType === 3 && node.parentNode
        && _zwVaIsAllowedChild(node, 'span')) {
      var shellSb = null;
      var declSb = '';
      var nSb = node.parentNode;
      var gSb = 0;
      while (nSb && gSb++ < 256) {
        if (nSb.nodeType !== 1 || !_zwJusIsEditable(nSb)) break;
        if (_zwJusIsEditingHost(nSb) || _zwJusIsBlockNode(nSb)) break;
        var tgSb = _zwJusTag(nSb);
        if (tgSb === 's' || tgSb === 'strike') {
          var dSb = '';
          try { dSb = _zwPlainDecl(nSb, 'text-decoration') || ''; } catch (_eVaSb) { dSb = ''; }
          if (dSb && String(dSb).toLowerCase().indexOf('line-through') === -1) {
            shellSb = nSb;
            declSb = dSb;
          }
          break;
        }
        nSb = nSb.parentNode;
      }
      if (shellSb) {
        _zwVaPeelOne(shellSb, true, command);
        // end 侧走既有裸移（_zwVaCarrierResidualStyling 对 u/s 命令恒 false，
        // 残余 el 保后段 decl 原样入 PeelCarry；本臂 ADD 面重物化门不入）。
        _zwVaPeelOne(shellSb, false, command);
        if (!node.parentNode) return;
        var canonSb = ['underline', 'overline', 'line-through'];
        var toksSb = String(declSb).toLowerCase().split(/\s+/);
        var survSb = [];
        for (var tSb = 0; tSb < toksSb.length; tSb++) {
          if (toksSb[tSb] && canonSb.indexOf(toksSb[tSb]) !== -1
              && toksSb[tSb] !== 'line-through'
              && survSb.indexOf(toksSb[tSb]) === -1) {
            survSb.push(toksSb[tSb]);
          }
        }
        survSb.sort(function (a, b) { return canonSb.indexOf(a) - canonSb.indexOf(b); });
        var wrapSb = null;
        var innerSb = null;
        if (_zwVaCssFlag()) {
          var mergedSb = survSb.slice();
          if (mergedSb.indexOf('line-through') === -1) mergedSb.push('line-through');
          mergedSb.sort(function (a, b) { return canonSb.indexOf(a) - canonSb.indexOf(b); });
          wrapSb = document.createElement('span');
          _zwVaSetStyleProp(wrapSb, 'text-decoration-line', mergedSb.join(' '));
          innerSb = wrapSb;
        } else {
          var nonMapSb = [];
          var uSb = null;
          for (var nSb2 = 0; nSb2 < survSb.length; nSb2++) {
            if (survSb[nSb2] !== 'underline') nonMapSb.push(survSb[nSb2]);
          }
          if (nonMapSb.length) {
            wrapSb = document.createElement('span');
            _zwVaSetStyleProp(wrapSb, 'text-decoration-line', nonMapSb.join(' '));
            innerSb = wrapSb;
          }
          if (survSb.indexOf('underline') !== -1) {
            uSb = document.createElement('u');
            if (innerSb) {
              innerSb.appendChild(uSb);
            } else {
              wrapSb = uSb;
            }
            innerSb = uSb;
          }
          var stSb = document.createElement('strike');
          if (innerSb) {
            innerSb.appendChild(stSb);
          } else {
            wrapSb = stSb;
          }
          innerSb = stSb;
        }
        var parSb = node.parentNode;
        var idxSb = _zwJusNodeIndex(node);
        _zwVaShiftEndpointsForInsert(parSb, idxSb);
        parSb.insertBefore(wrapSb, node);
        _zwJusMovePreservingRanges(node, innerSb, 0);
        return;
      }
    }
    var i;
    if (_zwVaIsAllowedChild(node, 'span')) {
      var sibCrit = function (n) {
        // DC-2 P2（2026-11-11）Chrome delta：css flag true 下 wrap 不并入 tag
        // 形兄弟——`<b>foo</b>[bar]<b>baz</b>` swcss(true) 录形 = 三段并列
        // `<b>foo</b><span style=font-weight:bold>bar</span><b>baz</b>`，
        // vendored sibCrit 无 flag 门（isSimpleModifiable 命中裸 b）→ 并入
        // 前后 b 得 `<b>foobarbaz</b>` 误形（multitest css-true 族 ~104 失败
        // 实证；Chrome 引擎不按 vendored 算法实现，录形为 oracle）。css 模式
        // 仅 span（style 形）兄弟可并（`<span style=…>foo</span>[bar]<span
        // style=…>baz</span>` css-true 录形仍合并单 span）；tag 模式不变
        //（swcss(false) 录形 `<b>foobarbaz</b>`）。sub/sup 例外：Chrome
        // css-true 亦合并单元素壳（`<span>[foo</span> <span>bar]</span>` sub
        // 期望 `<sub><span>foo</span> <span>bar</span></sub>` 单壳——subscript/
        // superscript 语料 2 变体实证，2026-11-11）。后续
        // _zwVaCrossTagContinuation 复用本 crit，css 模式同门语义一致。
        var nT = _zwJusTag(n);
        // DC-2 ②（2026-11-12）：bold 并值候选排除 strong——strong 邻接录形
        // 恒独立 fresh b 不并入（data/bold.js:270-282 `foo<strong>bar</strong>
        // [baz]`/`<strong>foo</strong>[bar]<strong>baz</strong>` 两翼 fresh b、
        // :285-287 `<b>foo</b>[bar]<strong>baz</strong>` 仅并入前 b、:298
        // strong|b css-false 仅并入后 b；vendored isSimpleModifiable 同判
        // strong/b 致误并）。b 候选不受此门（:287/:300 录形并入 b）。
        if (command === 'bold' && nT === 'strong') return false;
        if (_zwVaCssFlag() && nT !== 'span' && nT !== 'sub' && nT !== 'sup') {
          return false;
        }
        return _zwVaIsSimpleModifiable(n)
          && _zwVaEquivalentValues(command, _zwVaSpecifiedValue(n, command), newValue)
          && _zwVaLooselyEquivalent(command, _zwValEffectiveValue(n, command), newValue, true);
      };
      _zwVaWrap([node], sibCrit, function () { return null; }, command);
    }
    // Chrome delta（⑦ br 包裹面，2026-11-05）：invisible br（extraneous 形，
    // 如 `<p>[foo<p><br><p>bar]` 中块 `<p><br></p>`）不作 invisible 早退——
    // Chrome 对选区内 br 仍落独立值包裹，但仅 color 家族 + fontname：
    // forecolor compare 期望 `<p><span style=color><br></span></p>` 形、
    // fontname 同族包裹；**fontsize 期望 br 裸**（`<p><br></p>` 恒无
    // font/span 包裹，fontsize 语料 2 变体实证——首版无命令门 2 P→F 回归）。
    // 上游 wrap 合并臂已在上方尝试：br 有可并兄弟走合并（`foo<br>bar` 形，
    // 可见 br 本就不入本臂）；孤立 extraneous br 落尾部外裹臂。等值早退
    // （:751/:771）不受影响——已获同值继承的 br 仍零变更。其余 invisible
    // 节点维持 spec 早退（force-the-value :2753）。五语料 + multitest 全部
    // br compare fixture 改前零通过面（无保护面）。
    // https://w3c.github.io/editing/execCommand.html#force-the-value
    if (_zwJusIsInvisible(node)
        && !(node.nodeType === 1 && _zwJusIsHtmlElement(node, 'br')
             && command !== 'fontsize')) {
      return;
    }
    // F6b（2026-11-09 诊断 D3 定谳档）fontsize 等值门 script 缩放口径：
    // 引擎 gCS 盲 UA 元素规则（sub/sup 壳级读缺省 16px），medium(16px) ≡
    // 盲读即整门跳过 → C3/C4 值重放被吞（期望壳外 font size3）。**仅壳级**
    // （node 即 sub/sup 元素）按 0.83em 缩放比较（16→13.28 ≠ medium 桶3 →
    // 壳重放放行）；段文本级保持盲读——壳外裹后 text 已见壳值（font3 attr
    // 16px ≡ medium）内层自然溶解（D3b 双包实证：全 in-script 缩放会把
    // text 也裹一层）。C1/C2 等桶溶解在 restore 门已消化不达此处。非
    // fontsize 零差（F6b 域收敛——其余命令等值面维持原口径）。
    var effVaF6b = _zwValEffectiveValue(node, command);
    if (command === 'fontsize'
        && node.nodeType === 1
        && (_zwJusTag(node) === 'sub' || _zwJusTag(node) === 'sup')) {
      var pxVa = parseFloat(effVaF6b);
      if (!isNaN(pxVa)) effVaF6b = (pxVa * 0.83) + 'px';
    }
    if (_zwVaLooselyEquivalent(command, effVaF6b, newValue, true)) {
      return;
    }
    if (!_zwVaIsAllowedChild(node, 'span')) {
      var children = [];
      for (i = 0; i < node.childNodes.length; i++) {
        var kid = node.childNodes[i];
        if (kid.nodeType === 1) {
          var specKid = _zwVaSpecifiedValue(kid, command);
          if (specKid !== null && !_zwVaEquivalentValues(command, newValue, specKid)) {
            continue;
          }
        }
        children.push(kid);
      }
      for (i = 0; i < children.length; i++) {
        _zwVaForceValue(children[i], command, newValue);
      }
      return;
    }
    // 同上口径第二道门（下潜臂后）：仅壳级缩放、段文本级盲读（首道门
    // 注释——双包消解同源：text 盲读见壳值即等值溶解，不重裹内层）。
    var effVaF6b2 = _zwValEffectiveValue(node, command);
    if (command === 'fontsize'
        && node.nodeType === 1
        && (_zwJusTag(node) === 'sub' || _zwJusTag(node) === 'sup')) {
      var pxVa2 = parseFloat(effVaF6b2);
      if (!isNaN(pxVa2)) effVaF6b2 = (pxVa2 * 0.83) + 'px';
    }
    if (_zwVaLooselyEquivalent(command, effVaF6b2, newValue, true)) {
      return;
    }
    var property = command === 'fontsize' ? 'font-size'
      : (command === 'fontname' ? 'font-family'
        : (command === 'forecolor' ? 'color'
          // DC-2 P2 apply 面（2026-11-11）：六 format 命令 relevantCssProperty
          //（vendored :2857 同构）——CSS flag true 的 span 样式形 + 尾臂冗余
          // 声明判定消费。
          : (command === 'bold' ? 'font-weight'
            : (command === 'italic' ? 'font-style'
              : ((command === 'underline' || command === 'strikethrough')
                ? 'text-decoration'
                : (command === 'subscript' || command === 'superscript')
                  ? 'vertical-align' : 'background-color')))));
    // Chrome delta（DC-2 ① 载体原位面，2026-11-11）：u/s 命令对全含选区的
    // **decoration 载体 span**（inline decl 或作者规则面）的落点面——⑦ 下潜
    // 臂的 text-decoration 例外（strike/underline 语料 class 载体 + styled
    // span 录形；data 注释原文 "Should update text-decoration declaration in
    // the <span> which is a container of the range"）。裸 span（无任何
    // decoration 面）不入本门——跨 span 裸壳组录形 = 单外裹（`<span
    // [foo</span> <span>bar]</span>` css-true/css-false 均整组外裹单壳，
    // fix2 既有行为，本刀零漂移）。effectively contained 元素子级恒全含，
    // forceValue(span) 只在壳级全选形到达，局部选区（b[a]r 族）不入本门：
    //   - css flag true：原位并 style——新值与 inline 既有 line token 合并后按
    //     canonical 位序输出（[underline,overline,line-through] Chrome CSSOM
    //     序；strike 语料 underline 载体 + strike css-true 录形
    //     `underline line-through`，与 underline 载体同序，新值不前置；
    //     blink/color/style 长手丢弃。class 载体补 style attr，作者 token
    //     不入 inline 面——class=underline + strike css-true 录形仅
    //     line-through，余值仍由类规则级联供给）。
    //   - css flag false：inline decl 面 → 跳过 ⑦ 下潜、落尾臂外裹（styled
    //     span 录形 `<u><span style=…>…</span></u>`）；作者面（类载体）→
    //     下潜内容侧（`<span class=…><u>…</u></span>` 录形）。
    //   门内壳形 DC-2 ⑤ 扩（2026-11-12）：span 之外 ins/del 元素载体入同门
    //   ——ins/del 录形保壳短手原位并（`foo<ins>[bar]</ins>baz` +strike
    //   css-true = `<ins style="text-decoration:underline line-through">`，
    //   strike.js:389；`foo<del>[bar]</del>baz` +underline css-true =
    //   `<del style="text-decoration:underline line-through">`，
    //   underline.js:419——内容语义壳不溶解，与 u/s/strike 溶解臂分族）。
    //   ins/del bare 壳 tag face（underline/line-through）入 tdUsFace；css-
    //   false 走下潜内容侧（`<ins><strike>[bar]</strike></ins>`、
    //   `<del><u>[bar]</u></del>` strike.js:389/underline.js:424 实证）。
    //   bare ins + underline 无变化录形由 F6b effective 等值门早退、del +
    //   strike 零变化由 removal 不解 del 保（均不入本门）。
    //   门内仅 span/ins/del 形壳（u/s styled 壳维持 ⑦ 既有路径）；子级过滤臂
    //   同 ⑦ vendored step 原文。
    if (node.nodeType === 1) {
      var usSpan = (command === 'underline' || command === 'strikethrough')
        && (_zwJusTag(node) === 'span' || _zwJusTag(node) === 'ins'
          || _zwJusTag(node) === 'del')
        && _zwVaIsAllowedChild(node, 'span');
      var tdUsInline = '';
      var tdUsAuthor = '';
      var tdUsTag = '';
      if (usSpan) {
        try { tdUsInline = _zwPlainDecl(node, 'text-decoration') || ''; } catch (_eUcF1) { tdUsInline = ''; }
        if (!tdUsInline) {
          try { tdUsAuthor = _zwPlainAuthorDecl(node, 'text-decoration') || ''; } catch (_eUcF2) { tdUsAuthor = ''; }
        }
        if (!tdUsInline && !tdUsAuthor) {
          if (_zwJusTag(node) === 'ins') tdUsTag = 'underline';
          else if (_zwJusTag(node) === 'del') tdUsTag = 'line-through';
        }
      }
      var tdUsFace = tdUsInline || tdUsAuthor || tdUsTag;
      if (usSpan && tdUsFace && _zwVaCssFlag()) {
        var canonUs = ['underline', 'overline', 'line-through'];
        var mergeUs = [newValue];
        if (tdUsTag && mergeUs.indexOf(tdUsTag) === -1) mergeUs.push(tdUsTag);
        if (tdUsInline) {
          var toksUs = String(tdUsInline).toLowerCase().split(/\s+/);
          for (var ti = 0; ti < toksUs.length; ti++) {
            if (toksUs[ti] && canonUs.indexOf(toksUs[ti]) !== -1
                && mergeUs.indexOf(toksUs[ti]) === -1) {
              mergeUs.push(toksUs[ti]);
            }
          }
        }
        if (mergeUs.length > 1) {
          mergeUs.sort(function (a, b) { return canonUs.indexOf(a) - canonUs.indexOf(b); });
        }
        _zwVaSetStyleProp(node, 'text-decoration', mergeUs.join(' '));
        return;
      }
      if (usSpan && tdUsFace && !_zwVaCssFlag()) {
        if (tdUsInline) {
          // inline decl 面（styled）：跳过 ⑦ 下潜，落尾臂外裹。
        } else {
          // 作者面载体（类壳）：下潜内容侧（子级过滤臂同 ⑦ vendored step 原文）。
          var kidsUs = [];
          for (i = 0; i < node.childNodes.length; i++) {
            var kidUs = node.childNodes[i];
            if (kidUs.nodeType === 1) {
              var specKidUs = _zwVaSpecifiedValue(kidUs, command);
              if (specKidUs !== null && !_zwVaEquivalentValues(command, newValue, specKidUs)) {
                continue;
              }
            }
            kidsUs.push(kidUs);
          }
          for (i = 0; i < kidsUs.length; i++) {
            _zwVaForceValue(kidsUs[i], command, newValue);
          }
          return;
        }
      }
      // DC-2 ⑤（2026-11-12）：css-true bare 元素载体折叠溶解——u/s/strike
      //   全含壳（tag face，无 inline/author decl）自身溶解、tag 值并入新裹
      //   inline 面短手（Chrome 录形 `foo<u>[bar]</u>baz` +strike css-true =
      //   `<span style="text-decoration:underline line-through">[bar]</span>baz`，
      //   data/strikethrough.js:182、underline.js:241 `foo<s>[bar]</s>baz` +
      //   underline 同形；ins/del 不走本门——内容语义壳保形原位并，上方
      //   span/ins/del 门已截）。decl 面载体（inline/author，长手折叠/溶解
      //   族）不入本门。端点序 insertBefore→搬移→removeChild：par 层 insert
      //   的 +1 陈旧恰被同位 removeChild 抵消（(par,k≤idx) 与 (par,k≥idx+1)
      //   净移 0），壳内端点由 _zwJusMovePreservingRanges 随子搬移。
      if (_zwVaCssFlag() && node.nodeType === 1
          && (command === 'underline' || command === 'strikethrough')) {
        var tgDs = _zwJusTag(node);
        var faceDs = tgDs === 'u' ? 'underline'
          : (tgDs === 's' || tgDs === 'strike') ? 'line-through' : '';
        if (faceDs) {
          var declDs = '';
          try { declDs = _zwPlainDecl(node, 'text-decoration') || ''; } catch (_eDs1) { declDs = ''; }
          if (!declDs) {
            try { declDs = _zwPlainAuthorDecl(node, 'text-decoration') || ''; } catch (_eDs2) { declDs = ''; }
          }
          if (!declDs) {
            var canonDs = ['underline', 'overline', 'line-through'];
            var mergeDs = [newValue];
            if (mergeDs.indexOf(faceDs) === -1) mergeDs.push(faceDs);
            mergeDs.sort(function (a, b) { return canonDs.indexOf(a) - canonDs.indexOf(b); });
            var spanDs = document.createElement('span');
            _zwVaSetStyleProp(spanDs, 'text-decoration', mergeDs.join(' '));
            if (node.parentNode) {
              node.parentNode.insertBefore(spanDs, node);
              var kidsDs = [];
              for (i = 0; i < node.childNodes.length; i++) {
                kidsDs.push(node.childNodes[i]);
              }
              for (i = 0; i < kidsDs.length; i++) {
                _zwJusMovePreservingRanges(kidsDs[i], spanDs, spanDs.childNodes.length);
              }
              if (node.parentNode) node.parentNode.removeChild(node);
            }
            return;
          }
        }
      }
      // DC-2 ①（2026-11-12）：decl 面壳（inline text-decoration 替换 tag
      // face）ADD 分族——⑦ residualDecls 下潜门对 decl 壳产生内裹，Chrome
      // 录形按壳 tag 与命令 token 关系分族（data/underline.js:514-577、
      // data/strikethrough.js:474-563 录形直译）。仅 decl 面入本门（decl 空
      // = tag face 载体走 ⑤ 折叠臂/⑦ 下潜既有绿面）；junk token 丢弃、canon
      // 位序 [underline, overline, line-through]：
      //   - u 壳 + underline（命令 token = 壳 tag face）：外裹保壳——css-true
      //     span(text-decoration:newValue)（underline.js:516/:560）、css-false
      //     u（:521/:565）；
      //   - u 壳 + strike：css-true 溶解换形 shorthand span(decl∪line-through)
      //     （strikethrough.js:556）；css-false 外裹 strike 保壳（:561）；
      //   - s/strike 壳 + underline：css-true 溶解换形 shorthand span(decl∪
      //     underline)（underline.js:540）；css-false 外裹 u 保壳（:545）；
      //   - s/strike 壳 + strike（命令 token = 壳 tag face）：css-true 溶解
      //     换形 **longhand** span(text-decoration-line: decl∪line-through)
      //     （strikethrough.js:492/:532）；css-false 壳转形 + 内裹 strike——
      //     surviving=[underline] 换 u 剥 decl（:497）、余者换 span
      //     (text-decoration-line: surviving)（:537）。
      if ((command === 'underline' || command === 'strikethrough')
          && node.nodeType === 1) {
        var tgSa = _zwJusTag(node);
        var declSa = '';
        try {
          declSa = _zwPlainDecl(node, 'text-decoration') || '';
        } catch (_eVaSa0) { declSa = ''; }
        if ((tgSa === 'u' || tgSa === 's' || tgSa === 'strike') && declSa) {
          var canonSa = ['underline', 'overline', 'line-through'];
          var remSa = command === 'underline' ? 'underline' : 'line-through';
          var toksSa = String(declSa).toLowerCase().split(/\s+/);
          var survSa = [];
          for (var tSa = 0; tSa < toksSa.length; tSa++) {
            if (toksSa[tSa] && canonSa.indexOf(toksSa[tSa]) !== -1
                && survSa.indexOf(toksSa[tSa]) === -1) {
              survSa.push(toksSa[tSa]);
            }
          }
          survSa.sort(function (a, b) { return canonSa.indexOf(a) - canonSa.indexOf(b); });
          if (survSa.length) {
            var mergedSa = survSa.slice();
            if (mergedSa.indexOf(remSa) === -1) mergedSa.push(remSa);
            mergedSa.sort(function (a, b) { return canonSa.indexOf(a) - canonSa.indexOf(b); });
            if (_zwVaCssFlag()) {
              if (tgSa === 'u' && command === 'underline') {
                // 外裹 span(newValue)，壳原样（underline.js:516/:560）。
                var spanSa = document.createElement('span');
                _zwVaSetStyleProp(spanSa, 'text-decoration', newValue);
                node.parentNode.insertBefore(spanSa, node);
                _zwJusMovePreservingRanges(node, spanSa, 0);
                return;
              }
              if (tgSa === 'u' || command === 'underline') {
                // 溶解换形 shorthand span（strikethrough.js:556 /
                // underline.js:540）。
                var spanSa2 = _zwJusSetTagName(node, 'span');
                _zwVaWriteTdDeclOnly(spanSa2, 'text-decoration', mergedSa.join(' '));
                return;
              }
              // s/strike + strike css-true：溶解换形 longhand（:492/:532）。
              var spanSa3 = _zwJusSetTagName(node, 'span');
              _zwVaWriteTdDeclOnly(spanSa3, 'text-decoration-line', mergedSa.join(' '));
              return;
            }
            if (command === 'underline') {
              // css-false 外裹 u 保壳（underline.js:521/:545/:565）。
              var uSa = document.createElement('u');
              node.parentNode.insertBefore(uSa, node);
              _zwJusMovePreservingRanges(node, uSa, 0);
              return;
            }
            if (tgSa === 'u') {
              // u + strike css-false：外裹 strike 保壳（strikethrough.js:561）。
              var stSa = document.createElement('strike');
              node.parentNode.insertBefore(stSa, node);
              _zwJusMovePreservingRanges(node, stSa, 0);
              return;
            }
            // s/strike + strike css-false：壳转形 + 内裹 strike（:497/:537）。
            var carriedSa;
            if (survSa.length === 1 && survSa[0] === 'underline') {
              carriedSa = _zwJusSetTagName(node, 'u');
              _zwVaWriteTdDeclOnly(carriedSa, '', '');
            } else {
              carriedSa = _zwJusSetTagName(node, 'span');
              _zwVaWriteTdDeclOnly(carriedSa, 'text-decoration-line', survSa.join(' '));
            }
            var stSa2 = document.createElement('strike');
            carriedSa.insertBefore(stSa2, carriedSa.firstChild);
            var kidsSa = [].slice.call(carriedSa.childNodes);
            for (var kSa = 1; kSa < kidsSa.length; kSa++) {
              _zwJusMovePreservingRanges(kidsSa[kSa], stSa2, stSa2.childNodes.length);
            }
            return;
          }
        }
      }
      if (!usSpan || !tdUsFace) {
        var ownFacePre = '';
        var parFacePre = '';
        try { ownFacePre = __zwPlainComputed(node, property); } catch (_eVaPre) {}
        try {
          if (node.parentNode) parFacePre = __zwPlainComputed(node.parentNode, property);
        } catch (_eVaPar) {}
        // DC-2 ⑤（2026-11-12）：⑦ own face 补 tag face 臂——plain computed
        //   走查无 text-decoration 臂（part01c __zwPlainComputed 无该属性臂），
        //   bare u/s/strike 壳 ownFacePre 恒空致整壳外裹（css-false nest-
        //   inside 录形 `foo<u><strike>[bar]</strike></u>baz` = strike.js:182、
        //   underline.js:246 `foo<s><u>[bar]</u></s>baz` 同形——应下潜子级而
        //   非外绕；ins/del 由上方保壳门下潜截走，不达此处）。仅 u/s 命令面
        //   补臂，其余命令路径零影响。
        if ((command === 'underline' || command === 'strikethrough') && ownFacePre === '') {
          var tg7 = _zwJusTag(node);
          ownFacePre = tg7 === 'u' ? 'underline'
            : (tg7 === 's' || tg7 === 'strike') ? 'line-through' : '';
        }
        var ownGenuine = ownFacePre !== '' && ownFacePre !== parFacePre;
        var residualDecls = false;
        try {
          var declsPre = _zwPlainStyleDecls(node);
          for (var kr in declsPre) {
            if (Object.prototype.hasOwnProperty.call(declsPre, kr)) {
              residualDecls = true;
              break;
            }
          }
        } catch (_eRd) {}
        if (ownGenuine || residualDecls) {
          var kids3 = [];
          for (i = 0; i < node.childNodes.length; i++) {
            var kid3 = node.childNodes[i];
            if (kid3.nodeType === 1) {
              var specKid3 = _zwVaSpecifiedValue(kid3, command);
              if (specKid3 !== null && !_zwVaEquivalentValues(command, newValue, specKid3)) {
                continue;
              }
            }
            kids3.push(kid3);
          }
          for (i = 0; i < kids3.length; i++) {
            _zwVaForceValue(kids3[i], command, newValue);
          }
          return;
        }
      }
    }
    var newParent = null;
    // fontsize xxx-large 特例（spec :2696 原文 (!cssStylingFlag || newValue ==
    // "xxx-large")；vendored 同）——CSS flag true 也走 font size 属性形。
    if (!_zwVaCssFlag() || (command === 'fontsize' && newValue === 'xxx-large')) {
      // 关键字成员门（vendored :2860 原文第二条件 newValue ∈ 7 关键字）——
      // xx-small 等 CSS-only 关键字/2em 等相对值不可映射，落 span+style 臂；
      // 无门时 `<font size="undefined">` 错形（语料 p xx-small / p 2em fs3
      // flag=false 实证）。
      if (command === 'fontsize'
          && ['x-small', 'small', 'medium', 'large', 'x-large', 'xx-large', 'xxx-large']
               .indexOf(newValue) !== -1) {
        newParent = document.createElement('font');
        // Chrome delta（⑧ push-down 载体形，2026-11-05）：外段 raw attr 保留——
        // font 载体 size="6" 语料录形 ≠ cssSizeToLegacy 归一 "7"（font6 fs3
        // flag=false L604 实证；vendored 归一仅在无 raw 源时走）。仅 push-down
        // 携带路径传 carrySrc（主循环包裹不带源，恒归一）。
        if (carrySrc && carrySrc.getAttribute && carrySrc.getAttribute('size') !== null) {
          newParent.setAttribute('size', carrySrc.getAttribute('size'));
        } else {
          newParent.setAttribute('size', _zwVaCssSizeToLegacy(newValue));
        }
      }
      if (command === 'forecolor' && _zwVaParseSimpleColor(newValue)) {
        newParent = document.createElement('font');
        newParent.setAttribute('color', _zwVaParseSimpleColor(newValue));
      }
      if (command === 'fontname') {
        newParent = document.createElement('font');
        newParent.setAttribute('face', newValue);
      }
      // DC-2 P2 apply 面（2026-11-11）：六 format 命令元素臂（vendored
      // :2821-2853 原文——flag=false 恒标签形；值门同原文）。fresh 标签的
      // effective 读数（UA 表 b→700/strike→line-through）使 :1086 尾臂不加
      // 冗余 style 声明——录形恒裸 <b>/<i>/<u>/<strike>。Chrome delta
      // （DC-2 ③，2026-11-12）：strikethrough 新裹标签 Chrome 录形用
      // <strike>（语料 foo[bar]baz css-false 期望 foo<strike>[bar]</strike>baz
      // 实证；上游 vendored 原文 s 但 WPT 期望以 Chrome 录形为准）——原 <s>
      // 形 strike 文件 50 例 innerHTML 全簇红根因。
      if (command === 'bold' && (newValue === 'bold' || newValue === '700')) {
        newParent = document.createElement('b');
      }
      if (command === 'italic' && newValue === 'italic') {
        newParent = document.createElement('i');
      }
      if (command === 'strikethrough' && newValue === 'line-through') {
        newParent = document.createElement('strike');
      }
      if (command === 'underline' && newValue === 'underline') {
        newParent = document.createElement('u');
      }
    }
    // sub/sup 恒元素形无 flag 门（vendored :2860-2877 原文位置——Chrome
    // styleWithCSS true 亦落 <sub>/<sup>，无 span 样式形）。
    if ((command === 'subscript' || command === 'superscript')
        && newValue === 'subscript') {
      newParent = document.createElement('sub');
    }
    if ((command === 'subscript' || command === 'superscript')
        && newValue === 'superscript') {
      newParent = document.createElement('sup');
    }
    if (!newParent) {
      newParent = document.createElement('span');
    }
    // 插入点选区边界补偿（真浏览器 insertBefore 原生调整的等价面，_zwVaWrap
    // 同款）：新 parent 插在 node 前（idx = node 索引），落点 ≥ idx 的宿主选
    // 区端点 +1。缺补则端点漂到 newParent 与 node 之间，随后的 movePreserving
    // 端点随动把它带进 (newParent, 0)（语料 qV-after 读缺省黑实证——Chrome
    // 应落 (newParent, 1) bar 之后）。
    var insIdxFv = _zwJusNodeIndex(node);
    var selFv = globalThis.getSelection ? globalThis.getSelection() : null;
    var rngFv = selFv && selFv._ranges && selFv._ranges.length ? selFv._ranges[0] : null;
    if (rngFv) {
      if (rngFv.startContainer === node.parentNode && rngFv.startOffset >= insIdxFv) {
        try { rngFv.setStart(rngFv.startContainer, rngFv.startOffset + 1); } catch (_eFv1) {}
      }
      if (rngFv.endContainer === node.parentNode && rngFv.endOffset >= insIdxFv) {
        try { rngFv.setEnd(rngFv.endContainer, rngFv.endOffset + 1); } catch (_eFv2) {}
      }
    }
    var oldParentFv = node.parentNode;
    var insRefFv = node;
    if (node.nodeType === 3 && !node.__zwHandle && !node.__zwSelector
        && oldParentFv && oldParentFv.__zwHandle) {
      // M-wire 裸 ref 参考点退化：裸 text ref（无 handle/印）不能作 insertBefore
      // 参考点——part04.js 落位分发 handle 父支只收 印/handle/textEl/TEXT+TEXT
      // 形，handle 父 + 裸 text ref + 元素 newNode 全支落空 → newParent 壳 M 侧
      // 静默缺失，后续 movePreserving 把文本挪进未接线壳即序列化丢失。语料实证：
      // splitText 产物 ref 带 handle 故 a[b]c 过；残门克隆壳（shim createElement
      // 产物持 handle）内解析原件裸故 [a]bc 失。append（refNode==null 分支零参
      // 考点要求恒接线）与插 node 前同 JS 树，纯接线修复；文本运输仍走原
      // movePreserving（td3 SERIAL 证其 removeChild 恒接线，append 进已接线壳走
      // R265 textEl 恒接线）。门限限定 handle 父：解析父（无 handle）裸 ref 走
      // sel 支 pos-sel 参考位恒接线，append 化曾证伪 41 例（foo[bar…] 中列形
      // fontsize ?1-1000，fix14 复跑同集）；父 __zwSelector 判别无效（克隆与解
      // 析父代理面均不暴露，narrowed 复跑 47F 同集且 teal 复红）；node 末位门亦
      // 不足（全语料行上下文克隆多片段中列形，末位判定不触发）。
      insRefFv = null;
    }
    oldParentFv.insertBefore(newParent, insRefFv);
    // numericFace：fresh font 载体已落 attr 数值形（_zwVaParseSimpleColor），
    // 与 keyword 新值同色——补写 style 声明即冗余（:114/:269 录形 attr-only）。
    if (!_zwVaLooselyEquivalent(command, _zwValEffectiveValue(newParent, command), newValue, true)) {
      _zwVaSetStyleProp(newParent, property, newValue);
    }
    _zwJusMovePreservingRanges(node, newParent, newParent.childNodes.length);
    _zwVaCrossTagContinuation(newParent, command, sibCrit);
  }

  // ③'-d（tag 模式跨标签接续，:742 录形）：forceValue 尾臂裸插入的 font 包裹
  // 与相邻指定形 span 壳的文本-文本接点只拢 range 侧首文本，壳本体不动不摘
  // （`<font color="#0000ff">[ob]</font><span …>ar</span>` 录形）。同标签邻接
  // 不入此臂（wrap 臂 merge 面既有行为）；接续门复用 sibCrit（specified 等值
  // + loosely 等值），非等值壳不拢（防他色文本丢壳）。
  function _zwVaCrossTagContinuation(newParent, wrapCommand, sibCrit) {
    if (wrapCommand !== 'forecolor' || _zwVaCssFlag()) return;
    var nextEl = newParent.nextSibling;
    if (!nextEl || nextEl.nodeType !== 1 || !_zwJusIsEditable(nextEl)) return;
    if (_zwJusTag(nextEl) === 'font') return;
    if (!sibCrit || !sibCrit(nextEl)) return;
    if (newParent.lastChild && newParent.lastChild.nodeType === 3
        && nextEl.firstChild && nextEl.firstChild.nodeType === 3) {
      _zwJusMovePreservingRanges(nextEl.firstChild, newParent, -1);
    }
  }

  // —— reorderModifiableDescendants（参考实现 :2345）。
  function _zwVaReorderModifiableDescendants(node, command, newValue) {
    var candidate = node;
    var guard = 0;
    while (_zwVaIsModifiable(candidate)
           && candidate.childNodes && candidate.childNodes.length === 1
           && _zwVaIsModifiable(candidate.firstChild)
           && (!_zwVaIsSimpleModifiable(candidate)
               || !_zwVaEquivalentValues(command,
                    _zwVaSpecifiedValue(candidate, command), newValue))
           && guard++ < 256) {
      candidate = candidate.firstChild;
    }
    if (candidate === node
        || !_zwVaIsSimpleModifiable(candidate)
        || !_zwVaEquivalentValues(command, _zwVaSpecifiedValue(candidate, command), newValue)
        || !_zwVaLooselyEquivalent(command,
             _zwValEffectiveValue(candidate, command), newValue, true)) {
      return;
    }
    var guardM = 0;
    while (_zwJusHasKids(candidate) && guardM++ < 4096) {
      _zwJusMovePreservingRanges(candidate.firstChild, candidate.parentNode,
        _zwJusNodeIndex(candidate));
    }
    node.parentNode.insertBefore(candidate, node.nextSibling);
    _zwJusMovePreservingRanges(node, candidate, -1);
  }

  // —— 引擎安全 splitText（part04 splitText 仅 handle-based wrapper；plain 文本
  // 节点缺法——zz-probe-fs4 实证）——data 写 + createTextNode + insertBefore。
  // 选区边界不在此处补偿（setSelectionValue 调用点显式回写）。
  function _zwVaSplitText(node, offset) {
    var data = String(node.data);
    var after = data.slice(offset);
    var newText = document.createTextNode(after);
    node.data = data.slice(0, offset);
    var p = node.parentNode;
    var next = node.nextSibling;
    try {
      if (next) p.insertBefore(newText, next);
      else p.appendChild(newText);
    } catch (_eSt) {
      try { p.appendChild(newText); } catch (_eSt2) {}
    }
    return newText;
  }

  // —— Chromium delta（⑦ exec 包裹形状主攻，2026-11-04）：部分含选区的
  // specified 冲突祖先在选区边界切分——行内祖先 peel（外段子级移入原元素克隆、
  // 中段原地待 clearValue），块级祖先 push-down（非选中子级以 span 携带
  // specified 值包裹、块自身去属性）。语料录 Chrome 形
  // `<font size=1>foo[bar]baz</font>` +fontsize4 →
  // `<font size=1>foo</font><font size=4>bar</font><font size=1>baz</font>`、
  // `<p style="font-size:xx-small">foo[bar]baz</p>` →
  // `<p><span style="font-size:xx-small">foo</span>[bar]<span…>baz</span></p>`；
  // spec/vendored setSelectionValue 无此臂（spec forceValue 只做嵌套包裹，
  // corpus 119 案主族）。参考 chromium ApplyStyleCommand::applyInlineStyle 对
  // partially selected styled element 的 split 面（第三方法 wrap 外先拆层）。

  // 选区端点插入补偿（forceValue 同款）：parent 的 idx 落点插入新节点后，端点
  // 引用 (parent, ≥idx) 者 +1（shim insertBefore 不自动随动选区端点）。
  function _zwVaShiftEndpointsForInsert(parent, idx) {
    var sel = globalThis.getSelection ? globalThis.getSelection() : null;
    var rng = sel && sel._ranges && sel._ranges.length ? sel._ranges[0] : null;
    if (!rng) return;
    if (rng.startContainer === parent && rng.startOffset >= idx) {
      try { rng.setStart(parent, rng.startOffset + 1); } catch (_eSh1) {}
    }
    if (rng.endContainer === parent && rng.endOffset >= idx) {
      try { rng.setEnd(parent, rng.endOffset + 1); } catch (_eSh2) {}
    }
  }

  // 边界复合子索引：把 (cont, off) 换算成 el 的子间隙位。中间层（cont 与 el
  // 之间）边界必须贴子边缘（0 或全长）——中位意味着边界嵌在无关包裹里，peel
  // 无子间隙语义，返 -1 维持旧嵌套行为（cont === el 时任意子间隙合法）。
  function _zwVaEdgeComposite(el, cont, off) {
    if (cont === el) return off;
    var idx = off;
    var p = cont;
    var guard = 0;
    while (p && p !== el && guard++ < 256) {
      var par = p.parentNode;
      if (!par) return -1;
      var plen = p.nodeType === 3 ? _zwJusNodeLength(p) : p.childNodes.length;
      if (idx !== 0 && idx !== plen) return -1;
      idx = (idx === 0 ? 0 : 1) + _zwJusNodeIndex(p);
      p = par;
    }
    return p === el ? idx : -1;
  }

  // peel 后端点回写记录：{side, leaf}——leaf = el 中段首/末保留子级（pre-move
  // 快照身份，movePreserving 全程保恒）。管线尾把选区端点修到 leaf 内容缘
  // （Chrome 录形选区随内容进新包裹：`fo<span large>[ob]</span>…` 端点在 wrap
  // 内——qCV after 读新值，语料 26 案实证）。
  var _zwVaPeelRestore = [];
  // ③'-a：canonicalize keyword 保留形登记（流程尾自归一，见 canonicalize 处注释）。
  var _zwVaKwPreservePending = [];

  // peel 载体记录：{el}——start 侧 = 属性克隆，end 侧 = 残余 el（外段后缀
  // 载体）。供 _zwVaRematerializeCarries 嵌套 flag-form 重物化消费。
  var _zwVaPeelCarry = [];
  // u/s 移除面 start 克隆「保 tag」登记（DC-2 ①，2026-11-12）：移除区贯穿
  // 元素尾（选区末端容器在 el 之外）→ css-true 幸存前段保原 tag 不换形
  // span（`foo<u>ba[r</u>]baz` css-true = `foo<u>ba</u>rbaz` 录形
  // underline.js:658-662、`<strike>fo[o</strike><s>b]ar</s>` 保 strike
  // :710-714）；末端在 el 内容内（含末文本尾界，`<u>foo[bar]baz</u>`）
  // → 内段形照旧换形 span（:178-182 录形）。供
  // _zwVaRematerializeCarries 查询；_zwVaSplitConflictingInline 入口随
  // _zwVaPeelCarry 一并清空。
  var _zwVaPeelKeepTag = [];

  // 单祖先单侧 peel（无边界重写设计——随动自动保界）。两侧机制不对称：
  // · start 侧：外段前缀 [0..ci) 移入 el 的属性克隆（保样式），克隆以 **ref=el**
  //   插挂（解析元素 ref——M 代理 insertBefore wire 的 sel 印 ref 臂，实证落地面）；
  //   子级后搬入（forceValue wrap 臂同款：先空挂后搬子）。start 端点容器 =
  //   保留中段首子级（kids[ci]，不移动）→ 边界原值即正确，无需重写。
  // · end 侧：**不克隆**——内段前缀 [0..ci) 裸搬出 el 到 parent（clearValue
  //   simple-modifiable 同款上移，全程桥实证面），el 原样保外段后缀 [ci..len)
  //   属性。end 端点容器 = 末搬出子级（随动 epInNode 臂保界）→ 边界原值即正确。
  // 不对称的桥根因：M 代理 insertBefore 的 wire 分支只接 sel 印/handle 章/
  // textEl/plain-元素 ref——**解析文本 ref（无印无章）全分支 miss 静默丢弃**，
  // 故「克隆后插到 el 之后」的任何 ref（baz 等文本兄弟）不可用；start 侧克隆
  // 落 el 之前天然避开，end 侧免克隆彻底绕开（fix4 trace8-9 实证链）。
  // peel 残料 Chrome delta（⑩，2026-11-11）：移出片段后壳内残余**全为不可见
  // 节点**（extraneous br / 空白文本）时壳整体溶解（余料裸落、元素移除）——
  // `foo<sup>[bar]<br></sup>` 解除录形 = `foobar<br>`，peel 端侧 bare-move 后
  // sup 留装 br（vendored 无此门，forceValue 独立重裹形同左），Chrome 不留
  // 空壳（superscript 语料 2 变体实证）。残余含可见内容时保持既有保留行为。
  function _zwVaPeelDissolveInvisibleRemnant(el) {
    if (!el || !el.parentNode) return;
    var remKids = [];
    for (var rk = 0; rk < el.childNodes.length; rk++) {
      remKids.push(el.childNodes[rk]);
    }
    if (!remKids.length) return;
    for (var rk2 = 0; rk2 < remKids.length; rk2++) {
      var rkN = remKids[rk2];
      var inv = rkN.nodeType === 3 ? _zwVaIsWsText(rkN) : _zwJusIsInvisible(rkN);
      if (!inv) return;
    }
    var elIdxR = _zwJusNodeIndex(el);
    for (var rk3 = 0; rk3 < remKids.length; rk3++) {
      _zwJusMovePreservingRanges(remKids[rk3], el.parentNode, elIdxR + rk3);
    }
    if (el.parentNode) el.parentNode.removeChild(el);
  }

  function _zwVaPeelOne(el, isStart, command) {
    var rng = _zwVaGetActiveRange();
    if (!rng || !el.parentNode) return;
    var cont = isStart ? rng.startContainer : rng.endContainer;
    var off = isStart ? rng.startOffset : rng.endOffset;
    var ci = _zwVaEdgeComposite(el, cont, off);
    if (ci < 0) return;
    var len = el.childNodes.length;
    // 保形登记须在子级搬移前判定（搬移后 childNodes.length 已缩、容器已随动）。
    var keepTagP = false;
    if (isStart && (command === 'underline' || command === 'strikethrough')) {
      var ecK = rng.endContainer;
      var inElK = false;
      try { inElK = el === ecK || el.contains(ecK); } catch (_ePk) { inElK = false; }
      if (!inElK) {
        // 贯穿尾界分命令：strike 克隆恒保 tag（`<strike>fo[o</strike><s>b]ar
        // </s>` = 保 strike，strike.js:710-714——出口已持线值仍保）；u 克隆仅
        // 出口无命令线值时保（`foo<u>ba[r</u>]baz` 出口落普通内容 = 保 u，
        // underline.js:658-662；`<u>fo[o</u><ins>b]ar</ins>` 出口入持
        // underline 的 ins = 换形 span，:713-717）。
        if (command === 'strikethrough') {
          keepTagP = true;
        } else {
          var effK = null;
          try { effK = _zwValEffectiveValue(ecK, command); } catch (_ePk2) { effK = null; }
          keepTagP = !(effK && String(effK).indexOf('underline') !== -1);
        }
      }
    }
    if (isStart ? ci <= 0 : ci >= len) return;
    var parent = el.parentNode;
    var kids = [].slice.call(el.childNodes);
    _zwVaPeelRestore.push({
      side: isStart ? 'S' : 'E',
      leaf: kids[isStart ? ci : ci - 1]
    });
    if (isStart) {
      var clone = document.createElement(_zwJusTag(el));
      try {
        var at = el.attributes;
        for (var a = 0; a < at.length; a++) {
          try {
            var an = at[a].name, av = at[a].value;
            if (an.toLowerCase() === 'style') av = _zwVaNormalizeStyleAttrCss(av);
            clone.setAttribute(an, av);
          } catch (_ePa) {}
        }
      } catch (_ePb) {}
      _zwVaShiftEndpointsForInsert(parent, _zwJusNodeIndex(el));
      parent.insertBefore(clone, el);
      for (var k = 0; k < ci; k++) {
        _zwJusMovePreservingRanges(kids[k], clone, k);
      }
      _zwVaPeelDissolveInvisibleRemnant(el);
      if (keepTagP) _zwVaPeelKeepTag.push(clone);
      _zwVaPeelCarry.push(clone);
    } else {
      var elIdx = _zwJusNodeIndex(el);
      // Chrome delta（③ carry-clone，2026-11-05）：end 侧移出片段仅在载体
      // 持有命令等价面之外的残余样式时包载体克隆。fontsize teal `[a]bc`/
      // `a[b]c` 期望 bg-carry 形 `<span style=bg><font size=5>…</font></span>`
      // ——bare-move 丢 carry（got `<font size="5">a</font>` 实证）；残余
      // 声明（bg）经克隆 clearValue 摘冲突 prop 存活，forceValue 残余下潜臂
      // 把新包裹收进克隆内（G2 同机制）。语料 18 例 P2F（fix12 门禁实证，
      // fontsize/fontname/forecolor 单等价载体 end-straddle）：仅命令等价面
      // （size/font-size、color/color、face/font-family）的载体 Chrome 不留
      // 克隆——移出片段裸落并与前邻已裹片段并拢（`ob` 合一形）；克隆隔断
      // adjacency 后 wrap merge-only 并不拢，故必须真裸移。
      if (_zwVaCarrierResidualStyling(el, command)) {
        var cloneE = document.createElement(_zwJusTag(el));
        try {
          var atE = el.attributes;
          for (var aE = 0; aE < atE.length; aE++) {
            try {
              var anE = atE[aE].name, avE = atE[aE].value;
              if (anE.toLowerCase() === 'style') avE = _zwVaNormalizeStyleAttrCss(avE);
              cloneE.setAttribute(anE, avE);
            } catch (_ePaE) {}
          }
        } catch (_ePbE) {}
        _zwVaShiftEndpointsForInsert(parent, elIdx);
        parent.insertBefore(cloneE, el);
        for (var k2 = 0; k2 < ci; k2++) {
          _zwJusMovePreservingRanges(kids[k2], cloneE, k2);
        }
        _zwVaPeelCarry.push(cloneE);
      } else {
        for (var k2 = 0; k2 < ci; k2++) {
          _zwJusMovePreservingRanges(kids[k2], parent, elIdx + k2);
        }
      }
      _zwVaPeelDissolveInvisibleRemnant(el);
      if (el.parentNode) _zwVaPeelCarry.push(el);
    }
  }

  // 行内切分主臂：start/end 两边界各自爬升（editing host / 块级 / 不可编辑
  // 祖先截止），收集 specified 冲突的可编辑行内祖先链，自外向内逐层 peel
  // （外层切分只动外段，内层链元相对位置不变，先外后内保索引）。
  // backcolor/hilitecolor Chrome delta（fix9 复判）：仅当两边界爬升的首个
  // specified 冲突祖先相同（fcS===fcE——元素整体包住选区、两侧中间无其他
  // 冲突载体，M1 `b[ar]` 形 fcS=fcE=tan 拆分实证 `foo<span tan>b</span>arbaz`）
  // 才 peel 该元素；跨界形/嵌套载体形（fcS≠fcE）Chrome 录形恒整壳保留、新
  // 包裹收进内容侧（L273 `ba[r</span>]`、L298、L243 嵌套 transparent 三形
  // 实证——fix4 首版禁切 24+ 回归即本族）。forecolor/fontsize/fontname 维持
  // 逐层 peel-all（跨元素形 Chrome 亦拆 end 侧壳，语料 L647 实证）。全选区
  // 贴边形由 peel 侧 ci 门天然 no-op。
  function _zwVaSplitConflictingInline(command, newValue) {
    _zwVaPeelRestore = [];
    _zwVaPeelCarry = [];
    _zwVaPeelKeepTag = [];
    var isBg = command === 'backcolor' || command === 'hilitecolor';
    var fcS = null;
    if (isBg) {
      var rng0 = _zwVaGetActiveRange();
      if (!rng0) return;
      var firstConflict = function (cont) {
        var n = cont;
        var guard = 0;
        while (n && guard++ < 256) {
          var par = n.parentNode;
          if (!par || par.nodeType !== 1) break;
          if (_zwJusIsEditingHost(par) || _zwJusIsBlockNode(par)
              || !_zwJusIsEditable(par)) break;
          var spec = _zwVaSpecifiedValue(par, command);
          if (spec !== null && !_zwVaEquivalentValues(command, spec, newValue)) {
            return par;
          }
          n = par;
        }
        return null;
      };
      fcS = firstConflict(rng0.startContainer);
      var fcE = firstConflict(rng0.endContainer);
      if (!fcS || fcS !== fcE) return;
    }
    var isStart = false;
    for (var side = 0; side < 2; side++) {
      isStart = side === 0;
      var rng = _zwVaGetActiveRange();
      if (!rng) return;
      var cont = isStart ? rng.startContainer : rng.endContainer;
      if (!cont) continue;
      var chain = [];
      var n = cont;
      var guard = 0;
      while (n && guard++ < 256) {
        var par = n.parentNode;
        if (!par || par.nodeType !== 1) break;
        if (_zwJusIsEditingHost(par) || _zwJusIsBlockNode(par)
            || !_zwJusIsEditable(par)) break;
        var spec = _zwVaSpecifiedValue(par, command);
        // DC-2 ②（2026-11-12）：bold ADD（newValue 为激活值 bold/600-900）
        // 恒不切分带 bold 指定值祖先——非激活数值壳（100-500/normal）与
        // 激活数值壳（600/800，bold 等值集仅 700 故仍入 conflict 分支）录形
        // 均保壳内裹（data/bold.js:593-700 数值权重、:988-1000 300|400/
        // 400|500/500|600 邻接对、:926-1005 邻接 span 簇：`<span 500>fo[o
        // </span><span 600>b]ar</span>` 期望两壳原形 + 壳内 bold 载体）。
        // toggle-off（newValue null/'normal'）切分面不受此门。
        if (spec !== null && !_zwVaEquivalentValues(command, spec, newValue)
            && (!isBg || par === fcS)
            && !(command === 'bold'
                && ((parseInt(newValue, 10) || 0) >= 600 || newValue === 'bold'))) {
          chain.push(par);
        }
        n = par;
      }
      for (var i = chain.length - 1; i >= 0; i--) {
        _zwVaPeelOne(chain[i], isStart, command);
      }
    }
  }

  // peel 载体嵌套 flag-form 重物化（Chrome delta ⑧，2026-11-05）：flag=true 且
  // 载体（start 属性克隆 / end 残余 el）为 font 属性形、且载体父链（editable
  // 行内域）存在 specified 值祖先（嵌套形）→ 转 span style 形。语料分形实证：
  // 嵌套 font1-in-font4 fs4 flag=true 期望 span x-small 载体（fs4a L433）；flat
  // font1 fs4 flag=true 期望 font attr 载体恒保形（L378，tA2 恒绿面）；flag=
  // false 嵌套恒 font attr（fs4b L438）；span/style 载体已 flag-form（克隆归一
  // 臂 + 祖先规范形直出）不动。
  function _zwVaRematerializeCarries(command, newValue) {
    var cssFlag = _zwVaCssFlag();
    for (var i = 0; i < _zwVaPeelCarry.length; i++) {
      var cw = _zwVaPeelCarry[i];
      if (!cw || !cw.parentNode) continue;
      // DC-2 ①（2026-11-12）：u/s 移除面（newValue null）peel 幸存壳重物化
      // ——克隆/残余 el 携壳 tag + inline decl（或裸 tag face）按 Chrome 分解
      // 录形转形（data/underline.js:504-513 s-underline、:520-528 u-lineThru
      // +strike、:648-656 u 残 el、:703-721 跨 ins 载体；data/strikethrough.js
      // 同构）。decl 残值 = canon∩decl − 命令 token（junk 丢弃、canon 位序）：
      //   css-true：换形 span，decl := 残值非空 ? 残值 : [removed]（:506
      //     span(removed) 录形——选中段裸化、外围段保完整视觉）；
      //   css-false：decl 残值非空保 tag 改 decl（无语料保守形）；残值空换
      //     tag(removed)（underline→u、line-through→strike，:511/:527 录形）
      //     剥 decl；裸 tag face（无 decl）保 tag 不动（css-false 既有绿面
      //     :651-655 u 残壳、styled-child css-false 族）。
      if (newValue === null
          && (command === 'underline' || command === 'strikethrough')) {
        var declRm = '';
        try { declRm = _zwPlainDecl(cw, 'text-decoration') || ''; } catch (_eRm0) { declRm = ''; }
        var tgRm = _zwJusTag(cw);
        if (declRm || tgRm === 'u' || tgRm === 's' || tgRm === 'strike') {
          var remRm = command === 'underline' ? 'underline' : 'line-through';
          var canonRm = ['underline', 'overline', 'line-through'];
          var toksRm = declRm ? String(declRm).toLowerCase().split(/\s+/) : [];
          var survRm = [];
          for (var tRm = 0; tRm < toksRm.length; tRm++) {
            if (toksRm[tRm] && canonRm.indexOf(toksRm[tRm]) !== -1
                && toksRm[tRm] !== remRm && survRm.indexOf(toksRm[tRm]) === -1) {
              survRm.push(toksRm[tRm]);
            }
          }
          survRm.sort(function (a, b) { return canonRm.indexOf(a) - canonRm.indexOf(b); });
          if (cssFlag) {
            // bare 壳保形门（DC-2 ①）：s 裸壳恒保 tag（strikethrough.js
            // :271-280/:655-663 内段/进首段录形——s 幸存段不换形 span，与
            // u/strike 分解形不同）；u/strike 移除区贯穿元素尾的 start 克隆
            // 保 tag（underline.js:658-662、strike.js:710-718 录形，登记见
            // _zwVaPeelKeepTag）。decl 面壳与内段 u/strike 照旧换形 span
            //（underline.js:178-182、strike.js:331-335、:648-652 录形）。
            if (!declRm && (tgRm === 's'
                || _zwVaPeelKeepTag.indexOf(cw) !== -1)) {
              continue;
            }
            var spanRm = _zwJusSetTagName(cw, 'span');
            _zwVaSetStyleProp(spanRm, 'text-decoration',
              (survRm.length ? survRm : [remRm]).join(' '));
          } else if (declRm) {
            if (survRm.length) {
              _zwVaSetStyleProp(cw, 'text-decoration', survRm.join(' '));
            } else {
              var repRm = _zwJusSetTagName(cw, remRm === 'underline' ? 'u' : 'strike');
              _zwVaSetStyleProp(repRm, 'text-decoration', '');
              try {
                var atRm = repRm.getAttribute('style');
                if (atRm !== null && String(atRm).trim() === '') {
                  repRm.removeAttribute('style');
                }
              } catch (_eRm1) {}
            }
          }
          // 裸 tag face（无 decl）css-false：保 tag 原样（:651-655 u 残壳、
          // styled-child css-false 族既有绿面）。
          continue;
        }
      }
      // DC-2 ②（2026-11-12）：bold toggle-off（newValue 'normal'）peel 幸存
      // 壳重物化——css-false 且壳 decl ≡700（bold 等值）转 b 剥 decl（
      // data/bold.js:479-483 录形两翼 fresh b：`<span style="font-weight:
      // 700">foo[bar]baz</span>` → `<b>foo</b>bar<b>baz</b>`）；600/800/900
      // 非等值壳保 span 原形（:484/:489/:700 录形两翼 span-N 保形——D12/
      // D13 600|700、700|800 邻接形同证）；css-true 恒 span-N 保形（:476
      // 录形，下方 cssFlag+span 既有 continue 面即绿）；tag 面 b/strong 无
      // decl 不入转形（strong 部分选区无录形，保守保原形）。
      if (command === 'bold' && newValue === 'normal') {
        if (!cssFlag) {
          var fwB0 = '';
          try { fwB0 = _zwPlainDecl(cw, 'font-weight') || ''; } catch (_eBw0) { fwB0 = ''; }
          if (fwB0
              && _zwVaEquivalentValues('bold', String(fwB0).trim().toLowerCase(),
                'bold')) {
            var bB0 = _zwJusSetTagName(cw, 'b');
            _zwVaSetStyleProp(bB0, 'font-weight', '');
            try {
              var atB0 = bB0.getAttribute('style');
              if (atB0 !== null && String(atB0).trim() === '') {
                bB0.removeAttribute('style');
              }
            } catch (_eBw1) {}
          }
        }
        continue;
      }
      // ③'-b（tag 模式 forecolor span 载体臂）：peel 克隆载体恒为 span+style
      // 形（克隆路径 _zwVaNormalizeStyleAttrCss 归一 rgb），tag 模式语料期望
      // legacy font attr 形（forecolor.js :441 `foo<font color="#a52a2a">b</
      // font>[ar]baz` 录形实证）。窄门：forecolor、纯单声明载体（唯一属性
      // style 且仅 color 一声明）、嵌套形（祖先链持命令 specified，镜像下方
      // css 模式嵌套门）、简单色可解析——不满足保守不换（rgba/junk 载体、
      // 多声明载体、fontsize/fontname/bg 不入此臂：font 载体本就走下方
      // css 模式转换臂，bg 无 legacy attr 形）。
      if (!cssFlag && command === 'forecolor' && _zwJusTag(cw) === 'span') {
        var pure = false;
        try {
          var atB = cw.attributes;
          if (atB && atB.length === 1
              && String(atB[0].name || '').toLowerCase() === 'style') {
            var declsB = _zwPlainStyleDecls(cw);
            var hasOwn = Object.prototype.hasOwnProperty;
            var cnt = 0;
            var only = false;
            for (var dB in declsB) {
              if (!hasOwn.call(declsB, dB)) continue;
              cnt++;
              if (dB === 'color') only = true;
            }
            pure = cnt === 1 && only;
          }
        } catch (_eRmB) {}
        if (pure) {
          var specB = _zwVaSpecifiedValue(cw, command);
          var nestedB = false;
          var nB = cw.parentNode;
          var guardB = 0;
          while (nB && guardB++ < 256) {
            if (nB.nodeType !== 1 || !_zwJusIsEditable(nB)) break;
            if (_zwJusIsEditingHost(nB) || _zwJusIsBlockNode(nB)) break;
            if (_zwVaSpecifiedValue(nB, command) !== null) { nestedB = true; break; }
            nB = nB.parentNode;
          }
          var hexB = specB === null ? null : _zwVaParseSimpleColor(specB);
          if (nestedB && hexB) {
            var fontB = document.createElement('font');
            fontB.setAttribute('color', hexB);
            var parentB = cw.parentNode;
            var idxB = _zwJusNodeIndex(cw);
            _zwVaShiftEndpointsForInsert(parentB, idxB);
            parentB.insertBefore(fontB, cw);
            var kidsB = [].slice.call(cw.childNodes);
            for (var kB = 0; kB < kidsB.length; kB++) {
              _zwJusMovePreservingRanges(kidsB[kB], fontB, kB);
            }
            parentB.removeChild(cw);
          }
        }
        continue;
      }
      if (cssFlag && _zwJusTag(cw) === 'span') continue;
      // ③'-fs 收口：下方 flag-form 重物化臂（⑨）本属 css 模式——③'-b 抽走
      // 函数首 tag 模式早退后，tag 模式 font 载体（fontsize/fontname/forecolor
      // carry）误入此臂被换形 span CSS 形（fontsize :438 straddle A/B 实证：
      // a04dd1522 前绿 got=font 壳恒一，539200c21 后红 got=span x-small）。
      // tag 模式在此停臂（镜像改前整函数早退语义），font 载体保持 legacy
      // attr 形；forecolor tag span 载体由上方 ③'-b 臂消费后 continue 不及此。
      if (!cssFlag) continue;
      var spec = _zwVaSpecifiedValue(cw, command);
      if (spec === null) continue;
      var nested = false;
      var n = cw.parentNode;
      var guard = 0;
      while (n && guard++ < 256) {
        if (n.nodeType !== 1 || !_zwJusIsEditable(n)) break;
        if (_zwJusIsEditingHost(n) || _zwJusIsBlockNode(n)) break;
        if (_zwVaSpecifiedValue(n, command) !== null) { nested = true; break; }
        n = n.parentNode;
      }
      if (!nested) continue;
      var prop = command === 'fontsize' ? 'font-size'
        : (command === 'fontname' ? 'font-family'
          : (command === 'forecolor' ? 'color' : 'background-color'));
      var span = document.createElement('span');
      _zwVaSetStyleProp(span, prop, spec);
      var parent = cw.parentNode;
      var idx = _zwJusNodeIndex(cw);
      _zwVaShiftEndpointsForInsert(parent, idx);
      parent.insertBefore(span, cw);
      var kids = [].slice.call(cw.childNodes);
      for (var k = 0; k < kids.length; k++) {
        _zwJusMovePreservingRanges(kids[k], span, k);
      }
      parent.removeChild(cw);
    }
  }

  // 块级 specified 冲突祖先（`<p style=font-size:…>foo[bar]baz</p>` 族）不切分
  // 不 push-down——语料实证 Chrome 原样保块属性、包裹只收进块内（forceValue
  // 嵌套臂既有行为，fix4 首版 push-down 臂已按 6 内HTML回归证据移除）。

  // legacy font 载体判定（并值臂门槛）：font 元素且属性集 ⊆ {size, color,
  // face}——带无关属性（id 等）不并值，走常规包裹面。
  function _zwVaIsLegacyFontCarrier(el) {
    try {
      var at = el.attributes;
      if (!at) return true;
      for (var k = 0; k < at.length; k++) {
        var nm = String(at[k].name || '').toLowerCase();
        if (nm !== 'size' && nm !== 'color' && nm !== 'face') return false;
      }
      return true;
    } catch (_eLc) {
      return false;
    }
  }

  // ③ 并值载体自带样式面判定：style 属性 ≥1 声明，或 legacy font 的
  // size/color/face 属性（带无关属性如 id 的 font 形不并——:1460 并值臂
  // 同口径；裸 span/backcolor L18 归外裹）。attributes API 不可用保守不并。
  function _zwVaCarrierHasOwnStyling(el) {
    try {
      var at = el.attributes;
      if (!at) return false;
      for (var k = 0; k < at.length; k++) {
        var nm = String(at[k].name || '').toLowerCase();
        if (nm === 'style') {
          var decls = _zwPlainStyleDecls(el);
          for (var d in decls) {
            if (Object.prototype.hasOwnProperty.call(decls, d)) return true;
          }
        } else if (_zwJusTag(el) === 'font'
            && (nm === 'size' || nm === 'color' || nm === 'face')) {
          return true;
        }
      }
      return false;
    } catch (_eCa) {
      return false;
    }
  }

  // ③ 克隆残余判据（Chrome delta，2026-11-05）：载体去掉命令等价面后仍持有
  // 其他样式信号才准克隆。等价面映射：fontsize/fontname/forecolor 对应
  // legacy font size/face/color 属性与 style font-size/font-family/color
  // 声明；hilitecolor/backcolor 对应 style background-color 声明；其余命令
  // 无等价面（任何样式均算残余，但非 font 属性形载体按裸移保守走旧路径）。
  // 语料 18 例 P2F（fix12 门禁 fontsize/fontname/forecolor 单等价载体
  // end-straddle）实证仅等价面时 Chrome 不留克隆；带残余（teal bg 案）
  // 克隆存活 carry。attributes API 不可用保守不克隆。
  function _zwVaCarrierResidualStyling(el, command) {
    var cmd = String(command || '').toLowerCase();
    // 克隆仅限本轮门禁覆盖的五个 font 簇命令；其余命令（bold 等 multitest
    // 基线均在裸移下取得）保守不克隆，维持 pre-③ 裸移路径。
    if (cmd !== 'fontsize' && cmd !== 'fontname' && cmd !== 'forecolor'
        && cmd !== 'hilitecolor' && cmd !== 'backcolor') return false;
    var cmdAttr = cmd === 'fontsize' ? 'size'
      : cmd === 'fontname' ? 'face'
      : cmd === 'forecolor' ? 'color' : '';
    var cmdDecl = cmd === 'fontsize' ? 'font-size'
      : cmd === 'fontname' ? 'font-family'
      : cmd === 'forecolor' ? 'color'
      : (cmd === 'hilitecolor' || cmd === 'backcolor') ? 'background-color' : '';
    try {
      var at = el.attributes;
      if (!at) return false;
      for (var k = 0; k < at.length; k++) {
        var nm = String(at[k].name || '').toLowerCase();
        if (nm === 'style') {
          var decls = _zwPlainStyleDecls(el);
          for (var d in decls) {
            if (Object.prototype.hasOwnProperty.call(decls, d)
                && String(d).toLowerCase() !== cmdDecl) return true;
          }
        } else if (_zwJusTag(el) === 'font'
            && (nm === 'size' || nm === 'color' || nm === 'face')
            && nm !== cmdAttr) {
          return true;
        }
      }
      return false;
    } catch (_eCr) {
      return false;
    }
  }

  // 属性全空判定（unwrap 臂门槛）：attributes API 不可用视作非空（保守不剥）。
  function _zwVaIsAttributeless(el) {
    try {
      var at = el.attributes;
      return !!at && at.length === 0;
    } catch (_eAl) {
      return false;
    }
  }

  // 解包：children 回亲（movePreserving 端点随动）、端点引用 el 本体者重映射
  // 到亲（子间隙平移 el 原索引）、移除 el。
  function _zwVaUnwrapElement(el) {
    var parent = el.parentNode;
    if (!parent) return;
    var idx = _zwJusNodeIndex(el);
    var kids = [].slice.call(el.childNodes);
    for (var k = 0; k < kids.length; k++) {
      _zwJusMovePreservingRanges(kids[k], parent, idx + k);
    }
    var rng = _zwVaGetActiveRange();
    if (rng) {
      if (rng.startContainer === el) {
        try {
          rng.setStart(parent, Math.min(idx + rng.startOffset, parent.childNodes.length));
        } catch (_eUw1) {}
      }
      if (rng.endContainer === el) {
        try {
          rng.setEnd(parent, Math.min(idx + rng.endOffset, parent.childNodes.length));
        } catch (_eUw2) {}
      }
    }
    parent.removeChild(el);
  }

  // —— state/value override（参考实现 :1102-1171）——存储范围快照不匹配即重置
  // （selection 变化作废旧 override）。backcolor/hilitecolor 双向镜像。
  var _zwVaStateOverrides = {};
  var _zwVaValueOverrides = {};
  var _zwVaStoredRange = null;

  function _zwVaOverridesReset() {
    var r = null;
    try { r = _zwVaGetActiveRange(); } catch (_eOr) {}
    var cur = r
      ? { sc: r.startContainer, so: r.startOffset, ec: r.endContainer, eo: r.endOffset }
      : null;
    var stale = !_zwVaStoredRange || !cur
      || _zwVaStoredRange.sc !== cur.sc || _zwVaStoredRange.so !== cur.so
      || _zwVaStoredRange.ec !== cur.ec || _zwVaStoredRange.eo !== cur.eo;
    if (stale) {
      _zwVaStateOverrides = {};
      _zwVaValueOverrides = {};
      _zwVaStoredRange = cur;
    }
  }

  // F6（2026-11-02）：delete/forwarddelete preservesOverrides 等价面——把 stored
  // range 快照重锚到当前活动选区但**不清** override。vendored override 存储非
  // range 绑定、preservesOverrides 命令不重置（myExecCommand :8495 包裹）；本仓
  // 惰性 staleness 清除模型下 delete 必然移动选区 → 下一次 override 读会被误判
  // 陈旧清空（qV-after-delete 语料期望 override 存活实证）。由 part01f
  // __zw_ce_delete_apply 在删除落定后单点调用；其余命令的清除行为不变。
  function _zwVaReanchorOverrides() {
    var r = null;
    try { r = _zwVaGetActiveRange(); } catch (_eRa) {}
    _zwVaStoredRange = r
      ? { sc: r.startContainer, so: r.startOffset, ec: r.endContainer, eo: r.endOffset }
      : null;
  }

  // F6（2026-11-02）：preservesOverrides 包裹的原始快照/回写（不经 staleness
  // reset）——删除动作内部的 override 读（值查询/effective value 面）会在选区
  // 位移后触发 reset 误清存储；快照在动作前抓（彼时 stored==current，reset 无
  // 效），动作后重锚并整体回写。g17 multitest 实证：无包裹时 bold qS-after-
  // delete 期望 true 得 false（~30 断言）、fontsize qV-after 期望 "4" 得 "3"。
  function _zwVaOverridesTakeRaw() {
    var s = {}, v = {}, k;
    for (k in _zwVaStateOverrides) {
      if (Object.prototype.hasOwnProperty.call(_zwVaStateOverrides, k)) {
        s[k] = _zwVaStateOverrides[k];
      }
    }
    for (k in _zwVaValueOverrides) {
      if (Object.prototype.hasOwnProperty.call(_zwVaValueOverrides, k)) {
        v[k] = _zwVaValueOverrides[k];
      }
    }
    return { state: s, value: v };
  }

  function _zwVaOverridesPutRaw(snap) {
    if (!snap) return;
    _zwVaStateOverrides = snap.state || {};
    _zwVaValueOverrides = snap.value || {};
  }

  function _zwVaSetValueOverride(command, newValue) {
    _zwVaOverridesReset();
    _zwVaValueOverrides[command] = newValue;
    if (command === 'backcolor') _zwVaValueOverrides.hilitecolor = newValue;
    else if (command === 'hilitecolor') _zwVaValueOverrides.backcolor = newValue;
  }

  function _zwVaGetValueOverride(command) {
    _zwVaOverridesReset();
    return _zwVaValueOverrides[command];
  }

  function _zwVaUnsetValueOverride(command) {
    _zwVaOverridesReset();
    delete _zwVaValueOverrides[command];
    if (command === 'backcolor') delete _zwVaValueOverrides.hilitecolor;
    else if (command === 'hilitecolor') delete _zwVaValueOverrides.backcolor;
  }

  function _zwVaSetStateOverride(command, newState) {
    _zwVaOverridesReset();
    _zwVaStateOverrides[command] = newState;
  }

  function _zwVaGetStateOverride(command) {
    _zwVaOverridesReset();
    return _zwVaStateOverrides[command];
  }

  // normalizeFontSize（参考实现 :3192）——非法 → null；clamp 1-7；±3 相对模式。
  function _zwVaNormalizeFontSize(value) {
    value = String(value == null ? '' : value).trim();
    if (!/^[-+]?[0-9]+(\.[0-9]+)?([eE][-+]?[0-9]+)?$/.test(value)) return null;
    var mode;
    if (value.charAt(0) === '+') { value = value.slice(1); mode = 'relative-plus'; }
    else if (value.charAt(0) === '-') { value = value.slice(1); mode = 'relative-minus'; }
    else { mode = 'absolute'; }
    var num = parseInt(value, 10);
    if (mode === 'relative-plus') num += 3;
    if (mode === 'relative-minus') num = 3 - num;
    if (num < 1) num = 1;
    if (num > 7) num = 7;
    return { 1: 'x-small', 2: 'small', 3: 'medium', 4: 'large',
      5: 'x-large', 6: 'xx-large', 7: 'xxx-large' }[num];
  }

  // 空白-only 文本节点判定（wrap 拉邻放宽 / br 抑制共用；white-space 感知）。
  function _zwVaIsWsText(n) {
    return !!n && n.nodeType === 3 && _zwJusIsWhitespaceNode(n);
  }
  // —— setSelectionValue（参考实现 :2965）——五命令主流程。
  // 祖先链 style 属性规范形直出（刀口 2b）：两边界容器爬升至 editing host
  // （本体不动——tests.js normalizeSerializedStyle(testDiv) 只归一后代），
  // 逐层把 style attr 经 _zwVaNormalizeStyleAttrCss 重写为 Chrome cssText
  // 规范形。空结果保留原文（junk attr 不动，防序列化空 attr 面漂移）。
  function _zwVaCanonicalizeAncestorStyleAttrs(rng) {
    _zwVaKwPreservePending = [];
    var sides = [rng.startContainer, rng.endContainer];
    for (var s = 0; s < 2; s++) {
      var n = sides[s];
      var guard = 0;
      while (n && guard++ < 256) {
        if (n.nodeType !== 1) { n = n.parentNode; continue; }
        if (_zwJusIsEditingHost(n)) break;
        if (typeof n.getAttribute === 'function') {
          var raw = n.getAttribute('style');
          if (raw) {
            // ③'-a：keyword 原文保留（指定形比较前置面），余形照常归一。保留
            // 形登记 _zwVaKwPreservePending——本仓 CSSOM 保 keyword 原文，
            // 语料端 vendored normalizeColor 走 gCS 通道对 brown 族解析落黑
            // （zz-probe-cssom 实测 brown→rgb(0, 0, 0)），harness 归一不可
            // 依赖，流程尾自归一回 rgb 序列形（克隆路径同形，kept-half 录形）。
            var normKw = _zwVaNormalizeStyleAttrCss(raw, true);
            var normPlain = _zwVaNormalizeStyleAttrCss(raw);
            if (normKw && normKw !== raw) {
              try { n.setAttribute('style', normKw); } catch (_eCa) {}
            }
            if (normKw !== normPlain) _zwVaKwPreservePending.push(n);
          }
        }
        n = n.parentNode;
      }
    }
  }

  function _zwVaSetSelectionValue(command, newValue) {
    var rng = _zwVaGetActiveRange();
    if (!rng) return;
    // 无 contained formattable → override 记账臂（collapsed 选区 Chrome 口径）。
    var containedAll = _zwValGetAllEffectivelyContained(rng, function () { return true; });
    var anyFormattable = false;
    var i;
    for (i = 0; i < containedAll.length; i++) {
      if (_zwValIsFormattable(containedAll[i])) { anyFormattable = true; break; }
    }
    if (!anyFormattable) {
      // spec setSelectionValue 无 formattable 臂（vendored :2972-2991）：六
      // inline 命令（即带 inlineCommandActivatedValues 者）按激活值命中写
      // state override（bold state 假落 'bold' 命中 → true；真落 'normal'
      // 未命中 → false，解除面 qS-after=false 期望源）；sub/sup 互斥 unset
      // 对侧。此前只记账 value override（P1 FIXME 挂账）。
      var actF = (typeof globalThis.__zw_ce_format_activated === 'function')
        ? globalThis.__zw_ce_format_activated : null;
      var sixSw = command === 'bold' || command === 'italic'
        || command === 'underline' || command === 'strikethrough'
        || command === 'subscript' || command === 'superscript';
      if (sixSw && actF) _zwVaSetStateOverride(command, !!actF(command, newValue));
      if (command === 'subscript') delete _zwVaStateOverrides.superscript;
      else if (command === 'superscript') delete _zwVaStateOverrides.subscript;
      _zwVaSetValueOverride(command, newValue);
      return;
    }

    // 边界 splitText（start 侧）——vendored 的 removeAllRanges/addRange 舞蹈在其
    // globalRange 存在时是无效面（getActiveRange 恒返回 globalRange 本体，真机制
    // = getActiveRange().setStart(newNode, 0) 原位写、end 不动）；本仓无
    // globalRange，sel._ranges[0] 即该面。半成品 newActiveRange（不同容器臂 end
    // 未设 → setStart 塌缩）经舞蹈重绑后 rng 丢失真端点 → contained 集空 → 整段
    // apply 零变异（`foo[bar<b>baz]qoz</b>quz` 族五语料各 2 变体实证）——改为对
    // rng 全程原位写，不做选区重绑。
    if (_zwJusIsEditable(rng.startContainer)
        && rng.startContainer.nodeType === 3
        && rng.startOffset !== 0
        && rng.startOffset !== _zwJusNodeLength(rng.startContainer)) {
      if (rng.startContainer === rng.endContainer) {
        var newEndOffset = rng.endOffset - rng.startOffset;
        var newNode = _zwVaSplitText(rng.startContainer, rng.startOffset);
        rng.setEnd(newNode, newEndOffset);
        rng.setStart(newNode, 0);
      } else {
        var newNode2 = _zwVaSplitText(rng.startContainer, rng.startOffset);
        rng.setStart(newNode2, 0);
      }
    }
    // 边界 splitText（end 侧）。
    if (_zwJusIsEditable(rng.endContainer)
        && rng.endContainer.nodeType === 3
        && rng.endOffset !== 0
        && rng.endOffset !== _zwJusNodeLength(rng.endContainer)) {
      var newStart = [rng.startContainer, rng.startOffset];
      var newEnd = [rng.endContainer, rng.endOffset];
      _zwVaSplitText(rng.endContainer, rng.endOffset);
      rng.setStart(newStart[0], newStart[1]);
      rng.setEnd(newEnd[0], newEnd[1]);
      var selE = globalThis.getSelection();
      // DC-2 P2（2026-11-11）：重绑走公共 API 会经 _zwSync 的 swcss 选区清除臂
      //（part06 F1'-B）在 apply 中途清 __zwStyleWithCSS——forceValue span/元素
      // 取型在重绑之后读 flag（`foo[bar]baz` swcss(true) 尾文本触发本臂 →
      // <b> 误形，multitest 161 css-true 失败实证；`foobar` 端点=长度不触发本
      // 臂故孤立探针不可复现）。Chrome 恒不清该 flag（document 全局）；本仓
      // 清除臂是 typing 隔离近似，跨案隔离仍由每案 setup 公共 addRange 承担，
      // 此处内部重绑按 apply 前值回填。override 清除不回填（apply 不读
      // override，跨案 setup 清点同前）。
      var swPrev = globalThis.__zwStyleWithCSS;
      selE.removeAllRanges();
      selE.addRange(rng);
      globalThis.__zwStyleWithCSS = swPrev;
    }

    // 祖先链 style 属性规范形直出：Chrome 录制端 normalizeSerializedStyle 对
    // wrapper 后代 [style] 元素恒经 CSSOM 往返归一（Chrome getter 恒可用），语料
    // 期望串中选区祖先的 keyword 色 attr 恒为 `prop:rgb(...)` 规范形（M3 div
    // aqua→rgb(0, 255, 255) 实证）。我方 harness 归一器命中面受命令后 sel/查询
    // 态影响不稳定（zz-probe-nstyle ⑧ 命令后 getter 全空、div 面空间格未剥实证），
    // 故引擎在 setSelectionValue 入口对两边界祖先链（editing host 本体除外——
    // tests.js 归一范围 = wrapper 后代）直出规范形，got 与录制面无条件对齐。
    _zwVaCanonicalizeAncestorStyleAttrs(rng);

    // DC-2 ①（2026-11-12）：span decl 壳切刻面——u/s 移除面部分选区、壳
    // decl 剥除命令 token 后仍有 canon 残值时，Chrome 录形不切分壳：壳 decl
    // 原位重写为残值、选区外首尾段在壳内以移除 token 形内裹（underline.js
    // :484-493 `foo<span style="text-decoration: underline line-through">
    // b[a]r</span>baz` css-true = 壳保 line-through + b/r 段
    // `<span style="text-decoration:underline">` 内裹、css-false 内裹 u；
    // strikethrough.js:479-487 内裹 span/strike 同构）。重写后 specified
    // 含命令 token 面为 null → split 冲突判定自然跳过（不 peel 保壳）；
    // 全选残值形（junk 族 :725-761）同链只落 decl 重写（首尾段为空）。
    if ((command === 'underline' || command === 'strikethrough')
        && newValue === null) {
      var remSk = command === 'underline' ? 'underline' : 'line-through';
      var canonSk = ['underline', 'overline', 'line-through'];
      var seenSk = [];
      var edgesSk = [rng.startContainer, rng.endContainer];
      for (var eSk = 0; eSk < edgesSk.length; eSk++) {
        var nSk = edgesSk[eSk];
        if (nSk && nSk.nodeType !== 1) nSk = nSk.parentNode;
        var gSk = 0;
        while (nSk && nSk.nodeType === 1 && gSk++ < 256) {
          if (!_zwJusIsEditable(nSk)
              || _zwJusIsEditingHost(nSk) || _zwJusIsBlockNode(nSk)) break;
          if (_zwJusTag(nSk) === 'span' && seenSk.indexOf(nSk) === -1) {
            seenSk.push(nSk);
            var dSk = '';
            try { dSk = _zwPlainDecl(nSk, 'text-decoration') || ''; } catch (_eSk0) { dSk = ''; }
            var toksSk = dSk ? String(dSk).toLowerCase().split(/\s+/) : [];
            var survSk = [];
            for (var tSk = 0; tSk < toksSk.length; tSk++) {
              if (toksSk[tSk] && canonSk.indexOf(toksSk[tSk]) !== -1
                  && toksSk[tSk] !== remSk && survSk.indexOf(toksSk[tSk]) === -1) {
                survSk.push(toksSk[tSk]);
              }
            }
            if (survSk.length) {
              var kidsSk = [].slice.call(nSk.childNodes);
              var sPosSk = -1;
              var ePosSk = -1;
              for (var pSk = 0; pSk < kidsSk.length; pSk++) {
                var hitSk = kidsSk[pSk];
                var mSk = rng.startContainer;
                while (mSk && mSk.parentNode !== nSk) mSk = mSk.parentNode;
                if (mSk === hitSk) sPosSk = pSk;
                mSk = rng.endContainer;
                while (mSk && mSk.parentNode !== nSk) mSk = mSk.parentNode;
                if (mSk === hitSk) ePosSk = pSk;
              }
              // 端点确实落本壳内（含全选形——首尾段为空仅 decl 重写）才动；
              // 链上无关 decl 壳零触碰。
              if (sPosSk !== -1 || ePosSk !== -1) {
                survSk.sort(function (a, b) {
                  return canonSk.indexOf(a) - canonSk.indexOf(b);
                });
                // 直写 attr（不走 SetStyleProp）：既有解析壳 CSSOM 有 pending
                // 值，setValue 路径 cur==valS 跳手写回时 getAttribute/序列化
                // 见 CSSOM `k: v` 空格形（cssom-1），Chrome 录制端 cssText
                // 往返为 `k:v` 无空格规范形。setAttribute 外写使 CSSOM store
                // 按 attr 重放（规范形直出同链 _zwVaCanonicalizeAncestorStyle
                // Attrs 既有语义）。
                var declsSk = _zwPlainStyleDecls(nSk);
                declsSk['text-decoration'] = survSk.join(' ');
                var partsSk = [];
                for (var kSk in declsSk) {
                  if (Object.prototype.hasOwnProperty.call(declsSk, kSk)) {
                    partsSk.push(kSk + ':' + declsSk[kSk]);
                  }
                }
                nSk.setAttribute('style', partsSk.join('; '));
                var cutSk = remSk === 'underline' ? 'u' : 'strike';
                for (var hSk = 0; hSk < sPosSk; hSk++) {
                  var wSk = document.createElement(_zwVaCssFlag() ? 'span' : cutSk);
                  if (_zwVaCssFlag()) {
                    _zwVaSetStyleProp(wSk, 'text-decoration', remSk);
                  }
                  nSk.insertBefore(wSk, kidsSk[hSk]);
                  _zwJusMovePreservingRanges(kidsSk[hSk], wSk, 0);
                }
                for (var tSk2 = ePosSk + 1; tSk2 < kidsSk.length; tSk2++) {
                  var xSk = document.createElement(_zwVaCssFlag() ? 'span' : cutSk);
                  if (_zwVaCssFlag()) {
                    _zwVaSetStyleProp(xSk, 'text-decoration', remSk);
                  }
                  nSk.insertBefore(xSk, kidsSk[tSk2]);
                  _zwJusMovePreservingRanges(kidsSk[tSk2], xSk, 0);
                }
              }
            }
          }
          nSk = nSk.parentNode;
        }
      }
    }

    // Chromium delta 切分 pass（⑦ 包裹形状，2026-11-04）：部分含选区的
    // specified 冲突祖先行内 peel / 块级 push-down，先于 contained 清值
    // （peel 后中段元素升格 contained → 既有 clearValue 简单可改性摘壳臂
    // （:598 简单可改性分支）自然收口中段裸元素）。
    _zwVaSplitConflictingInline(command, newValue);
    // peel 载体嵌套 flag-form 重物化（⑧）：见函数注。
    _zwVaRematerializeCarries(command, newValue);

    // contained element clearValue pass。
    var elements = _zwValGetAllEffectivelyContained(rng, function (node) {
      return _zwJusIsEditable(node) && node.nodeType === 1;
    });
    for (i = 0; i < elements.length; i++) {
      var elC = elements[i];
      var specC = _zwVaSpecifiedValue(elC, command);
      _zwVaClearValue(elC, command);
      // Chromium delta：fully contained 的 legacy font 并值成属性（不另起
      // 包裹）——`<font size=7>[abc]</font>` +forecolor 期望
      // `<font color="#ff0000" size="7">`、三属性 font +fontname 期望 face
      // 原位替换（记录变体多序等价）。属性集限 size/color/face（带 id 等
      // 无关属性不并，走常规包裹）；冲突值已由 clearValue 摘属性臂（非简单
      // 可改性）或摘壳臂（简单可改性）处理，并值面直接落新值。
      var attrC = command === 'fontsize' ? 'size'
        : (command === 'fontname' ? 'face'
          : (command === 'forecolor' ? 'color' : null));
      if (attrC !== null && elC.parentNode && _zwJusTag(elC) === 'font'
          && _zwVaIsLegacyFontCarrier(elC)) {
        var attrVal = command === 'fontsize' ? _zwVaCssSizeToLegacy(newValue)
          : (command === 'fontname' ? newValue : _zwVaParseSimpleColor(newValue));
        if (attrVal != null) {
          try { elC.setAttribute(attrC, String(attrVal)); } catch (_eSaC) {}
        }
      }
      // Chromium delta：specified 值被摘净且元素再无属性 → 解包（children
      // 回亲、el 移除）。简单可改性元素已由 clearValue 摘壳臂覆盖；本臂补
      // 非简单可改性形态（带多余属性的 span 摘净 style 后滞留空壳，forceValue
      // 会把新包裹套进旧壳）。
      if (specC !== null && elC.parentNode && _zwVaIsAttributeless(elC)) {
        _zwVaUnwrapElement(elC);
      }
    }

    // node pushDownValues + forceValue pass。
    var nodes = _zwValGetAllEffectivelyContained(rng, _zwJusIsEditable);
    for (i = 0; i < nodes.length; i++) {
      var node = nodes[i];
      _zwVaPushDownValues(node, command, newValue);
      // Chromium delta（③ 载体形 merge，2026-11-05）：bg 命令对全含样式载体
      // 并值不外裹——Chrome 录形 `<font size=6>[foo]</font>` / xx-large span
      // 全选单成员 run 并值到载体（wrap 外裹后单成员归一收缩），裸 span 无
      // 样式不并（backcolor L18 期望外裹形）、run 多成员不并
      //（`[foo<font size=6>bar</font>baz]` 期望单外裹、font6 摘净——多成员
      // 经 :786 sibCrit 合并面既有行为）。并值后后代经 forceValue :805 等值
      // 早退自然免裹。非 bg 命令维持外裹（fontsize teal 走 carry-clone 面）。
      var bgMerge = (command === 'hilitecolor' || command === 'backcolor')
        && node.nodeType === 1
        && _zwVaSpecifiedValue(node, command) === null
        && _zwVaCarrierHasOwnStyling(node)
        && _zwVaIsAllowedChild(node, 'span');
      if (bgMerge) {
        var sibJoined = false;
        for (var mi = 0; mi < nodes.length; mi++) {
          if (nodes[mi] === node.previousSibling
              || nodes[mi] === node.nextSibling) {
            sibJoined = true;
            break;
          }
        }
        if (!sibJoined) {
          _zwVaSetStyleProp(node, 'background-color', newValue);
          continue;
        }
      }
      if (_zwVaIsAllowedChild(node, 'span')) {
        _zwVaForceValue(node, command, newValue);
      }
    }

    // peel 端点回写（Chrome 录形选区随内容进新包裹）：leaf 已随 movePreserving
    // 落位（中段内容并入新 wrap 或原位），端点修到其内容缘——qCV/qCS 读新值
    // 面（语料 `fo[o<font size=2>b]ar</font>` qCV after=4 实证）。leaf 若已被
    // 后续臂移除（异常面）保持管线边界不动。
    for (i = 0; i < _zwVaPeelRestore.length; i++) {
      var pr = _zwVaPeelRestore[i];
      var leaf = pr.leaf;
      if (!leaf || !leaf.parentNode) continue;
      try {
        if (pr.side === 'E') {
          var eLeaf = leaf;
          while (eLeaf.nodeType === 1 && eLeaf.childNodes.length) {
            eLeaf = eLeaf.childNodes[eLeaf.childNodes.length - 1];
          }
          if (eLeaf.nodeType === 3) rng.setEnd(eLeaf, _zwJusNodeLength(eLeaf));
        } else {
          rng.setStart(leaf, 0);
        }
      } catch (_ePr) {}
    }
    _zwVaPeelRestore = [];

    // ③'-a：keyword 保留形流程尾自归一（rgb 序列形对齐克隆路径与既有基线
    // 形；语料端 vendored normalizeColor 不可依赖——canonicalize 处注释）。
    for (i = 0; i < _zwVaKwPreservePending.length; i++) {
      var elKw = _zwVaKwPreservePending[i];
      if (!elKw || !elKw.parentNode) continue;
      try {
        var rawKw = elKw.getAttribute('style');
        if (rawKw) {
          var normKw2 = _zwVaNormalizeStyleAttrCss(rawKw);
          if (normKw2 && normKw2 !== rawKw) elKw.setAttribute('style', normKw2);
        }
      } catch (_eKw2) {}
    }
    _zwVaKwPreservePending = [];
  }

  // —— 纯校验导出（part06 编辑分支 beforeinput 前置）——无效值在事件前拒绝
  //（Chrome 口径：action 校验失败零事件、execCommand 返 false）。fontname 任意
  // 串合法（vendored action 无校验，:3180）。
  globalThis.__zw_ce_value_validate = function (cmd, rawValue) {
    if (!_zwVaFive[String(cmd)]) return null;
    // enabled 面（语料 fontname tt 两形实证 [false]：起点入 tt[ce=false] 与
    // 跨 tt 交叉两形均返 false）——选区任一端点不在可编辑内容（宿主自身或
    // 宿主内可编辑节点）→ 命令未启用，execCommand 返 false 零事件。
    var selEn = globalThis.getSelection ? globalThis.getSelection() : null;
    var rngEn = selEn && selEn._ranges && selEn._ranges.length ? selEn._ranges[0] : null;
    var boundEditable = function (n) {
      return !!n && (_zwJusIsEditingHost(n) || _zwJusIsEditable(n));
    };
    if (rngEn && (!boundEditable(rngEn.startContainer)
        || !boundEditable(rngEn.endContainer))) {
      return false;
    }
    var value = rawValue == null ? '' : String(rawValue);
    if (cmd === 'fontsize') return _zwVaNormalizeFontSize(value) !== null;
    if (cmd === 'forecolor' || cmd === 'backcolor' || cmd === 'hilitecolor') {
      // Chrome 口径：非法色不前置拒绝（语料 [true] 实证 return true 零变更），
      // 仅 currentColor 维持 spec 拒绝（语料 [false]、vendored 同）。
      return value.toLowerCase() !== 'currentcolor';
    }
    return true;
  };

  // —— override 读数导出（part01c __zw_ce_value_query 消费）——未设返
  // undefined（调用方按 undefined 区分「无 override」）；读前走 reset（选区
  // 变动作废旧 override——同 vendored getValueOverride 惯用法）。fontsize 消费
  // 面 = getLegacyFontSize（vendored myQueryCommandValue :655-657）——
  // _zwVaLegacyValueOf 两段式（关键字直转 + normalize 链 + px 中点映射），
  // 不可映射返 ''（queryCommandValue 串面契约）。
  globalThis.__zw_ce_value_override_value = function (cmd) {
    var v = _zwVaGetValueOverride(String(cmd));
    if (v === undefined) return undefined;
    if (String(cmd) === 'fontsize') {
      var legacy = _zwVaLegacyValueOf(v);
      return legacy === null ? '' : String(legacy);
    }
    if (String(cmd) === 'forecolor' || String(cmd) === 'backcolor'
        || String(cmd) === 'hilitecolor') {
      // Chrome override 存色对象、读出即 CSSOM 序列化形（语料 collapsed
      // #0000FF → qV 'rgb(0, 0, 255)' 实证——vendored 存原始串直返是 delta）。
      return _zwVaNormalizeCssColor(v) || v;
    }
    return v;
  };

  // —— state override 读数导出（part01c __zw_ce_format_query 与 part06
  // _zwQueryFormatState 消费）——未设返 undefined（调用方区分「无 override」）；
  // 读前走 reset（选区变动作废旧 override，同 value 侧惯用法）。vendored
  // getStateOverride :1124-1128 对应面。
  globalThis.__zw_ce_value_state_override = function (cmd) {
    return _zwVaGetStateOverride(String(cmd));
  };

  // —— typing 值 override 快照导出（part01 __zw_ce_inserttext_mutate 消费）
  // ——insertText 变异核开头（世界同步/选区移动前）捕获。读侧 reset 语义同
  // vendored recordCurrentOverrides（:4351，插入前记录）——变异移动选区后
  // stored range 失配、事后读恒空，必须快照。fontname/fontsize/forecolor/
  // hilitecolor 四命令（记录序同 vendored；backcolor 经镜像落 hilitecolor 槽）。
  globalThis.__zw_ce_value_override_snapshot = function () {
    var cmds = ['fontname', 'fontsize', 'forecolor', 'hilitecolor'];
    var snap = null;
    for (var i = 0; i < cmds.length; i++) {
      var v = _zwVaGetValueOverride(cmds[i]);
      if (v !== undefined) {
        if (!snap) snap = {};
        snap[cmds[i]] = v;
      }
    }
    return snap;
  };

  // —— 页面脚本选区变更 override 清除导出（part06 Selection shim _zwSync 消费）
  // ——Chrome typing style + spec overrides 随程序化选区设置清空（multitest
  // qV-before 默认值实证：前一子测 backcolor #00FFFF 执行后次案 qV-before 恒
  // "rgba(0, 0, 0, 0)"）。F1' styled 壳重建刀（2026-11-08）根因面：跨子测陈旧
  // override 经 delete take/putRaw + reanchor（preservesOverrides 语义）保鲜，
  // insertText 快照读命中 → typed 段以 monospace/7/red/cyan 陈旧壳重裹
  // （mt 25 键 F1'-A 族）。清点挂 _zwSync（Selection 公共 API 唯一通道）；
  // 内部路径不经此（typed caret 走 _ranges 直写、delete 走 reanchor+putRaw、
  // 结构命令 preservesOverrides take/put 包裹尾回写恢复）。stored range 置
  // null → 下一次读走既有 staleness reset 重锚当前选区。
  // https://w3c.github.io/selection-api/ （程序化选区变更语义）
  globalThis.__zw_ce_value_overrides_clear = function () {
    _zwVaStateOverrides = {};
    _zwVaValueOverrides = {};
    _zwVaStoredRange = null;
  };

  // —— typing 值 override 重放导出（part01 __zw_ce_inserttext_mutate 变异尾
  // 消费）——vendored insertText action 的 restoreStatesAndValues（:4458）：
  // 插入后对覆盖插入段的 active range 逐条值 override 等值比较（areEquivalent
  // 门——forceValue 本体无等值门，缺门会对既有同值内容产生冗余包裹），非等值
  // 按 command action 重放。引擎侧经临时 range 进 _zwVaSetSelectionValue 主
  // 流程（边界 splitText 只分出插入段 + clearValue + pushDownValues +
  // forceValue——就地 splice 使 caret 锚为整节点，直接 forceValue 会过裹全串）
  // ；fontsize 记录侧已 normalize 关键字，直进主流程（legacy 换算在 attr 臂）。
  // 尾部 caret 塌到 range 终点（vendored 尾部 collapseToEnd 对应面）。
  // F6b 门机具（2026-11-08 oracle 定谳档）：node 位 DOM sub/sup 祖先判定
  // （host 止）+ 关键字→桶（part01 __zw_f6b_bucket_of 消费，px 表
  // x-small..xxx-large = 10/13/16/18/24/32/48 与 part01d sizeMapData 同源）。
  function _zwVaNodeInScriptF6b(node) {
    var n = (node && node.nodeType === 1) ? node : (node ? node.parentNode : null);
    var guard = 0;
    while (n && n.nodeType === 1 && guard++ < 256) {
      var tn = String(n.tagName || '').toLowerCase();
      if (tn === 'sub' || tn === 'sup') return true;
      if (typeof _zwJusIsEditingHost === 'function' && _zwJusIsEditingHost(n)) return false;
      n = n.parentNode;
    }
    return false;
  }
  function _zwVaKwBucketF6b(kw) {
    var pxMap = { 'x-small': 10, small: 13, medium: 16, large: 18,
      'x-large': 24, 'xx-large': 32, 'xxx-large': 48 };
    var px = pxMap[String(kw)];
    if (px == null || typeof globalThis.__zw_f6b_bucket_of !== 'function') return null;
    return globalThis.__zw_f6b_bucket_of(px);
  }

  globalThis.__zw_ce_value_restore_overrides = function (node, off0, off1, snap) {
    if (!node || !snap) return;
    var sel = globalThis.getSelection ? globalThis.getSelection() : null;
    if (!sel) return;
    var cmds = ['fontname', 'fontsize', 'forecolor', 'hilitecolor'];
    var r = document.createRange();
    try {
      r.setStart(node, off0);
      r.setEnd(node, off1);
    } catch (_eRr) { return; }
    sel._ranges = [r];
    // F2 通道 B（2026-11-08 link-absorption 刀）：armed pending link 时
    // forecolor 参照钉死过滤（#0000ff = Chrome UA link 参照色，本仓 UA 无
    // link 色规则不读 computed——待判别②）。归一 ≡ 参照剔除不 materialize
    //（multitest :1994-:2012 族期望无 font）；非参照值绕等值门强制重放——
    // UA 填充初值黑（_zwValEffectiveValue 无声明返 'rgb(0, 0, 0)'）使 black
    // 与 link 内 computed 恒等值、等值门恒过跳过（census black got `fooabar`
    // 无 font、blue/brown 有 font 双向实证）。qS/qV override 读面不动
    //（override 解耦法则②③）。fontname/fontsize/hilitecolor 无 F2 语料键
    // 不滤（F6b 域）。
    var armedLink = globalThis.__zwTypingLink;
    var linkArmed = armedLink != null && armedLink !== '';
    var f6bDissolvedKw = null;
    for (var i = 0; i < cmds.length; i++) {
      var cmd = cmds[i];
      var v = snap[cmd];
      if (v === undefined || v === null || v === '') continue;
      if (linkArmed && cmd === 'forecolor') {
        if (_zwVaNormalizeColorKey(v) === 'rgb(0, 0, 255)') continue;
        try { _zwVaSetSelectionValue(cmd, v); } catch (_eRoF2) {}
        continue;
      }
      // F6b（2026-11-08 oracle 定谳档）fontsize 重放等值门 script 缩放桶
      // 口径：caret 位 DOM sub/sup 时按 0.83em 缩放最近档比较（语料 C1/C2
      // `sub,fontsize=2,inserttext` 期望无 font——指定 small 桶2 ≡ sub 内
      // 13.28px 桶2；C3/C4 fontsize=3 期望 font size3 壳外——medium 桶3 ≠
      // 桶2 重放）。walk_px 恒 16px 缺省口径盲作者 inline 字号，故非
      // script 上下文保旧口径零行为差（script 位 author 字号不在 F6b 语料
      // 形内，FIXME: script×author-font-size 混合形未接）。
      if (cmd === 'fontsize'
          && typeof globalThis.__zw_f6b_bucket_of === 'function'
          && typeof globalThis.__zw_f6b_walk_px === 'function'
          && _zwVaNodeInScriptF6b(node)) {
        var f6bCb = globalThis.__zw_f6b_bucket_of(globalThis.__zw_f6b_walk_px(node));
        var f6bVb = _zwVaKwBucketF6b(v);
        if (f6bCb != null && f6bVb != null && f6bCb === f6bVb) {
          f6bDissolvedKw = v;
          continue;
        }
      } else if (_zwVaEquivalentValues(cmd, _zwValEffectiveValue(node, cmd), v, true)) {
        continue;
      }
      try { _zwVaSetSelectionValue(cmd, v); } catch (_eRo) {}
    }
    try {
      var end2 = (sel._ranges && sel._ranges[0]) ? sel._ranges[0] : r;
      var c = document.createRange();
      c.setStart(end2.endContainer, end2.endOffset);
      c.collapse(true);
      sel._ranges = [c];
    } catch (_eRo2) {}
    // F6b qV 载体重绑（2026-11-09 门禁批 P2F 2 判读）：等桶溶解后树无 font
    // 壳，基线 qV("fontsize")="2" 经壳树读偶然满足（重放裹壳）；语料期望值
    // 命令效果仍可查（multitest.js :2059/:2064 qV-after "2"）——溶解臂收
    // 集、caret 塌到位后重绑 override（reset 以插入段尾 caret 重锚存活）。
    // 跨案清除走既有 _zwSync 选区变更面，无泄漏。包 wrap 路径（C3/C4）与
    // 非 script 等值跳过路径不动（壳在，树读即真）。
    if (f6bDissolvedKw != null) {
      try { _zwVaSetValueOverride('fontsize', f6bDissolvedKw); } catch (_eRoF6b) {}
    }
  };

  // 选区整体是否落在同一 a[href] 内（Chrome foreColor 不改链内样式的 delta
  // 门；语料 3 例 `<a href>foo[bar]baz</a>` 实证：DOM 不变 + [true] + qV 读
  // 新值）。两端点各自最近 a[href] 祖先同且非空即判命中。
  function _zwVaInHrefAnchor() {
    var sel = globalThis.getSelection ? globalThis.getSelection() : null;
    var rng = sel && sel._ranges && sel._ranges.length ? sel._ranges[0] : null;
    if (!rng || rng.collapsed) return false;
    var up = function (n) {
      // 文本节点起点直接判元素会漏（while 条件首查即败）——沿父链逐级查。
      while (n) {
        if (n.nodeType === 1 && _zwJusTag(n) === 'a'
            && n.getAttribute && n.getAttribute('href') != null) return n;
        n = n.parentNode;
      }
      return null;
    };
    var a1 = up(rng.startContainer);
    return !!a1 && a1 === up(rng.endContainer);
  }

  // —— 引擎入口（part06 execCommand 五命令臂单点调用）——返回 false = 无效值
  // （execCommand 返 false——语料 return value 断言面）。fontsize 归一后以
  // 关键字进流程；色三命令 currentColor 拒（spec），非法色 Chrome 口径返
  // true 零变更（语料 [true] + DOM 不变实证，与 vendored return false 异）。
  globalThis.__zw_ce_value_apply = function (cmd, rawValue) {
    if (!_zwVaFive[String(cmd)]) return null;
    var value = rawValue == null ? '' : String(rawValue);
    if (cmd === 'fontsize') {
      var kw = _zwVaNormalizeFontSize(value);
      if (kw === null) return false;
      _zwVaSetSelectionValue('fontsize', kw);
      return true;
    }
    if (cmd === 'forecolor' || cmd === 'backcolor' || cmd === 'hilitecolor') {
      if (/^([0-9a-fA-F]{3}){1,2}$/.test(value)) value = '#' + value;
      // currentColor 独走 spec 拒绝（语料 [false]，vendored 同）。
      if (value.toLowerCase() === 'currentcolor') return false;
      // Chrome 口径（语料实证、与 vendored return false 相异的 delta）：非法
      // 色返 true 且零变更——DOM 不动、override 不记（qV-after 落缺省黑）。
      if (!_zwVaNormalizeCssColor(value)) return true;
      // Chrome delta（语料 forecolor 3 例实证，仅 forecolor 观测面）：选区整
      // 体在 a[href] 内不落 DOM——返 true + 值 override 记账（qV-after 读新
      // 值）。vendored 无此门。
      if (cmd === 'forecolor' && _zwVaInHrefAnchor()) {
        _zwVaSetValueOverride(cmd, _zwVaNormalizeCssColor(value));
        return true;
      }
      _zwVaSetSelectionValue(cmd, value);
      return true;
    }
    _zwVaSetSelectionValue('fontname', value);
    return true;
  };

  // —— 六 format 命令 action 入口（part06 execCommand 非 collapsed 臂单点调
  // 用；DC-2 P2 apply 面，2026-11-11）——vendored action 原文直译（bold :3119/
  // italic :3464/strikethrough :3592/underline :3674/subscript :3609/
  // superscript :3641）：state 真落解除值（bold/italic 落 normal、underline/
  // strikethrough 落 null），假落激活值；sub/sup 两段式（先 null 清对侧与既
  // 有值，state false 再落目标值——互斥经 clearValue 摘壳自然达成）。state 读
  // = spec myQueryCommandState 激活面（P1 __zw_ce_format_query；state override
  // 未接同 P1 挂账）。非六命令返 null（调用方回落）。
  globalThis.__zw_ce_format_apply = function (cmd) {
    var c = String(cmd == null ? '' : cmd).toLowerCase();
    var six = { bold: 1, italic: 1, underline: 1, strikethrough: 1,
      subscript: 1, superscript: 1 };
    if (!six[c]) return null;
    // Chrome delta（⑪ 不可编辑端点门，2026-11-11；rc=false 化 DC-2 ②
    // 2026-11-12）：选区任一边界位置落在 contenteditable=false 子树内 →
    // 整条命令 no-op（rc=false，DOM 零变更）——vendored 按 editable 过滤
    // contained 后仍会对可编辑片段落值；Chrome 对边界入非编辑区的命令
    // 恒不应用且 execCommand 返回 false（data/bold.js:103-167 录形：
    // `fo[o<span cef>b]ar</span>baz`/`<span cef>fo[o<span ce>b]ar</span>
    // baz</span>` 期望原样 + [false]；对照中段穿越形 `fo[o<span cef>
    // bar</span>b]az` 边界均在可编辑区 → rc=true 两翼落值）。余五命令
    // 语料无 cef 录形，门仅 bold 实证、按命令面共享逻辑保持一致。边界
    // 位置解析：文本容器即本节点；元素容器取 offset 处子节点（offset=
    // 末端为宿主缘，可编辑——`{…}` 选形端点落在 editing host 元素上
    // 不触发）。
    try {
      var rngCef = _zwVaGetActiveRange();
      if (rngCef) {
        var cefBn = function (cont, off) {
          if (!cont) return null;
          if (cont.nodeType === 3) return cont;
          var kidsB = cont.childNodes;
          if (off < kidsB.length) return kidsB[off];
          return null;
        };
        var bnS = cefBn(rngCef.startContainer, rngCef.startOffset);
        var bnE = cefBn(rngCef.endContainer, rngCef.endOffset);
        if ((bnS !== null && !_zwJusIsEditable(bnS)
              && !_zwJusIsEditingHost(bnS))
            || (bnE !== null && !_zwJusIsEditable(bnE)
              && !_zwJusIsEditingHost(bnE))) {
          return false;
        }
      }
    } catch (_eCefG) {}
    var state = false;
    var fq = (typeof globalThis.__zw_ce_format_query === 'function')
      ? globalThis.__zw_ce_format_query(c) : null;
    if (fq) state = fq.state;
    if (c === 'bold') {
      _zwVaSetSelectionValue('bold', state ? 'normal' : 'bold');
      return true;
    }
    if (c === 'italic') {
      _zwVaSetSelectionValue('italic', state ? 'normal' : 'italic');
      return true;
    }
    if (c === 'underline') {
      _zwVaSetSelectionValue('underline', state ? null : 'underline');
      return true;
    }
    if (c === 'strikethrough') {
      _zwVaSetSelectionValue('strikethrough', state ? null : 'line-through');
      return true;
    }
    _zwVaSetSelectionValue(c, null);
    if (!state) _zwVaSetSelectionValue(c, c);
    return true;
  };
