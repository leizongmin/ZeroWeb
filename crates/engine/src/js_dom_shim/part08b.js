// ============================================================================
// part08b — GLSL ES 1.0 前端子集：词法/语法/语义校验 + 活跃声明提取
// （webgl-compat goal M3-S2，2026-10-22）。
//
// 发布单点：globalThis._zwGlCompileGlsl(source, isVertex) →
//   { ok, infoLog, attributes, uniforms }
//   - ok：编译通过；infoLog：失败原因（GL infoLog 风格行，成功为 ''）
//   - attributes：顶点着色器**活跃** attribute（声明序）[{ name, type, size }]
//   - uniforms：本阶段**活跃** uniform（声明序）[{ name, type, size }]
//   （type 为 GL 枚举，size 数组长度或 1）。跨阶段 uniform 合并、location
//   分配、bindAttribLocation 覆盖由 part08 linkProgram 负责。
//
// 加载序：part08a → part08 → 本文件（仅依赖自身符号 + globalThis 发布；由
// part08 的 compileShader **运行时**调用——无加载期耦合）。
//
// 规范依据：GLSL ES 1.0.17（词法 §3 / 语法 §4 / 存储限定符 §4.3 / 保留字
// §3.7 / 内建变量 §7 / 内建函数 §8）；WebGL 1.0 §5.14.9（compileShader）。
//
// **子集边界（诚实记账）**：
//   - 不做表达式类型推导与检查（FIXME(管线需要时)）；赋值左值合法性不校验；
//   - struct 不支持（显式编译错误）；数组仅一维整型字面量长度；
//   - 内建函数按名表接受（不做 arity/实参类型检查）；用户函数先声明后调用；
//   - # 预处理指令不展开、按编译错误拒绝（FIXME）；片段着色器 float 默认
//     精度（§4.5.4 默认 NONE）不强制（宽松接受）；
//   - 浮点字面量 f/F 后缀按 §3.6 拒绝（GLSL ES 无后缀）。
// ============================================================================

// ── 保留字（GLSL ES 1.0 §3.7 关键字 + future-use + 全部类型名）——作标识符
// 即编译错误；类型名按关键字词法化以便类型位识别 ──────────────────────────
var _ZW_GLSL_RESERVED = {
  attribute: 1, const: 1, bool: 1, float: 1, int: 1, break: 1, continue: 1,
  do: 1, while: 1, if: 1, else: 1, for: 1, in: 1, out: 1, inout: 1,
  uniform: 1, varying: 1, void: 1, precision: 1, highp: 1, mediump: 1,
  lowp: 1, true: 1, false: 1, return: 1, discard: 1, struct: 1,
  vec2: 1, vec3: 1, vec4: 1, ivec2: 1, ivec3: 1, ivec4: 1,
  bvec2: 1, bvec3: 1, bvec4: 1, mat2: 1, mat3: 1, mat4: 1,
  sampler2D: 1, samplerCube: 1,
  // future-use 保留（§3.7 表——常用面）
  asm: 1, class: 1, union: 1, enum: 1, typedef: 1, template: 1, this: 1,
  packed: 1, goto: 1, switch: 1, default: 1, inline: 1, noinline: 1,
  volatile: 1, public: 1, static: 1, extern: 1, external: 1, interface: 1,
  flat: 1, long: 1, short: 1, double: 1, half: 1, fixed: 1, unsigned: 1,
  superp: 1, input: 1, output: 1, hvec2: 1, hvec3: 1, hvec4: 1,
  dvec: 1, dvec2: 1, fvec: 1, fvec2: 1, fvec3: 1, fvec4: 1,
  sampler1D: 1, sampler3D: 1, sampler1DShadow: 1, sampler2DShadow: 1,
  sampler2DRect: 1, sampler3DRect: 1, sampler2DRectShadow: 1,
  sizeof: 1, cast: 1, namespace: 1, using: 1
};

// ── 类型名 → GL 枚举（GLES2 Table 6.14；getActiveUniform/Attrib 类型面）──
// T_VOID 哨兵 0：仅函数返回类型合法（变量声明遇之报错）。
var _ZW_GLSL_T_VOID = 0;
var _ZW_GLSL_TYPES = {
  void: _ZW_GLSL_T_VOID,
  float: 0x1406,   // FLOAT
  int: 0x1404,     // INT
  bool: 0x8B56,    // BOOL
  vec2: 0x8B50, vec3: 0x8B51, vec4: 0x8B52,
  ivec2: 0x8B53, ivec3: 0x8B54, ivec4: 0x8B55,
  bvec2: 0x8B57, bvec3: 0x8B58, bvec4: 0x8B59,
  mat2: 0x8B5A, mat3: 0x8B5B, mat4: 0x8B5C,
  sampler2D: 0x8B5E, samplerCube: 0x8B60
};

// ── 内建函数名表（GLSL ES 1.0 §8 节选：三角/指数/几何/矩阵/纹理/比较族）──
var _ZW_GLSL_BUILTINS = {
  radians: 1, degrees: 1, sin: 1, cos: 1, tan: 1, asin: 1, acos: 1, atan: 1,
  pow: 1, exp: 1, log: 1, exp2: 1, log2: 1, sqrt: 1, inversesqrt: 1,
  abs: 1, sign: 1, floor: 1, ceil: 1, fract: 1, mod: 1, min: 1, max: 1,
  clamp: 1, mix: 1, step: 1, smoothstep: 1,
  length: 1, distance: 1, dot: 1, cross: 1, normalize: 1,
  faceforward: 1, reflect: 1, refract: 1, matrixCompMult: 1,
  texture2D: 1, texture2DProj: 1, textureCube: 1,
  lessThan: 1, lessThanEqual: 1, greaterThan: 1, greaterThanEqual: 1,
  equal: 1, notEqual: 1, any: 1, all: 1, not: 1
};

// ── 词法（§3）：token = { t: 'id'|'kw'|'inum'|'fnum'|'op'|'eof', v, pos } ──
// 数字值仅在数组长度位使用（inum 手工十进制解析——不依赖 parseInt）；
// fnum 存原文切片（无求值需求）。
function _zwGlslTokenize(src, err) {
  var toks = [];
  var i = 0;
  var n = src.length;
  function push(t, v) { toks.push({ t: t, v: v, pos: i }); }
  while (i < n) {
    var c = src.charAt(i);
    if (c === ' ' || c === '\t' || c === '\n' || c === '\r' || c === '\f') { i++; continue; }
    if (c === '/' && src.charAt(i + 1) === '/') { // 行注释（§3.1）
      while (i < n && src.charAt(i) !== '\n') i++;
      continue;
    }
    if (c === '/' && src.charAt(i + 1) === '*') { // 块注释（可跨行）
      var close = src.indexOf('*/', i + 2);
      if (close < 0) { err('unterminated block comment', i); return null; }
      i = close + 2;
      continue;
    }
    if (c === '#') { // 预处理指令——子集边界（头注声明）
      err('preprocessor directives are not supported', i);
      return null;
    }
    // 数字（§3.5/§3.6）：float 带小数点/指数；int 纯十进制；无后缀
    if ((c >= '0' && c <= '9') ||
      (c === '.' && src.charAt(i + 1) >= '0' && src.charAt(i + 1) <= '9')) {
      var start = i;
      var isFloat = false;
      if (c === '.') { isFloat = true; i++; } // .5 前导点形式
      while (i < n && src.charAt(i) >= '0' && src.charAt(i) <= '9') i++;
      if (src.charAt(i) === '.') {
        isFloat = true; i++;
        while (i < n && src.charAt(i) >= '0' && src.charAt(i) <= '9') i++;
      }
      if (src.charAt(i) === 'e' || src.charAt(i) === 'E') {
        isFloat = true; i++;
        if (src.charAt(i) === '+' || src.charAt(i) === '-') i++;
        if (!(src.charAt(i) >= '0' && src.charAt(i) <= '9')) { err('malformed exponent', start); return null; }
        while (i < n && src.charAt(i) >= '0' && src.charAt(i) <= '9') i++;
      }
      if (src.charAt(i) === 'f' || src.charAt(i) === 'F') { // §3.6——无后缀
        err('float literal suffix is not allowed', start);
        return null;
      }
      var text = src.slice(start, i);
      if (isFloat) push('fnum', text);
      else {
        var val = 0;
        for (var d = 0; d < text.length; d++) val = val * 10 + (text.charCodeAt(d) - 48);
        push('inum', val);
      }
      continue;
    }
    // 标识符/关键字（§3.4/§3.6）
    if ((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c === '_') {
      var s2 = i;
      while (i < n) {
        var c2 = src.charAt(i);
        if ((c2 >= 'a' && c2 <= 'z') || (c2 >= 'A' && c2 <= 'Z') ||
          (c2 >= '0' && c2 <= '9') || c2 === '_') i++;
        else break;
      }
      var word = src.slice(s2, i);
      push(_ZW_GLSL_RESERVED[word] ? 'kw' : 'id', word);
      continue;
    }
    // 运算符（§3.8——最长匹配）
    var three = src.substr(i, 3);
    if (three === '<<=' || three === '>>=') { push('op', three); i += 3; continue; }
    var two = src.substr(i, 2);
    if (two === '++' || two === '--' || two === '&&' || two === '||' || two === '==' ||
      two === '!=' || two === '<=' || two === '>=' || two === '<<' || two === '>>' ||
      two === '+=' || two === '-=' || two === '*=' || two === '/=' || two === '%=') {
      push('op', two); i += 2; continue;
    }
    if ('+-*/%=<>!&|^~?:;,.{}()[]'.indexOf(c) >= 0) { push('op', c); i++; continue; }
    err('unexpected character ' + _zwGlslQuote(c), i);
    return null;
  }
  push('eof', '');
  return toks;
}

function _zwGlslQuote(v) { return "'" + String(v) + "'"; }

// ── 语法 + 语义（一遍式：建作用域、精确记标识符使用、提取全局声明）────────
// 返回 { globals, used } 或 null（err 已产 infoLog 行）。
function _zwGlslCompile(src, isVertex, err) {
  var toks = _zwGlslTokenize(src, err);
  if (!toks) return null;
  var p = 0;
  function peek() { return toks[p]; }
  function at(v) { return toks[p].v === v && (toks[p].t === 'op' || toks[p].t === 'kw'); }
  function eat(v) { if (at(v)) { p++; return true; } return false; }
  function expect(v, what) {
    if (eat(v)) return true;
    err('expected ' + _zwGlslQuote(v) + (what ? ' in ' + what : '') +
      ' but got ' + _zwGlslQuote(toks[p].v), toks[p].pos);
    return false;
  }
  function typeEnum(v) {
    return Object.prototype.hasOwnProperty.call(_ZW_GLSL_TYPES, v) ? _ZW_GLSL_TYPES[v] : null;
  }

  // 作用域栈（§4.2.1）：每层 { name: {kind: 'var'|'fn', type} }。
  // 预注入 §7 内建变量（用户重声明被 gl_ 前缀检查拦截）。
  var scopes = [{
    gl_Position: { kind: 'var', type: 0x8B52 },
    gl_PointSize: { kind: 'var', type: 0x1406 },
    gl_FragColor: { kind: 'var', type: 0x8B52 },
    gl_FragData: { kind: 'var', type: 0x8B52 },
    gl_FrontFacing: { kind: 'var', type: 0x8B56 },
    gl_PointCoord: { kind: 'var', type: 0x8B52 }
  }];
  var usedSet = {};
  function declare(name, kind, type, pos) {
    var top = scopes[scopes.length - 1];
    if (top[name]) { err('duplicate declaration of ' + _zwGlslQuote(name), pos); return false; }
    top[name] = { kind: kind, type: type };
    return true;
  }
  function lookup(name) {
    for (var i = scopes.length - 1; i >= 0; i--) {
      if (scopes[i][name]) return scopes[i][name];
    }
    return null;
  }
  function use(name, pos) { // 变量引用：必须已声明并记使用（活跃性精确记账）
    if (!lookup(name)) {
      err('undeclared identifier ' + _zwGlslQuote(name), pos);
      return false;
    }
    usedSet[name] = true;
    return true;
  }

  var globals = []; // 声明序 attribute/uniform（声明同时注入作用域）

  // ── 表达式（§5——C 风格优先级；结构解析 + 使用记账，无类型推导）──
  function parseExpr() {
    if (!parseAssign()) return false;
    while (eat(',')) { if (!parseAssign()) return false; }
    return true;
  }
  function parseAssign() {
    if (!parseCond()) return false;
    if (at('=') || at('+=') || at('-=') || at('*=') || at('/=') || at('%=') ||
      at('<<=') || at('>>=')) {
      p++;
      return parseAssign();
    }
    return true;
  }
  function parseCond() {
    if (!parseBin(0)) return false;
    if (eat('?')) {
      if (!parseExpr() || !expect(':', 'ternary')) return false;
      if (!parseAssign()) return false;
    }
    return true;
  }
  var _ZW_BIN_LEVELS = [['||'], ['&&'], ['|'], ['^'], ['&'], ['==', '!='],
    ['<', '>', '<=', '>='], ['<<', '>>'], ['+', '-'], ['*', '/', '%']];
  function parseBin(lv) {
    if (lv >= _ZW_BIN_LEVELS.length) return parseUnary();
    if (!parseBin(lv + 1)) return false;
    while (toks[p].t === 'op' && _ZW_BIN_LEVELS[lv].indexOf(toks[p].v) >= 0) {
      p++;
      if (!parseBin(lv + 1)) return false;
    }
    return true;
  }
  function parseUnary() {
    if (at('++') || at('--') || at('+') || at('-') || at('!') || at('~')) {
      p++;
      return parseUnary();
    }
    return parsePostfix();
  }
  function parsePostfix() {
    if (!parsePrimary()) return false;
    while (true) {
      if (eat('.')) { // 字段选择/swizzle——字段名非变量引用，不记使用
        if (toks[p].t !== 'id') { err('expected field name after .', toks[p].pos); return false; }
        p++;
      } else if (eat('[')) {
        if (!parseExpr() || !expect(']', 'index')) return false;
      } else if (at('++') || at('--')) {
        p++;
      } else break;
    }
    return true;
  }
  function parsePrimary() {
    var tk = toks[p];
    if (tk.t === 'inum' || tk.t === 'fnum') { p++; return true; }
    if (tk.v === 'true' || tk.v === 'false') { p++; return true; }
    if (eat('(')) { if (!parseExpr() || !expect(')', 'paren')) return false; return true; }
    // 类型构造器（§5.2/§5.4）——类型名为 kw；裸类型名（无括号）非法
    if (tk.t === 'kw') {
      var teCtor = typeEnum(tk.v);
      if (teCtor !== null && teCtor !== _ZW_GLSL_T_VOID) {
        p++;
        if (!expect('(', 'type constructor')) return false;
        return parseArgs();
      }
    }
    if (tk.t === 'id') {
      p++;
      if (eat('(')) { // 函数调用：内建名表 / 类型构造器 / 已声明用户函数（§4.2.1 先声明后用）
        if (!_ZW_GLSL_BUILTINS[tk.v] && !typeEnum(tk.v)) {
          var ent = lookup(tk.v);
          if (!ent || ent.kind !== 'fn') {
            err('call to undefined function ' + _zwGlslQuote(tk.v), tk.pos);
            return false;
          }
        }
        return parseArgs();
      }
      return use(tk.v, tk.pos); // 变量引用
    }
    err('unexpected token ' + _zwGlslQuote(tk.v) + ' in expression', tk.pos);
    return false;
  }
  function parseArgs() {
    if (eat(')')) return true;
    while (true) {
      if (!parseAssign()) return false;
      if (eat(',')) continue;
      return expect(')', 'arguments');
    }
  }

  // ── 声明（§4.1.9 init_declarator_list + §4.1.8 数组）──
  function parseDeclarators(typeE, storage, isGlobal) {
    while (true) {
      var tk = toks[p];
      if (tk.t !== 'id') { err('expected identifier in declaration', tk.pos); return false; }
      if (tk.v.slice(0, 3) === 'gl_') { // §3.7：gl_ 前缀用户代码禁用
        err('reserved prefix gl_ in identifier ' + _zwGlslQuote(tk.v), tk.pos);
        return false;
      }
      p++;
      var size = 1;
      if (eat('[')) {
        var num = toks[p];
        if (num.t !== 'inum') { err('array size must be an integer literal', num.pos); return false; }
        size = num.v;
        p++;
        if (!expect(']', 'array declaration')) return false;
      }
      var hasInit = false;
      if (at('=')) {
        // §4.3.3/§4.3.5——attribute/varying/uniform 无初始化器
        if (storage === 'attribute' || storage === 'varying' || storage === 'uniform') {
          err(storage + ' cannot have an initializer', tk.pos);
          return false;
        }
        p++;
        hasInit = true;
        if (!parseAssign()) return false;
      }
      if (storage === 'const' && !hasInit) { // §4.3.2 const 必须初始化
        err('const declaration requires an initializer', tk.pos);
        return false;
      }
      if (!declare(tk.v, 'var', typeE, tk.pos)) return false;
      if (isGlobal && (storage === 'attribute' || storage === 'uniform')) {
        globals.push({ name: tk.v, type: typeE, size: size, storage: storage });
      }
      if (eat(',')) continue;
      return expect(';', 'declaration');
    }
  }

  function parsePrecision() { // 'precision' 已消费：qualifier typeSpec ;（§4.5.3）
    var q = toks[p];
    if (q.v !== 'highp' && q.v !== 'mediump' && q.v !== 'lowp') {
      err('expected precision qualifier', q.pos);
      return false;
    }
    p++;
    var te = typeEnum(toks[p].v);
    if (toks[p].t !== 'kw' || te === null || te === _ZW_GLSL_T_VOID) {
      err('expected type after precision qualifier', toks[p].pos);
      return false;
    }
    p++;
    return expect(';', 'precision statement');
  }

  // 语句（§4.2）——loopDepth/fnRet 沿调用链传递（break/continue/void-return 语义）
  function parseStatement(loopDepth, fnRet) {
    var tk = peek();
    if (tk.v === '{') return parseCompound(loopDepth, fnRet);
    if (tk.v === ';') { p++; return true; }
    if (tk.v === 'precision') { p++; return parsePrecision(); }
    if (tk.v === 'if') {
      p++;
      if (!expect('(', 'if') || !parseExpr() || !expect(')', 'if')) return false;
      if (!parseStatement(loopDepth, fnRet)) return false;
      if (eat('else')) { if (!parseStatement(loopDepth, fnRet)) return false; }
      return true;
    }
    if (tk.v === 'for') {
      p++;
      if (!expect('(', 'for')) return false;
      scopes.push({});
      if (toks[p].v !== ';') { // for-init：声明或表达式，可空
        var teF = toks[p].t === 'kw' ? typeEnum(toks[p].v) : null;
        if (teF !== null) {
          if (teF === _ZW_GLSL_T_VOID) {
            scopes.pop();
            err('void invalid for variable declaration', toks[p].pos);
            return false;
          }
          p++;
          if (!parseDeclarators(teF, 'local', false)) { scopes.pop(); return false; }
        } else {
          if (!parseExpr()) { scopes.pop(); return false; }
          if (!expect(';', 'for-init')) { scopes.pop(); return false; }
        }
      } else p++;
      if (toks[p].v !== ';' && !parseExpr()) { scopes.pop(); return false; }
      if (!expect(';', 'for-cond')) { scopes.pop(); return false; }
      if (toks[p].v !== ')' && !parseExpr()) { scopes.pop(); return false; }
      if (!expect(')', 'for')) { scopes.pop(); return false; }
      if (!parseStatement(loopDepth + 1, fnRet)) { scopes.pop(); return false; }
      scopes.pop();
      return true;
    }
    if (tk.v === 'while') {
      p++;
      if (!expect('(', 'while') || !parseExpr() || !expect(')', 'while')) return false;
      return parseStatement(loopDepth + 1, fnRet);
    }
    if (tk.v === 'do') {
      p++;
      if (!parseStatement(loopDepth + 1, fnRet)) return false;
      if (!expect('while', 'do-while') || !expect('(', 'do-while') || !parseExpr() ||
        !expect(')', 'do-while') || !expect(';', 'do-while')) return false;
      return true;
    }
    if (tk.v === 'break' || tk.v === 'continue') { // §4.2.4 循环内限定
      if (loopDepth === 0) { err(tk.v + ' outside of loop', tk.pos); return false; }
      p++;
      return expect(';', tk.v);
    }
    if (tk.v === 'return') { // §4.2.5——void 函数 return 不得带值
      p++;
      if (toks[p].v !== ';') {
        if (fnRet && fnRet.voidFn) { err('void function cannot return a value', tk.pos); return false; }
        if (!parseExpr()) return false;
      }
      return expect(';', 'return');
    }
    if (tk.v === 'discard') { p++; return expect(';', 'discard'); } // §4.2.6 片元
    if (tk.v === 'const') { // 局部 const 声明（§4.3.2）
      p++;
      var teC = typeEnum(peek().v);
      if (peek().t !== 'kw' || teC === null || teC === _ZW_GLSL_T_VOID) {
        err('expected type after const', peek().pos);
        return false;
      }
      p++;
      return parseDeclarators(teC, 'const', false);
    }
    var te = toks[p].t === 'kw' ? typeEnum(toks[p].v) : null; // 声明语句：类型名开头
    if (te !== null) {
      if (te === _ZW_GLSL_T_VOID) { err('void invalid for variable declaration', tk.pos); return false; }
      p++;
      return parseDeclarators(te, 'local', false);
    }
    if (!parseExpr()) return false;
    return expect(';', 'expression statement');
  }

  function parseCompound(loopDepth, fnRet) {
    if (!expect('{', 'compound statement')) return false;
    scopes.push({});
    while (toks[p].v !== '}' && toks[p].t !== 'eof') {
      if (!parseStatement(loopDepth, fnRet)) return false;
    }
    scopes.pop();
    return expect('}', 'compound statement');
  }

  // 参数表（§4.2.1）：空 | void | ([in|out|inout] type [name])(, ...)*
  function parseParams() {
    if (eat(')')) return true;
    if (eat('void') && eat(')')) return true;
    while (true) {
      if (at('in') || at('out') || at('inout')) p++;
      var te = typeEnum(toks[p].v);
      if (toks[p].t !== 'kw' || te === null || te === _ZW_GLSL_T_VOID) {
        err('expected parameter type', toks[p].pos);
        return false;
      }
      p++;
      var tk = toks[p];
      if (tk.t === 'id') { // 参数名可省（§4.2.1）
        p++;
        if (toks[p].v === '[') { // 数组参数——子集拒绝（FIXME）
          err('array parameters are not supported', tk.pos);
          return false;
        }
        if (!declare(tk.v, 'var', te, tk.pos)) return false;
      }
      if (eat(',')) continue;
      return expect(')', 'parameter list');
    }
  }

  // ── 顶层声明（§4.3 全局变量 + §4.2.1 函数定义）──
  function parseGlobal() {
    var tk = peek();
    if (tk.v === 'precision') { p++; return parsePrecision(); }
    var storage = 'local';
    if (tk.v === 'attribute' || tk.v === 'uniform' || tk.v === 'varying' || tk.v === 'const') {
      storage = tk.v;
      if (storage === 'attribute' && !isVertex) { // §4.3.3——仅顶点着色器
        err('attribute is only available in vertex shaders', tk.pos);
        return false;
      }
      p++;
      tk = peek();
    }
    var te = tk.t === 'kw' ? typeEnum(tk.v) : null;
    if (te === null) {
      err('expected type specifier at global scope, got ' + _zwGlslQuote(tk.v), tk.pos);
      return false;
    }
    p++;
    // 函数定义：type name ( params ) { body }——参数入函数作用域（不漏全局）
    var nameTk = toks[p];
    if (nameTk && nameTk.t === 'id' && toks[p + 1] && toks[p + 1].v === '(') {
      if (storage !== 'local') { err('function cannot have storage qualifier ' + storage, tk.pos); return false; }
      if (nameTk.v.slice(0, 3) === 'gl_') { // §3.7
        err('reserved prefix gl_ in identifier ' + _zwGlslQuote(nameTk.v), nameTk.pos);
        return false;
      }
      p += 2;
      if (!declare(nameTk.v, 'fn', te, nameTk.pos)) return false;
      scopes.push({});
      if (!parseParams()) { scopes.pop(); return false; }
      var okBody = parseCompound(0, { voidFn: te === _ZW_GLSL_T_VOID });
      scopes.pop();
      return okBody;
    }
    // 全局变量声明（含 storage——attribute/uniform/varying/const 全部入作用域）
    if (te === _ZW_GLSL_T_VOID) { err('void invalid for variable declaration', tk.pos); return false; }
    return parseDeclarators(te, storage, true);
  }

  while (toks[p].t !== 'eof') {
    if (!parseGlobal()) return null;
  }
  // §4.2.1——main() 函数必须存在（缺失为编译期错误）
  var mainEnt = scopes[0].main;
  if (!mainEnt || mainEnt.kind !== 'fn') {
    err('missing main() function', src.length);
    return null;
  }
  return { globals: globals, used: usedSet };
}

// ── 发布单点（part08 compileShader 运行时调用）────────────────────────────
globalThis._zwGlCompileGlsl = function (source, isVertex) {
  var log = [];
  var s = String(source);
  function lineOf(pos) {
    var ln = 1;
    for (var i = 0; i < pos && i < s.length; i++) {
      if (s.charAt(i) === '\n') ln++;
    }
    return ln;
  }
  function err(msg, pos) {
    if (log.length < 8) log.push('ERROR: 0:' + lineOf(pos) + ': ' + msg);
  }
  var result = _zwGlslCompile(s, !!isVertex, err);
  if (!result) {
    return { ok: false, infoLog: log.join('\n') || 'compile failed', attributes: [], uniforms: [] };
  }
  var attributes = [];
  var uniforms = [];
  for (var i = 0; i < result.globals.length; i++) {
    var g = result.globals[i];
    if (!result.used[g.name]) continue; // 未引用 → 非 active（§5.14.10 优化剔除）
    if (g.storage === 'attribute') attributes.push({ name: g.name, type: g.type, size: g.size });
    else if (g.storage === 'uniform') uniforms.push({ name: g.name, type: g.type, size: g.size });
  }
  return { ok: true, infoLog: '', attributes: attributes, uniforms: uniforms };
};
