// ============================================================================
// part08 — WebGL 接口面（webgl-compat goal M2，2026-10-22）。
//
// 范围（2026-10-22 用户四项裁决——门控解除）：
//   ① getContext('webgl' | 'experimental-webgl') 上下文对象面（'webgl2' 返 null，
//      WebGL2 仅接口对象存在——idlharness 资产面，实例面远期挂账）；
//   ② getError 状态机 + buffer/texture 错误语义（INVALID_ENUM/INVALID_OPERATION/
//      NO_ERROR，pin rev corpus 7 案 15 subtests 验收）；
//   ③ WebIDL 校验面——必参 TypeError、TexImageSource union 品牌检查、
//      boolean 转换不触 valueOf（uniformMatrix*fv transpose）；
//   ④ clear() 经**既有 host 2d 上下文**光栅化（source-over 不透明填充 =
//      GL 替换语义，scissor 生效），
//      不另起 wgpu 实例；GL→wgpu 映射与 shader/纹理上传管线 = M3（FIXME 标注）。
//
// 接口面数据表（常量 297+262 / 方法 125+83）由 webgl1.idl / webgl2.idl 派生，
// 在 part08a.js（前置加载，globalThis 单点交接）；idlharness（webgl/
// idlharness.any.js）按 WebIDL 逐项断言接口对象/原型（常量值与描述符、方法
// existence/.length/.name、this 品牌 TypeError），本文件与之逐项对齐。
//
// 作用域：part07 末尾已闭合外层 IIFE，本文件运行在**顶层作用域**——与 part04/05
//（同一大闭包内，无法反向引用本文件局部）经 `globalThis._zwMakeCtxWebGL` 单点
// 发布对接（part01 `globalThis._zwGetInnerText` 先例）；其余符号全部闭包内。
// ============================================================================

(function () {
  'use strict';

  // ── 接口面数据表 ──
  // 由 part08a.js 派生注入（2000 行准则拆分；来源/派生口径见彼处头注，勿手改）。
  var _zwWglT = globalThis._zwWglIdlTables;
  globalThis._zwWglIdlTables = null; // 用后即清
  var _ZW_WGL1C = _zwWglT.w1c;
  var _ZW_WGL1M = _zwWglT.w1m;
  var _ZW_WGL2C = _zwWglT.w2c;
  var _ZW_WGL2M = _zwWglT.w2m;

  // ── 品牌/子型 ─────────────────────────────────────────────────────────
  // 实例标记：上下文 `_zwGlCtx` + `_zwGlIface`（'WebGLRenderingContext' |
  // 'WebGL2RenderingContext'）；GL 对象族 `_zwGlObj` + `_zwGlIface`（接口名）。
  // WebIDL 品牌检查：this 必须是本接口平台对象（idlharness 以 this=null/{} 调
  // 全部方法与 getter 断言 TypeError）。
  var _ZW_IFACE = 'WebGLRenderingContext';
  var _ZW_IFACE2 = 'WebGL2RenderingContext';

  // ── 接口定义机械 ─────────────────────────────────────────────────────
  // WebIDL es-interface（https://webidl.spec.whatwg.org/#es-interface）：
  //   接口对象（Function.prototype 派生或基接口接口对象派生）+ prototype
  //  {w:false,e:false,c:false} + 常量双落（接口对象与 prototype，
  //  {w:false,e:true,c:false}）+ 方法（prototype 数据属性
  //  {w:true,e:true,c:true}，.length=必参个数、.name=成员名）+
  //  属性（prototype 访问器 {e:true,c:true}，品牌外 this 抛 TypeError）。
  // 无声明构造器的接口 `new` 亦抛 TypeError（idlharness test_self 构造断言面）。
  function _zwGlDefineInterface(spec) {
    var ifaceName = spec.name;
    var constructible = spec.constructible || null;
    var ifaceObj = function () {
      if (!(this instanceof ifaceObj)) {
        throw new TypeError('Illegal constructor');
      }
      if (!constructible) {
        throw new TypeError('Illegal invocation');
      }
      return constructible(this, arguments);
    };
    // 接口对象 [[Prototype]]：基接口**接口对象**（无基 → Function.prototype，
    // 函数字面量默认即满足）——idlharness test_self 原型链断言面。
    if (spec.base) Object.setPrototypeOf(ifaceObj, globalThis[spec.base]);
    if (constructible) {
      Object.defineProperty(ifaceObj, 'length', { value: 1 });
    }
    Object.defineProperty(ifaceObj, 'name', {
      value: ifaceName, writable: false, enumerable: false, configurable: true
    });
    var proto = spec.base
      ? Object.create(globalThis[spec.base].prototype)
      : Object.create(Object.prototype);
    Object.defineProperty(ifaceObj, 'prototype', {
      value: proto, writable: false, enumerable: false, configurable: false
    });
    // prototype.constructor（WebIDL {w:true,e:false,c:true}）。
    Object.defineProperty(proto, 'constructor', {
      value: ifaceObj, writable: true, enumerable: false, configurable: true
    });
    var consts = spec.constants || {};
    for (var cname in consts) {
      var cval = consts[cname];
      Object.defineProperty(ifaceObj, cname, {
        value: cval, writable: false, enumerable: true, configurable: false
      });
      Object.defineProperty(proto, cname, {
        value: cval, writable: false, enumerable: true, configurable: false
      });
    }
    // 方法由 _zwGlDefineContextInterface 按品牌包装注入——对象族接口与 WebIDL
    // 一致，仅承载常量/属性。
    var attrs = spec.attributes || [];
    for (var i = 0; i < attrs.length; i++) {
      (function (a) {
        // 访问器命名（idlharness attribute 断言面：getter name 'get X'、setter
        // name 'set X'）。
        var getter = function () { return a.get(this); };
        Object.defineProperty(getter, 'name', { value: 'get ' + a.name });
        var prop = { get: getter, enumerable: true, configurable: true };
        if (a.set) {
          var setter = function (v) { a.set(this, v); };
          Object.defineProperty(setter, 'name', { value: 'set ' + a.name });
          prop.set = setter;
        }
        Object.defineProperty(proto, a.name, prop);
      })(attrs[i]);
    }
    // 全局注册（WebIDL：{w:true,e:false,c:true}——idlharness test_self 描述符面）。
    Object.defineProperty(globalThis, ifaceName, {
      value: ifaceObj, writable: true, enumerable: false, configurable: true
    });
    return ifaceObj;
  }

  function _zwGlBrandObj(self) {
    if (!self || !self._zwGlObj) {
      throw new TypeError('Illegal invocation');
    }
  }

  // ── 数值转换（WebIDL）─────────────────────────────────────────────────
  // ToInteger（ES 规范）：NaN → 0、±Infinity → ±2^53-1 截断后取整——
  // bufferSubData 的 NaN offset 经此落 0（corpus bufferSubData.html 断言
  // NO_ERROR 的机制面：WebGL 1.0 §6.20 NaN 无错误语义 + GLintptr 转换）。
  function _zwGlToInteger(v) {
    var n = +v;
    if (isNaN(n)) return 0;
    if (n === Infinity || n === -Infinity) return n > 0 ? 9007199254740991 : -9007199254740991;
    return n < 0 ? -Math.floor(-n) : Math.floor(n);
  }

  // TexImageSource union 品牌检查（webgl1.idl typedef：ImageBitmap or ImageData
  // or HTMLImageElement or HTMLCanvasElement or HTMLVideoElement or
  // OffscreenCanvas or VideoFrame）。union 转换按接口逐个品牌判定——非成员对象
  // 抛 TypeError 且**不触达其属性 getter**（corpus texImage2D/texSubImage2D 的
  // {width/height getter 抛异常} 对象期望 TypeError 而非透传异常）。
  function _zwGlIsTexImageSource(src) {
    if (!src || typeof src !== 'object') return false;
    try {
      if (src instanceof globalThis.ImageData) return true;
    } catch (_e1) {}
    if (src._zwImageBitmap) return true;
    if (src._zwOffscreenCanvas) return true;
    try {
      var tag = src.tagName;
      if (tag === 'CANVAS' || tag === 'IMG' || tag === 'VIDEO') return true;
    } catch (_e2) {}
    return false;
  }

  // ── 状态机原语 ────────────────────────────────────────────────────────
  // GL 错误单旗标（WebGL getError：返首错并清除；旗标置位期间新错误不覆盖）。
  function _zwGlSetError(state, code) {
    if (state._err === 0) state._err = code;
  }

  // webglcontextlost/restored 事件派发（WebGL 1.0 §5.15——WebGLContextEvent 于
  // canvas 元素上触发；纯 shim 单测环境无 canvas/无 dispatchEvent 时静默跳过，
  // 丢失状态机仍可经 isContextLost 观测）。after 回调在派发后读 defaultPrevented。
  function _zwGlFireContextEvent(state, type, cancelable, after) {
    var canvas = state._canvas;
    if (!canvas || typeof canvas.dispatchEvent !== 'function' ||
      typeof globalThis.WebGLContextEvent !== 'function') {
      return;
    }
    var ev;
    try {
      ev = new globalThis.WebGLContextEvent(type, {
        bubbles: false, cancelable: !!cancelable, statusMessage: ''
      });
    } catch (_eCtor) { return; }
    try { canvas.dispatchEvent(ev); } catch (_eDispatch) { return; }
    if (after) after(ev);
  }

  function _zwGlNewObject(ifaceName) {
    var obj = { _zwGlObj: true, _zwGlIface: ifaceName, _label: '' };
    // 实例原型链接（WebIDL 对象创建步）：接口经 _zwGlDefineInterface 全局注册
    // 后可查——属性访问器（WebGLObject.label、WebGLShaderPrecisionFormat.
    // rangeMin 等）在原型上，不链接则实例属性读取落 undefined。未注册名退
    // Object.prototype。
    var ctor = globalThis[ifaceName];
    if (ctor && ctor.prototype) Object.setPrototypeOf(obj, ctor.prototype);
    return obj;
  }

  // create*/delete*/is* 的对象名映射（op 名 → 接口名）。
  var _ZW_CREATE_TAGS = {
    createBuffer: 'WebGLBuffer', createFramebuffer: 'WebGLFramebuffer',
    createProgram: 'WebGLProgram', createRenderbuffer: 'WebGLRenderbuffer',
    createShader: 'WebGLShader', createTexture: 'WebGLTexture',
    createQuery: 'WebGLQuery', createSampler: 'WebGLSampler',
    createSync: 'WebGLSync', createTransformFeedback: 'WebGLTransformFeedback',
    createVertexArray: 'WebGLVertexArrayObject'
  };
  var _ZW_DELETE_TAGS = {
    deleteBuffer: 'WebGLBuffer', deleteFramebuffer: 'WebGLFramebuffer',
    deleteProgram: 'WebGLProgram', deleteRenderbuffer: 'WebGLRenderbuffer',
    deleteShader: 'WebGLShader', deleteTexture: 'WebGLTexture',
    deleteQuery: 'WebGLQuery', deleteSampler: 'WebGLSampler',
    deleteSync: 'WebGLSync', deleteTransformFeedback: 'WebGLTransformFeedback',
    deleteVertexArray: 'WebGLVertexArrayObject'
  };
  var _ZW_IS_TAGS = {
    isBuffer: 'WebGLBuffer', isFramebuffer: 'WebGLFramebuffer',
    isProgram: 'WebGLProgram', isRenderbuffer: 'WebGLRenderbuffer',
    isShader: 'WebGLShader', isTexture: 'WebGLTexture',
    isQuery: 'WebGLQuery', isSampler: 'WebGLSampler',
    isSync: 'WebGLSync', isTransformFeedback: 'WebGLTransformFeedback',
    isVertexArray: 'WebGLVertexArrayObject'
  };

  // 压缩纹理扩展（裁决③ corpus 批 3 的条件依赖面）：常量暴露 + per-context
  // 使能记录。FIXME(M3 纹理簇)：压缩像素解码未实现——使能后格式的真实上传以
  // INVALID_OPERATION 拒绝（base image 缺席同码），扩展常量面先行（WPT
  // compressed-tex-sub-image-2d-without-base-image 的 getExtension 依赖面）。
  var _ZW_EXT_S3TC = {
    COMPRESSED_RGB_S3TC_DXT1_EXT: 0x83F0,
    COMPRESSED_RGBA_S3TC_DXT1_EXT: 0x83F1,
    COMPRESSED_RGBA_S3TC_DXT3_EXT: 0x83F2,
    COMPRESSED_RGBA_S3TC_DXT5_EXT: 0x83F3
  };
  var _ZW_EXT_ETC1 = { COMPRESSED_RGB_ETC1_WEBGL: 0x8D64 };
  var _ZW_EXT_NAMES = ['WEBGL_compressed_texture_etc1', 'WEBGL_compressed_texture_s3tc',
    'WEBGL_lose_context'];

  // GL 枚举集（错误语义校验用；值同数据表）。
  var _ZW_BIND_TARGETS = { 0x8892: 1, 0x8893: 1 }; // ARRAY_BUFFER / ELEMENT_ARRAY_BUFFER
  var _ZW_TEX_BIND_TARGETS = { 0x0DE1: 1, 0x8513: 1 }; // TEXTURE_2D / TEXTURE_CUBE_MAP
  var _ZW_USAGE = { 0x88E0: 1, 0x88E1: 1, 0x88E2: 1, 0x88E3: 1, 0x88E4: 1 }; // STREAM_/STATIC_/DYNAMIC_{READ,DRAW,COPY}
  var _ZW_BEGIN_MODES = { 0: 1, 1: 1, 2: 1, 3: 1, 4: 1, 5: 1, 6: 1 };
  var _ZW_CLEAR_MASKS = 0x4100; // DEPTH|STENCIL|COLOR_BUFFER_BIT
  // enable/disable/isEnabled 的合法 cap 集（GLES2 2.0.25 Table 6.10——WebGL 1.0
  // §5.14.3：cap 非法 → INVALID_ENUM）。
  var _ZW_CAPS = {
    0x0BE2: 1, // BLEND
    0x0B44: 1, // CULL_FACE
    0x0B71: 1, // DEPTH_TEST
    0x0BD0: 1, // DITHER
    0x8037: 1, // POLYGON_OFFSET_FILL
    0x809E: 1, // SAMPLE_ALPHA_TO_COVERAGE
    0x80A0: 1, // SAMPLE_COVERAGE
    0x0C11: 1, // SCISSOR_TEST
    0x0B90: 1  // STENCIL_TEST
  };
  // 深度比较函数集（depthFunc；Table 6.9 CompareFunc 列）。
  var _ZW_DEPTH_FUNCS = { 0x0200: 1, 0x0201: 1, 0x0202: 1, 0x0203: 1, 0x0204: 1, 0x0205: 1, 0x0206: 1, 0x0207: 1 }; // NEVER/LESS/EQUAL/LEQUAL/GREATER/NOTEQUAL/GEQUAL/ALWAYS
  var _ZW_FRONT_FACES = { 0x0900: 1, 0x0901: 1 }; // CW/CCW
  var _ZW_CULL_MODES = { 0x0404: 1, 0x0405: 1, 0x0408: 1 }; // FRONT/BACK/FRONT_AND_BACK
  // blend 方程/因子集（GLES2 Table 6.9；MIN/MAX 与 4 参因子同集）。
  var _ZW_BLEND_EQUATIONS = { 0x8006: 1, 0x8007: 1, 0x8008: 1, 0x800B: 1, 0x800A: 1, 0x8005: 1, 0x8004: 1 }; // FUNC_ADD/SUBTRACT/REVERSE_SUBTRACT/MIN/MAX(FUNC_REVERSE_?)——MIN 0x8007? 见行内注释
  var _ZW_BLEND_FACTORS = {
    0: 1, 1: 1,               // ZERO/ONE
    0x0300: 1, 0x0301: 1,     // SRC_COLOR/ONE_MINUS_SRC_COLOR
    0x0302: 1, 0x0303: 1,     // SRC_ALPHA/ONE_MINUS_SRC_ALPHA
    0x0304: 1, 0x0305: 1,     // DST_ALPHA/ONE_MINUS_DST_ALPHA
    0x0306: 1, 0x0307: 1,     // DST_COLOR/ONE_MINUS_DST_COLOR
    0x0308: 1, 0x0309: 1,     // SRC_ALPHA_SATURATE/CONSTANT_COLOR? —— 见行内注释
    0x8001: 1, 0x8002: 1,     // CONSTANT_COLOR/ONE_MINUS_CONSTANT_COLOR
    0x8003: 1, 0x8004: 1      // CONSTANT_ALPHA/ONE_MINUS_CONSTANT_ALPHA
  };

  function _zwGlIsBufferSource(v) {
    if (!v || typeof v !== 'object') return false;
    return typeof v.byteLength === 'number' &&
      (typeof ArrayBuffer !== 'undefined' ? (v instanceof ArrayBuffer || ArrayBuffer.isView(v)) : false);
  }

  // ── 方法实现表（op 名 → impl(self, args)）────────────────────────────
  // args 为宿主 Arguments；未列出的 op 落 FIXME 桩（品牌检查后 no-op 返 undefined
  // ——M3 起按簇补真实语义，接口 existence/.length 面已由数据表对齐 idlharness）。
  var _zwGlOps = {
    // —— getError 状态机 ——
    // 上下文丢失后首次查询返 CONTEXT_LOST_WEBGL、其后 NO_ERROR（WebGL 1.0 §5.15.1
    // ——丢失上下文不再产生新错误）。
    getError: function (state, args) {
      if (state._lost) {
        if (state._lostErrPending) { state._lostErrPending = false; return 0x9242; }
        return 0;
      }
      var err = state._err;
      state._err = 0;
      return err;
    },

    // —— 对象族 ——
    createShader: function (state, args) {
      // §5.14.9——type 仅 VERTEX_SHADER/FRAGMENT_SHADER，否则 INVALID_ENUM + null。
      var type = +args[0];
      if (type !== 0x8B31 && type !== 0x8B30) { _zwGlSetError(state, 0x0500); return null; }
      var sh = _zwGlNewObject('WebGLShader');
      sh._zwType = type;
      return sh;
    },
    isContextLost: function (state, args) { return !!state._lost; },
    getContextAttributes: function (state, args) {
      // 丢失上下文 → null（WebGL 1.0 §5.14.2）；否则返创建属性副本（调用方
      // 改写不得回写状态）。attrs 尊重 getContext 第二参——FIXME(M3)：shim
      // getContext 现仅透传 colorSpace。
      if (state._lost) return null;
      var out = {};
      for (var k in state._attrs) out[k] = state._attrs[k];
      return out;
    },
    getSupportedExtensions: function (state, args) {
      return _ZW_EXT_NAMES.slice();
    },
    // getShaderPrecisionFormat（WebGL 1.0 §5.14.5；GLES2 §6.2 Table 6.6 最低
    // 精度面——编译管线缺席不依赖具体 shader 对象，shaderType/precisionType
    // 域校验先行）。
    getShaderPrecisionFormat: function (state, args) {
      var shaderType = +args[0];
      var precisionType = +args[1];
      if (shaderType !== 0x8B31 && shaderType !== 0x8B30) { // VERTEX_SHADER/FRAGMENT_SHADER
        _zwGlSetError(state, 0x0500);
        return null;
      }
      var range = {
        0x8DF0: [15, 15, 10],  // LOW_FLOAT
        0x8DF1: [15, 15, 10],  // MEDIUM_FLOAT
        0x8DF2: [127, 127, 23],// HIGH_FLOAT
        0x8DF3: [31, 30, 0],   // LOW_INT
        0x8DF4: [31, 30, 0],   // MEDIUM_INT
        0x8DF5: [31, 30, 0]    // HIGH_INT
      }[precisionType];
      if (!range) { _zwGlSetError(state, 0x0500); return null; }
      var fmt = _zwGlNewObject('WebGLShaderPrecisionFormat');
      fmt._rangeMin = range[0];
      fmt._rangeMax = range[1];
      fmt._precision = range[2];
      return fmt;
    },
    getExtension: function (state, args) {
      var name = args.length > 0 ? String(args[0]) : '';
      var ext = null;
      if (name === 'WEBGL_compressed_texture_s3tc') {
        state._compressedFormats[0x83F0] = 1;
        state._compressedFormats[0x83F1] = 1;
        state._compressedFormats[0x83F2] = 1;
        state._compressedFormats[0x83F3] = 1;
        ext = _ZW_EXT_S3TC;
      } else if (name === 'WEBGL_compressed_texture_etc1') {
        state._compressedFormats[0x8D64] = 1;
        ext = _ZW_EXT_ETC1;
      } else if (name === 'WEBGL_lose_context') {
        // WEBGL_lose_context（WebGL 1.0 §5.15.3）：lose/restore 触发
        // webglcontextlost/restored 事件（WebGLContextEvent，lost 可取消——
        // preventDefault 抑制后续 restore）。
        ext = {
          loseContext: function () {
            if (state._lost) return;
            state._lost = true;
            state._lostErrPending = true;
            state._err = 0;
            _zwGlFireContextEvent(state, 'webglcontextlost', true, function (ev) {
              if (ev.defaultPrevented) state._restoreSuppressed = true;
            });
          },
          restoreContext: function () {
            if (!state._lost || state._restoreSuppressed) return;
            state._lost = false;
            state._restoreSuppressed = false;
            state._lostErrPending = false;
            _zwGlFireContextEvent(state, 'webglcontextrestored', false, null);
          }
        };
      }
      if (!ext) return null;
      // 每扩展返回稳定对象（幂等语义）。
      if (!state._extObjects[name]) state._extObjects[name] = Object.create({}, {
        _zwGlExt: { value: true }
      });
      var obj = state._extObjects[name];
      for (var k in ext) {
        if (!(k in obj)) Object.defineProperty(obj, k, {
          value: ext[k], writable: false, enumerable: true, configurable: false
        });
      }
      return obj;
    },

    // —— buffer 对象（corpus bufferSubData.html）——
    bindBuffer: function (state, args) {
      var target = +args[0];
      if (!_ZW_BIND_TARGETS[target]) { _zwGlSetError(state, 0x0500); return; }
      var buf = args[1];
      if (buf !== null && buf !== undefined) {
        if (!buf._zwGlObj || buf._zwGlIface !== 'WebGLBuffer') {
          throw new TypeError('parameter 2 is not of type WebGLBuffer');
        }
        if (buf._zwDeleted) { _zwGlSetError(state, 0x0502); return; }
      }
      state._bindings[target] = buf || null;
    },
    bufferData: function (state, args) {
      var target = +args[0];
      if (!_ZW_BIND_TARGETS[target]) { _zwGlSetError(state, 0x0500); return; }
      var buf = state._bindings[target];
      if (!buf) { _zwGlSetError(state, 0x0502); return; }
      var data = args[1];
      var usage = +args[2];
      if (!_ZW_USAGE[usage]) { _zwGlSetError(state, 0x0500); return; }
      if (typeof data === 'number') {
        if (isNaN(data) || data < 0) { _zwGlSetError(state, 0x0501); return; } // INVALID_VALUE
        buf._zwSize = _zwGlToInteger(data);
      } else if (_zwGlIsBufferSource(data)) {
        buf._zwSize = data.byteLength;
      } else if (data === null) {
        buf._zwSize = 0;
      } else {
        throw new TypeError('parameter 2 is not of type BufferSource');
      }
    },
    bufferSubData: function (state, args) {
      var target = +args[0];
      if (!_ZW_BIND_TARGETS[target]) { _zwGlSetError(state, 0x0500); return; }
      var buf = state._bindings[target];
      if (!buf) { _zwGlSetError(state, 0x0502); return; }
      if (!_zwGlIsBufferSource(args[2])) {
        throw new TypeError('parameter 3 is not of type BufferSource');
      }
      // NaN offset 经 WebIDL GLintptr 转换落 0（不产生错误——corpus 断言面）。
      var offset = _zwGlToInteger(args[1]);
      var size = args[2].byteLength;
      if (offset < 0 || offset + size > (buf._zwSize || 0)) {
        _zwGlSetError(state, 0x0501); // INVALID_VALUE
      }
    },

    // —— texture 对象 + 压缩/上传校验语义（corpus compressedTexImage2D /
    //    compressedTexSubImage2D / texImage2D / texSubImage2D）——
    bindTexture: function (state, args) {
      var target = +args[0];
      if (!_ZW_TEX_BIND_TARGETS[target]) { _zwGlSetError(state, 0x0500); return; }
      var tex = args[1];
      if (tex !== null && tex !== undefined) {
        if (!tex._zwGlObj || tex._zwGlIface !== 'WebGLTexture') {
          throw new TypeError('parameter 2 is not of type WebGLTexture');
        }
        if (tex._zwDeleted) { _zwGlSetError(state, 0x0502); return; }
      }
      state._texBindings[target] = tex || null;
    },
    compressedTexImage2D: function (state, args) {
      if (args.length < 7) {
        throw new TypeError('compressedTexImage2D: requires 7 arguments');
      }
      var target = +args[0];
      if (!_ZW_TEX_BIND_TARGETS[target]) { _zwGlSetError(state, 0x0500); return; }
      var format = +args[2];
      // 格式校验先于其余参数（spec 5.14.8：未使能格式 → INVALID_ENUM——
      // corpus 两案的 S3TC 常量在不使能扩展的上下文恒 INVALID_ENUM）。
      if (!state._compressedFormats[format]) { _zwGlSetError(state, 0x0500); return; }
      var tex = state._texBindings[0x0DE1];
      if (!tex) { _zwGlSetError(state, 0x0502); return; }
      var level = _zwGlToInteger(args[1]);
      var w = _zwGlToInteger(args[3]);
      var h = _zwGlToInteger(args[4]);
      var border = _zwGlToInteger(args[5]);
      var data = args[6];
      if (level < 0 || border !== 0 || w <= 0 || h <= 0 || !_zwGlIsBufferSource(data)) {
        _zwGlSetError(state, 0x0501);
        return;
      }
      // FIXME(M3 纹理簇)：压缩解码与像素存储未实现——base image 记账后按
      // 未上传处理（compressedTexSubImage2D 的 base 前置判定消费此标记）。
      tex._zwHasCompressedBase = true;
    },
    compressedTexSubImage2D: function (state, args) {
      if (args.length < 8) {
        throw new TypeError('compressedTexSubImage2D: requires 8 arguments');
      }
      var target = +args[0];
      if (!_ZW_TEX_BIND_TARGETS[target]) { _zwGlSetError(state, 0x0500); return; }
      var format = +args[6];
      if (!state._compressedFormats[format]) { _zwGlSetError(state, 0x0500); return; }
      var tex = state._texBindings[0x0DE1];
      if (!tex || !tex._zwHasCompressedBase) {
        // 无既有 base image → INVALID_OPERATION（corpus 批 3 断言面）。
        _zwGlSetError(state, 0x0502);
        return;
      }
      var x = _zwGlToInteger(args[2]);
      var y = _zwGlToInteger(args[3]);
      var w = _zwGlToInteger(args[4]);
      var h = _zwGlToInteger(args[5]);
      var data = args[7];
      if (x < 0 || y < 0 || w <= 0 || h <= 0 || !_zwGlIsBufferSource(data)) {
        _zwGlSetError(state, 0x0501);
        return;
      }
      // FIXME(M3 纹理簇)：同 compressedTexImage2D。
    },
    texImage2D: function (state, args) {
      // 重载分派按参数个数（WebIDL 重载解析）：9 参 pixels 形 / 6 参 source 形；
      // 7-8 参落不进任何重载 → TypeError。
      var n = args.length;
      var tex = state._texBindings[0x0DE1];
      if (n >= 9) {
        var target = +args[0];
        if (!_ZW_TEX_BIND_TARGETS[target]) { _zwGlSetError(state, 0x0500); return; }
        var level = _zwGlToInteger(args[1]);
        var w = _zwGlToInteger(args[4]);
        var h = _zwGlToInteger(args[5]);
        var pixels = args[8];
        if (level < 0 || w <= 0 || h <= 0 ||
          (pixels !== null && pixels !== undefined && !_zwGlIsBufferSource(pixels))) {
          _zwGlSetError(state, 0x0501);
          return;
        }
        if (tex) tex._zwHasBase = true;
        // FIXME(M3 纹理簇)：像素上传 no-op——M2 无采样方，行为不可观测。
      } else if (n === 6) {
        // source 形：union 转换失败 → TypeError（品牌判定不触源对象 getter）。
        if (!_zwGlIsTexImageSource(args[5])) {
          throw new TypeError('parameter 6 is not of type TexImageSource');
        }
        if (tex) tex._zwHasBase = true;
        // FIXME(M3 纹理簇)：同上。
      } else {
        throw new TypeError('texImage2D: no matching overload');
      }
    },
    texSubImage2D: function (state, args) {
      var n = args.length;
      var tex = state._texBindings[0x0DE1];
      if (n >= 9) {
        var target = +args[0];
        if (!_ZW_TEX_BIND_TARGETS[target]) { _zwGlSetError(state, 0x0500); return; }
        var level = _zwGlToInteger(args[1]);
        var x = _zwGlToInteger(args[2]);
        var y = _zwGlToInteger(args[3]);
        var w = _zwGlToInteger(args[4]);
        var h = _zwGlToInteger(args[5]);
        var pixels = args[8];
        if (level < 0 || x < 0 || y < 0 || w <= 0 || h <= 0 ||
          (pixels !== null && pixels !== undefined && !_zwGlIsBufferSource(pixels))) {
          _zwGlSetError(state, 0x0501);
          return;
        }
        if (tex) tex._zwHasBase = true;
      } else if (n === 7) {
        if (!_zwGlIsTexImageSource(args[6])) {
          throw new TypeError('parameter 7 is not of type TexImageSource');
        }
        if (tex) tex._zwHasBase = true;
      } else {
        throw new TypeError('texSubImage2D: no matching overload');
      }
    },

    // —— program / uniform 管线（M3-S2；corpus uniformMatrixNfv.html 的
    // null location no-throw 面保持：未链接 program 恒 INVALID_OPERATION + null，
    // null location 静默）——
    getUniformLocation: function (state, args) {
      var program = args[0];
      _zwGlBrandCheck(program, 'WebGLProgram', 1);
      if (!program._zwLinkOk) { // 未成功链接 → INVALID_OPERATION + null（§5.14.9）
        _zwGlSetError(state, 0x0502);
        return null;
      }
      var name = args.length > 1 ? String(args[1]) : 'null';
      var link = program._zwLink;
      var ent = link.uniformMap[name];
      if (!ent) {
        // 数组元素形 'u[N]'：基名为数组 uniform 且 N < size → 基址 + N；
        // 其余未知名 → null 无错误（§5.14.9）。
        var m = _ZW_ARRAY_LOC_RE.exec(name);
        if (m) {
          var base = link.uniformMap[m[1]];
          if (base && base.size > 1 && +m[2] < base.size) {
            return _zwGlUniformLocation(program, base, name, base.index + +m[2]);
          }
        }
        return null;
      }
      return _zwGlUniformLocation(program, ent, name, ent.index);
    },
    bindAttribLocation: function (state, args) {
      // GLES2 §2.10.3：index ≥ MAX_VERTEX_ATTRIBS → INVALID_VALUE；gl_ 前缀 →
      // INVALID_OPERATION（§3.7 保留前缀）；绑定在下次 linkProgram 生效。
      var program = args[0];
      _zwGlBrandCheck(program, 'WebGLProgram', 1);
      var index = _zwGlToInteger(args[1]);
      var name = String(args[2]);
      if (index < 0 || index >= _ZW_MAX_VERTEX_ATTRIBS) { _zwGlSetError(state, 0x0501); return; }
      if (name.slice(0, 3) === 'gl_') { _zwGlSetError(state, 0x0502); return; }
      var binds = program._zwAttribBinds || (program._zwAttribBinds = {});
      binds[name] = index;
    },
    uniform1f: function (state, args) { _zwGlUniformLoad(state, args, 1, false, false, false); },
    uniform2f: function (state, args) { _zwGlUniformLoad(state, args, 2, false, false, false); },
    uniform3f: function (state, args) { _zwGlUniformLoad(state, args, 3, false, false, false); },
    uniform4f: function (state, args) { _zwGlUniformLoad(state, args, 4, false, false, false); },
    uniform1i: function (state, args) { _zwGlUniformLoad(state, args, 1, true, false, false); },
    uniform2i: function (state, args) { _zwGlUniformLoad(state, args, 2, true, false, false); },
    uniform3i: function (state, args) { _zwGlUniformLoad(state, args, 3, true, false, false); },
    uniform4i: function (state, args) { _zwGlUniformLoad(state, args, 4, true, false, false); },
    uniform1fv: function (state, args) { _zwGlUniformLoad(state, args, 1, false, false, true); },
    uniform2fv: function (state, args) { _zwGlUniformLoad(state, args, 2, false, false, true); },
    uniform3fv: function (state, args) { _zwGlUniformLoad(state, args, 3, false, false, true); },
    uniform4fv: function (state, args) { _zwGlUniformLoad(state, args, 4, false, false, true); },
    uniform1iv: function (state, args) { _zwGlUniformLoad(state, args, 1, true, false, true); },
    uniform2iv: function (state, args) { _zwGlUniformLoad(state, args, 2, true, false, true); },
    uniform3iv: function (state, args) { _zwGlUniformLoad(state, args, 3, true, false, true); },
    uniform4iv: function (state, args) { _zwGlUniformLoad(state, args, 4, true, false, true); },
    uniformMatrix2fv: function (state, args) { _zwGlUniformLoad(state, args, 4, false, true, true); },
    uniformMatrix3fv: function (state, args) { _zwGlUniformLoad(state, args, 9, false, true, true); },
    uniformMatrix4fv: function (state, args) { _zwGlUniformLoad(state, args, 16, false, true, true); },
    getUniform: function (state, args) {
      var program = args[0], loc = args[1];
      _zwGlBrandCheck(program, 'WebGLProgram', 1);
      _zwGlBrandCheck(loc, 'WebGLUniformLocation', 2);
      // location 不属该 program / 已失效（重链）→ INVALID_OPERATION + null。
      if (!program._zwLinkOk || loc._zwProgram !== program ||
        loc._zwStamp !== program._zwStamp) {
        _zwGlSetError(state, 0x0502);
        return null;
      }
      var ent = loc._zwEnt;
      var comps = _zwGlTypeComponents(ent.type) || 1;
      var run = program._zwLink.uniformRuns[ent.index];
      var arr = run.arr;
      if (!arr) { // 未上传 → 规范初值全 0（GLES2 §2.10.4）
        arr = run.arr = _zwGlIsIntFamily(ent.type)
          ? new Int32Array(ent.size * comps)
          : new Float32Array(ent.size * comps);
      }
      var off = (loc._zwIndex - ent.index) * comps;
      if (comps === 1) {
        var v0 = arr[off];
        return ent.type === 0x8B56 ? !!v0 : v0; // BOOL uniform → boolean（§5.14.9 表）
      }
      var out = _zwGlIsIntFamily(ent.type) ? new Int32Array(comps) : new Float32Array(comps);
      for (var i = 0; i < comps; i++) out[i] = arr[off + i];
      return out;
    },

    // —— 绘制状态机（corpus 无直接案；draw 状态语义为 M2 Mission 面）——
    drawArrays: function (state, args) {
      var mode = +args[0];
      if (!_ZW_BEGIN_MODES[mode]) { _zwGlSetError(state, 0x0500); return; }
      // 无合法 program → INVALID_OPERATION（GL spec；M2 program 恒未链接）。
      if (!state._program) { _zwGlSetError(state, 0x0502); return; }
      _zwGlSetError(state, 0x0502); // FIXME(M3-S4 draw 光栅化)：真实绘制管线。
    },
    drawElements: function (state, args) {
      var mode = +args[0];
      if (!_ZW_BEGIN_MODES[mode]) { _zwGlSetError(state, 0x0500); return; }
      var type = +args[2];
      if (type !== 0x1401 && type !== 0x1403) { _zwGlSetError(state, 0x0500); return; } // UNSIGNED_BYTE/SHORT
      if (!state._program) { _zwGlSetError(state, 0x0502); return; }
      _zwGlSetError(state, 0x0502); // FIXME(M3-S4 draw 光栅化)。
    },
    clearColor: function (state, args) {
      state._clearColor[0] = +args[0] || 0;
      state._clearColor[1] = +args[1] || 0;
      state._clearColor[2] = +args[2] || 0;
      state._clearColor[3] = args.length > 3 ? +args[3] : 0;
      // NaN 分量取 0（WebGL §6.20：NaN 入状态调用无错误，按 0 处理）。
    },
    colorMask: function (state, args) {
      state._colorMask[0] = !!args[0];
      state._colorMask[1] = !!args[1];
      state._colorMask[2] = !!args[2];
      state._colorMask[3] = !!args[3];
    },
    scissor: function (state, args) {
      var x = _zwGlToInteger(args[0]);
      var y = _zwGlToInteger(args[1]);
      var w = _zwGlToInteger(args[2]);
      var h = _zwGlToInteger(args[3]);
      if (w < 0 || h < 0) { _zwGlSetError(state, 0x0501); return; }
      state._scissor = [x, y, w, h];
    },
    viewport: function (state, args) {
      state._viewport = [_zwGlToInteger(args[0]), _zwGlToInteger(args[1]),
        _zwGlToInteger(args[2]), _zwGlToInteger(args[3])];
    },
    enable: function (state, args) {
      var cap = +args[0];
      if (!_ZW_CAPS[cap]) { _zwGlSetError(state, 0x0500); return; }
      state._enabled[cap] = true;
    },
    disable: function (state, args) {
      var cap = +args[0];
      if (!_ZW_CAPS[cap]) { _zwGlSetError(state, 0x0500); return; }
      state._enabled[cap] = false;
    },

    // —— 状态设置族（M3-S1 查询面对应写入口；GLES2 Table 6.9/6.10 域校验）——
    isEnabled: function (state, args) {
      var cap = +args[0];
      if (!_ZW_CAPS[cap]) { _zwGlSetError(state, 0x0500); return false; }
      return !!state._enabled[cap];
    },
    depthMask: function (state, args) { state._depthMask = !!args[0]; },
    frontFace: function (state, args) {
      var mode = +args[0];
      if (!_ZW_FRONT_FACES[mode]) { _zwGlSetError(state, 0x0500); return; }
      state._frontFace = mode;
    },
    cullFace: function (state, args) {
      var mode = +args[0];
      if (!_ZW_CULL_MODES[mode]) { _zwGlSetError(state, 0x0500); return; }
      state._cullFaceMode = mode;
    },
    depthFunc: function (state, args) {
      var fn = +args[0];
      if (!_ZW_DEPTH_FUNCS[fn]) { _zwGlSetError(state, 0x0500); return; }
      state._depthFunc = fn;
    },
    depthRange: function (state, args) {
      // zNear/zFar clamp 到 [0,1]（GLES2 §2.13——越界截取不报错）。
      var near = Math.min(Math.max(+args[0] || 0, 0), 1);
      var far = Math.min(Math.max(+args[1] || 0, 0), 1);
      state._depthRange = [isNaN(near) ? 0 : near, isNaN(far) ? 1 : far];
    },
    lineWidth: function (state, args) {
      var w = +args[0];
      if (isNaN(w) || w <= 0) { _zwGlSetError(state, 0x0501); return; } // INVALID_VALUE
      // WebGL §6.22：非 1 宽度恒不受支持（ALIASED_LINE_WIDTH_RANGE [1,1]）——
      // 状态记录、实际线宽恒 1。
      state._lineWidth = Math.min(Math.max(w, 1), 1);
    },
    sampleCoverage: function (state, args) {
      var v = +args[0];
      state._sampleCoverageValue = isNaN(v) ? 1 : Math.min(Math.max(v, 0), 1);
      state._sampleCoverageInvert = !!args[1];
    },
    blendColor: function (state, args) {
      for (var i = 0; i < 4; i++) state._blendColor[i] = +args[i] || 0; // NaN→0（§6.20）
    },
    blendFunc: function (state, args) {
      var src = +args[0];
      var dst = +args[1];
      if (!_ZW_BLEND_FACTORS[src] || !_ZW_BLEND_FACTORS[dst]) {
        _zwGlSetError(state, 0x0500);
        return;
      }
      state._blendSrcRGB = src; state._blendDstRGB = dst;
      state._blendSrcAlpha = src; state._blendDstAlpha = dst;
    },
    blendEquation: function (state, args) {
      var mode = +args[0];
      // WebGL 1.0 core 仅 FUNC_ADD/FUNC_SUBTRACT/FUNC_REVERSE_SUBTRACT
      //（MIN/MAX 属 EXT_blend_minmax，不在 webgl1.idl 常量集）。
      if (mode !== 0x8006 && mode !== 0x800A && mode !== 0x800B) {
        _zwGlSetError(state, 0x0500);
        return;
      }
      state._blendEquationRGB = mode; state._blendEquationAlpha = mode;
    },
    stencilFunc: function (state, args) {
      var fn = +args[0];
      if (!_ZW_DEPTH_FUNCS[fn]) { _zwGlSetError(state, 0x0500); return; }
      state._stencilFunc = fn;
      state._stencilRef = +args[1] || 0;
      var m = args.length > 2 ? +args[2] : 0xFFFFFFFF;
      state._stencilValueMask = isNaN(m) ? 0xFFFFFFFF : (m >>> 0); // mask=0 合法，不得回退全掩码
    },

    // —— clear（标尺①：本地 reftest 光栅化面）——
    // 颜色清屏经**既有 host 2d 上下文**（source-over 不透明填充 = GL 替换语义
    // 逐位一致；scissor 开启时按 GL 底左原点换算裁剪矩形）。不用 'copy' 合成：
    // host path 光栅对 source-exclusive 合成附带 uncovered 清除，会擦掉裁剪矩形
    // 外的既有像素（违反 GL clear 只写 scissor 矩形的语义——part66 scissor 用例
    // 实证）。a<1 的 premultiply/替换约定差与 depth/stencil 缺席 = FIXME(M3
    // 独立 GL 呈现面)。host 未注册（纯 shim 单测环境）→ 静默跳过光栅。
    clear: function (state, args) {
      var mask = args.length > 0 ? (+args[0] || 0) : 0;
      if (isNaN(mask) || (mask & ~_ZW_CLEAR_MASKS) !== 0) {
        _zwGlSetError(state, 0x0500);
        return;
      }
      if ((mask & 0x4000) === 0) return; // COLOR_BUFFER_BIT
      var m = state._colorMask;
      if (!m[0] && !m[1] && !m[2] && !m[3]) return;
      var w = state._drawingBufferWidth;
      var h = state._drawingBufferHeight;
      if (w <= 0 || h <= 0) return;
      var x = 0;
      var y = 0;
      var rw = w;
      var rh = h;
      if (state._enabled[0x0C11]) { // SCISSOR_TEST
        var s = state._scissor;
        // GL 底左原点 → host 2d 顶左原点：y' = h - y - sh。
        x = s[0];
        y = h - s[1] - s[3];
        rw = s[2];
        rh = s[3];
        if (x < 0) { rw += x; x = 0; }
        if (y < 0) { rh += y; y = 0; }
        if (rw > w - x) rw = w - x;
        if (rh > h - y) rh = h - y;
        if (rw <= 0 || rh <= 0) return;
      }
      if (typeof globalThis.__zw_canvas_op !== 'function' || !state._handle) return;
      var c = state._clearColor;
      var q = function (v) {
        var b = Math.round(Math.min(Math.max(v, 0), 1) * 255);
        return isFinite(b) ? b : 0;
      };
      var style = c[3] >= 1
        ? 'rgb(' + q(c[0]) + ',' + q(c[1]) + ',' + q(c[2]) + ')'
        : 'rgba(' + q(c[0]) + ',' + q(c[1]) + ',' + q(c[2]) + ',' +
          Math.min(Math.max(c[3], 0), 1) + ')';
      globalThis.__zw_canvas_op(state._handle, 'setCompositeOperation', 'source-over');
      globalThis.__zw_canvas_op(state._handle, 'setGlobalAlpha', '1');
      globalThis.__zw_canvas_op(state._handle, 'setFillStyle', style);
      globalThis.__zw_canvas_op(state._handle, 'fillRect', String(x), String(y),
        String(rw), String(rh));
    },

    // —— 参数查询（M3-S1 全表；WebGL 1.0 §5.14.6 + GLES2 2.0.25 Table 6.9/6.10/
    //     6.18 + WebGL §6.4 最小支持上限）——
    // 实现相关上限取 WebGL §6.4 表最小值（FIXME(M3 管线簇)：接 render-foundation
    // 真实 limits）；绑定/状态值读自 state；typed array 返回形按 GLES2 表列型。
    getParameter: function (state, args) {
      var pname = args.length > 0 ? +args[0] : 0;
      switch (pname) {
        // —— 字符串身份（WebGL §5.14.5）——
        case 0x1F00: return 'ZeroWeb';                  // VENDOR
        case 0x1F01: return 'ZeroWeb';                  // RENDERER
        case 0x1F02: return 'WebGL 1.0';                // VERSION
        case 0x8B8C: return 'WebGL GLSL ES 1.0';        // SHADING_LANGUAGE_VERSION
        // —— 绘图缓冲构成 ——
        case 0x0D50: return 4;                          // SUBPIXEL_BITS（Table 6.18 固定 4）
        case 0x0D52: return 8;                          // RED_BITS
        case 0x0D53: return 8;                          // GREEN_BITS
        case 0x0D54: return 8;                          // BLUE_BITS
        case 0x0D55: return state._attrs.alpha ? 8 : 0; // ALPHA_BITS
        case 0x0D56: return 0;                          // DEPTH_BITS——FIXME(M3)：无 depth attachment
        case 0x0D57: return 0;                          // STENCIL_BITS——同上
        case 0x80A8: return 0;                          // SAMPLE_BUFFERS（无 MSAA 绘图缓冲）
        case 0x80A9: return 0;                          // SAMPLES
        // —— 实现相关上限（WebGL §6.4 最小支持表）——
        case 0x0D33: return 4096;                       // MAX_TEXTURE_SIZE
        case 0x851C: return 4096;                       // MAX_CUBE_MAP_TEXTURE_SIZE
        case 0x84E8: return 4096;                       // MAX_RENDERBUFFER_SIZE
        case 0x8869: return 16;                         // MAX_VERTEX_ATTRIBS
        case 0x8DFB: return 128;                        // MAX_VERTEX_UNIFORM_VECTORS
        case 0x8DFD: return 16;                         // MAX_FRAGMENT_UNIFORM_VECTORS
        case 0x8DFC: return 16;                         // MAX_VARYING_VECTORS
        case 0x8872: return 16;                         // MAX_TEXTURE_IMAGE_UNITS
        case 0x8B4C: return 16;                         // MAX_VERTEX_TEXTURE_IMAGE_UNITS
        case 0x8B4D: return 32;                         // MAX_COMBINED_TEXTURE_IMAGE_UNITS
        case 0x0D3A:                                    // MAX_VIEWPORT_DIMS（Int32Array）
          return new Int32Array([16384, 16384]);        // FIXME(M3)：对齐 MAX_CANVAS_DIM
        case 0x846D: return new Float32Array([1, 1024]);  // ALIASED_POINT_SIZE_RANGE
        case 0x846E: return new Float32Array([1, 1]);     // ALIASED_LINE_WIDTH_RANGE（§6.22 恒 [1,1]）
        // —— 压缩纹理格式表（使能扩展写入 _compressedFormats）——
        case 0x86A3: {                                  // COMPRESSED_TEXTURE_FORMATS（Uint32Array；pin rev IDL 0x86A3）
          var arr = [];
          for (var kc in state._compressedFormats) arr.push(+kc);
          return new Uint32Array(arr);
        }
        case 0x8B9B: return 0x1908;                     // IMPLEMENTATION_COLOR_READ_FORMAT = RGBA
        case 0x8B9A: return 0x1401;                     // IMPLEMENTATION_COLOR_READ_TYPE = UNSIGNED_BYTE
        // —— 像素/深度/模板状态（Table 6.9/6.10）——
        case 0x0BA2: return state._viewport;            // VIEWPORT
        case 0x0C10: return state._scissor;             // SCISSOR_BOX
        case 0x0B70: return new Float32Array(state._depthRange);  // DEPTH_RANGE
        case 0x0B21: return state._lineWidth;           // LINE_WIDTH
        case 0x0B44: return !!state._enabled[0x0B44];   // CULL_FACE
        case 0x0B45: return state._cullFaceMode;        // CULL_FACE_MODE
        case 0x0B46: return state._frontFace;           // FRONT_FACE
        case 0x0B71: return !!state._enabled[0x0B71];   // DEPTH_TEST
        case 0x0B74: return state._depthFunc;           // DEPTH_FUNC
        case 0x0B72: return state._depthMask;           // DEPTH_WRITEMASK（pin rev IDL 0x0B72）
        case 0x0B73: return 1;                          // DEPTH_CLEAR_VALUE（pin rev IDL 0x0B73；标量，Table 6.9 初始 1）
        case 0x0B90: return !!state._enabled[0x0B90];   // STENCIL_TEST
        case 0x0B92: return state._stencilFunc;         // STENCIL_FUNC
        case 0x0B93: return state._stencilValueMask;    // STENCIL_VALUE_MASK
        case 0x0B94: case 0x0B95: case 0x0B96:          // STENCIL_FAIL/PASS_DEPTH_FAIL/PASS_DEPTH_PASS
          return 0x1E00;                                // KEEP 默认（stencilOp setter M3-S3）
        case 0x0B97: return state._stencilRef;          // STENCIL_REF
        case 0x0B98: return state._stencilWritemask;    // STENCIL_WRITEMASK
        case 0x8005: return state._blendColor ? new Float32Array(state._blendColor) : new Float32Array([0, 0, 0, 0]); // BLEND_COLOR（pin rev IDL 0x8005）
        case 0x0BE2: return !!state._enabled[0x0BE2];   // BLEND
        case 0x8009: return state._blendEquationRGB;    // BLEND_EQUATION（RGB 同名同值）
        case 0x883D: return state._blendEquationAlpha;  // BLEND_EQUATION_ALPHA
        case 0x80C9: return state._blendSrcRGB;         // BLEND_SRC_RGB
        case 0x80C8: return state._blendDstRGB;         // BLEND_DST_RGB
        case 0x80CB: return state._blendSrcAlpha;       // BLEND_SRC_ALPHA
        case 0x80CA: return state._blendDstAlpha;       // BLEND_DST_ALPHA
        case 0x80AA: return state._sampleCoverageValue;   // SAMPLE_COVERAGE_VALUE
        case 0x80AB: return state._sampleCoverageInvert;  // SAMPLE_COVERAGE_INVERT
        case 0x809E: return !!state._enabled[0x809E];   // SAMPLE_ALPHA_TO_COVERAGE
        case 0x80A0: return !!state._enabled[0x80A0];   // SAMPLE_COVERAGE
        case 0x0BD0: return !!state._enabled[0x0BD0];   // DITHER
        case 0x8037: return !!state._enabled[0x8037];   // POLYGON_OFFSET_FILL（pin rev IDL 0x8037）
        case 0x0C11: return !!state._enabled[0x0C11];   // SCISSOR_TEST
        case 0x0C23: return state._colorMask.slice();   // COLOR_WRITEMASK
        case 0x0C22: return new Float32Array(state._clearColor); // COLOR_CLEAR_VALUE（pin rev IDL 0x0C22）
        case 0x0B91: return state._stencilClearValue !== undefined ? state._stencilClearValue : 0; // STENCIL_CLEAR_VALUE（Table 6.9 初始 0）
        // —— 绑定对象（§5.14.6 绑定查询）——
        case 0x8894: return state._bindings[0x8892];    // ARRAY_BUFFER_BINDING
        case 0x8895: return state._bindings[0x8893];    // ELEMENT_ARRAY_BUFFER_BINDING
        case 0x8069: return state._texBindings[0x0DE1]; // TEXTURE_BINDING_2D
        case 0x8514: return state._texBindings[0x8513]; // TEXTURE_BINDING_CUBE_MAP
        case 0x8B8D: return state._program;             // CURRENT_PROGRAM
        // —— WebGL 专有像素存取参数（WebGL §5.14.8；经 pixelStorei 入 _pixelStore）——
        case 0x9240: return !!state._pixelStore['0x9240'];  // UNPACK_FLIP_Y_WEBGL
        case 0x9241: return !!state._pixelStore['0x9241'];  // UNPACK_PREMULTIPLY_ALPHA_WEBGL
        case 0x9243:                                          // UNPACK_COLORSPACE_CONVERSION_WEBGL
          return state._pixelStore['0x9243'] !== undefined ? state._pixelStore['0x9243'] : 0x9244; // BROWSER_DEFAULT_WEBGL
        default:
          _zwGlSetError(state, 0x0500);
          return null;
      }
    },
    pixelStorei: function (state, args) {
      var pname = +args[0];
      if (pname === 0x0CF5) { // UNPACK_ALIGNMENT（GLES2 §3.6.1——仅 1/2/4/8）
        var align = _zwGlToInteger(args[1]);
        if (align !== 1 && align !== 2 && align !== 4 && align !== 8) {
          _zwGlSetError(state, 0x0501); // INVALID_VALUE
          return;
        }
      }
      state._pixelStore[String(pname)] = _zwGlToInteger(args[1]);
    },

    // —— shader/program 管线（M3-S2：GLSL ES 1.0 编译 + 链接 + 反射面）——
    shaderSource: function (state, args) {
      var sh = args[0];
      if (!sh || !sh._zwGlObj || sh._zwGlIface !== 'WebGLShader') {
        throw new TypeError('parameter 1 is not of type WebGLShader');
      }
      sh._zwSource = args.length > 1 ? String(args[1]) : '';
    },
    compileShader: function (state, args) {
      var sh = args[0];
      _zwGlBrandCheck(sh, 'WebGLShader', 1);
      // GLSL ES 1.0 前端编译（part08b 单点 _zwGlCompileGlsl——校验 + 活跃声明
      // 提取）。编译失败不产生 GL 错误：COMPILE_STATUS false + infoLog 非空
      //（WebGL 1.0 §5.14.9）。
      var res = globalThis._zwGlCompileGlsl(sh._zwSource || '', sh._zwType === 0x8B31);
      sh._zwCompileOk = !!res.ok;
      sh._zwInfoLog = res.infoLog || '';
      sh._zwAttribs = res.attributes || [];
      sh._zwUniforms = res.uniforms || [];
    },
    attachShader: function (state, args) {
      // GLES2 §2.10.3：同一 shader 重复挂接同一 program → INVALID_OPERATION。
      var program = args[0], sh = args[1];
      _zwGlBrandCheck(program, 'WebGLProgram', 1);
      _zwGlBrandCheck(sh, 'WebGLShader', 2);
      var list = program._zwShaders || (program._zwShaders = []);
      for (var i = 0; i < list.length; i++) {
        if (list[i] === sh) { _zwGlSetError(state, 0x0502); return; }
      }
      list.push(sh);
    },
    detachShader: function (state, args) {
      // 未挂接 → INVALID_OPERATION（§2.10.3）。
      var program = args[0], sh = args[1];
      _zwGlBrandCheck(program, 'WebGLProgram', 1);
      _zwGlBrandCheck(sh, 'WebGLShader', 2);
      var list = program._zwShaders || [];
      for (var i = 0; i < list.length; i++) {
        if (list[i] === sh) {
          list.splice(i, 1);
          return;
        }
      }
      _zwGlSetError(state, 0x0502);
    },
    linkProgram: function (state, args) {
      var program = args[0];
      _zwGlBrandCheck(program, 'WebGLProgram', 1);
      _zwGlLinkProgram(program);
    },
    useProgram: function (state, args) {
      var p = args[0];
      if (p !== null && p !== undefined &&
        (!p._zwGlObj || p._zwGlIface !== 'WebGLProgram')) {
        throw new TypeError('parameter 1 is not of type WebGLProgram');
      }
      state._program = p || null;
    },
    getShaderParameter: function (state, args) {
      var sh = args[0];
      _zwGlBrandCheck(sh, 'WebGLShader', 1);
      var pname = args.length > 1 ? +args[1] : 0;
      switch (pname) {
        case 0x8B81: return !!sh._zwCompileOk;    // COMPILE_STATUS
        case 0x8B80: return !!sh._zwDeleted;      // DELETE_STATUS
        case 0x8B4F: return sh._zwType || 0;      // SHADER_TYPE
        default: _zwGlSetError(state, 0x0500); return null;
      }
    },
    getProgramParameter: function (state, args) {
      var prog = args[0];
      _zwGlBrandCheck(prog, 'WebGLProgram', 1);
      var pname = args.length > 1 ? +args[1] : 0;
      switch (pname) {
        case 0x8B82: return !!prog._zwLinkOk;     // LINK_STATUS
        case 0x8B83: return false;                // VALIDATE_STATUS FIXME(S4)：validateProgram 真语义随 draw 簇
        case 0x8B80: return !!prog._zwDeleted;    // DELETE_STATUS
        case 0x8B85: return prog._zwShaders ? prog._zwShaders.length : 0; // ATTACHED_SHADERS
        case 0x8B86: return prog._zwLinkOk ? prog._zwLink.uniformList.length : 0; // ACTIVE_UNIFORMS
        case 0x8B89: return prog._zwLinkOk ? prog._zwLink.attribList.length : 0;  // ACTIVE_ATTRIBUTES
        default: _zwGlSetError(state, 0x0500); return null;
      }
    },
    getShaderInfoLog: function (state, args) {
      var sh = args[0];
      _zwGlBrandCheck(sh, 'WebGLShader', 1);
      return sh._zwInfoLog || '';
    },
    getProgramInfoLog: function (state, args) {
      var prog = args[0];
      _zwGlBrandCheck(prog, 'WebGLProgram', 1);
      return prog._zwLinkLog || '';
    },
    getAttribLocation: function (state, args) {
      var program = args[0];
      _zwGlBrandCheck(program, 'WebGLProgram', 1);
      if (!program._zwLinkOk) { // 未链接 → INVALID_OPERATION + -1（M2 面保持）
        _zwGlSetError(state, 0x0502);
        return -1;
      }
      var name = args.length > 1 ? String(args[1]) : 'null';
      var ent = program._zwLink.attribMap[name];
      return ent ? ent.index : -1; // 未知名 → -1 无错误（§5.14.9）
    },
    getActiveAttrib: function (state, args) {
      var program = args[0];
      _zwGlBrandCheck(program, 'WebGLProgram', 1);
      if (!program._zwLinkOk) { _zwGlSetError(state, 0x0502); return null; }
      var index = _zwGlToInteger(args[1]);
      var list = program._zwLink.attribList;
      if (index < 0 || index >= list.length) { _zwGlSetError(state, 0x0501); return null; }
      var ent = list[index];
      var info = _zwGlNewObject('WebGLActiveInfo');
      info._name = ent.name; info._size = ent.size; info._type = ent.type;
      return info;
    },
    getActiveUniform: function (state, args) {
      var program = args[0];
      _zwGlBrandCheck(program, 'WebGLProgram', 1);
      if (!program._zwLinkOk) { _zwGlSetError(state, 0x0502); return null; }
      var index = _zwGlToInteger(args[1]);
      var list = program._zwLink.uniformList;
      if (index < 0 || index >= list.length) { _zwGlSetError(state, 0x0501); return null; }
      var ent = list[index];
      var info = _zwGlNewObject('WebGLActiveInfo');
      // 数组 uniform 的 name 面为 'u[0]'（WebGL 1.0 §5.14.10）。
      info._name = ent.isArray ? ent.name + '[0]' : ent.name;
      info._size = ent.size; info._type = ent.type;
      return info;
    }
  };

  // ── shader/program 管线共享机械（M3-S2）────────────────────────────────
  function _zwGlBrandCheck(obj, iface, paramNo) {
    if (!obj || !obj._zwGlObj || obj._zwGlIface !== iface) {
      throw new TypeError('parameter ' + paramNo + ' is not of type ' + iface);
    }
  }

  var _ZW_MAX_VERTEX_ATTRIBS = 16; // WebGL §6.4 最小上限（getParameter 同源）
  var _ZW_ARRAY_LOC_RE = /^(.+)\[(\d+)\]$/;

  // GLSL 类型 → 分量数（GLES2 Table 6.14——uniform 加载族匹配面）。
  function _zwGlTypeComponents(t) {
    switch (t) {
      case 0x1406: case 0x1404: case 0x8B56: return 1; // FLOAT/INT/BOOL
      case 0x8B50: case 0x8B53: case 0x8B57: return 2; // *_VEC2
      case 0x8B51: case 0x8B54: case 0x8B58: return 3; // *_VEC3
      case 0x8B52: case 0x8B55: case 0x8B59: return 4; // *_VEC4
      case 0x8B5A: return 4;                            // MAT2
      case 0x8B5B: return 9;                            // MAT3
      case 0x8B5C: return 16;                           // MAT4
      default: return 0;                                // 采样器按标量处理
    }
  }
  function _zwGlIsSampler(t) { return t === 0x8B5E || t === 0x8B60; }
  // getUniform 返回 Int32Array/整数的类型面（int/bool 族 + 采样器）。
  function _zwGlIsIntFamily(t) {
    return t === 0x1404 || t === 0x8B56 || t === 0x8B53 || t === 0x8B54 ||
      t === 0x8B55 || t === 0x8B57 || t === 0x8B58 || t === 0x8B59 || _zwGlIsSampler(t);
  }

  // linkProgram（GLES2 §2.10.5 / WebGL 1.0 §5.14.9）：链接失败**不产生 GL 错误**
  // ——LINK_STATUS false + infoLog 非空。active uniform 跨阶段合并（同名必同型
  // 同长）；location 声明序分配（顶点阶段在前、片元独有续后，数组每元素一槽）；
  // active attribute 先应用 bindAttribLocation 再自动分配（冲突 → 失败）。
  // 链接使全部旧 location 失效（stamp 递增——§5.14.9 重链失效面）。
  function _zwGlLinkProgram(program) {
    program._zwStamp = (program._zwStamp || 0) + 1;
    function fail(log) {
      program._zwLinkOk = false;
      program._zwLinkLog = log;
      program._zwLink = { ok: false, uniformMap: {}, uniformList: [],
        attribMap: {}, attribList: [], uniformRuns: {}, locCache: {} };
    }
    var attached = program._zwShaders || [];
    var vs = null, fs = null, vsCount = 0, fsCount = 0, i, j;
    for (i = 0; i < attached.length; i++) {
      if (attached[i]._zwType === 0x8B31) { vsCount++; vs = attached[i]; }
      else if (attached[i]._zwType === 0x8B30) { fsCount++; fs = attached[i]; }
    }
    if (vsCount !== 1) {
      return fail(vsCount === 0 ? 'no vertex shader attached' : 'multiple vertex shaders attached');
    }
    if (fsCount !== 1) {
      return fail(fsCount === 0 ? 'no fragment shader attached' : 'multiple fragment shaders attached');
    }
    if (!vs._zwCompileOk) return fail('vertex shader has not been compiled successfully');
    if (!fs._zwCompileOk) return fail('fragment shader has not been compiled successfully');

    // active uniform 合并：顶点阶段声明序在前，片元独有续后；同名必同型同长。
    var uniformMap = {}, uniformList = [], uniformRuns = {}, locCache = {};
    var stages = [vs._zwUniforms, fs._zwUniforms];
    for (i = 0; i < stages.length; i++) {
      for (j = 0; j < stages[i].length; j++) {
        var u = stages[i][j];
        if (uniformMap[u.name]) {
          var prev = uniformMap[u.name];
          if (prev.type !== u.type || prev.size !== u.size) {
            return fail('uniform ' + u.name +
              ' declared with different type or size in vertex and fragment shaders');
          }
          continue;
        }
        var ent = { name: u.name, type: u.type, size: u.size, isArray: u.size > 1 };
        uniformMap[u.name] = ent;
        if (ent.isArray) uniformMap[u.name + '[0]'] = ent; // 'u' 与 'u[0]' 同址查询面
        uniformList.push(ent);
      }
    }
    var next = 0;
    for (i = 0; i < uniformList.length; i++) {
      uniformList[i].index = next;
      uniformRuns[next] = { comps: _zwGlTypeComponents(uniformList[i].type) || 1, arr: null };
      next += uniformList[i].size;
    }

    // active attribute：bindAttribLocation 先行（冲突 → 失败），其余自动分配。
    var attribMap = {}, attribList = [], used = {};
    var binds = program._zwAttribBinds || {};
    var vattrs = vs._zwAttribs;
    for (i = 0; i < vattrs.length; i++) {
      var a = vattrs[i];
      var bindIdx = binds[a.name];
      if (bindIdx !== undefined) {
        if (used[bindIdx]) {
          return fail('two active attributes bound to the same index ' + bindIdx);
        }
        used[bindIdx] = true;
        attribMap[a.name] = { name: a.name, type: a.type, size: a.size, index: bindIdx };
      } else {
        attribList.push(a); // 待自动分配（保持声明序）
      }
    }
    var autoIdx = 0;
    for (i = 0; i < attribList.length; i++) {
      while (used[autoIdx]) autoIdx++;
      if (autoIdx >= _ZW_MAX_VERTEX_ATTRIBS) {
        return fail('out of vertex attribute indices');
      }
      used[autoIdx] = true;
      attribMap[attribList[i].name] = {
        name: attribList[i].name, type: attribList[i].type,
        size: attribList[i].size, index: autoIdx
      };
      autoIdx++;
    }
    var ordered = [];
    for (i = 0; i < vattrs.length; i++) ordered.push(attribMap[vattrs[i].name]);

    program._zwLinkOk = true;
    program._zwLinkLog = '';
    program._zwLink = { ok: true, uniformMap: uniformMap, uniformList: uniformList,
      attribMap: attribMap, attribList: ordered, uniformRuns: uniformRuns, locCache: locCache };
  }

  // location 工厂：同 program 同名同 stamp 返回同一对象（§5.14.9 对象同一性面）；
  // 缓存挂 link 状态——重链后缓存重建，旧对象经 stamp 失配自然失效。
  function _zwGlUniformLocation(program, ent, name, index) {
    var cache = program._zwLink.locCache;
    if (cache[name]) return cache[name];
    var loc = _zwGlNewObject('WebGLUniformLocation');
    loc._zwProgram = program;
    loc._zwStamp = program._zwStamp;
    loc._zwIndex = index;
    loc._zwEnt = ent;
    cache[name] = loc;
    return loc;
  }

  // uniform* 通用加载（GLES2 §2.10.4 + WebGL §6.20）：location null → 静默
  //（corpus uniformMatrixNfv no-throw 面）；品牌 → TypeError；不属当前 program/
  // 重链失效 → INVALID_OPERATION。transpose ToBoolean（不触 valueOf——corpus
  // {valueOf: throw} 转换面），WebGL 1.0 仅接受 false。float 值 NaN/±Inf →
  // INVALID_VALUE（§6.20）。类型匹配：标量 setter 仅标量 uniform；向量 setter
  // 按分量数；矩阵 setter 仅同阶矩阵；采样器仅 uniform1i/1iv。值按 uniform
  // 类型族存储（Float32Array/Int32Array），偏移 = (location 槽 - 基槽) × 分量。
  function _zwGlUniformLoad(state, args, comps, isInt, isMatrix, seq) {
    var loc = args[0];
    if (loc !== null && loc !== undefined &&
      (!loc._zwGlObj || loc._zwGlIface !== 'WebGLUniformLocation')) {
      throw new TypeError('parameter 1 is not of type WebGLUniformLocation');
    }
    if (loc === null || loc === undefined) return;
    if (isMatrix && !!args[1]) { // WebGL 1.0——transpose 必须为 false
      _zwGlSetError(state, 0x0501);
      return;
    }
    var program = state._program;
    if (!program || loc._zwProgram !== program ||
      loc._zwStamp !== program._zwStamp || !program._zwLinkOk) {
      _zwGlSetError(state, 0x0502);
      return;
    }
    // 值读取：v 形取 sequence（长度须为分量数整数倍且 ≤ uniform 元素容量）；
    // 分量形逐参取 args[1..comps]。
    var values = [], nv = 0, v, i;
    if (seq) {
      v = args[isMatrix ? 2 : 1];
      if (!Array.isArray(v) && !(typeof v === 'object' && v && typeof v.length === 'number')) {
        throw new TypeError('parameter ' + (isMatrix ? 3 : 2) + ' is not of type sequence');
      }
      nv = v.length >>> 0;
      if (nv === 0 || nv % comps !== 0) { _zwGlSetError(state, 0x0502); return; }
      for (i = 0; i < nv; i++) {
        var num = +v[i];
        if (!isInt && !isFinite(num)) { _zwGlSetError(state, 0x0501); return; } // §6.20
        values.push(isInt ? _zwGlToInteger(num) : num);
      }
    } else {
      for (i = 0; i < comps; i++) {
        var num2 = +args[1 + i];
        if (!isInt && !isFinite(num2)) { _zwGlSetError(state, 0x0501); return; }
        values.push(isInt ? _zwGlToInteger(num2) : num2);
      }
      nv = comps;
    }
    var ent = loc._zwEnt;
    var t = ent.type;
    if (isMatrix) {
      if (t !== (comps === 4 ? 0x8B5A : comps === 9 ? 0x8B5B : 0x8B5C)) {
        _zwGlSetError(state, 0x0502);
        return;
      }
    } else if (_zwGlIsSampler(t)) {
      if (!isInt || comps !== 1) { _zwGlSetError(state, 0x0502); return; } // 采样器仅 1i/1iv
    } else {
      var uc = _zwGlTypeComponents(t);
      if ((comps === 1 && uc !== 1) || (comps > 1 && uc !== comps)) {
        _zwGlSetError(state, 0x0502);
        return;
      }
    }
    var count = nv / comps;
    if (ent.size > 1 ? count > ent.size : count !== 1) {
      _zwGlSetError(state, 0x0502); // 容量越界（数组）/ 非单元素（标量）
      return;
    }
    var run = program._zwLink.uniformRuns[ent.index];
    if (!run.arr) {
      run.arr = _zwGlIsIntFamily(t) ? new Int32Array(ent.size * comps) : new Float32Array(ent.size * comps);
    }
    var off = (loc._zwIndex - ent.index) * comps;
    for (i = 0; i < nv; i++) run.arr[off + i] = values[i];
  }

  // create*/delete*/is* 族共享机械。
  (function () {
    var name;
    for (name in _ZW_CREATE_TAGS) {
      if (_zwGlOps[name]) continue; // 特化实现优先——通用机械不覆盖（createShader 类型域校验）
      (function (tag) {
        _zwGlOps[name] = function (state, args) { return _zwGlNewObject(tag); };
      })(_ZW_CREATE_TAGS[name]);
    }
    for (name in _ZW_DELETE_TAGS) {
      (function (tag) {
        _zwGlOps[name] = function (state, args) {
          var obj = args[0];
          if (obj !== null && obj !== undefined) {
            if (!obj._zwGlObj || obj._zwGlIface !== tag) {
              throw new TypeError('parameter 1 is not of type ' + tag);
            }
            obj._zwDeleted = true;
            // 删除已绑定对象：绑定置 null（spec——binding 保留删除对象的实现
            // 差异面，WebGL 按立即解除处理）。
            var k;
            for (k in state._bindings) {
              if (state._bindings[k] === obj) state._bindings[k] = null;
            }
            for (k in state._texBindings) {
              if (state._texBindings[k] === obj) state._texBindings[k] = null;
            }
            if (state._program === obj) state._program = null;
          }
        };
      })(_ZW_DELETE_TAGS[name]);
    }
    for (name in _ZW_IS_TAGS) {
      (function (tag) {
        _zwGlOps[name] = function (state, args) {
          var obj = args[0];
          return !!obj && obj._zwGlObj && obj._zwGlIface === tag && !obj._zwDeleted;
        };
      })(_ZW_IS_TAGS[name]);
    }
  })();

  // ── 上下文工厂（part04/part05 getContext 经 globalThis 单点调用）──────
  // hostHandle = 既有 host 2d 上下文 id（getContext2d op 建；clear 光栅 +
  // canvas toDataURL/painter 桥共用同一 bitmap——canvas 单上下文类型语义下
  // 无 2d/webgl 并存面）。'webgl2' 实例面远期（M4 挂账评估）——现仅接口对象
  // 存在（idlharness 资产面），getContext('webgl2') 返 null。
  globalThis._zwMakeCtxWebGL = function (hostHandle, width, height, canvasEl) {
    var ctx = Object.create(globalThis.WebGLRenderingContext.prototype);
    ctx._zwGlCtx = true;
    ctx._zwGlIface = _ZW_IFACE;
    ctx._handle = String(hostHandle);
    ctx._zwGlState = {
      _err: 0,
      _handle: String(hostHandle),
      _canvas: canvasEl || null,
      _drawingBufferWidth: width,
      _drawingBufferHeight: height,
      _bindings: {},          // ARRAY_BUFFER / ELEMENT_ARRAY_BUFFER
      _texBindings: {},       // TEXTURE_2D / TEXTURE_CUBE_MAP
      _program: null,
      _clearColor: [0, 0, 0, 0],
      _colorMask: [true, true, true, true],
      // GLES2 初始 scissor box = (0,0,首绑绘图缓冲尺寸)（GLES2 2.0.25 §2.8.1——
      // enable(SCISSOR_TEST) 前不参与 clear 裁剪，M2 clear 语义不受影响）。
      _scissor: [0, 0, width, height],
      _viewport: [0, 0, width, height],
      _enabled: {},           // cap 枚举 → bool
      _pixelStore: {},
      _compressedFormats: {}, // 使能的压缩格式（getExtension 写入）
      _extObjects: {},
      // —— M3-S1 查询面状态（GLES2 2.0.25 Table 6.9/6.10 初始值）——
      // webgl1.idl dictionary 同构的创建属性（getContext 第二参 attrs 的
      // 尊重面 FIXME(M3)：shim getContext 现仅透传 colorSpace——默认值按
      // WebGL 1.0 §5.2 表）。
      _attrs: {
        alpha: true, depth: true, stencil: false, antialias: true,
        premultipliedAlpha: true, preserveDrawingBuffer: false,
        powerPreference: 'default', failIfMajorPerformanceCaveat: false,
        desynchronized: false
      },
      _frontFace: 0x0901,       // CCW（Table 6.10）
      _cullFaceMode: 0x0405,    // BACK
      _depthFunc: 0x0201,       // LESS
      _depthMask: true,
      _depthRange: [0, 1],
      _lineWidth: 1,
      _sampleCoverageValue: 1,
      _sampleCoverageInvert: false,
      _stencilFunc: 0x0207,     // ALWAYS
      _stencilRef: 0,
      _stencilValueMask: 0xFFFFFFFF,
      _stencilWritemask: 0xFFFFFFFF,
      _blendColor: [0, 0, 0, 0],
      _blendSrcRGB: 1, _blendDstRGB: 0,   // glBlendFunc 默认 ONE/ZERO（no-op blend）
      _blendSrcAlpha: 1, _blendDstAlpha: 0,
      _blendEquationRGB: 0x8006,          // FUNC_ADD
      _blendEquationAlpha: 0x8006,
      // —— 上下文丢失（WebGL 1.0 §5.15——WEBGL_lose_context 驱动）——
      _lost: false,
      _lostErrPending: false,   // 丢失后首次 getError 返 CONTEXT_LOST_WEBGL
      _restoreSuppressed: false // webglcontextlost 被 preventDefault → 恢复抑制
    };
    return ctx;
  };

  // ── 接口定义 ─────────────────────────────────────────────────────────
  // 属性 getter/setter 均带 this 品牌守卫（idlharness attribute 断言面——
  // this=null/{} 调 getter 期望 TypeError）；品牌接口名经包装层闭包注入。

  function _zwGlMakeCtxAttrs() {
    var defs = [
      { name: 'canvas', readonly: true,
        get: function (st) { return st._canvas; } },
      { name: 'drawingBufferWidth', readonly: true,
        get: function (st) { return st._drawingBufferWidth; } },
      { name: 'drawingBufferHeight', readonly: true,
        get: function (st) { return st._drawingBufferHeight; } },
      { name: 'drawingBufferFormat', readonly: true,
        get: function (st) { return 'rgba8unorm'; } },
      { name: 'drawingBufferColorSpace', readonly: false,
        get: function (st) { return st._drawingBufferColorSpace; },
        set: function (st, v) { st._drawingBufferColorSpace = String(v); } },
      { name: 'unpackColorSpace', readonly: false,
        get: function (st) { return 'srgb'; },
        set: function (st, v) { /* FIXME(M3)：色彩空间转换未实现——静默。 */ } }
    ];
    return defs;
  }

  // WebGLObject（label 可读写 USVString；品牌 = GL 对象族）。
  _zwGlDefineInterface({
    name: 'WebGLObject',
    base: null,
    attributes: [{
      name: 'label', readonly: false,
      get: function (self) { _zwGlBrandObj(self); return self._label || ''; },
      set: function (self, v) { _zwGlBrandObj(self); self._label = String(v); }
    }],
    constants: {}
  });

  // WebGLObject 子型（空接口；原型链 → WebGLObject.prototype）。
  var _zwGlObjSubtypes = ['WebGLBuffer', 'WebGLFramebuffer', 'WebGLProgram',
    'WebGLRenderbuffer', 'WebGLShader', 'WebGLTexture'];
  for (var _zwI = 0; _zwI < _zwGlObjSubtypes.length; _zwI++) {
    _zwGlDefineInterface({
      name: _zwGlObjSubtypes[_zwI], base: 'WebGLObject', constants: {}
    });
  }

  // 独立对象接口（原型链 → Object.prototype）。
  _zwGlDefineInterface({ name: 'WebGLUniformLocation', base: null, constants: {} });
  _zwGlDefineInterface({
    name: 'WebGLActiveInfo', base: null, constants: {},
    attributes: [
      { name: 'size', readonly: true,
        get: function (self) { _zwGlBrandObj(self); return self._size || 0; } },
      { name: 'type', readonly: true,
        get: function (self) { _zwGlBrandObj(self); return self._type || 0; } },
      { name: 'name', readonly: true,
        get: function (self) { _zwGlBrandObj(self); return self._name || ''; } }
    ]
  });
  _zwGlDefineInterface({
    name: 'WebGLShaderPrecisionFormat', base: null, constants: {},
    attributes: [
      { name: 'rangeMin', readonly: true,
        get: function (self) { _zwGlBrandObj(self); return self._rangeMin || 0; } },
      { name: 'rangeMax', readonly: true,
        get: function (self) { _zwGlBrandObj(self); return self._rangeMax || 0; } },
      { name: 'precision', readonly: true,
        get: function (self) { _zwGlBrandObj(self); return self._precision || 0; } }
    ]
  });

  // WebGL2 对象子型（原型链 → WebGLObject.prototype）。
  var _zwGl2ObjSubtypes = ['WebGLQuery', 'WebGLSampler', 'WebGLSync',
    'WebGLTransformFeedback', 'WebGLVertexArrayObject'];
  for (var _zwJ = 0; _zwJ < _zwGl2ObjSubtypes.length; _zwJ++) {
    _zwGlDefineInterface({
      name: _zwGl2ObjSubtypes[_zwJ], base: 'WebGLObject', constants: {}
    });
  }

  // WebGLContextEvent : Event（唯一可构造接口；constructor(type, init)）。
  _zwGlDefineInterface({
    name: 'WebGLContextEvent', base: 'Event', constants: {},
    constructible: function (self, args) {
      var type = args.length > 0 ? String(args[0]) : '';
      var init = args.length > 1 && args[1] && typeof args[1] === 'object' ? args[1] : {};
      // 借 shim Event 构造（type/bubbles/cancelable 等 init 通道），再换原型
      // 挂 WebGLContextEvent 专属成员。
      var ev = new globalThis.Event(type, init);
      Object.setPrototypeOf(ev, globalThis.WebGLContextEvent.prototype);
      try {
        Object.defineProperty(ev, 'statusMessage', {
          value: typeof init.statusMessage === 'string' ? init.statusMessage : '',
          writable: false, enumerable: true, configurable: true
        });
      } catch (_eS) {}
      return ev;
    },
    attributes: [{
      name: 'statusMessage', readonly: true,
      get: function (self) {
        if (!self || !(self instanceof globalThis.WebGLContextEvent)) {
          throw new TypeError('Illegal invocation');
        }
        return String(self.statusMessage || '');
      }
    }]
  });

  // 丢失态仍可调用的方法白名单（WebGL 1.0 §5.15.1 查询族）。
  var _ZW_LOST_ALLOW = {
    getError: 1, isContextLost: 1, getContextAttributes: 1,
    getSupportedExtensions: 1, getExtension: 1
  };

  // ── 上下文接口（常量/方法按数据表；属性经包装层注入品牌接口名）────────
  // 注：_zwGlDefineInterface 的属性包装面向对象族（self 品牌内嵌 getter）；
  // 上下文属性的接口名品牌在 wrapper 生成时无法从 self 推——改由方法 wrapper
  // 同款 dispatch（getter 走 _zwGlDispatch 参数化通道）。此处直接构造。
  function _zwGlDefineContextInterface(spec) {
    var ifaceName = spec.name;
    var ifaceObj = _zwGlDefineInterface({
      name: ifaceName, base: spec.base, constants: spec.constants
    });
    // 方法：WebIDL 品牌检查（this=null/{} 抛 TypeError——idlharness operation
    // 断言面）+ 必参个数校验（arguments.length < minOverloadLength → TypeError
    // ——WebIDL 重载解析，corpus compressedTexImage2D 缺参案断言面）。
    var methods = spec.methods;
    for (var mname in methods) {
      var wrapper = (function (op, len, iname) {
        var fn = function () {
          if (!this || !this._zwGlCtx || this._zwGlIface !== iname) {
            throw new TypeError('Illegal invocation');
          }
          if (arguments.length < len) {
            throw new TypeError(mname + ': ' + len +
              ' argument' + (len === 1 ? '' : 's') + ' required');
          }
          var st = this._zwGlState;
          // 上下文丢失：除查询/扩展族外全部方法立即返回（WebGL 1.0 §5.15.1——
          // 无错误、无状态副作用）；getError/getContextAttributes 的丢失语义在
          // impl 内。FIXME(M4)：按 §5.15.1 逐项核对扩展/查询族在丢失态的
          // 返回值清单（getSupportedExtensions/getExtension 保持可用——
          // restoreContext 触达面）。
          if (st._lost && !_ZW_LOST_ALLOW[op]) return undefined;
          var impl = _zwGlOps[op];
          return impl ? impl(st, arguments) : undefined;
        };
        Object.defineProperty(fn, 'length', { value: len });
        Object.defineProperty(fn, 'name', { value: op });
        return fn;
      })(mname, methods[mname], ifaceName);
      Object.defineProperty(ifaceObj.prototype, mname, {
        value: wrapper, writable: true, enumerable: true, configurable: true
      });
    }
    var defs = _zwGlMakeCtxAttrs();
    for (var i = 0; i < defs.length; i++) {
      (function (d) {
        // 访问器命名（idlharness attribute 断言面：getter 'get X' / setter 'set X'）。
        var getter = function () {
          if (!this || !this._zwGlCtx || this._zwGlIface !== ifaceName) {
            throw new TypeError('Illegal invocation');
          }
          return d.get(this._zwGlState);
        };
        Object.defineProperty(getter, 'name', { value: 'get ' + d.name });
        var prop = { get: getter, enumerable: true, configurable: true };
        if (d.set) {
          var setter = function (v) {
            if (!this || !this._zwGlCtx || this._zwGlIface !== ifaceName) {
              throw new TypeError('Illegal invocation');
            }
            d.set(this._zwGlState, v);
          };
          Object.defineProperty(setter, 'name', { value: 'set ' + d.name });
          prop.set = setter;
        }
        Object.defineProperty(ifaceObj.prototype, d.name, prop);
      })(defs[i]);
    }
    return ifaceObj;
  }

  // WebGL2 = webgl1 ∪ 独有（IDL includes WebGLRenderingContextBase）——
  // idlharness 常量/方法断言用 assert_own_property（原型链继承不满足），故
  // WebGL2 接口对象与 prototype 落**自有**属性；重声明键两表值一致（派生脚本
  // 校验：常量零交集、方法 min length 无分歧），浅合并无冲突覆盖。
  var _ZW_W2C = {};
  var _ZW_W2M = {};
  var _zwK;
  for (_zwK in _ZW_WGL1C) _ZW_W2C[_zwK] = _ZW_WGL1C[_zwK];
  for (_zwK in _ZW_WGL2C) _ZW_W2C[_zwK] = _ZW_WGL2C[_zwK];
  for (_zwK in _ZW_WGL1M) _ZW_W2M[_zwK] = _ZW_WGL1M[_zwK];
  for (_zwK in _ZW_WGL2M) _ZW_W2M[_zwK] = _ZW_WGL2M[_zwK];
  _zwGlDefineContextInterface({
    name: _ZW_IFACE, base: null,
    constants: _ZW_WGL1C, methods: _ZW_WGL1M
  });
  // WebGL2 无基接口（pin rev webgl2.idl：WebGL2RenderingContext 仅 mixin includes，
  // 无 ': WebGLRenderingContext' 继承）——接口对象 [[Prototype]] = Function.prototype、
  // prototype [[Prototype]] = Object.prototype（idlharness test_self 断言面，
  // part66 基线实证）；webgl1 成员经上方合并表落自有属性。
  _zwGlDefineContextInterface({
    name: _ZW_IFACE2, base: null,
    constants: _ZW_W2C, methods: _ZW_W2M
  });
})();
