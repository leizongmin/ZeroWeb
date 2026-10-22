// ── part08d.js — GLSL 绘制求值器 + draw 光栅化后端 ─────────────────────────
//
// 职责：part08b（词法/语法/语义校验——单遍无 AST，infoLog 错误面字节钉死）
// 之后的"求值"半边：共享 part08b 词法器构建绘制用 AST、类型化求值执行
// shader、drawArrays/drawElements 顶点装配 → 裁剪 → 视口变换 → 像素中心
// 重心光栅 → host 位图替换写回（putImageData 对偶 getImageData 读底）。
//
// 求值值模型：标量 = JS number（bool 0/1）；向量 = {k, c:[…]}；矩阵 =
// {k, m:[[列0…],[列1…]]} 列主序（m[列][行]）；sampler 占位 0。
//
// 范围边界（FIXME 汇总——含此类构造的 shader 在 draw 期解析/求值失败 →
// 静默跳过绘制，spec 不允许报错、像素保持既有内容）：
//   - 评估器子集拒绝面：位运算/取模/while/do/用户函数/数组声明/
//     struct/带参函数定义（前后缀 ++/-- 已支持，GLSL ES 1.0 §5.9）。
//   - int/int 除法按浮点语义（标量无类型痕迹；常驻测试不依赖）。
//   - 裁剪仅做"任一顶点 w≤0 → 整图元丢弃"，无平面插值裁剪。
//   - depth/stencil 测试未实现（状态面已存，绘制恒通过）；blend 非默认
//     因子（ONE/ZERO）按替换写。
//   - LINE 光栅走参数采样近似（非 GL 线光栅精确规则）；POINTS 恒 1×1
//     （gl_PointSize 未消费）。
//   - texture2D/textureCube 采样返回不透明黑（sampler 上传链未接）。
//   - FBO 绑定时的绘制与 readPixels 读回（render-to-texture）未实现。
//   - UNPACK 对齐/premultiply 状态不参与顶点数据解释（GLES2 §2.8 客户端
//     数组无像素状态面）。
//
// 交接：消费 part08b 的 globalThis._zwGlslShared（tokenize/types/builtins/
// reserved/tVoid——用后即清，part01 globalThis 单点先例）；发布
// globalThis._zwGlDrawBackend = { drawArrays, drawElements } 供 part08 绘制
// op 调用期查表（加载期无耦合，仅要求 include 顺序 part08b → part08d）。
// mode/负值/program null 等前置域校验留在 part08（corpus 首错序已钉）。

(function () {
  'use strict';

  var H = globalThis._zwGlslShared;
  globalThis._zwGlslShared = null;
  if (!H) return; // part08b 未载（include 顺序错误）——本簇整体不注册
  var _zwGlslTokenize = H.tokenize;
  var _ZW_GLSL_TYPES = H.types;
  var _ZW_GLSL_T_VOID = H.tVoid;

  // ── 类型族别（枚举值与 part08b _ZW_GLSL_TYPES 同源，GLES2 Table 6.14）──
  var _T_FLOAT = 0x1406, _T_INT = 0x1404, _T_BOOL = 0x8B56;
  var _T_VEC2 = 0x8B50, _T_VEC3 = 0x8B51, _T_VEC4 = 0x8B52;
  var _T_IVEC2 = 0x8B53, _T_IVEC4 = 0x8B55;
  var _T_BVEC2 = 0x8B57, _T_BVEC4 = 0x8B59;
  var _T_MAT2 = 0x8B5A, _T_MAT3 = 0x8B5B, _T_MAT4 = 0x8B5C;
  var _T_SAMP2D = 0x8B5E, _T_SAMPCUBE = 0x8B60;

  var _ZW_WIDTHS = {};
  _ZW_WIDTHS[_T_FLOAT] = 1; _ZW_WIDTHS[_T_INT] = 1; _ZW_WIDTHS[_T_BOOL] = 1;
  _ZW_WIDTHS[_T_VEC2] = 2; _ZW_WIDTHS[_T_VEC3] = 3; _ZW_WIDTHS[_T_VEC4] = 4;
  _ZW_WIDTHS[_T_IVEC2] = 2; _ZW_WIDTHS[_T_IVEC4] = 4;
  _ZW_WIDTHS[_T_BVEC2] = 2; _ZW_WIDTHS[_T_BVEC4] = 4;
  _ZW_WIDTHS[_T_MAT2] = 4; _ZW_WIDTHS[_T_MAT3] = 9; _ZW_WIDTHS[_T_MAT4] = 16;
  _ZW_WIDTHS[_T_SAMP2D] = 1; _ZW_WIDTHS[_T_SAMPCUBE] = 1;

  function _zwTIsMat(k) { return k === _T_MAT2 || k === _T_MAT3 || k === _T_MAT4; }
  function _zwTIsVec(k) { return (k >= _T_VEC2 && k <= _T_VEC4) ||
    (k >= _T_IVEC2 && k <= _T_IVEC4) || (k >= _T_BVEC2 && k <= _T_BVEC4); }
  function _zwTIsInt(k) { return k === _T_INT || (k >= _T_IVEC2 && k <= _T_IVEC4); }
  function _zwTSampler(k) { return k === _T_SAMP2D || k === _T_SAMPCUBE; }
  function _zwTWidth(k) { return _ZW_WIDTHS[k] || 1; }
  function _zwTMatN(k) { return k === _T_MAT2 ? 2 : k === _T_MAT3 ? 3 : k === _T_MAT4 ? 4 : 0; }
  function _zwTVecOf(n) { return n === 2 ? _T_VEC2 : n === 3 ? _T_VEC3 : _T_VEC4; }

  function _zwTZero(k) { // 未初始化声明按零（GLSL 未定义行为——确定性取零）
    if (_zwTIsMat(k)) {
      var n = _zwTMatN(k), m = [], c, r;
      for (c = 0; c < n; c++) { m.push([]); for (r = 0; r < n; r++) m[c].push(0); }
      return { k: k, m: m };
    }
    if (_zwTIsVec(k)) {
      var w = _zwTWidth(k), a = [], i;
      for (i = 0; i < w; i++) a.push(0);
      return { k: k, c: a };
    }
    return 0;
  }

  // ── AST 解析（求值子集；校验错误面归 part08b——本解析失败 → draw 跳过）──
  // 语句：block/decl/if/for/return/discard/break/continue/expr；
  // 表达式：num/var/bin/un/assign(= += -= *= /=)/sel(.xyzw rgba stpq)/
  // idx/call(类型构造/内建)/cond(?:)。
  function _zwGlParseDraw(src) {
    var toks = _zwGlslTokenize(src, function () {}); // 词法失败同走 draw 跳过面
    if (!toks) return { ok: false, reason: 'tokenize failed' };
    var p = 0;
    function fail(m) { throw new Error(m); }
    function tk() { return toks[Math.min(p, toks.length - 1)]; }
    function isOp(v) { var t = tk(); return t.t === 'op' && t.v === v; }
    function isKw(v) { var t = tk(); return t.t === 'kw' && t.v === v; }
    function isTyKw() { var t = tk(); return t.t === 'kw' && _ZW_GLSL_TYPES[t.v] !== undefined; }
    function eatOp(v) { if (isOp(v)) { p++; return true; } return false; }
    function eatKw(v) { if (isKw(v)) { p++; return true; } return false; }
    function needOp(v) { if (!eatOp(v)) fail("expected '" + v + "'"); }
    function needId() {
      if (tk().t !== 'id') fail('expected identifier');
      return toks[p++].v;
    }
    function parseType() {
      if (!isTyKw()) fail('expected type');
      return _ZW_GLSL_TYPES[toks[p++].v];
    }

    var prog = { ok: true, globals: [], varyings: [], main: null };
    function parseGlobalDecl(storage) { // 限定词已消费
      var k = parseType();
      for (;;) {
        var name = needId();
        if (eatOp('[')) fail('arrays not supported');
        var init = null;
        if (eatOp('=')) init = parseAssign();
        prog.globals.push({ storage: storage, k: k, name: name, init: init });
        if (storage === 'varying') prog.varyings.push(name);
        if (!eatOp(',')) break;
      }
      needOp(';');
    }
    function parseFunction() {
      var k = parseType();
      var name = needId();
      needOp('(');
      if (!isOp(')') && !eatKw('void')) fail('function parameters not supported');
      needOp(')');
      if (name !== 'main') fail('user-defined functions not supported');
      if (k !== _ZW_GLSL_T_VOID) fail('main must return void');
      if (prog.main) fail('multiple main definitions');
      prog.main = parseBlock();
    }
    function parseBlock() {
      needOp('{');
      var body = [];
      while (!isOp('}')) {
        if (tk().t === 'eof') fail('unterminated block');
        body.push(parseStmt());
      }
      p++;
      return { st: 'block', body: body };
    }
    function parseLocalDecl() {
      eatKw('const');
      var k = parseType();
      var list = [];
      for (;;) {
        var name = needId();
        if (eatOp('[')) fail('arrays not supported');
        var init = null;
        if (eatOp('=')) init = parseAssign();
        list.push({ name: name, init: init });
        if (!eatOp(',')) break;
      }
      needOp(';');
      return { st: 'decl', k: k, list: list };
    }
    function parseStmt() {
      if (isOp('{')) return parseBlock();
      if (eatKw('if')) {
        needOp('(');
        var c = parseExpr();
        needOp(')');
        var th = parseStmt();
        var el = eatKw('else') ? parseStmt() : null;
        return { st: 'if', c: c, th: th, el: el };
      }
      if (eatKw('for')) {
        needOp('(');
        var init = null;
        if (isTyKw() || isKw('const')) init = parseLocalDecl(); // decl 形自带 ';'
        else if (!isOp(';')) { init = { st: 'expr', e: parseExpr() }; needOp(';'); }
        else p++;
        var cond = isOp(';') ? null : parseExpr();
        needOp(';');
        var nx = isOp(')') ? null : parseExpr();
        needOp(')');
        return { st: 'for', init: init, cond: cond, nx: nx, body: parseStmt() };
      }
      if (eatKw('while') || eatKw('do')) fail('while/do loops not supported');
      if (eatKw('return')) {
        var rv = isOp(';') ? null : parseExpr();
        needOp(';');
        return { st: 'ret', e: rv };
      }
      if (eatKw('discard')) { needOp(';'); return { st: 'discard' }; }
      if (eatKw('break')) { needOp(';'); return { st: 'brk' }; }
      if (eatKw('continue')) { needOp(';'); return { st: 'cont' }; }
      if (isTyKw() || isKw('const')) return parseLocalDecl();
      var e = parseExpr();
      needOp(';');
      return { st: 'expr', e: e };
    }
    var _ASSIGN_OPS = { '=': 1, '+=': 1, '-=': 1, '*=': 1, '/=': 1 };
    function parseExpr() {
      return parseAssign();
    }
    function parseAssign() {
      var left = parseCond();
      var t = tk();
      if (t.t === 'op' && _ASSIGN_OPS[t.v]) {
        p++;
        if (left.t !== 'var' && left.t !== 'sel' && left.t !== 'idx') {
          fail('invalid assignment target');
        }
        return { t: 'assign', op: t.v, target: left, value: parseAssign() };
      }
      if (t.t === 'op' && (t.v === '%=' || t.v === '<<=' || t.v === '>>=')) {
        fail('bit-wise ops not supported');
      }
      return left;
    }
    function parseCond() {
      var c = parseOr();
      if (!eatOp('?')) return c;
      var a = parseAssign();
      needOp(':');
      return { t: 'cond', c: c, a: a, b: parseAssign() };
    }
    function binChain(sub, ops) {
      var l = sub();
      for (;;) {
        var t = tk();
        if (t.t === 'op' && ops[t.v]) { p++; l = { t: 'bin', op: t.v, l: l, r: sub() }; }
        else return l;
      }
    }
    function parseOr() { return binChain(parseAnd, { '||': 1 }); }
    function parseAnd() { return binChain(parseEq, { '&&': 1 }); }
    function parseEq() { return binChain(parseRel, { '==': 1, '!=': 1 }); }
    function parseRel() { return binChain(parseAdd, { '<': 1, '>': 1, '<=': 1, '>=': 1 }); }
    function parseAdd() { return binChain(parseMul, { '+': 1, '-': 1 }); }
    function parseMul() {
      var l = parseUnary();
      for (;;) {
        var t = tk();
        if (t.t === 'op' && (t.v === '*' || t.v === '/')) {
          p++; l = { t: 'bin', op: t.v, l: l, r: parseUnary() };
        } else if (t.t === 'op' && (t.v === '%' || t.v === '&' || t.v === '|' ||
          t.v === '^' || t.v === '<<' || t.v === '>>')) {
          fail('bit-wise/modulo ops not supported');
        } else return l;
      }
    }
    function parseUnary() {
      var t = tk();
      if (t.t === 'op' && (t.v === '-' || t.v === '+' || t.v === '!')) {
        p++; return { t: 'un', op: t.v, e: parseUnary() };
      }
      if (t.t === 'op' && (t.v === '++' || t.v === '--')) { // 前缀自增/自减
        p++; return { t: 'pre', op: t.v, e: parseUnary() };
      }
      if (t.t === 'op' && t.v === '~') fail('bit-wise ops not supported');
      return parsePostfix();
    }
    function parsePostfix() {
      var e = parsePrimary();
      for (;;) {
        if (eatOp('.')) {
          var f = tk();
          if (f.t !== 'id' && f.t !== 'kw') fail('expected member name');
          p++;
          e = { t: 'sel', e: e, sel: f.v };
        } else if (eatOp('[')) {
          var ix = parseExpr();
          needOp(']');
          e = { t: 'idx', e: e, i: ix };
        } else if (isOp('++') || isOp('--')) { // 后缀自增/自减（左值约束同赋值）
          if (e.t !== 'var' && e.t !== 'sel' && e.t !== 'idx') fail('invalid increment target');
          var op = tk().v;
          p++;
          e = { t: 'post', op: op, target: e };
        } else return e;
      }
    }
    function parseArgs() {
      var args = [];
      if (!isOp(')')) {
        args.push(parseAssign());
        while (eatOp(',')) args.push(parseAssign());
      }
      needOp(')');
      return args;
    }
    function parsePrimary() {
      var t = tk();
      if (t.t === 'inum' || t.t === 'fnum') {
        p++; return { t: 'num', v: t.t === 'inum' ? t.v : parseFloat(t.v) };
      }
      if (t.t === 'kw') {
        if (t.v === 'true') { p++; return { t: 'num', v: 1 }; }
        if (t.v === 'false') { p++; return { t: 'num', v: 0 }; }
        if (_ZW_GLSL_TYPES[t.v] !== undefined) { // 类型构造调用（vec3(…)/mat2(…)）
          var k = _ZW_GLSL_TYPES[t.v];
          p++; needOp('(');
          return { t: 'call', ctor: k, args: parseArgs() };
        }
        fail('unsupported keyword ' + t.v);
      }
      if (t.t === 'id') {
        p++;
        if (eatOp('(')) return { t: 'call', fn: t.v, args: parseArgs() };
        return { t: 'var', name: t.v };
      }
      if (eatOp('(')) { var e = parseExpr(); needOp(')'); return e; }
      fail('unexpected token ' + (t.v === '' ? 'eof' : t.v));
    }

    try {
      for (;;) {
        if (tk().t === 'eof') break;
        if (eatKw('precision')) { // precision 语句：precision 限定符 类型 ';'
          if (!eatKw('highp') && !eatKw('mediump') && !eatKw('lowp')) {
            fail('expected precision qualifier');
          }
          parseType();
          needOp(';');
          continue;
        }
        if (isKw('attribute') || isKw('uniform') || isKw('varying') || isKw('const')) {
          var st = tk().v;
          p++;
          parseGlobalDecl(st);
          continue;
        }
        if (isTyKw() || isKw('void')) { parseFunction(); continue; }
        fail('unsupported global construct ' + (tk().v || 'eof'));
      }
      if (!prog.main) fail('missing main()');
    } catch (e) {
      return { ok: false, reason: String((e && e.message) || e) };
    }
    return prog;
  }

  // ── 求值 ────────────────────────────────────────────────────────────────
  // env = { scopes: [全局 scope, …局部链] }；ctx = 执行控制流 + 步预算。
  var _ZW_STEP_BUDGET = 4000000; // 单次 draw 求值步预算（防失控循环；超出 → 弃绘制）

  function _zwTruth(v) { return typeof v === 'number' ? v !== 0 : !!v; }
  function _zwLookup(env, name) {
    for (var i = env.scopes.length - 1; i >= 0; i--) {
      var s = env.scopes[i];
      if (s.hasOwnProperty(name)) return s[name];
    }
    return undefined;
  }
  function _zwScopeOf(env, name) {
    for (var i = env.scopes.length - 1; i >= 0; i--) {
      if (env.scopes[i].hasOwnProperty(name)) return env.scopes[i];
    }
    return null;
  }

  var _SWZ = { x: 0, y: 1, z: 2, w: 3, r: 0, g: 1, b: 2, a: 3, s: 0, t: 1, p: 2, q: 3 };
  function _zwSwizSet(ch) {
    return ch === 'x' || ch === 'y' || ch === 'z' || ch === 'w' ? 0 :
      ch === 'r' || ch === 'g' || ch === 'b' || ch === 'a' ? 1 : 2;
  }
  function _zwSwizIdx(sel, distinct) { // 读允许重复/任意 ≤4；写要求互异
    var idx = [], set = -1, i, ch;
    for (i = 0; i < sel.length; i++) {
      ch = sel.charAt(i);
      if (_SWZ[ch] === undefined) throw new Error('bad swizzle .' + sel);
      if (set < 0) set = _zwSwizSet(ch);
      else if (set !== _zwSwizSet(ch)) throw new Error('mixed swizzle sets .' + sel);
      if (distinct && idx.indexOf(_SWZ[ch]) >= 0) throw new Error('duplicate swizzle .' + sel);
      idx.push(_SWZ[ch]);
    }
    return idx;
  }
  function _zwSwizzleRead(v, sel) {
    var idx = _zwSwizIdx(sel, false);
    if (typeof v === 'number') {
      if (idx.length === 1) return v;
      throw new Error('swizzle on scalar');
    }
    if (_zwTIsMat(v.k)) { // 列主序线性下标 → 分量
      var n = _zwTMatN(v.k), out = [], i;
      for (i = 0; i < idx.length; i++) {
        out.push(v.m[idx[i] % n][(idx[i] / n) | 0]);
      }
      return idx.length === 1 ? out[0] : { k: _zwTVecOf(idx.length), c: out };
    }
    if (_zwTIsVec(v.k)) {
      if (idx.length === 1) return v.c[idx[0]];
      var base = v.k >= _T_BVEC2 ? _T_BOOL : v.k >= _T_IVEC2 ? _T_INT : _T_FLOAT;
      var cs = [], j;
      for (j = 0; j < idx.length; j++) cs.push(v.c[idx[j]]);
      var kk = base === _T_BOOL ? (idx.length === 2 ? _T_BVEC2 : idx.length === 3 ? _T_BVEC3 : _T_BVEC4)
        : base === _T_INT ? (idx.length === 2 ? _T_IVEC2 : idx.length === 3 ? _T_IVEC3 : _T_IVEC4)
        : (idx.length === 2 ? _T_VEC2 : idx.length === 3 ? _T_VEC3 : _T_VEC4);
      return { k: kk, c: cs };
    }
    throw new Error('swizzle on unsupported type');
  }
  function _zwIndexRead(v, i) {
    if (typeof v === 'number') throw new Error('index on scalar');
    if (v && v.__arr) { // uniform 数组：{__arr, els}
      if (i < 0 || i >= v.els.length) throw new Error('array index out of range');
      return v.els[i];
    }
    if (_zwTIsVec(v.k)) {
      if (i < 0 || i >= v.c.length) throw new Error('vector index out of range');
      return v.c[i];
    }
    if (_zwTIsMat(v.k)) {
      var n = _zwTMatN(v.k);
      if (i < 0 || i >= n) throw new Error('matrix index out of range');
      return { k: _zwTVecOf(n), c: v.m[i] };
    }
    throw new Error('index on unsupported type');
  }

  function _zwMapCw(v, f) { // 逐分量单参
    if (typeof v === 'number') return f(v);
    if (_zwTIsVec(v.k)) {
      var cs = [], i;
      for (i = 0; i < v.c.length; i++) cs.push(f(v.c[i]));
      return { k: v.k, c: cs };
    }
    if (_zwTIsMat(v.k)) {
      var m = [], c, r, n = _zwTMatN(v.k);
      for (c = 0; c < n; c++) { m.push([]); for (r = 0; r < n; r++) m[c].push(f(v.m[c][r])); }
      return { k: v.k, m: m };
    }
    throw new Error('component-wise on unsupported type');
  }
  function _zwZip(a, b, f) { // 双参逐分量（标量广播——GLSL ES 1.0 §5.3）
    if (typeof a === 'number' && typeof b === 'number') return f(a, b);
    if (typeof a === 'number' && typeof b === 'object') {
      var t = a; a = b; b = t; // 交换后 a 为聚合
    }
    if (typeof a === 'object' && typeof b === 'number') {
      if (_zwTIsVec(a.k)) {
        var cs = [], i;
        for (i = 0; i < a.c.length; i++) cs.push(f(a.c[i], b));
        return { k: a.k, c: cs };
      }
      if (_zwTIsMat(a.k)) {
        var m = [], c, r, n = _zwTMatN(a.k);
        for (c = 0; c < n; c++) { m.push([]); for (r = 0; r < n; r++) m[c].push(f(a.m[c][r], b)); }
        return { k: a.k, m: m };
      }
    }
    if (typeof a === 'object' && typeof b === 'object' && a.k === b.k) {
      if (_zwTIsVec(a.k)) {
        var cs2 = [], j;
        for (j = 0; j < a.c.length; j++) cs2.push(f(a.c[j], b.c[j]));
        return { k: a.k, c: cs2 };
      }
      if (_zwTIsMat(a.k)) {
        var m2 = [], cc, rr, n2 = _zwTMatN(a.k);
        for (cc = 0; cc < n2; cc++) {
          m2.push([]);
          for (rr = 0; rr < n2; rr++) m2[cc].push(f(a.m[cc][rr], b.m[cc][rr]));
        }
        return { k: a.k, m: m2 };
      }
    }
    throw new Error('operand shape mismatch');
  }
  function _zwMul(a, b) { // *：分量乘 + 矩阵代数（列主序）
    if (typeof a === 'number' && typeof b === 'number') return a * b;
    if (typeof a === 'number' || typeof b === 'number') return _zwZip(a, b, function (x, y) { return x * y; });
    if (_zwTIsVec(a.k) && _zwTIsVec(b.k)) {
      if (a.c.length !== b.c.length) throw new Error('vec*vec width mismatch');
      return _zwZip(a, b, function (x, y) { return x * y; });
    }
    if (_zwTIsMat(a.k) && _zwTIsVec(b.k)) { // mat*vec：result_i = Σ_j m[j][i]*v[j]
      var n = _zwTMatN(a.k), out = [], i, j;
      if (b.c.length !== n) throw new Error('mat*vec width mismatch');
      for (i = 0; i < n; i++) {
        var s = 0;
        for (j = 0; j < n; j++) s += a.m[j][i] * b.c[j];
        out.push(s);
      }
      return { k: _zwTVecOf(n), c: out };
    }
    if (_zwTIsVec(a.k) && _zwTIsMat(b.k)) { // vec*mat：result_i = Σ_j v[j]*m[i][j]
      var n2 = _zwTMatN(b.k), out2 = [], i2, j2;
      if (a.c.length !== n2) throw new Error('vec*mat width mismatch');
      for (i2 = 0; i2 < n2; i2++) {
        var s2 = 0;
        for (j2 = 0; j2 < n2; j2++) s2 += a.c[j2] * b.m[i2][j2];
        out2.push(s2);
      }
      return { k: _zwTVecOf(n2), c: out2 };
    }
    if (_zwTIsMat(a.k) && _zwTIsMat(b.k)) { // mat*mat：result[i][j] = Σ_k a[i][k]*b[k][j]
      var n3 = _zwTMatN(a.k);
      if (_zwTMatN(b.k) !== n3) throw new Error('mat*mat size mismatch');
      var m3 = [], c3, r3, k3;
      for (c3 = 0; c3 < n3; c3++) { // 结果第 c3 列
        var col = [];
        for (r3 = 0; r3 < n3; r3++) {
          var s3 = 0;
          for (k3 = 0; k3 < n3; k3++) s3 += a.m[k3][r3] * b.m[c3][k3];
          col.push(s3);
        }
        m3.push(col);
      }
      return { k: a.k, m: m3 };
    }
    throw new Error('operand shape mismatch');
  }
  function _zwValEq(a, b) { // ==/!= 深比较（分量级）
    if (typeof a === 'number' && typeof b === 'number') return a === b;
    if (typeof a === 'object' && typeof b === 'object' && a.k === b.k) {
      var i;
      if (_zwTIsVec(a.k)) {
        if (a.c.length !== b.c.length) return false;
        for (i = 0; i < a.c.length; i++) if (a.c[i] !== b.c[i]) return false;
        return true;
      }
      if (_zwTIsMat(a.k)) {
        var n = _zwTMatN(a.k), c, r;
        for (c = 0; c < n; c++) for (r = 0; r < n; r++) if (a.m[c][r] !== b.m[c][r]) return false;
        return true;
      }
    }
    return false;
  }

  function _zwStepVal(v, d) { // ++/-- 逐分量 ±1（GLSL ES 1.0 §5.9，作用于标量/向量/矩阵）
    if (typeof v === 'number') return v + d;
    if (typeof v === 'object') {
      if (_zwTIsVec(v.k)) return { k: v.k, c: v.c.map(function (x) { return x + d; }) };
      if (_zwTIsMat(v.k)) {
        return { k: v.k, m: v.m.map(function (col) { return col.map(function (x) { return x + d; }); }) };
      }
    }
    throw new Error('increment on non-numeric');
  }

  function _zwEvalExpr(n, env, ctx) {
    if (++ctx.steps > _ZW_STEP_BUDGET) throw { budget: true };
    switch (n.t) {
      case 'num': return n.v;
      case 'var': {
        var v = _zwLookup(env, n.name);
        if (v === undefined) throw new Error('unbound variable ' + n.name);
        return v;
      }
      case 'sel': return _zwSwizzleRead(_zwEvalExpr(n.e, env, ctx), n.sel);
      case 'idx': return _zwIndexRead(_zwEvalExpr(n.e, env, ctx), _zwEvalExpr(n.i, env, ctx) | 0);
      case 'un': {
        var u = _zwEvalExpr(n.e, env, ctx);
        if (n.op === '+') return u;
        if (n.op === '!') return _zwTruth(u) ? 0 : 1;
        return _zwMapCw(u, function (x) { return -x; });
      }
      case 'bin': {
        var op = n.op;
        if (op === '&&') {
          return _zwTruth(_zwEvalExpr(n.l, env, ctx)) && _zwTruth(_zwEvalExpr(n.r, env, ctx)) ? 1 : 0;
        }
        if (op === '||') {
          return _zwTruth(_zwEvalExpr(n.l, env, ctx)) || _zwTruth(_zwEvalExpr(n.r, env, ctx)) ? 1 : 0;
        }
        var a = _zwEvalExpr(n.l, env, ctx);
        var b = _zwEvalExpr(n.r, env, ctx);
        if (op === '+') return _zwZip(a, b, function (x, y) { return x + y; });
        if (op === '-') return _zwZip(a, b, function (x, y) { return x - y; });
        if (op === '*') return _zwMul(a, b);
        if (op === '/') return _zwZip(a, b, function (x, y) { return x / y; });
        if (op === '<') return a < b ? 1 : 0;
        if (op === '>') return a > b ? 1 : 0;
        if (op === '<=') return a <= b ? 1 : 0;
        if (op === '>=') return a >= b ? 1 : 0;
        if (op === '==') return _zwValEq(a, b) ? 1 : 0;
        if (op === '!=') return _zwValEq(a, b) ? 0 : 1;
        throw new Error('unsupported operator ' + op);
      }
      case 'cond':
        return _zwTruth(_zwEvalExpr(n.c, env, ctx))
          ? _zwEvalExpr(n.a, env, ctx)
          : _zwEvalExpr(n.b, env, ctx);
      case 'pre': case 'post': { // 自增/自减：前置返回新值，后缀返回旧值（§5.9）
        var old = _zwEvalExpr(n.target, env, ctx);
        var nv = _zwStepVal(old, n.op === '++' ? 1 : -1);
        _zwStoreLValue(env, n.target, nv);
        return n.t === 'pre' ? nv : old;
      }
      case 'assign': return _zwEvalAssign(n, env, ctx);
      case 'call':
        return n.ctor !== undefined ? _zwEvalCtor(n.ctor, n.args, env, ctx) : _zwEvalCall(n, env, ctx);
      default:
        throw new Error('unsupported expression ' + n.t);
    }
  }

  function _zwStoreLValue(env, target, val) {
    if (target.t === 'var') {
      var sc = _zwScopeOf(env, target.name);
      if (!sc) throw new Error('unbound variable ' + target.name);
      sc[target.name] = val;
      return;
    }
    if (target.t === 'sel') { // 向量分量写：宿主必须具名变量（FIXME 嵌套左值）
      if (target.e.t !== 'var') throw new Error('unsupported assignment target');
      var v = _zwLookup(env, target.e.name);
      if (typeof v !== 'object' || !_zwTIsVec(v.k)) throw new Error('swizzle write on non-vector');
      var idx = _zwSwizIdx(target.sel, true); // 写通道须互异（GLSL ES 1.0 §5.5）
      var w = typeof val === 'number' ? 1 : _zwTWidth(val.k);
      if (idx.length !== w) throw new Error('swizzle write width mismatch');
      for (var i = 0; i < idx.length; i++) v.c[idx[i]] = typeof val === 'number' ? val : val.c[i];
      return;
    }
    if (target.t === 'idx') {
      if (target.e.t !== 'var') throw new Error('unsupported assignment target');
      var vv = _zwLookup(env, target.e.name);
      var ix = _zwEvalExpr(target.i, env, ctx) | 0;
      if (typeof vv === 'object' && vv.__arr) {
        if (ix < 0 || ix >= vv.els.length) throw new Error('array index out of range');
        vv.els[ix] = val;
        return;
      }
      if (typeof vv === 'object' && _zwTIsVec(vv.k)) {
        if (ix < 0 || ix >= vv.c.length) throw new Error('vector index out of range');
        vv.c[ix] = typeof val === 'number' ? val : val.c[0];
        return;
      }
      throw new Error('index write on unsupported type');
    }
    throw new Error('unsupported assignment target');
  }
  function _zwEvalAssign(n, env, ctx) {
    var val;
    if (n.op === '=') {
      val = _zwEvalExpr(n.value, env, ctx);
    } else { // 复合赋值：读改写
      var cur = _zwEvalExpr(n.target, env, ctx);
      var rhs = _zwEvalExpr(n.value, env, ctx);
      val = n.op === '+=' ? _zwZip(cur, rhs, function (x, y) { return x + y; })
        : n.op === '-=' ? _zwZip(cur, rhs, function (x, y) { return x - y; })
        : n.op === '*=' ? _zwMul(cur, rhs)
        : _zwZip(cur, rhs, function (x, y) { return x / y; });
    }
    _zwStoreLValue(env, n.target, val);
    return val;
  }

  function _zwEvalCtor(k, args, env, ctx) {
    if (_zwTSampler(k)) return 0; // sampler 构造占位（采样未实现——FIXME 头注）
    var flat = [], i, v;
    for (i = 0; i < args.length; i++) {
      v = _zwEvalExpr(args[i], env, ctx);
      if (typeof v === 'number') flat.push(v);
      else if (_zwTIsVec(v.k)) flat.push.apply(flat, v.c);
      else throw new Error('bad constructor argument');
    }
    if (_zwTIsMat(k)) {
      var n = _zwTMatN(k), m = [], c, r;
      if (flat.length === 1) { // 单标量 → 对角阵
        var s = flat[0];
        for (c = 0; c < n; c++) {
          m.push([]);
          for (r = 0; r < n; r++) m[c].push(c === r ? s : 0);
        }
        return { k: k, m: m };
      }
      if (flat.length !== n * n) throw new Error('matrix constructor arity'); // 列主序扁平
      for (c = 0; c < n; c++) {
        m.push([]);
        for (r = 0; r < n; r++) m[c].push(flat[c * n + r]);
      }
      return { k: k, m: m };
    }
    var w = _zwTWidth(k);
    if (flat.length === 1) { var s0 = flat[0]; flat = []; for (i = 0; i < w; i++) flat.push(s0); }
    if (flat.length !== w) throw new Error('constructor arity');
    if (w === 1) return flat[0];
    return { k: k, c: flat };
  }

  // ── 内建函数库（GLSL ES 1.0 §8 子集——part08b _ZW_GLSL_BUILTINS 名集）──
  function _zwCallCw1(a, f) { return _zwMapCw(a, f); }
  function _zwCallCw2(a, b, f) { return _zwZip(a, b, f); }
  function _zwNum(v) { return typeof v === 'number' ? v : 0; }
  function _zwFlat(v) { // genType 参数 → 分量数组
    if (typeof v === 'number') return [v];
    if (_zwTIsVec(v.k)) return v.c.slice();
    throw new Error('bad builtin argument');
  }
  var _ZW_BUILTIN_IMPL = {
    radians: function (a) { return _zwCallCw1(a[0], function (x) { return x * Math.PI / 180; }); },
    degrees: function (a) { return _zwCallCw1(a[0], function (x) { return x * 180 / Math.PI; }); },
    sin: function (a) { return _zwCallCw1(a[0], Math.sin); },
    cos: function (a) { return _zwCallCw1(a[0], Math.cos); },
    tan: function (a) { return _zwCallCw1(a[0], Math.tan); },
    asin: function (a) { return _zwCallCw1(a[0], Math.asin); },
    acos: function (a) { return _zwCallCw1(a[0], Math.acos); },
    atan: function (a) {
      return a.length > 1 ? _zwCallCw2(a[0], a[1], Math.atan2)
        : _zwCallCw1(a[0], Math.atan);
    },
    pow: function (a) { return _zwCallCw2(a[0], a[1], Math.pow); },
    exp: function (a) { return _zwCallCw1(a[0], Math.exp); },
    log: function (a) { return _zwCallCw1(a[0], Math.log); },
    exp2: function (a) { return _zwCallCw1(a[0], function (x) { return Math.pow(2, x); }); },
    log2: function (a) { return _zwCallCw1(a[0], Math.log2); },
    sqrt: function (a) { return _zwCallCw1(a[0], Math.sqrt); },
    inversesqrt: function (a) { return _zwCallCw1(a[0], function (x) { return 1 / Math.sqrt(x); }); },
    abs: function (a) { return _zwCallCw1(a[0], Math.abs); },
    sign: function (a) { return _zwCallCw1(a[0], function (x) { return x > 0 ? 1 : x < 0 ? -1 : 0; }); },
    floor: function (a) { return _zwCallCw1(a[0], Math.floor); },
    ceil: function (a) { return _zwCallCw1(a[0], Math.ceil); },
    fract: function (a) { return _zwCallCw1(a[0], function (x) { return x - Math.floor(x); }); },
    mod: function (a) { // GLSL mod：x - y*floor(x/y)
      return _zwCallCw2(a[0], a[1], function (x, y) { return x - y * Math.floor(x / y); });
    },
    min: function (a) { return _zwCallCw2(a[0], a[1], Math.min); },
    max: function (a) { return _zwCallCw2(a[0], a[1], Math.max); },
    clamp: function (a) { // clamp(x, minVal, maxVal)：genType 或标量 min/max
      var x = a[0], lo = a[1], hi = a[2];
      var f = function (v, l, h) { return Math.min(Math.max(v, l), h); };
      if (typeof lo === 'number' && typeof hi === 'number') return _zwCallCw1(x, function (v) { return f(v, lo, hi); });
      var xs = _zwFlat(x), ls = _zwFlat(lo), hs = _zwFlat(hi), out = [], i;
      for (i = 0; i < xs.length; i++) out.push(f(xs[i], ls[i], hs[i]));
      return typeof x === 'number' ? out[0] : { k: x.k, c: out };
    },
    mix: function (a) {
      var x = a[0], y = a[1], al = a[2];
      if (typeof al === 'number') {
        return _zwZip(x, y, function (u, v) { return u * (1 - al) + v * al; });
      }
      var xs = _zwFlat(x), ys = _zwFlat(y), as = _zwFlat(al), out = [], i;
      for (i = 0; i < xs.length; i++) out.push(xs[i] * (1 - as[i]) + ys[i] * as[i]);
      return { k: x.k, c: out };
    },
    step: function (a) {
      var e = a[0], x = a[1];
      if (typeof e === 'number') return _zwCallCw1(x, function (v) { return v < e ? 0 : 1; });
      return _zwCallCw2(e, x, function (ev, xv) { return xv < ev ? 0 : 1; });
    },
    smoothstep: function (a) {
      var e0 = _zwFlat(a[0]), e1 = _zwFlat(a[1]), xs = _zwFlat(a[2]), out = [];
      var f = function (e0v, e1v, xv) {
        var t = Math.min(Math.max((xv - e0v) / (e1v - e0v), 0), 1);
        return t * t * (3 - 2 * t);
      };
      for (var i = 0; i < xs.length; i++) out.push(f(e0[i], e1[i], xs[i]));
      return typeof a[2] === 'number' ? out[0] : { k: a[2].k, c: out };
    },
    length: function (a) {
      var c = _zwFlat(a[0]), s = 0, i;
      for (i = 0; i < c.length; i++) s += c[i] * c[i];
      return Math.sqrt(s);
    },
    distance: function (a) {
      var p = _zwFlat(a[0]), q = _zwFlat(a[1]), s = 0, i;
      for (i = 0; i < p.length; i++) { var d = p[i] - q[i]; s += d * d; }
      return Math.sqrt(s);
    },
    dot: function (a) {
      var p = _zwFlat(a[0]), q = _zwFlat(a[1]), s = 0, i;
      for (i = 0; i < p.length; i++) s += p[i] * q[i];
      return s;
    },
    cross: function (a) {
      var p = _zwFlat(a[0]), q = _zwFlat(a[1]);
      return { k: _T_VEC3, c: [
        p[1] * q[2] - p[2] * q[1], p[2] * q[0] - p[0] * q[2], p[0] * q[1] - p[1] * q[0]
      ] };
    },
    normalize: function (a) {
      var c = _zwFlat(a[0]), s = 0, i;
      for (i = 0; i < c.length; i++) s += c[i] * c[i];
      var l = Math.sqrt(s);
      if (l === 0) return a[0]; // 零向量未定义——恒等返回
      var out = [];
      for (i = 0; i < c.length; i++) out.push(c[i] / l);
      return typeof a[0] === 'number' ? out[0] : { k: a[0].k, c: out };
    },
    faceforward: function (a) { // N → dot(Nref, I) < 0 ? N : -N
      var d = _ZW_BUILTIN_IMPL.dot([a[1], a[2]]);
      return d < 0 ? a[0] : _zwMapCw(a[0], function (x) { return -x; });
    },
    reflect: function (a) { // I - 2*dot(N, I)*N
      var d = _ZW_BUILTIN_IMPL.dot([a[1], a[0]]);
      return _zwZip(a[0], _zwMapCw(a[1], function (x) { return x * 2 * d; }),
        function (x, y) { return x - y; });
    },
    refract: function (a) { // k = 1 - eta²(1-dot(N,I)²)；k<0 → 0 向量
      var iv = _zwFlat(a[0]), nv = _zwFlat(a[1]), eta = _zwNum(a[2]);
      var d = 0, i;
      for (i = 0; i < iv.length; i++) d += nv[i] * iv[i];
      var k = 1 - eta * eta * (1 - d * d);
      if (k < 0) {
        var z = [];
        for (i = 0; i < iv.length; i++) z.push(0);
        return typeof a[0] === 'number' ? 0 : { k: a[0].k, c: z };
      }
      var f = eta * d + Math.sqrt(k), out = [];
      for (i = 0; i < iv.length; i++) out.push(eta * iv[i] - f * nv[i]);
      return { k: a[0].k, c: out };
    },
    matrixCompMult: function (a) { return _zwCallCw2(a[0], a[1], function (x, y) { return x * y; }); },
    // 采样未实现（FIXME 头注）——确定值：不透明黑
    texture2D: function () { return { k: _T_VEC4, c: [0, 0, 0, 1] }; },
    texture2DProj: function () { return { k: _T_VEC4, c: [0, 0, 0, 1] }; },
    textureCube: function () { return { k: _T_VEC4, c: [0, 0, 0, 1] }; },
    lessThan: function (a) { return _zwBvecCmp(a, function (x, y) { return x < y; }); },
    lessThanEqual: function (a) { return _zwBvecCmp(a, function (x, y) { return x <= y; }); },
    greaterThan: function (a) { return _zwBvecCmp(a, function (x, y) { return x > y; }); },
    greaterThanEqual: function (a) { return _zwBvecCmp(a, function (x, y) { return x >= y; }); },
    equal: function (a) { return _zwBvecCmp(a, function (x, y) { return x === y; }); },
    notEqual: function (a) { return _zwBvecCmp(a, function (x, y) { return x !== y; }); },
    any: function (a) {
      var c = _zwFlat(a[0]), i;
      for (i = 0; i < c.length; i++) if (c[i]) return 1;
      return 0;
    },
    all: function (a) {
      var c = _zwFlat(a[0]), i;
      for (i = 0; i < c.length; i++) if (!c[i]) return 0;
      return 1;
    },
    not: function (a) { return _zwMapCw(a[0], function (x) { return x ? 0 : 1; }); }
  };
  function _zwBvecCmp(a, f) {
    var p = _zwFlat(a[0]), q = _zwFlat(a[1]), out = [], i;
    for (i = 0; i < p.length; i++) out.push(f(p[i], q[i]) ? 1 : 0);
    var n = p.length, base = a[0].k >= _T_IVEC2 && a[0].k <= _T_IVEC4 ? _T_BVEC2
      : a[0].k >= _T_VEC2 && a[0].k <= _T_VEC4 ? _T_BVEC2 : _T_BOOL;
    if (n === 1) return out[0];
    return { k: base + (n - 2), c: out }; // BVEC2/3/4 连续枚举
  }
  function _zwEvalCall(n, env, ctx) {
    var bi = _ZW_BUILTIN_IMPL[n.fn];
    if (!bi) throw new Error('unsupported function ' + n.fn);
    var args = [], i;
    for (i = 0; i < n.args.length; i++) args.push(_zwEvalExpr(n.args[i], env, ctx));
    return bi(args);
  }

  function _zwExecStmt(st, env, ctx) {
    if (++ctx.steps > _ZW_STEP_BUDGET) throw { budget: true };
    switch (st.st) {
      case 'block': {
        env.scopes.push({});
        for (var i = 0; i < st.body.length; i++) {
          _zwExecStmt(st.body[i], env, ctx);
          if (ctx.ret || ctx.discard || ctx.brk || ctx.cont) break;
        }
        env.scopes.pop();
        return;
      }
      case 'decl': {
        var sc = env.scopes[env.scopes.length - 1];
        for (var j = 0; j < st.list.length; j++) {
          var d = st.list[j];
          sc[d.name] = d.init ? _zwEvalExpr(d.init, env, ctx) : _zwTZero(st.k);
        }
        return;
      }
      case 'expr':
        _zwEvalExpr(st.e, env, ctx);
        return;
      case 'if':
        if (_zwTruth(_zwEvalExpr(st.c, env, ctx))) _zwExecStmt(st.th, env, ctx);
        else if (st.el) _zwExecStmt(st.el, env, ctx);
        return;
      case 'for': {
        env.scopes.push({});
        try {
          if (st.init) _zwExecStmt(st.init, env, ctx);
          var guard = 0;
          while (guard++ < 65536) {
            if (st.cond && !_zwTruth(_zwEvalExpr(st.cond, env, ctx))) break;
            _zwExecStmt(st.body, env, ctx);
            if (ctx.ret || ctx.discard) break;
            if (ctx.brk) { ctx.brk = false; break; }
            ctx.cont = false; // continue → 仍执行第 3 节
            if (st.nx) _zwEvalExpr(st.nx, env, ctx);
          }
          if (guard > 65536) throw new Error('for-loop iteration cap');
        } finally {
          env.scopes.pop();
        }
        return;
      }
      case 'ret':
        ctx.retV = st.e ? _zwEvalExpr(st.e, env, ctx) : undefined;
        ctx.ret = true;
        return;
      case 'discard': ctx.discard = true; return;
      case 'brk': ctx.brk = true; return;
      case 'cont': ctx.cont = true; return;
      default:
        throw new Error('unsupported statement ' + st.st);
    }
  }

  // ── draw 管线 ───────────────────────────────────────────────────────────
  // GLES2 §2.8 VertexAttribPointer 数据类型域（INT/UNSIGNED_INT 不在表内；
  // FIXED 为 16.16 定点 4 字节）。字节序按宿主小端（FIXME 记账——客户端数组
  // 字节序 GLES2 未规定，常驻测试/reftest 自产数据两侧一致）。
  var _ZW_VATTR_TYPES = {};
  _ZW_VATTR_TYPES[0x1400] = 1; // BYTE
  _ZW_VATTR_TYPES[0x1401] = 1; // UNSIGNED_BYTE
  _ZW_VATTR_TYPES[0x1402] = 2; // SHORT
  _ZW_VATTR_TYPES[0x1403] = 2; // UNSIGNED_SHORT
  _ZW_VATTR_TYPES[0x1406] = 4; // FLOAT
  _ZW_VATTR_TYPES[0x140C] = 4; // FIXED

  var _ZW_AB = 0x8892;       // ARRAY_BUFFER（pin rev IDL 值序——part08a 同源）
  var _ZW_EAB = 0x8893;      // ELEMENT_ARRAY_BUFFER
  var _ZW_FBO = 0x8D40;      // FRAMEBUFFER

  function _zwDrawProgOf(prog) { // 绘制 AST 缓存（重链时 part08 置 null 失效）
    var dp = prog._zwDrawProg;
    if (dp) return dp;
    var vs = null, fs = null, sh = prog._zwShaders || [], i;
    for (i = 0; i < sh.length; i++) {
      if (sh[i]._zwType === 0x8B31) vs = sh[i];
      else if (sh[i]._zwType === 0x8B30) fs = sh[i];
    }
    dp = {
      vs: _zwGlParseDraw(vs && vs._zwSource || ''),
      fs: _zwGlParseDraw(fs && fs._zwSource || '')
    };
    dp.ok = dp.vs.ok && dp.fs.ok;
    prog._zwDrawProg = dp;
    return dp;
  }

  function _zwUniformElement(k, arr, off) {
    if (_zwTIsMat(k)) {
      var n = _zwTMatN(k), m = [], c, r;
      for (c = 0; c < n; c++) {
        m.push([]);
        for (r = 0; r < n; r++) m[c].push(arr[off + c * n + r]);
      }
      return { k: k, m: m };
    }
    if (_zwTIsVec(k)) {
      var w = _zwTWidth(k), cs = [], i;
      for (i = 0; i < w; i++) cs.push(arr[off + i]);
      return { k: k, c: cs };
    }
    return arr[off];
  }
  function _zwUniformValues(link) {
    var out = {};
    for (var i = 0; i < link.uniformList.length; i++) {
      var ent = link.uniformList[i];
      var run = link.uniformRuns[ent.index];
      if (!run) continue;
      var arr = run.arr;
      if (!arr) arr = run.arr = new Float32Array(ent.size * run.comps); // 未加载 → 零
      if (ent.size > 1 || ent.isArray) { // 数组 uniform：{__arr, els} 支持下标读
        var els = [], e;
        for (e = 0; e < ent.size; e++) els.push(_zwUniformElement(ent.type, arr, e * run.comps));
        out[ent.name] = { __arr: true, els: els };
        out[ent.name + '[0]'] = { __arr: true, els: els };
      } else {
        out[ent.name] = _zwUniformElement(ent.type, arr, 0);
      }
    }
    return out;
  }

  function _zwFetchAttrib(dv, va, typeSize, elem) { // 顶点数据 → float vec4
    var stride = va.stride || va.size * typeSize;
    var base = va.offset + elem * stride;
    var c = [0, 0, 0, 1], i;
    for (i = 0; i < va.size; i++) {
      var o = base + i * typeSize, v = 0;
      switch (va.type) {
        case 0x1406: v = dv.getFloat32(o, true); break;
        case 0x1400: v = dv.getInt8(o); break;
        case 0x1401: v = dv.getUint8(o); if (va.normalized) v /= 255; break;
        case 0x1402: v = dv.getInt16(o, true); if (va.normalized) v /= 32767; break;
        case 0x1403: v = dv.getUint16(o, true); if (va.normalized) v /= 65535; break;
        case 0x140C: v = dv.getInt32(o, true) / 65536; break; // 16.16 定点
      }
      c[i] = v;
    }
    return { k: _T_VEC4, c: c };
  }

  // 全局 scope 构建：attribute/uniform/varying/const/普通全局按声明序落位；
  // const/普通全局的初始化式按序求值（可引用更早声明）。
  function _zwBuildScope(prog, inputs) {
    var g = {};
    var env = { scopes: [g] };
    for (var i = 0; i < prog.globals.length; i++) {
      var d = prog.globals[i];
      var v = _zwTZero(d.k);
      if (d.storage === 'attribute') v = inputs.attribs[d.name] !== undefined ? inputs.attribs[d.name] : v;
      else if (d.storage === 'uniform') v = inputs.uniforms[d.name] !== undefined ? inputs.uniforms[d.name] : v;
      else if (d.storage === 'varying') v = inputs.varyings && inputs.varyings[d.name] !== undefined
        ? inputs.varyings[d.name] : v;
      g[d.name] = v;
      if (d.init) {
        g[d.name] = _zwEvalExpr(d.init, env, inputs.ctx);
      }
    }
    return g;
  }

  function _zwRunVertex(dp, g, shared) {
    var env = { scopes: [g, {}] };
    g.gl_Position = { k: _T_VEC4, c: [0, 0, 0, 1] };
    g.gl_PointSize = 1;
    g.gl_FrontFacing = 1;
    shared.ret = false; shared.discard = false; shared.brk = false; shared.cont = false;
    _zwExecStmt(dp.vs.main, env, shared);
    var pos = g.gl_Position;
    if (typeof pos !== 'object' || !_zwTIsVec(pos.k) || pos.c.length !== 4) {
      throw new Error('gl_Position not written as vec4');
    }
    var vary = {};
    for (var i = 0; i < dp.vs.varyings.length; i++) {
      var nm = dp.vs.varyings[i];
      vary[nm] = g[nm];
    }
    return { pos: pos, vary: vary };
  }

  function _zwRunFragment(dp, g, shared) {
    var env = { scopes: [g, {}] };
    g.gl_FragColor = { k: _T_VEC4, c: [0, 0, 0, 0] };
    g.gl_FrontFacing = 1;
    g.gl_PointCoord = { k: _T_VEC2, c: [0.5, 0.5] };
    shared.ret = false; shared.discard = false; shared.brk = false; shared.cont = false;
    _zwExecStmt(dp.fs.main, env, shared);
    if (shared.discard) return null;
    var col = g.gl_FragColor;
    if (typeof col === 'object' && _zwTIsVec(col.k)) {
      return [
        Math.min(Math.max(col.c[0], 0), 1), Math.min(Math.max(col.c[1], 0), 1),
        Math.min(Math.max(col.c[2], 0), 1), Math.min(Math.max(col.c[3] === undefined ? 1 : col.c[3], 0), 1)
      ];
    }
    return [0, 0, 0, 1];
  }

  // host 位图读底（getImageData 线格式 "W:H;r,g,b,a,..." 顶行起行主序）；
  // host 未注册（纯 shim 单测环境）→ 全零底（初始 drawing buffer 语义）。
  function _zwSurfaceOf(state) {
    var W = state._drawingBufferWidth, Hh = state._drawingBufferHeight;
    var pix = null;
    if (typeof globalThis.__zw_canvas_op === 'function' && state._handle && W > 0 && Hh > 0) {
      var wire = globalThis.__zw_canvas_op(state._handle, 'getImageData',
        '0', '0', String(W), String(Hh));
      pix = _zwParseWire(wire, W, Hh);
    }
    if (!pix) pix = new Uint8ClampedArray(Math.max(W, 0) * Math.max(Hh, 0) * 4);
    return { W: W, H: Hh, pix: pix };
  }
  function _zwParseWire(wire, W, Hh) {
    if (typeof wire !== 'string') return null;
    var sc = wire.indexOf(';');
    if (sc <= 0) return null;
    var dims = wire.slice(0, sc).split(':');
    if (+dims[0] !== W || +dims[1] !== Hh) return null;
    var parts = wire.slice(sc + 1).split(',');
    var out = new Uint8ClampedArray(W * Hh * 4);
    var n = Math.min(parts.length, out.length);
    for (var i = 0; i < n; i++) out[i] = +parts[i];
    return out;
  }

  // GL 底左视口 → host 顶左像素坐标；返回图元顶点 {x, y, w, vary}（host 浮点坐标）。
  function _zwProject(state, Hh, vout, uni) {
    var vp = state._viewport;
    var w = vout.pos.c[3];
    var ndcX = vout.pos.c[0] / w, ndcY = vout.pos.c[1] / w;
    var x = vp[0] + (ndcX * 0.5 + 0.5) * vp[2];
    var yGL = vp[1] + (ndcY * 0.5 + 0.5) * vp[3];
    return { x: x, y: Hh - yGL, w: w, vary: vout.vary, uni: uni };
  }

  function _zwWritePix(sf, scis, cm, px, py, col) {
    if (px < 0 || py < 0 || px >= sf.W || py >= sf.H) return;
    if (scis) {
      if (px < scis[0] || py < scis[1] || px >= scis[0] + scis[2] || py >= scis[1] + scis[3]) return;
    }
    var o = (py * sf.W + px) * 4;
    if (cm[0]) sf.pix[o] = Math.round(col[0] * 255);
    if (cm[1]) sf.pix[o + 1] = Math.round(col[1] * 255);
    if (cm[2]) sf.pix[o + 2] = Math.round(col[2] * 255);
    if (cm[3]) sf.pix[o + 3] = Math.round(col[3] * 255);
  }

  // 重心坐标变化量插值（透视校正：Σ(a_i·λ_i/w_i)/Σ(λ_i/w_i)——GLSL ES 1.0
  // §5.6 varying 语义）。标量/向量逐分量。
  function _zwInterp(a, b, c, l0, l1, l2, iw0, iw1, iw2) {
    var s = iw0 + iw1 + iw2;
    function mix3(va, vb, vc) {
      if (typeof va === 'number') {
        return (va * iw0 + vb * iw1 + vc * iw2) / s;
      }
      var cs = [], i;
      for (i = 0; i < va.c.length; i++) {
        cs.push((va.c[i] * iw0 + vb.c[i] * iw1 + vc.c[i] * iw2) / s);
      }
      return { k: va.k, c: cs };
    }
    return mix3(a, b, c);
  }

  // FS varying 零值表：FS 声明而 VS 未写（链路失配）的 varying 按声明类型取零。
  function _zwFsVaryZeros(dp) {
    var z = {};
    for (var i = 0; i < dp.fs.globals.length; i++) {
      var d = dp.fs.globals[i];
      if (d.storage === 'varying') z[d.name] = _zwTZero(d.k);
    }
    return z;
  }

  function _zwRasterTri(dp, sf, scis, cm, tri, shared) {
    var v0 = tri[0], v1 = tri[1], v2 = tri[2];
    var minX = Math.max(0, Math.floor(Math.min(v0.x, v1.x, v2.x)));
    var maxX = Math.min(sf.W - 1, Math.ceil(Math.max(v0.x, v1.x, v2.x)));
    var minY = Math.max(0, Math.floor(Math.min(v0.y, v1.y, v2.y)));
    var maxY = Math.min(sf.H - 1, Math.ceil(Math.max(v0.y, v1.y, v2.y)));
    if (scis) { // host 系裁剪矩形相交
      minX = Math.max(minX, scis[0]); minY = Math.max(minY, scis[1]);
      maxX = Math.min(maxX, scis[0] + scis[2] - 1); maxY = Math.min(maxY, scis[1] + scis[3] - 1);
    }
    if (minX > maxX || minY > maxY) return;
    var area = (v1.x - v0.x) * (v2.y - v0.y) - (v2.x - v0.x) * (v1.y - v0.y);
    if (area === 0) return; // 退化图元
    var varyNames = dp.fs.varyings, vz = _zwFsVaryZeros(dp), vi;
    for (var py = minY; py <= maxY; py++) {
      for (var px = minX; px <= maxX; px++) {
        var cx = px + 0.5, cy = py + 0.5;
        // 边函数（与 area 同号判内——双向环绕通吃；像素中心采样，
        // 顶左填充规则未实现 FIXME——reftest 标尺为自绘孪生，不依赖）
        var l0 = ((v2.x - v1.x) * (cy - v1.y) - (v2.y - v1.y) * (cx - v1.x)) / area;
        var l1 = ((v0.x - v2.x) * (cy - v2.y) - (v0.y - v2.y) * (cx - v2.x)) / area;
        var l2 = 1 - l0 - l1;
        if (l0 < 0 || l1 < 0 || l2 < 0) continue;
        var iw0 = l0 / v0.w, iw1 = l1 / v1.w, iw2 = l2 / v2.w;
        var g = _zwBuildScope(dp.fs, {
          uniforms: v0.uni, attribs: {}, varyings: null, ctx: shared
        });
        for (vi = 0; vi < varyNames.length; vi++) {
          var nm = varyNames[vi];
          var va = v0.vary[nm] !== undefined ? v0.vary[nm] : vz[nm];
          var vb = v1.vary[nm] !== undefined ? v1.vary[nm] : vz[nm];
          var vc = v2.vary[nm] !== undefined ? v2.vary[nm] : vz[nm];
          g[nm] = _zwInterp(va, vb, vc, l0, l1, l2, iw0, iw1, iw2);
        }
        g.gl_FragCoord = { k: _T_VEC4, c: [cx, sf.H - cy, 0, 1] }; // GL 窗口系（底左原点）
        var col = _zwRunFragment(dp, g, shared);
        if (col) _zwWritePix(sf, scis, cm, px, py, col);
        if (shared.aborted) return;
      }
    }
  }

  function _zwRasterLine(dp, sf, scis, cm, a, b, shared) {
    var steps = Math.max(1, Math.ceil(Math.max(Math.abs(b.x - a.x), Math.abs(b.y - a.y))));
    var varyNames = dp.fs.varyings, vz = _zwFsVaryZeros(dp), vi;
    for (var i = 0; i <= steps; i++) {
      if (shared.aborted) return;
      var t = i / steps;
      var iw0 = (1 - t) / a.w, iw1 = t / b.w, s = iw0 + iw1;
      var x = a.x * (1 - t) + b.x * t, y = a.y * (1 - t) + b.y * t;
      var px = Math.floor(x), py = Math.floor(y);
      var g = _zwBuildScope(dp.fs, { uniforms: a.uni, attribs: {}, varyings: null, ctx: shared });
      for (vi = 0; vi < varyNames.length; vi++) {
        var nm = varyNames[vi];
        var va = a.vary[nm] !== undefined ? a.vary[nm] : vz[nm];
        var vb = b.vary[nm] !== undefined ? b.vary[nm] : vz[nm];
        g[nm] = _zwInterp(va, vb, vb, 1 - t, t, 0, iw0, iw1, 0);
      }
      g.gl_FragCoord = { k: _T_VEC4, c: [x, sf.H - y, 0, 1] };
      var col = _zwRunFragment(dp, g, shared);
      if (col) _zwWritePix(sf, scis, cm, px, py, col);
    }
  }

  function _zwRasterPoint(dp, sf, scis, cm, v, shared) {
    var g = _zwBuildScope(dp.fs, { uniforms: v.uni, attribs: {}, varyings: null, ctx: shared });
    var varyNames = dp.fs.varyings, vz = _zwFsVaryZeros(dp), vi;
    for (vi = 0; vi < varyNames.length; vi++) {
      var nm = varyNames[vi];
      g[nm] = v.vary[nm] !== undefined ? v.vary[nm] : vz[nm];
    }
    g.gl_FragCoord = { k: _T_VEC4, c: [Math.floor(v.x) + 0.5, sf.H - v.y - 0.5, 0, 1] };
    var col = _zwRunFragment(dp, g, shared);
    if (col) _zwWritePix(sf, scis, cm, Math.floor(v.x), Math.floor(v.y), col);
  }

  // 图元装配（GLES2 §2.8）：返回顶点槽位三元组/二元组/单元组。
  function _zwAssemble(mode, n, get) {
    var prims = [], i;
    if (mode === 0) { for (i = 0; i < n; i++) prims.push([get(i)]); } // POINTS
    else if (mode === 1) { // LINES
      for (i = 0; i + 1 < n; i += 2) prims.push([get(i), get(i + 1)]);
    } else if (mode === 2) { // LINE_LOOP
      for (i = 0; i + 1 < n; i++) prims.push([get(i), get(i + 1)]);
      if (n > 2) prims.push([get(n - 1), get(0)]);
    } else if (mode === 3) { // LINE_STRIP
      for (i = 0; i + 1 < n; i++) prims.push([get(i), get(i + 1)]);
    } else if (mode === 4) { // TRIANGLES
      for (i = 0; i + 2 < n; i += 3) prims.push([get(i), get(i + 1), get(i + 2)]);
    } else if (mode === 5) { // TRIANGLE_STRIP
      for (i = 0; i + 2 < n; i++) prims.push([get(i), get(i + 1), get(i + 2)]);
    } else if (mode === 6) { // TRIANGLE_FAN
      for (i = 1; i + 1 < n; i++) prims.push([get(0), get(i), get(i + 1)]);
    }
    return prims;
  }

  // 顶点数据取回值按声明 attribute 类型收窄（取回恒为 float vec4——
  // GLES2 §2.8 表：分量不足补 (0,0,1)；matN attribute 占多槽位未支持 → null）。
  function _zwCoerceAttrib(v, k) {
    if (_zwTIsVec(k)) {
      if (v.k === k) return v;
      var w = _zwTWidth(k), cs = [], i;
      for (i = 0; i < w; i++) cs.push(v.c[i]);
      return { k: k, c: cs };
    }
    if (_zwTIsMat(k) || _zwTSampler(k)) return null;
    return v.c[0]; // float/int/bool 标量
  }

  // 绘制共享尾段：逐顶点求值（缓存）→ 图元装配 → w≤0 整图元丢弃（FIXME 完整
  // 裁剪）→ 光栅 → 呈现。getElem(slot) → 顶点数据元素下标（arrays 直通、
  // elements 经索引表）；求值/预算异常 → 弃本次绘制（已算像素不呈现）。
  function _zwExecute(state, dp, mode, count, dvs, uni, getElem) {
    var scis = null;
    if (state._enabled[0x0C11]) { // SCISSOR_TEST → host 系矩形（clear 同款换算）
      var s = state._scissor, Hh0 = state._drawingBufferHeight;
      scis = [s[0], Hh0 - s[1] - s[3], s[2], s[3]];
    }
    var shared = { steps: 0, ret: false, discard: false, brk: false, cont: false, aborted: false };
    var list = dp.vsLinkList;   // 活跃 attribute 名（与 dvs 同序——调用侧落位）
    var types = dp.vsLinkTypes; // 同序声明类型
    var vcache = {};
    function shade(slot) {
      var el = getElem(slot);
      if (vcache[el] !== undefined) return vcache[el];
      var g = _zwBuildScope(dp.vs, { uniforms: uni, attribs: {}, varyings: null, ctx: shared });
      for (var ai = 0; ai < dvs.length; ai++) {
        var vv = dvs[ai];
        if (vv.enabled) {
          var fetched = _zwFetchAttrib(vv.dv, vv.va, vv.typeSize, el);
          var coerced = _zwCoerceAttrib(fetched, types[ai]);
          if (coerced === null) { shared.aborted = true; return null; }
          g[list[ai]] = coerced;
        } else {
          var curv = _zwCoerceAttrib({ k: _T_VEC4, c: vv.cur.slice() }, types[ai]);
          if (curv === null) { shared.aborted = true; return null; }
          g[list[ai]] = curv;
        }
      }
      var r;
      try {
        r = _zwRunVertex(dp, g, shared);
      } catch (e) {
        shared.aborted = true; // 求值失败（含步预算超限）→ 弃绘制
        return null;
      }
      vcache[el] = _zwProject(state, state._drawingBufferHeight, r, uni);
      return vcache[el];
    }
    try {
      var prims = _zwAssemble(mode, count, shade);
      if (!shared.aborted && prims.length > 0) {
        var sf = _zwSurfaceOf(state);
        if (sf.W > 0 && sf.H > 0) {
          var cm = state._colorMask;
          for (var i = 0; i < prims.length && !shared.aborted; i++) {
            var pr = prims[i], j, clip = false;
            for (j = 0; j < pr.length; j++) {
              if (!pr[j] || pr[j].w <= 0) { clip = true; break; }
            }
            if (clip) continue;
            if (pr.length === 3) _zwRasterTri(dp, sf, scis, cm, pr, shared);
            else if (pr.length === 2) _zwRasterLine(dp, sf, scis, cm, pr[0], pr[1], shared);
            else _zwRasterPoint(dp, sf, scis, cm, pr[0], shared);
          }
          if (!shared.aborted) _zwPresent(state, sf);
        }
      }
    } catch (e) {
      shared.aborted = true; // 光栅面异常（含步预算超限）→ 弃绘制（FIXME 头注记账）
    }
  }

  function _zwDrawArrays(state, mode, first, count) {
    var prog = state._program;
    var link = prog && prog._zwLink;
    if (!link || !link.ok) return 0x0502; // 未链接程序（GLES2 §2.8）
    if (state._bindings[_ZW_FBO]) return 0; // FBO 绑定绘制未实现（FIXME 头注）——静默
    var dp = _zwDrawProgOf(prog);
    if (!dp.ok) return 0; // 评估器子集外（FIXME 头注）——spec 不允许报错
    var attrs = state._vertexAttribs;
    var list = link.attribList;
    var dvs = [], names = [], types = [];
    // 范围检查（WebGL "Enabled Vertex Attributes and Range Checking"——实现取
    // INVALID_OPERATION + 不绘制的 robust 选项；enabled 但未被程序消费的数组
    // 恒不报错、不参与检查）。
    for (var i = 0; i < list.length; i++) {
      var ent = list[i], va = attrs[ent.index];
      names.push(ent.name);
      types.push(ent.type);
      if (!va || !va.enabled) {
        dvs.push({ enabled: false, cur: va && va.cur ? va.cur : [0, 0, 0, 1] });
        continue;
      }
      var typeSize = _ZW_VATTR_TYPES[va.type];
      if (!typeSize) return 0x0502;
      var buf = va.buffer;
      if (!buf || !buf._zwData) return 0x0502; // enabled 无缓冲
      var stride = va.stride || va.size * typeSize;
      var need = va.offset + (count - 1) * stride + va.size * typeSize;
      if (need > buf._zwData.length) return 0x0502; // 越界取数 → INVALID_OPERATION
      dvs.push({
        enabled: true, dv: new DataView(buf._zwData.buffer,
          buf._zwData.byteOffset, buf._zwData.byteLength),
        va: va, typeSize: typeSize
      });
    }
    dp.vsLinkList = names;
    dp.vsLinkTypes = types;
    _zwExecute(state, dp, mode, count, dvs, _zwUniformValues(link),
      function (slot) { return first + slot; });
    return 0;
  }

  function _zwDrawElements(state, mode, count, type, offset, elBuf) {
    var prog = state._program;
    var link = prog && prog._zwLink;
    if (!link || !link.ok) return 0x0502;
    if (state._bindings[_ZW_FBO]) return 0;
    var dp = _zwDrawProgOf(prog);
    if (!dp.ok) return 0;
    var isz = type === 0x1403 ? 2 : 1; // UNSIGNED_SHORT / UNSIGNED_BYTE（part08 已拒其余）
    var data = elBuf && elBuf._zwData;
    if (!data) return 0x0502; // 无索引缓冲（count>0 已在 part08 挡 null 绑定）
    if (offset + count * isz > data.length) return 0x0502; // 索引越界（WebGL Range Checking）
    var edv = new DataView(data.buffer, data.byteOffset, data.byteLength);
    var idx = [], maxIdx = -1, i;
    for (i = 0; i < count; i++) {
      var v = isz === 2 ? edv.getUint16(offset + i * 2, true) : edv.getUint8(offset + i);
      if (v > maxIdx) maxIdx = v;
      idx.push(v);
    }
    var attrs = state._vertexAttribs;
    var list = link.attribList;
    var dvs = [], names = [], types = [];
    for (i = 0; i < list.length; i++) {
      var ent = list[i], va = attrs[ent.index];
      names.push(ent.name);
      types.push(ent.type);
      if (!va || !va.enabled) {
        dvs.push({ enabled: false, cur: va && va.cur ? va.cur : [0, 0, 0, 1] });
        continue;
      }
      var typeSize = _ZW_VATTR_TYPES[va.type];
      if (!typeSize) return 0x0502;
      var buf = va.buffer;
      if (!buf || !buf._zwData) return 0x0502;
      var stride = va.stride || va.size * typeSize;
      var need = va.offset + maxIdx * stride + va.size * typeSize;
      if (need > buf._zwData.length) return 0x0502;
      dvs.push({
        enabled: true, dv: new DataView(buf._zwData.buffer,
          buf._zwData.byteOffset, buf._zwData.byteLength),
        va: va, typeSize: typeSize
      });
    }
    dp.vsLinkList = names;
    dp.vsLinkTypes = types;
    _zwExecute(state, dp, mode, count, dvs, _zwUniformValues(link),
      function (slot) { return idx[slot]; });
    return 0;
  }

  function _zwPresent(state, sf) {
    if (typeof globalThis.__zw_canvas_op !== 'function' || !state._handle) return;
    var parts = new Array(sf.pix.length);
    for (var i = 0; i < sf.pix.length; i++) parts[i] = sf.pix[i];
    globalThis.__zw_canvas_op(state._handle, 'putImageData',
      '0', '0', String(sf.W), String(sf.H), parts.join(','));
  }

  globalThis._zwGlDrawBackend = {
    drawArrays: _zwDrawArrays,
    drawElements: _zwDrawElements
  };
})();
