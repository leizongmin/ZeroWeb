// ============================================================================
// part08c — WebGL GL 对象存储簇（webgl-compat goal M3-S3，2026-10-22；
// 自 part08 按职责拆分——2000 行准则，part08 保留状态机/查询/program 簇）。
//
// 范围：buffer 真字节存储 + 纹理参数/上传校验链 + 压缩纹理记账 +
// framebuffer/renderbuffer 附件骨架（实现口径与边界记账见
// docs/goal/webgl-compat/evidence/2026-10-22-m3-s3-object-storage.md；
// 枚举值序以 pin rev webgl1.idl 派生表为源）。
//
// 作用域与交接：本文件运行在顶层作用域（part07 末闭合外层 IIFE 同款），
// part08 末尾经 globalThis._zwGlStorageHandoff 单点交接共享原语与 cap 值
//（part01 先例，用后即清）；本文件 op 表运行时**合并回** part08 的
// _zwGlOps——dispatch 在方法调用期查表，无加载期耦合（仅要求 include
// 顺序 part08 → part08c；part08 的 create/delete/is 通用机械与本簇无
// 同名冲突）。
// ============================================================================

(function () {
  'use strict';

  // ── part08 共享原语与 cap 值交接（单点，用后即清；cap 单源 part08）──
  var _zwH = globalThis._zwGlStorageHandoff;
  globalThis._zwGlStorageHandoff = null;
  var _zwGlOps = _zwH.ops;
  var _zwGlSetError = _zwH.setError;
  var _zwGlToInteger = _zwH.toInteger;
  var _zwGlIsTexImageSource = _zwH.isTexImageSource;
  var _ZW_MAX_TEX_SIZE = _zwH.maxTexSize;
  var _ZW_MAX_TEX_LEVEL = _zwH.maxTexLevel;
  var _ZW_MAX_BUFFER_BYTES = _zwH.maxBufferBytes;

  // ── 枚举集（buffer/纹理/framebuffer 校验用）──
  // GL 枚举集（错误语义校验用；值同数据表）。
  var _ZW_BIND_TARGETS = { 0x8892: 1, 0x8893: 1 }; // ARRAY_BUFFER / ELEMENT_ARRAY_BUFFER
  var _ZW_TEX_BIND_TARGETS = { 0x0DE1: 1, 0x8513: 1 }; // TEXTURE_2D / TEXTURE_CUBE_MAP
  // usage 九值集（pin rev webgl1.idl 值序：STREAM 0x88E0-2、STATIC 0x88E4-6、
  // DYNAMIC 0x88E8-0xA——w1 接口仅暴露 *_DRAW 三常量，READ/COPY 为 GLES2 语义
  // 内合法值）。M2 旧表含幻值 0x88E3 且漏 DYNAMIC_DRAW 0x88E8（M3-S3 修正）。
  var _ZW_USAGE = { 0x88E0: 1, 0x88E1: 1, 0x88E2: 1, 0x88E4: 1, 0x88E5: 1,
    0x88E6: 1, 0x88E8: 1, 0x88E9: 1, 0x88EA: 1 };
  // —— M3-S3 对象真存储枚举集 ——
  // 纹理采样参数名（GLES2 Table 6.9 Sampler；WRAP_S/T = 0x2802/0x2803）+
  // 各名合法值集（§3.7.7 精确匹配）。
  var _ZW_TEX_PARAMS = { 0x2800: 1, 0x2801: 1, 0x2802: 1, 0x2803: 1 }; // MAG/MIN_FILTER, WRAP_S/T
  var _ZW_TEX_MAG_FILTERS = { 0x2600: 1, 0x2601: 1 }; // NEAREST/LINEAR
  var _ZW_TEX_MIN_FILTERS = { 0x2600: 1, 0x2601: 1, 0x2700: 1, 0x2701: 1, 0x2702: 1, 0x2703: 1 };
  var _ZW_TEX_WRAPS = { 0x2901: 1, 0x812F: 1, 0x8370: 1 }; // REPEAT/CLAMP_TO_EDGE/MIRRORED_REPEAT
  // texImage2D/texSubImage2D 的格式/类型域（GLES2 Table 3.2 WebGL 1.0 core 全集
  // ——WebGL §6.13：internalformat 必须等于 format）。
  var _ZW_TEX_FORMATS = { 0x1906: 1, 0x1909: 1, 0x190A: 1, 0x1907: 1, 0x1908: 1 }; // ALPHA/LUMINANCE/LUMINANCE_ALPHA/RGB/RGBA
  var _ZW_TEX_TYPES = { 0x1401: 1, 0x8032: 1, 0x8033: 1, 0x8363: 1 }; // UNSIGNED_BYTE/SHORT_5_5_5_1/4_4_4_4/5_6_5
  var _ZW_TEX_COMBOS = {
    0x1908: { 0x1401: 1, 0x8033: 1, 0x8032: 1 }, // RGBA
    0x1907: { 0x1401: 1, 0x8363: 1 },            // RGB
    0x190A: { 0x1401: 1 },                       // LUMINANCE_ALPHA
    0x1909: { 0x1401: 1 },                       // LUMINANCE
    0x1906: { 0x1401: 1 }                        // ALPHA
  };
  // 上传/子上传合法 target：TEXTURE_2D + 六 cube face（TEXTURE_CUBE_MAP 本身
  // 非法——GLES2 §3.7.2）。
  var _ZW_TEX_IMAGE_TARGETS = { 0x0DE1: 1, 0x8515: 1, 0x8516: 1, 0x8517: 1,
    0x8518: 1, 0x8519: 1, 0x851A: 1 };
  // framebuffer 附件点（GLES2 Table 4.3——COLOR_ATTACHMENT0/DEPTH/STENCIL/
  // DEPTH_STENCIL）+ renderbuffer 内部格式（WebGL §5.14.14 core 五格式——值按
  // pin rev webgl1.idl：RGBA4=0x8056、DEPTH_COMPONENT16=0x81A5）。
  var _ZW_FB_ATTACHMENTS = { 0x8CE0: 1, 0x8D00: 1, 0x8D20: 1, 0x821A: 1 };
  var _ZW_RB_FORMATS = { 0x8056: 1, 0x8057: 1, 0x8D62: 1, 0x81A5: 1, 0x8D48: 1 };

  // ── 共享助手（字节视图/像素字节数/纹理参数与上传校验/level 表）──
  function _zwGlIsBufferSource(v) {
    if (!v || typeof v !== 'object') return false;
    return typeof v.byteLength === 'number' &&
      (typeof ArrayBuffer !== 'undefined' ? (v instanceof ArrayBuffer || ArrayBuffer.isView(v)) : false);
  }

  // BufferSource → 其字节视图（拷贝源；ArrayBuffer 本体无 byteOffset/view）。
  function _zwGlBufferSourceBytes(v) {
    return new Uint8Array(v.buffer, v.byteOffset || 0, v.byteLength);
  }

  // 像素字节数（紧凑排布，GLES2 Table 3.2 尺寸列——UNSIGNED_SHORT_* 恒 2 字节）。
  function _zwGlTexPixelBytes(format, type) {
    if (type !== 0x1401) return 2;
    switch (format) {
      case 0x1908: return 4; // RGBA
      case 0x1907: return 3; // RGB
      case 0x190A: return 2; // LUMINANCE_ALPHA
      default: return 1;     // LUMINANCE/ALPHA
    }
  }

  // texParameteri/f 共享实现（GLES2 §3.7.7）——isFloat 时 param 按 GLfloat 数值
  // 精确匹配枚举（非整数值不匹配任何合法枚举 → INVALID_ENUM）。
  function _zwGlTexParameteriImpl(state, args, isFloat) {
    var target = +args[0];
    if (!_ZW_TEX_BIND_TARGETS[target]) { _zwGlSetError(state, 0x0500); return; }
    var tex = state._texBindings[target];
    if (!tex) { _zwGlSetError(state, 0x0502); return; }
    var pname = +args[1];
    if (!_ZW_TEX_PARAMS[pname]) { _zwGlSetError(state, 0x0500); return; }
    var param = isFloat ? +args[2] : _zwGlToInteger(args[2]);
    var ok = (pname === 0x2800 && _ZW_TEX_MAG_FILTERS[param]) ||
      (pname === 0x2801 && _ZW_TEX_MIN_FILTERS[param]) ||
      ((pname === 0x2802 || pname === 0x2803) && _ZW_TEX_WRAPS[param]);
    if (!ok) { _zwGlSetError(state, 0x0500); return; }
    if (!tex._zwParams) tex._zwParams = {};
    tex._zwParams[pname] = param;
  }

  // texImage2D/texSubImage2D 共享参数域校验（GLES2 §3.7.2 + WebGL §6.13）：
  // 调用方按各自形参布局抽取原始值后传入；返回 null = 已置错误，否则回填
  // {target,level,w,h,format,type}。
  function _zwGlTexUploadValidate(state, tex, target, level, w, h, format, type, border) {
    if (!_ZW_TEX_IMAGE_TARGETS[target]) { _zwGlSetError(state, 0x0500); return null; }
    if (!tex) { _zwGlSetError(state, 0x0502); return null; } // 无绑定纹理
    if (!_ZW_TEX_FORMATS[format]) { _zwGlSetError(state, 0x0500); return null; }
    if (!_ZW_TEX_TYPES[type]) { _zwGlSetError(state, 0x0500); return null; }
    // WebGL §6.13：合法格式/类型但不构成合法组合 → INVALID_OPERATION。
    if (!_ZW_TEX_COMBOS[format] || !_ZW_TEX_COMBOS[format][type]) {
      _zwGlSetError(state, 0x0502);
      return null;
    }
    if (level < 0 || level > _ZW_MAX_TEX_LEVEL || border !== 0 ||
      w <= 0 || h <= 0 || w > _ZW_MAX_TEX_SIZE || h > _ZW_MAX_TEX_SIZE) {
      _zwGlSetError(state, 0x0501);
      return null;
    }
    return { target: target, level: level, w: w, h: h, format: format, type: type };
  }

  // 取/建纹理 image 表（key = target:level；纹理真存储单点）。
  function _zwGlTexImages(tex) {
    return tex._zwImages || (tex._zwImages = {});
  }

  // ── op 面（对象字面量保持 part08 原文，合并见文件尾）──
  var _zwStorageOps = {
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
      // 真存储（GLES2 §2.5）：重分配即丢弃旧 store——_zwData 唯一持有字节。
      if (typeof data === 'number') {
        if (isNaN(data) || data < 0) { _zwGlSetError(state, 0x0501); return; } // INVALID_VALUE
        var size = _zwGlToInteger(data);
        if (size > _ZW_MAX_BUFFER_BYTES) { _zwGlSetError(state, 0x0501); return; }
        buf._zwData = new Uint8Array(size);
        buf._zwSize = size;
      } else if (_zwGlIsBufferSource(data)) {
        var src = _zwGlBufferSourceBytes(data);
        buf._zwData = new Uint8Array(src.length);
        buf._zwData.set(src);
        buf._zwSize = src.length;
      } else if (data === null) {
        buf._zwData = new Uint8Array(0);
        buf._zwSize = 0;
      } else {
        throw new TypeError('parameter 2 is not of type BufferSource');
      }
      buf._zwUsage = usage;
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
      // 无既有 store → INVALID_OPERATION（GLES2 §2.5 bufferSubData 前置）。
      if (buf._zwData === undefined) { _zwGlSetError(state, 0x0502); return; }
      if (offset < 0 || offset + size > (buf._zwSize || 0)) {
        _zwGlSetError(state, 0x0501); // INVALID_VALUE
        return;
      }
      buf._zwData.set(_zwGlBufferSourceBytes(args[2]), offset);
    },
    getBufferParameter: function (state, args) {
      var target = +args[0];
      if (!_ZW_BIND_TARGETS[target]) { _zwGlSetError(state, 0x0500); return null; }
      var buf = state._bindings[target];
      if (!buf) { _zwGlSetError(state, 0x0502); return null; }
      var pname = +args[1];
      switch (pname) {
        case 0x8764: return buf._zwSize || 0;    // BUFFER_SIZE（Table 6.8 初始 0）
        case 0x8765: return buf._zwUsage || 0x88E4; // BUFFER_USAGE（初始 STATIC_DRAW=0x88E4）
        default: _zwGlSetError(state, 0x0500); return null;
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
        // 同一纹理不得改绑异型 target（GLES2 §3.7.13——首绑定型）。
        if (tex._zwBindTarget !== undefined && tex._zwBindTarget !== target) {
          _zwGlSetError(state, 0x0502);
          return;
        }
        tex._zwBindTarget = target;
      }
      state._texBindings[target] = tex || null;
    },
    // 纹理采样参数（GLES2 §3.7.7）：参数名/值域校验（精确匹配），落纹理对象。
    texParameteri: function (state, args) {
      _zwGlTexParameteriImpl(state, args, false);
    },
    texParameterf: function (state, args) {
      _zwGlTexParameteriImpl(state, args, true);
    },
    getTexParameter: function (state, args) {
      var target = +args[0];
      if (!_ZW_TEX_BIND_TARGETS[target]) { _zwGlSetError(state, 0x0500); return null; }
      var tex = state._texBindings[target];
      if (!tex) { _zwGlSetError(state, 0x0502); return null; }
      var pname = +args[1];
      if (!_ZW_TEX_PARAMS[pname]) { _zwGlSetError(state, 0x0500); return null; }
      // 未设置返 GLES2 Table 6.9 初始值（MIN=NEAREST_MIPMAP_LINEAR、MAG=LINEAR、
      // WRAP=REPEAT）。
      var defaults = { 0x2800: 0x2601, 0x2801: 0x2702, 0x2802: 0x2901, 0x2803: 0x2901 };
      var v = tex._zwParams ? tex._zwParams[pname] : undefined;
      return v !== undefined ? v : defaults[pname];
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
        // 9 参形：(target, level, internalformat, w, h, border, format, type,
        // pixels)——WebGL §6.13 internalformat 必须等于 format。
        var ent = _zwGlTexUploadValidate(state, tex, +args[0], _zwGlToInteger(args[1]),
          _zwGlToInteger(args[3]), _zwGlToInteger(args[4]), _zwGlToInteger(args[6]),
          _zwGlToInteger(args[7]), _zwGlToInteger(args[5]));
        if (!ent) return;
        // WebGL §6.13：internalformat 必须等于 format（合法枚举但不同 →
        // INVALID_OPERATION）。
        if (_zwGlToInteger(args[2]) !== ent.format) { _zwGlSetError(state, 0x0502); return; }
        var pixels = args[8];
        if (pixels !== null && pixels !== undefined && !_zwGlIsBufferSource(pixels)) {
          _zwGlSetError(state, 0x0501);
          return;
        }
        var need = ent.w * ent.h * _zwGlTexPixelBytes(ent.format, ent.type);
        var data = null;
        if (pixels) {
          if (pixels.byteLength < need) { _zwGlSetError(state, 0x0502); return; } // §6.13 容量不足
          data = new Uint8Array(_zwGlBufferSourceBytes(pixels)); // 拷贝持有
        } // pixels null → store 已分配未初始化（data null——GLES2 §3.7.1）
        var imgs = _zwGlTexImages(tex);
        imgs[ent.target + ':' + ent.level] =
          { w: ent.w, h: ent.h, format: ent.format, type: ent.type, data: data };
        tex._zwHasBase = true;
        // FIXME(M3-S4 纹理簇)：UNPACK_FLIP_Y/PREMULTIPLY/COLORSPACE_CONVERSION
        // 像素变换未实现（当前按源字节直存）；UNPACK_ALIGNMENT 行跨距按 1。
      } else if (n === 6) {
        // source 形：union 转换失败 → TypeError（品牌判定不触源对象 getter）。
        if (!_zwGlIsTexImageSource(args[5])) {
          throw new TypeError('parameter 6 is not of type TexImageSource');
        }
        // FIXME(M3 纹理簇)：源解码未实现——仅记 level 元数据（RGBA/UNSIGNED_BYTE
        // 口径），像素字节留空；dims 取源对象数值属性（不可读则 0）。
        var src2 = args[5];
        var sw = _zwGlToInteger(src2.width);
        var sh2 = _zwGlToInteger(src2.height);
        if (tex) {
          _zwGlTexImages(tex)['0x0DE1:' + _zwGlToInteger(args[1])] =
            { w: sw || 1, h: sh2 || 1, format: 0x1908, type: 0x1401, data: null };
          tex._zwHasBase = true;
        }
      } else {
        throw new TypeError('texImage2D: no matching overload');
      }
    },
    texSubImage2D: function (state, args) {
      var n = args.length;
      var tex = state._texBindings[0x0DE1];
      if (n >= 9) {
        // 9 参形：(target, level, x, y, w, h, format, type, pixels)。
        var ent = _zwGlTexUploadValidate(state, tex, +args[0], _zwGlToInteger(args[1]),
          _zwGlToInteger(args[4]), _zwGlToInteger(args[5]), _zwGlToInteger(args[6]),
          _zwGlToInteger(args[7]), 0);
        if (!ent) return;
        var x = _zwGlToInteger(args[2]);
        var y = _zwGlToInteger(args[3]);
        var pixels = args[8];
        if (pixels !== null && pixels !== undefined && !_zwGlIsBufferSource(pixels)) {
          _zwGlSetError(state, 0x0501);
          return;
        }
        // 既有 level image 必须存在且 format/type 一致（GLES2 §3.7.2）。
        var imgs = _zwGlTexImages(tex);
        var img = imgs[ent.target + ':' + ent.level];
        if (!img || img.format !== ent.format || img.type !== ent.type) {
          _zwGlSetError(state, 0x0502);
          return;
        }
        if (x < 0 || y < 0 || x + ent.w > img.w || y + ent.h > img.h) {
          _zwGlSetError(state, 0x0501); // 区域越界 INVALID_VALUE
          return;
        }
        var need = ent.w * ent.h * _zwGlTexPixelBytes(ent.format, ent.type);
        if (pixels) {
          if (pixels.byteLength < need) { _zwGlSetError(state, 0x0502); return; }
          // null data = 已分配未初始化——区域写前物化为全零（GLES2 §3.7.1）。
          if (!img.data) {
            img.data = new Uint8Array(img.w * img.h * _zwGlTexPixelBytes(img.format, img.type));
          }
          var srcBytes = _zwGlBufferSourceBytes(pixels);
          var dst = img.data;
          var bpp = _zwGlTexPixelBytes(ent.format, ent.type);
          for (var row = 0; row < ent.h; row++) {
            // 紧凑源行 → 目标行跨距 img.w（UNPACK_ALIGNMENT=1 口径，FIXME 同上）。
            dst.set(srcBytes.subarray(row * ent.w * bpp, (row + 1) * ent.w * bpp),
              ((y + row) * img.w + x) * bpp);
          }
        }
        tex._zwHasBase = true;
      } else if (n === 7) {
        if (!_zwGlIsTexImageSource(args[6])) {
          throw new TypeError('parameter 7 is not of type TexImageSource');
        }
        // source 形同需既有 base（GLES2 §3.7.2——corpus 无 base INVALID_OPERATION
        // 同码）；FIXME(M3 纹理簇)：源解码未实现，仅置 base 标记。
        if (!tex || !tex._zwHasBase) { _zwGlSetError(state, 0x0502); return; }
        tex._zwHasBase = true;
      } else {
        throw new TypeError('texSubImage2D: no matching overload');
      }
    },

    // —— framebuffer / renderbuffer 骨架（M3-S3；GLES2 §4.4 + WebGL §5.14.14）——
    // 附件状态为真存储（attachments 表）；颜色面光栅化仍由 M2 clear/host 2d 承担，
    // 附件消费 = M3-S4 draw 簇（FIXME）。FRAMEBUFFER/RENDERBUFFER 绑定复用
    // _bindings 表（deleteFramebuffer/deleteRenderbuffer 的解除绑定机械免费生效）。
    bindFramebuffer: function (state, args) {
      var target = +args[0];
      if (target !== 0x8D40) { _zwGlSetError(state, 0x0500); return; } // FRAMEBUFFER
      var fbo = args[1];
      if (fbo !== null && fbo !== undefined) {
        if (!fbo._zwGlObj || fbo._zwGlIface !== 'WebGLFramebuffer') {
          throw new TypeError('parameter 2 is not of type WebGLFramebuffer');
        }
        if (fbo._zwDeleted) { _zwGlSetError(state, 0x0502); return; }
      }
      state._bindings[0x8D40] = fbo || null;
    },
    framebufferTexture2D: function (state, args) {
      var target = +args[0];
      if (target !== 0x8D40) { _zwGlSetError(state, 0x0500); return; }
      var attachment = +args[1];
      if (!_ZW_FB_ATTACHMENTS[attachment]) { _zwGlSetError(state, 0x0500); return; }
      var fbo = state._bindings[0x8D40];
      if (!fbo) { _zwGlSetError(state, 0x0502); return; }
      var texture = args[3];
      if (texture !== null && texture !== undefined) {
        if (!texture._zwGlObj || texture._zwGlIface !== 'WebGLTexture') {
          throw new TypeError('parameter 4 is not of type WebGLTexture');
        }
        var textarget = +args[2];
        // textarget 域（TEXTURE_2D + cube face）与 level=0 前置（GLES2 §4.4.3）。
        if (!_ZW_TEX_IMAGE_TARGETS[textarget]) { _zwGlSetError(state, 0x0500); return; }
        var level = _zwGlToInteger(args[4]);
        if (level !== 0) { _zwGlSetError(state, 0x0501); return; }
        if (texture._zwDeleted) { _zwGlSetError(state, 0x0502); return; }
        if (!fbo._zwAttachments) fbo._zwAttachments = {};
        fbo._zwAttachments[attachment] =
          { texture: texture, textarget: textarget, level: level };
      } else if (fbo._zwAttachments) {
        // texture null → 解除该附件点（GLES2 §4.4.3）。
        delete fbo._zwAttachments[attachment];
      }
    },
    framebufferRenderbuffer: function (state, args) {
      var target = +args[0];
      if (target !== 0x8D40) { _zwGlSetError(state, 0x0500); return; }
      var attachment = +args[1];
      if (!_ZW_FB_ATTACHMENTS[attachment]) { _zwGlSetError(state, 0x0500); return; }
      if (+args[2] !== 0x8D41) { _zwGlSetError(state, 0x0500); return; } // RENDERBUFFER
      var fbo = state._bindings[0x8D40];
      if (!fbo) { _zwGlSetError(state, 0x0502); return; }
      var rb = args[3];
      if (rb !== null && rb !== undefined) {
        if (!rb._zwGlObj || rb._zwGlIface !== 'WebGLRenderbuffer') {
          throw new TypeError('parameter 4 is not of type WebGLRenderbuffer');
        }
        if (rb._zwDeleted) { _zwGlSetError(state, 0x0502); return; }
        if (!fbo._zwAttachments) fbo._zwAttachments = {};
        fbo._zwAttachments[attachment] = { rb: rb };
      } else if (fbo._zwAttachments) {
        delete fbo._zwAttachments[attachment];
      }
    },
    checkFramebufferStatus: function (state, args) {
      if (+args[0] !== 0x8D40) { _zwGlSetError(state, 0x0500); return 0; }
      var fbo = state._bindings[0x8D40];
      // 默认 framebuffer 视为 complete（drawing buffer 恒在——GLES2 §4.4.5）。
      if (!fbo) return 0x8CD5; // FRAMEBUFFER_COMPLETE
      var atts = fbo._zwAttachments;
      var any = false, name;
      for (name in atts) { if (atts[name]) { any = true; break; } }
      if (!any) return 0x8CD7; // INCOMPLETE_MISSING_ATTACHMENT
      var dw = -1, dh = -1;
      for (name in atts) {
        var ent = atts[name];
        if (!ent) continue;
        // 已删对象仍挂附件 → incomplete（§4.4.2.3 删除解除仅及当前绑定 FBO）。
        var img = null;
        if (ent.texture) {
          img = ent.texture._zwDeleted ? null : _zwGlTexImages(ent.texture)[ent.textarget + ':' + ent.level];
        } else if (ent.rb) {
          img = ent.rb._zwDeleted ? null : (ent.rb._zwStorage || null);
        }
        if (!img) return 0x8CD6; // FRAMEBUFFER_INCOMPLETE_ATTACHMENT
        if (dw < 0) { dw = img.w; dh = img.h; }
        else if (img.w !== dw || img.h !== dh) return 0x8CD9; // INCOMPLETE_DIMENSIONS
      }
      return 0x8CD5;
    },
    getFramebufferAttachmentParameter: function (state, args) {
      if (+args[0] !== 0x8D40) { _zwGlSetError(state, 0x0500); return null; }
      var attachment = +args[1];
      if (!_ZW_FB_ATTACHMENTS[attachment]) { _zwGlSetError(state, 0x0500); return null; }
      var pname = +args[2];
      var fbo = state._bindings[0x8D40];
      if (!fbo) {
        // 默认 framebuffer：仅 OBJECT_TYPE 可查（GLES2 §6.1.13——返
        // FRAMEBUFFER_DEFAULT，其余 INVALID_OPERATION）。
        if (pname === 0x8CD0) return 0x8218; // FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE
        _zwGlSetError(state, 0x0502);
        return null;
      }
      var ent = fbo._zwAttachments ? fbo._zwAttachments[attachment] : null;
      if (!ent) {
        // 空附件点：OBJECT_TYPE NONE，其余返 0（GLES2 Table 6.15）。
        switch (pname) {
          case 0x8CD0: return 0;
          case 0x8CD1: case 0x8CD2: case 0x8CD3: return 0;
          default: _zwGlSetError(state, 0x0500); return null;
        }
      }
      switch (pname) {
        // OBJECT_TYPE 值取 pin rev IDL 常量口径（TEXTURE=0x1702、RENDERBUFFER=
        // 0x8D41——corpus 断言与上下文常量同源）。
        case 0x8CD0: return ent.texture ? 0x1702 : 0x8D41; // TEXTURE / RENDERBUFFER
        case 0x8CD1: return ent.texture || ent.rb;         // OBJECT_NAME
        case 0x8CD2: case 0x8CD3:
          // 纹理级/cube face 查询仅纹理附件可答（GLES2 §6.1.13——renderbuffer
          // 附件点 → INVALID_OPERATION）。
          if (!ent.texture) { _zwGlSetError(state, 0x0502); return null; }
          if (pname === 0x8CD2) return ent.level;
          return ent.textarget !== 0x0DE1 ? ent.textarget : 0;
        default:
          _zwGlSetError(state, 0x0500);
          return null;
      }
    },
    bindRenderbuffer: function (state, args) {
      var target = +args[0];
      if (target !== 0x8D41) { _zwGlSetError(state, 0x0500); return; } // RENDERBUFFER
      var rb = args[1];
      if (rb !== null && rb !== undefined) {
        if (!rb._zwGlObj || rb._zwGlIface !== 'WebGLRenderbuffer') {
          throw new TypeError('parameter 2 is not of type WebGLRenderbuffer');
        }
        if (rb._zwDeleted) { _zwGlSetError(state, 0x0502); return; }
      }
      state._bindings[0x8D41] = rb || null;
    },
    renderbufferStorage: function (state, args) {
      var target = +args[0];
      if (target !== 0x8D41) { _zwGlSetError(state, 0x0500); return; }
      var rb = state._bindings[0x8D41];
      if (!rb) { _zwGlSetError(state, 0x0502); return; }
      var internalformat = +args[1];
      if (!_ZW_RB_FORMATS[internalformat]) { _zwGlSetError(state, 0x0500); return; }
      var w = _zwGlToInteger(args[2]);
      var h = _zwGlToInteger(args[3]);
      // 尺寸域 1..MAX_RENDERBUFFER_SIZE（GLES2 §4.4.3——RenderbufferStorage）。
      if (w <= 0 || h <= 0 || w > 4096 || h > 4096) { _zwGlSetError(state, 0x0501); return; }
      rb._zwStorage = { format: internalformat, w: w, h: h };
    },
    getRenderbufferParameter: function (state, args) {
      var target = +args[0];
      if (target !== 0x8D41) { _zwGlSetError(state, 0x0500); return null; }
      var rb = state._bindings[0x8D41];
      if (!rb) { _zwGlSetError(state, 0x0502); return null; }
      var st = rb._zwStorage;
      switch (+args[1]) {
        // WIDTH/HEIGHT/INTERNAL_FORMAT 按 pin rev webgl1.idl 值序（0x8D42/43/44
        // ——与上下文常量同源）。
        case 0x8D42: return st ? st.w : 0;            // RENDERBUFFER_WIDTH
        case 0x8D43: return st ? st.h : 0;            // RENDERBUFFER_HEIGHT
        case 0x8D44: return st ? st.format : 0;       // RENDERBUFFER_INTERNAL_FORMAT
        case 0x8D50: case 0x8D51: case 0x8D52:        // RED/GREEN/BLUE_SIZE
        case 0x8D53: case 0x8D54: case 0x8D55:        // ALPHA/DEPTH/STENCIL_SIZE
          if (!st) return 0;
          var f = st.format;
          if (f === 0x8056) return 4;                 // RGBA4
          if (f === 0x8057) {                         // RGB5_A1
            return (+args[1] === 0x8D53) ? 1 : 5;
          }
          if (f === 0x8D62) {                         // RGB565
            return +args[1] === 0x8D51 ? 6 : (+args[1] === 0x8D52 ? 5 : 0);
          }
          if (f === 0x81A5) return +args[1] === 0x8D54 ? 16 : 0; // DEPTH_COMPONENT16
          return +args[1] === 0x8D55 ? 8 : 0;         // STENCIL_INDEX8
        default: _zwGlSetError(state, 0x0500); return null;
      }
    },
  };

  // ── op 表合并回 part08 的 _zwGlOps（调用期查表，见头注）──
  for (var _zwS3Op in _zwStorageOps) _zwGlOps[_zwS3Op] = _zwStorageOps[_zwS3Op];
})();
