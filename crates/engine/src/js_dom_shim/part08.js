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
// 派生口径见表头注释；idlharness（webgl/ idlharness.any.js）按 WebIDL 逐项断言
// 接口对象/原型（常量值与描述符、方法 existence/.length/.name、this 品牌
// TypeError），本文件与之逐项对齐。
//
// 作用域：part07 末尾已闭合外层 IIFE，本文件运行在**顶层作用域**——与 part04/05
//（同一大闭包内，无法反向引用本文件局部）经 `globalThis._zwMakeCtxWebGL` 单点
// 发布对接（part01 `globalThis._zwGetInnerText` 先例）；其余符号全部闭包内。
// ============================================================================

(function () {
  'use strict';

  // ── 接口面数据表（脚本注入，勿手改）──
  // 来源：WPT pin 315976933870 interfaces/webgl{1,2}.idl（webref 提取）——
  // WebGLRenderingContextBase+Overloads → _ZW_WGL1C/_ZW_WGL1M；
  // WebGL2RenderingContextBase+Overloads 全集（含重声明成员）→ _ZW_WGL2C/_ZW_WGL2M。
  // 常量值为 IDL 原文（十六进制/十进制逐字）；方法值为 WebIDL
  // minOverloadLength（必参个数，optional/default 前截断——idlharness.js 同口径），
  // 供必参 TypeError 与 prototype 方法 .length 对齐（idlharness 断言面）。
// ── 常量/方法面数据表（脚本派生，勿手改）──
// 来源：WPT pin 315976933870 interfaces/webgl{1,2}.idl（webref 提取）。
// _ZW_WGL1C/_ZW_WGL1M = webgl1.idl 全部 interface（含 Base/Overloads mixin）；
// _ZW_WGL2C/_ZW_WGL2M = webgl2.idl 全集（含重声明成员，合并时按键覆盖——
// 两表键值经派生脚本校验一致/无冲突）。常量值为 IDL 原文逐字（十六进制/
// 十进制/负数）；方法值为 WebIDL minOverloadLength（首 optional/default 前
// 的必参个数，多重载取最小——idlharness.js 同口径），供必参 TypeError 与
// prototype 方法 .length 对齐（idlharness interface: operation 断言面）。
var _ZW_WGL1C = {
  ACTIVE_ATTRIBUTES: 0x8B89,
  ACTIVE_TEXTURE: 0x84E0,
  ACTIVE_UNIFORMS: 0x8B86,
  ALIASED_LINE_WIDTH_RANGE: 0x846E,
  ALIASED_POINT_SIZE_RANGE: 0x846D,
  ALPHA: 0x1906,
  ALPHA_BITS: 0x0D55,
  ALWAYS: 0x0207,
  ARRAY_BUFFER: 0x8892,
  ARRAY_BUFFER_BINDING: 0x8894,
  ATTACHED_SHADERS: 0x8B85,
  BACK: 0x0405,
  BLEND: 0x0BE2,
  BLEND_COLOR: 0x8005,
  BLEND_DST_ALPHA: 0x80CA,
  BLEND_DST_RGB: 0x80C8,
  BLEND_EQUATION: 0x8009,
  BLEND_EQUATION_ALPHA: 0x883D,
  BLEND_EQUATION_RGB: 0x8009,
  BLEND_SRC_ALPHA: 0x80CB,
  BLEND_SRC_RGB: 0x80C9,
  BLUE_BITS: 0x0D54,
  BOOL: 0x8B56,
  BOOL_VEC2: 0x8B57,
  BOOL_VEC3: 0x8B58,
  BOOL_VEC4: 0x8B59,
  BROWSER_DEFAULT_WEBGL: 0x9244,
  BUFFER_SIZE: 0x8764,
  BUFFER_USAGE: 0x8765,
  BYTE: 0x1400,
  CCW: 0x0901,
  CLAMP_TO_EDGE: 0x812F,
  COLOR_ATTACHMENT0: 0x8CE0,
  COLOR_BUFFER_BIT: 0x00004000,
  COLOR_CLEAR_VALUE: 0x0C22,
  COLOR_WRITEMASK: 0x0C23,
  COMPILE_STATUS: 0x8B81,
  COMPRESSED_TEXTURE_FORMATS: 0x86A3,
  CONSTANT_ALPHA: 0x8003,
  CONSTANT_COLOR: 0x8001,
  CONTEXT_LOST_WEBGL: 0x9242,
  CULL_FACE: 0x0B44,
  CULL_FACE_MODE: 0x0B45,
  CURRENT_PROGRAM: 0x8B8D,
  CURRENT_VERTEX_ATTRIB: 0x8626,
  CW: 0x0900,
  DECR: 0x1E03,
  DECR_WRAP: 0x8508,
  DELETE_STATUS: 0x8B80,
  DEPTH_ATTACHMENT: 0x8D00,
  DEPTH_BITS: 0x0D56,
  DEPTH_BUFFER_BIT: 0x00000100,
  DEPTH_CLEAR_VALUE: 0x0B73,
  DEPTH_COMPONENT: 0x1902,
  DEPTH_COMPONENT16: 0x81A5,
  DEPTH_FUNC: 0x0B74,
  DEPTH_RANGE: 0x0B70,
  DEPTH_STENCIL: 0x84F9,
  DEPTH_STENCIL_ATTACHMENT: 0x821A,
  DEPTH_TEST: 0x0B71,
  DEPTH_WRITEMASK: 0x0B72,
  DITHER: 0x0BD0,
  DONT_CARE: 0x1100,
  DST_ALPHA: 0x0304,
  DST_COLOR: 0x0306,
  DYNAMIC_DRAW: 0x88E8,
  ELEMENT_ARRAY_BUFFER: 0x8893,
  ELEMENT_ARRAY_BUFFER_BINDING: 0x8895,
  EQUAL: 0x0202,
  FASTEST: 0x1101,
  FLOAT: 0x1406,
  FLOAT_MAT2: 0x8B5A,
  FLOAT_MAT3: 0x8B5B,
  FLOAT_MAT4: 0x8B5C,
  FLOAT_VEC2: 0x8B50,
  FLOAT_VEC3: 0x8B51,
  FLOAT_VEC4: 0x8B52,
  FRAGMENT_SHADER: 0x8B30,
  FRAMEBUFFER: 0x8D40,
  FRAMEBUFFER_ATTACHMENT_OBJECT_NAME: 0x8CD1,
  FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE: 0x8CD0,
  FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE: 0x8CD3,
  FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL: 0x8CD2,
  FRAMEBUFFER_BINDING: 0x8CA6,
  FRAMEBUFFER_COMPLETE: 0x8CD5,
  FRAMEBUFFER_INCOMPLETE_ATTACHMENT: 0x8CD6,
  FRAMEBUFFER_INCOMPLETE_DIMENSIONS: 0x8CD9,
  FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT: 0x8CD7,
  FRAMEBUFFER_UNSUPPORTED: 0x8CDD,
  FRONT: 0x0404,
  FRONT_AND_BACK: 0x0408,
  FRONT_FACE: 0x0B46,
  FUNC_ADD: 0x8006,
  FUNC_REVERSE_SUBTRACT: 0x800B,
  FUNC_SUBTRACT: 0x800A,
  GENERATE_MIPMAP_HINT: 0x8192,
  GEQUAL: 0x0206,
  GREATER: 0x0204,
  GREEN_BITS: 0x0D53,
  HIGH_FLOAT: 0x8DF2,
  HIGH_INT: 0x8DF5,
  IMPLEMENTATION_COLOR_READ_FORMAT: 0x8B9B,
  IMPLEMENTATION_COLOR_READ_TYPE: 0x8B9A,
  INCR: 0x1E02,
  INCR_WRAP: 0x8507,
  INT: 0x1404,
  INT_VEC2: 0x8B53,
  INT_VEC3: 0x8B54,
  INT_VEC4: 0x8B55,
  INVALID_ENUM: 0x0500,
  INVALID_FRAMEBUFFER_OPERATION: 0x0506,
  INVALID_OPERATION: 0x0502,
  INVALID_VALUE: 0x0501,
  INVERT: 0x150A,
  KEEP: 0x1E00,
  LEQUAL: 0x0203,
  LESS: 0x0201,
  LINEAR: 0x2601,
  LINEAR_MIPMAP_LINEAR: 0x2703,
  LINEAR_MIPMAP_NEAREST: 0x2701,
  LINES: 0x0001,
  LINE_LOOP: 0x0002,
  LINE_STRIP: 0x0003,
  LINE_WIDTH: 0x0B21,
  LINK_STATUS: 0x8B82,
  LOW_FLOAT: 0x8DF0,
  LOW_INT: 0x8DF3,
  LUMINANCE: 0x1909,
  LUMINANCE_ALPHA: 0x190A,
  MAX_COMBINED_TEXTURE_IMAGE_UNITS: 0x8B4D,
  MAX_CUBE_MAP_TEXTURE_SIZE: 0x851C,
  MAX_FRAGMENT_UNIFORM_VECTORS: 0x8DFD,
  MAX_RENDERBUFFER_SIZE: 0x84E8,
  MAX_TEXTURE_IMAGE_UNITS: 0x8872,
  MAX_TEXTURE_SIZE: 0x0D33,
  MAX_VARYING_VECTORS: 0x8DFC,
  MAX_VERTEX_ATTRIBS: 0x8869,
  MAX_VERTEX_TEXTURE_IMAGE_UNITS: 0x8B4C,
  MAX_VERTEX_UNIFORM_VECTORS: 0x8DFB,
  MAX_VIEWPORT_DIMS: 0x0D3A,
  MEDIUM_FLOAT: 0x8DF1,
  MEDIUM_INT: 0x8DF4,
  MIRRORED_REPEAT: 0x8370,
  NEAREST: 0x2600,
  NEAREST_MIPMAP_LINEAR: 0x2702,
  NEAREST_MIPMAP_NEAREST: 0x2700,
  NEVER: 0x0200,
  NICEST: 0x1102,
  NONE: 0,
  NOTEQUAL: 0x0205,
  NO_ERROR: 0,
  ONE: 1,
  ONE_MINUS_CONSTANT_ALPHA: 0x8004,
  ONE_MINUS_CONSTANT_COLOR: 0x8002,
  ONE_MINUS_DST_ALPHA: 0x0305,
  ONE_MINUS_DST_COLOR: 0x0307,
  ONE_MINUS_SRC_ALPHA: 0x0303,
  ONE_MINUS_SRC_COLOR: 0x0301,
  OUT_OF_MEMORY: 0x0505,
  PACK_ALIGNMENT: 0x0D05,
  POINTS: 0x0000,
  POLYGON_OFFSET_FACTOR: 0x8038,
  POLYGON_OFFSET_FILL: 0x8037,
  POLYGON_OFFSET_UNITS: 0x2A00,
  RED_BITS: 0x0D52,
  RENDERBUFFER: 0x8D41,
  RENDERBUFFER_ALPHA_SIZE: 0x8D53,
  RENDERBUFFER_BINDING: 0x8CA7,
  RENDERBUFFER_BLUE_SIZE: 0x8D52,
  RENDERBUFFER_DEPTH_SIZE: 0x8D54,
  RENDERBUFFER_GREEN_SIZE: 0x8D51,
  RENDERBUFFER_HEIGHT: 0x8D43,
  RENDERBUFFER_INTERNAL_FORMAT: 0x8D44,
  RENDERBUFFER_RED_SIZE: 0x8D50,
  RENDERBUFFER_STENCIL_SIZE: 0x8D55,
  RENDERBUFFER_WIDTH: 0x8D42,
  RENDERER: 0x1F01,
  REPEAT: 0x2901,
  REPLACE: 0x1E01,
  RGB: 0x1907,
  RGB565: 0x8D62,
  RGB5_A1: 0x8057,
  RGBA: 0x1908,
  RGBA4: 0x8056,
  RGBA8: 0x8058,
  SAMPLER_2D: 0x8B5E,
  SAMPLER_CUBE: 0x8B60,
  SAMPLES: 0x80A9,
  SAMPLE_ALPHA_TO_COVERAGE: 0x809E,
  SAMPLE_BUFFERS: 0x80A8,
  SAMPLE_COVERAGE: 0x80A0,
  SAMPLE_COVERAGE_INVERT: 0x80AB,
  SAMPLE_COVERAGE_VALUE: 0x80AA,
  SCISSOR_BOX: 0x0C10,
  SCISSOR_TEST: 0x0C11,
  SHADER_TYPE: 0x8B4F,
  SHADING_LANGUAGE_VERSION: 0x8B8C,
  SHORT: 0x1402,
  SRC_ALPHA: 0x0302,
  SRC_ALPHA_SATURATE: 0x0308,
  SRC_COLOR: 0x0300,
  STATIC_DRAW: 0x88E4,
  STENCIL_ATTACHMENT: 0x8D20,
  STENCIL_BACK_FAIL: 0x8801,
  STENCIL_BACK_FUNC: 0x8800,
  STENCIL_BACK_PASS_DEPTH_FAIL: 0x8802,
  STENCIL_BACK_PASS_DEPTH_PASS: 0x8803,
  STENCIL_BACK_REF: 0x8CA3,
  STENCIL_BACK_VALUE_MASK: 0x8CA4,
  STENCIL_BACK_WRITEMASK: 0x8CA5,
  STENCIL_BITS: 0x0D57,
  STENCIL_BUFFER_BIT: 0x00000400,
  STENCIL_CLEAR_VALUE: 0x0B91,
  STENCIL_FAIL: 0x0B94,
  STENCIL_FUNC: 0x0B92,
  STENCIL_INDEX8: 0x8D48,
  STENCIL_PASS_DEPTH_FAIL: 0x0B95,
  STENCIL_PASS_DEPTH_PASS: 0x0B96,
  STENCIL_REF: 0x0B97,
  STENCIL_TEST: 0x0B90,
  STENCIL_VALUE_MASK: 0x0B93,
  STENCIL_WRITEMASK: 0x0B98,
  STREAM_DRAW: 0x88E0,
  SUBPIXEL_BITS: 0x0D50,
  TEXTURE: 0x1702,
  TEXTURE0: 0x84C0,
  TEXTURE1: 0x84C1,
  TEXTURE10: 0x84CA,
  TEXTURE11: 0x84CB,
  TEXTURE12: 0x84CC,
  TEXTURE13: 0x84CD,
  TEXTURE14: 0x84CE,
  TEXTURE15: 0x84CF,
  TEXTURE16: 0x84D0,
  TEXTURE17: 0x84D1,
  TEXTURE18: 0x84D2,
  TEXTURE19: 0x84D3,
  TEXTURE2: 0x84C2,
  TEXTURE20: 0x84D4,
  TEXTURE21: 0x84D5,
  TEXTURE22: 0x84D6,
  TEXTURE23: 0x84D7,
  TEXTURE24: 0x84D8,
  TEXTURE25: 0x84D9,
  TEXTURE26: 0x84DA,
  TEXTURE27: 0x84DB,
  TEXTURE28: 0x84DC,
  TEXTURE29: 0x84DD,
  TEXTURE3: 0x84C3,
  TEXTURE30: 0x84DE,
  TEXTURE31: 0x84DF,
  TEXTURE4: 0x84C4,
  TEXTURE5: 0x84C5,
  TEXTURE6: 0x84C6,
  TEXTURE7: 0x84C7,
  TEXTURE8: 0x84C8,
  TEXTURE9: 0x84C9,
  TEXTURE_2D: 0x0DE1,
  TEXTURE_BINDING_2D: 0x8069,
  TEXTURE_BINDING_CUBE_MAP: 0x8514,
  TEXTURE_CUBE_MAP: 0x8513,
  TEXTURE_CUBE_MAP_NEGATIVE_X: 0x8516,
  TEXTURE_CUBE_MAP_NEGATIVE_Y: 0x8518,
  TEXTURE_CUBE_MAP_NEGATIVE_Z: 0x851A,
  TEXTURE_CUBE_MAP_POSITIVE_X: 0x8515,
  TEXTURE_CUBE_MAP_POSITIVE_Y: 0x8517,
  TEXTURE_CUBE_MAP_POSITIVE_Z: 0x8519,
  TEXTURE_MAG_FILTER: 0x2800,
  TEXTURE_MIN_FILTER: 0x2801,
  TEXTURE_WRAP_S: 0x2802,
  TEXTURE_WRAP_T: 0x2803,
  TRIANGLES: 0x0004,
  TRIANGLE_FAN: 0x0006,
  TRIANGLE_STRIP: 0x0005,
  UNPACK_ALIGNMENT: 0x0CF5,
  UNPACK_COLORSPACE_CONVERSION_WEBGL: 0x9243,
  UNPACK_FLIP_Y_WEBGL: 0x9240,
  UNPACK_PREMULTIPLY_ALPHA_WEBGL: 0x9241,
  UNSIGNED_BYTE: 0x1401,
  UNSIGNED_INT: 0x1405,
  UNSIGNED_SHORT: 0x1403,
  UNSIGNED_SHORT_4_4_4_4: 0x8033,
  UNSIGNED_SHORT_5_5_5_1: 0x8034,
  UNSIGNED_SHORT_5_6_5: 0x8363,
  VALIDATE_STATUS: 0x8B83,
  VENDOR: 0x1F00,
  VERSION: 0x1F02,
  VERTEX_ATTRIB_ARRAY_BUFFER_BINDING: 0x889F,
  VERTEX_ATTRIB_ARRAY_ENABLED: 0x8622,
  VERTEX_ATTRIB_ARRAY_NORMALIZED: 0x886A,
  VERTEX_ATTRIB_ARRAY_POINTER: 0x8645,
  VERTEX_ATTRIB_ARRAY_SIZE: 0x8623,
  VERTEX_ATTRIB_ARRAY_STRIDE: 0x8624,
  VERTEX_ATTRIB_ARRAY_TYPE: 0x8625,
  VERTEX_SHADER: 0x8B31,
  VIEWPORT: 0x0BA2,
  ZERO: 0,
};

var _ZW_WGL2C = {
  ACTIVE_UNIFORM_BLOCKS: 0x8A36,
  ALREADY_SIGNALED: 0x911A,
  ANY_SAMPLES_PASSED: 0x8C2F,
  ANY_SAMPLES_PASSED_CONSERVATIVE: 0x8D6A,
  COLOR: 0x1800,
  COLOR_ATTACHMENT1: 0x8CE1,
  COLOR_ATTACHMENT10: 0x8CEA,
  COLOR_ATTACHMENT11: 0x8CEB,
  COLOR_ATTACHMENT12: 0x8CEC,
  COLOR_ATTACHMENT13: 0x8CED,
  COLOR_ATTACHMENT14: 0x8CEE,
  COLOR_ATTACHMENT15: 0x8CEF,
  COLOR_ATTACHMENT2: 0x8CE2,
  COLOR_ATTACHMENT3: 0x8CE3,
  COLOR_ATTACHMENT4: 0x8CE4,
  COLOR_ATTACHMENT5: 0x8CE5,
  COLOR_ATTACHMENT6: 0x8CE6,
  COLOR_ATTACHMENT7: 0x8CE7,
  COLOR_ATTACHMENT8: 0x8CE8,
  COLOR_ATTACHMENT9: 0x8CE9,
  COMPARE_REF_TO_TEXTURE: 0x884E,
  CONDITION_SATISFIED: 0x911C,
  COPY_READ_BUFFER: 0x8F36,
  COPY_READ_BUFFER_BINDING: 0x8F36,
  COPY_WRITE_BUFFER: 0x8F37,
  COPY_WRITE_BUFFER_BINDING: 0x8F37,
  CURRENT_QUERY: 0x8865,
  DEPTH: 0x1801,
  DEPTH24_STENCIL8: 0x88F0,
  DEPTH32F_STENCIL8: 0x8CAD,
  DEPTH_COMPONENT24: 0x81A6,
  DEPTH_COMPONENT32F: 0x8CAC,
  DRAW_BUFFER0: 0x8825,
  DRAW_BUFFER1: 0x8826,
  DRAW_BUFFER10: 0x882F,
  DRAW_BUFFER11: 0x8830,
  DRAW_BUFFER12: 0x8831,
  DRAW_BUFFER13: 0x8832,
  DRAW_BUFFER14: 0x8833,
  DRAW_BUFFER15: 0x8834,
  DRAW_BUFFER2: 0x8827,
  DRAW_BUFFER3: 0x8828,
  DRAW_BUFFER4: 0x8829,
  DRAW_BUFFER5: 0x882A,
  DRAW_BUFFER6: 0x882B,
  DRAW_BUFFER7: 0x882C,
  DRAW_BUFFER8: 0x882D,
  DRAW_BUFFER9: 0x882E,
  DRAW_FRAMEBUFFER: 0x8CA9,
  DRAW_FRAMEBUFFER_BINDING: 0x8CA6,
  DYNAMIC_COPY: 0x88EA,
  DYNAMIC_READ: 0x88E9,
  FLOAT_32_UNSIGNED_INT_24_8_REV: 0x8DAD,
  FLOAT_MAT2x3: 0x8B65,
  FLOAT_MAT2x4: 0x8B66,
  FLOAT_MAT3x2: 0x8B67,
  FLOAT_MAT3x4: 0x8B68,
  FLOAT_MAT4x2: 0x8B69,
  FLOAT_MAT4x3: 0x8B6A,
  FRAGMENT_SHADER_DERIVATIVE_HINT: 0x8B8B,
  FRAMEBUFFER_ATTACHMENT_ALPHA_SIZE: 0x8215,
  FRAMEBUFFER_ATTACHMENT_BLUE_SIZE: 0x8214,
  FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING: 0x8210,
  FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE: 0x8211,
  FRAMEBUFFER_ATTACHMENT_DEPTH_SIZE: 0x8216,
  FRAMEBUFFER_ATTACHMENT_GREEN_SIZE: 0x8213,
  FRAMEBUFFER_ATTACHMENT_RED_SIZE: 0x8212,
  FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE: 0x8217,
  FRAMEBUFFER_ATTACHMENT_TEXTURE_LAYER: 0x8CD4,
  FRAMEBUFFER_DEFAULT: 0x8218,
  FRAMEBUFFER_INCOMPLETE_MULTISAMPLE: 0x8D56,
  HALF_FLOAT: 0x140B,
  INTERLEAVED_ATTRIBS: 0x8C8C,
  INT_2_10_10_10_REV: 0x8D9F,
  INT_SAMPLER_2D: 0x8DCA,
  INT_SAMPLER_2D_ARRAY: 0x8DCF,
  INT_SAMPLER_3D: 0x8DCB,
  INT_SAMPLER_CUBE: 0x8DCC,
  INVALID_INDEX: 0xFFFFFFFF,
  MAX: 0x8008,
  MAX_3D_TEXTURE_SIZE: 0x8073,
  MAX_ARRAY_TEXTURE_LAYERS: 0x88FF,
  MAX_CLIENT_WAIT_TIMEOUT_WEBGL: 0x9247,
  MAX_COLOR_ATTACHMENTS: 0x8CDF,
  MAX_COMBINED_FRAGMENT_UNIFORM_COMPONENTS: 0x8A33,
  MAX_COMBINED_UNIFORM_BLOCKS: 0x8A2E,
  MAX_COMBINED_VERTEX_UNIFORM_COMPONENTS: 0x8A31,
  MAX_DRAW_BUFFERS: 0x8824,
  MAX_ELEMENTS_INDICES: 0x80E9,
  MAX_ELEMENTS_VERTICES: 0x80E8,
  MAX_ELEMENT_INDEX: 0x8D6B,
  MAX_FRAGMENT_INPUT_COMPONENTS: 0x9125,
  MAX_FRAGMENT_UNIFORM_BLOCKS: 0x8A2D,
  MAX_FRAGMENT_UNIFORM_COMPONENTS: 0x8B49,
  MAX_PROGRAM_TEXEL_OFFSET: 0x8905,
  MAX_SAMPLES: 0x8D57,
  MAX_SERVER_WAIT_TIMEOUT: 0x9111,
  MAX_TEXTURE_LOD_BIAS: 0x84FD,
  MAX_TRANSFORM_FEEDBACK_INTERLEAVED_COMPONENTS: 0x8C8A,
  MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS: 0x8C8B,
  MAX_TRANSFORM_FEEDBACK_SEPARATE_COMPONENTS: 0x8C80,
  MAX_UNIFORM_BLOCK_SIZE: 0x8A30,
  MAX_UNIFORM_BUFFER_BINDINGS: 0x8A2F,
  MAX_VARYING_COMPONENTS: 0x8B4B,
  MAX_VERTEX_OUTPUT_COMPONENTS: 0x9122,
  MAX_VERTEX_UNIFORM_BLOCKS: 0x8A2B,
  MAX_VERTEX_UNIFORM_COMPONENTS: 0x8B4A,
  MIN: 0x8007,
  MIN_PROGRAM_TEXEL_OFFSET: 0x8904,
  OBJECT_TYPE: 0x9112,
  PACK_ROW_LENGTH: 0x0D02,
  PACK_SKIP_PIXELS: 0x0D04,
  PACK_SKIP_ROWS: 0x0D03,
  PIXEL_PACK_BUFFER: 0x88EB,
  PIXEL_PACK_BUFFER_BINDING: 0x88ED,
  PIXEL_UNPACK_BUFFER: 0x88EC,
  PIXEL_UNPACK_BUFFER_BINDING: 0x88EF,
  QUERY_RESULT: 0x8866,
  QUERY_RESULT_AVAILABLE: 0x8867,
  R11F_G11F_B10F: 0x8C3A,
  R16F: 0x822D,
  R16I: 0x8233,
  R16UI: 0x8234,
  R32F: 0x822E,
  R32I: 0x8235,
  R32UI: 0x8236,
  R8: 0x8229,
  R8I: 0x8231,
  R8UI: 0x8232,
  R8_SNORM: 0x8F94,
  RASTERIZER_DISCARD: 0x8C89,
  READ_BUFFER: 0x0C02,
  READ_FRAMEBUFFER: 0x8CA8,
  READ_FRAMEBUFFER_BINDING: 0x8CAA,
  RED: 0x1903,
  RED_INTEGER: 0x8D94,
  RENDERBUFFER_SAMPLES: 0x8CAB,
  RG: 0x8227,
  RG16F: 0x822F,
  RG16I: 0x8239,
  RG16UI: 0x823A,
  RG32F: 0x8230,
  RG32I: 0x823B,
  RG32UI: 0x823C,
  RG8: 0x822B,
  RG8I: 0x8237,
  RG8UI: 0x8238,
  RG8_SNORM: 0x8F95,
  RGB10_A2: 0x8059,
  RGB10_A2UI: 0x906F,
  RGB16F: 0x881B,
  RGB16I: 0x8D89,
  RGB16UI: 0x8D77,
  RGB32F: 0x8815,
  RGB32I: 0x8D83,
  RGB32UI: 0x8D71,
  RGB8: 0x8051,
  RGB8I: 0x8D8F,
  RGB8UI: 0x8D7D,
  RGB8_SNORM: 0x8F96,
  RGB9_E5: 0x8C3D,
  RGBA16F: 0x881A,
  RGBA16I: 0x8D88,
  RGBA16UI: 0x8D76,
  RGBA32F: 0x8814,
  RGBA32I: 0x8D82,
  RGBA32UI: 0x8D70,
  RGBA8I: 0x8D8E,
  RGBA8UI: 0x8D7C,
  RGBA8_SNORM: 0x8F97,
  RGBA_INTEGER: 0x8D99,
  RGB_INTEGER: 0x8D98,
  RG_INTEGER: 0x8228,
  SAMPLER_2D_ARRAY: 0x8DC1,
  SAMPLER_2D_ARRAY_SHADOW: 0x8DC4,
  SAMPLER_2D_SHADOW: 0x8B62,
  SAMPLER_3D: 0x8B5F,
  SAMPLER_BINDING: 0x8919,
  SAMPLER_CUBE_SHADOW: 0x8DC5,
  SEPARATE_ATTRIBS: 0x8C8D,
  SIGNALED: 0x9119,
  SIGNED_NORMALIZED: 0x8F9C,
  SRGB: 0x8C40,
  SRGB8: 0x8C41,
  SRGB8_ALPHA8: 0x8C43,
  STATIC_COPY: 0x88E6,
  STATIC_READ: 0x88E5,
  STENCIL: 0x1802,
  STREAM_COPY: 0x88E2,
  STREAM_READ: 0x88E1,
  SYNC_CONDITION: 0x9113,
  SYNC_FENCE: 0x9116,
  SYNC_FLAGS: 0x9115,
  SYNC_FLUSH_COMMANDS_BIT: 0x00000001,
  SYNC_GPU_COMMANDS_COMPLETE: 0x9117,
  SYNC_STATUS: 0x9114,
  TEXTURE_2D_ARRAY: 0x8C1A,
  TEXTURE_3D: 0x806F,
  TEXTURE_BASE_LEVEL: 0x813C,
  TEXTURE_BINDING_2D_ARRAY: 0x8C1D,
  TEXTURE_BINDING_3D: 0x806A,
  TEXTURE_COMPARE_FUNC: 0x884D,
  TEXTURE_COMPARE_MODE: 0x884C,
  TEXTURE_IMMUTABLE_FORMAT: 0x912F,
  TEXTURE_IMMUTABLE_LEVELS: 0x82DF,
  TEXTURE_MAX_LEVEL: 0x813D,
  TEXTURE_MAX_LOD: 0x813B,
  TEXTURE_MIN_LOD: 0x813A,
  TEXTURE_WRAP_R: 0x8072,
  TIMEOUT_EXPIRED: 0x911B,
  TIMEOUT_IGNORED: -1,
  TRANSFORM_FEEDBACK: 0x8E22,
  TRANSFORM_FEEDBACK_ACTIVE: 0x8E24,
  TRANSFORM_FEEDBACK_BINDING: 0x8E25,
  TRANSFORM_FEEDBACK_BUFFER: 0x8C8E,
  TRANSFORM_FEEDBACK_BUFFER_BINDING: 0x8C8F,
  TRANSFORM_FEEDBACK_BUFFER_MODE: 0x8C7F,
  TRANSFORM_FEEDBACK_BUFFER_SIZE: 0x8C85,
  TRANSFORM_FEEDBACK_BUFFER_START: 0x8C84,
  TRANSFORM_FEEDBACK_PAUSED: 0x8E23,
  TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN: 0x8C88,
  TRANSFORM_FEEDBACK_VARYINGS: 0x8C83,
  UNIFORM_ARRAY_STRIDE: 0x8A3C,
  UNIFORM_BLOCK_ACTIVE_UNIFORMS: 0x8A42,
  UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES: 0x8A43,
  UNIFORM_BLOCK_BINDING: 0x8A3F,
  UNIFORM_BLOCK_DATA_SIZE: 0x8A40,
  UNIFORM_BLOCK_INDEX: 0x8A3A,
  UNIFORM_BLOCK_REFERENCED_BY_FRAGMENT_SHADER: 0x8A46,
  UNIFORM_BLOCK_REFERENCED_BY_VERTEX_SHADER: 0x8A44,
  UNIFORM_BUFFER: 0x8A11,
  UNIFORM_BUFFER_BINDING: 0x8A28,
  UNIFORM_BUFFER_OFFSET_ALIGNMENT: 0x8A34,
  UNIFORM_BUFFER_SIZE: 0x8A2A,
  UNIFORM_BUFFER_START: 0x8A29,
  UNIFORM_IS_ROW_MAJOR: 0x8A3E,
  UNIFORM_MATRIX_STRIDE: 0x8A3D,
  UNIFORM_OFFSET: 0x8A3B,
  UNIFORM_SIZE: 0x8A38,
  UNIFORM_TYPE: 0x8A37,
  UNPACK_IMAGE_HEIGHT: 0x806E,
  UNPACK_ROW_LENGTH: 0x0CF2,
  UNPACK_SKIP_IMAGES: 0x806D,
  UNPACK_SKIP_PIXELS: 0x0CF4,
  UNPACK_SKIP_ROWS: 0x0CF3,
  UNSIGNALED: 0x9118,
  UNSIGNED_INT_10F_11F_11F_REV: 0x8C3B,
  UNSIGNED_INT_24_8: 0x84FA,
  UNSIGNED_INT_2_10_10_10_REV: 0x8368,
  UNSIGNED_INT_5_9_9_9_REV: 0x8C3E,
  UNSIGNED_INT_SAMPLER_2D: 0x8DD2,
  UNSIGNED_INT_SAMPLER_2D_ARRAY: 0x8DD7,
  UNSIGNED_INT_SAMPLER_3D: 0x8DD3,
  UNSIGNED_INT_SAMPLER_CUBE: 0x8DD4,
  UNSIGNED_INT_VEC2: 0x8DC6,
  UNSIGNED_INT_VEC3: 0x8DC7,
  UNSIGNED_INT_VEC4: 0x8DC8,
  UNSIGNED_NORMALIZED: 0x8C17,
  VERTEX_ARRAY_BINDING: 0x85B5,
  VERTEX_ATTRIB_ARRAY_DIVISOR: 0x88FE,
  VERTEX_ATTRIB_ARRAY_INTEGER: 0x88FD,
  WAIT_FAILED: 0x911D,
};

var _ZW_WGL1M = {
  activeTexture: 1,
  attachShader: 2,
  bindAttribLocation: 3,
  bindBuffer: 2,
  bindFramebuffer: 2,
  bindRenderbuffer: 2,
  bindTexture: 2,
  blendColor: 4,
  blendEquation: 1,
  blendEquationSeparate: 2,
  blendFunc: 2,
  blendFuncSeparate: 4,
  bufferData: 3,
  bufferSubData: 3,
  checkFramebufferStatus: 1,
  clear: 1,
  clearColor: 4,
  clearDepth: 1,
  clearStencil: 1,
  colorMask: 4,
  compileShader: 1,
  compressedTexImage2D: 7,
  compressedTexSubImage2D: 8,
  copyTexImage2D: 8,
  copyTexSubImage2D: 8,
  createBuffer: 0,
  createFramebuffer: 0,
  createProgram: 0,
  createRenderbuffer: 0,
  createShader: 1,
  createTexture: 0,
  cullFace: 1,
  deleteBuffer: 1,
  deleteFramebuffer: 1,
  deleteProgram: 1,
  deleteRenderbuffer: 1,
  deleteShader: 1,
  deleteTexture: 1,
  depthFunc: 1,
  depthMask: 1,
  depthRange: 2,
  detachShader: 2,
  disable: 1,
  disableVertexAttribArray: 1,
  drawArrays: 3,
  drawElements: 4,
  drawingBufferStorage: 3,
  enable: 1,
  enableVertexAttribArray: 1,
  finish: 0,
  flush: 0,
  framebufferRenderbuffer: 4,
  framebufferTexture2D: 5,
  frontFace: 1,
  generateMipmap: 1,
  getActiveAttrib: 2,
  getActiveUniform: 2,
  getAttachedShaders: 1,
  getAttribLocation: 2,
  getBufferParameter: 2,
  getContextAttributes: 0,
  getError: 0,
  getExtension: 1,
  getFramebufferAttachmentParameter: 3,
  getParameter: 1,
  getProgramInfoLog: 1,
  getProgramParameter: 2,
  getRenderbufferParameter: 2,
  getShaderInfoLog: 1,
  getShaderParameter: 2,
  getShaderPrecisionFormat: 2,
  getShaderSource: 1,
  getSupportedExtensions: 0,
  getTexParameter: 2,
  getUniform: 2,
  getUniformLocation: 2,
  getVertexAttrib: 2,
  getVertexAttribOffset: 2,
  hint: 2,
  isBuffer: 1,
  isContextLost: 0,
  isEnabled: 1,
  isFramebuffer: 1,
  isProgram: 1,
  isRenderbuffer: 1,
  isShader: 1,
  isTexture: 1,
  lineWidth: 1,
  linkProgram: 1,
  pixelStorei: 2,
  polygonOffset: 2,
  readPixels: 7,
  renderbufferStorage: 4,
  sampleCoverage: 2,
  scissor: 4,
  shaderSource: 2,
  stencilFunc: 3,
  stencilFuncSeparate: 4,
  stencilMask: 1,
  stencilMaskSeparate: 2,
  stencilOp: 3,
  stencilOpSeparate: 4,
  texImage2D: 6,
  texParameterf: 3,
  texParameteri: 3,
  texSubImage2D: 7,
  uniform1f: 2,
  uniform1fv: 2,
  uniform1i: 2,
  uniform1iv: 2,
  uniform2f: 3,
  uniform2fv: 2,
  uniform2i: 3,
  uniform2iv: 2,
  uniform3f: 4,
  uniform3fv: 2,
  uniform3i: 4,
  uniform3iv: 2,
  uniform4f: 5,
  uniform4fv: 2,
  uniform4i: 5,
  uniform4iv: 2,
  uniformMatrix2fv: 3,
  uniformMatrix3fv: 3,
  uniformMatrix4fv: 3,
  useProgram: 1,
  validateProgram: 1,
  vertexAttrib1f: 2,
  vertexAttrib1fv: 2,
  vertexAttrib2f: 3,
  vertexAttrib2fv: 2,
  vertexAttrib3f: 4,
  vertexAttrib3fv: 2,
  vertexAttrib4f: 5,
  vertexAttrib4fv: 2,
  vertexAttribPointer: 6,
  viewport: 4,
};

var _ZW_WGL2M = {
  beginQuery: 2,
  beginTransformFeedback: 1,
  bindBufferBase: 3,
  bindBufferRange: 5,
  bindSampler: 2,
  bindTransformFeedback: 2,
  bindVertexArray: 1,
  blitFramebuffer: 10,
  bufferData: 3,
  bufferSubData: 3,
  clearBufferfi: 4,
  clearBufferfv: 3,
  clearBufferiv: 3,
  clearBufferuiv: 3,
  clientWaitSync: 3,
  compressedTexImage2D: 7,
  compressedTexImage3D: 8,
  compressedTexSubImage2D: 8,
  compressedTexSubImage3D: 10,
  copyBufferSubData: 5,
  copyTexSubImage3D: 9,
  createQuery: 0,
  createSampler: 0,
  createTransformFeedback: 0,
  createVertexArray: 0,
  deleteQuery: 1,
  deleteSampler: 1,
  deleteSync: 1,
  deleteTransformFeedback: 1,
  deleteVertexArray: 1,
  drawArraysInstanced: 4,
  drawBuffers: 1,
  drawElementsInstanced: 5,
  drawRangeElements: 6,
  endQuery: 1,
  endTransformFeedback: 0,
  fenceSync: 2,
  framebufferTextureLayer: 5,
  getActiveUniformBlockName: 2,
  getActiveUniformBlockParameter: 3,
  getActiveUniforms: 3,
  getBufferSubData: 3,
  getFragDataLocation: 2,
  getIndexedParameter: 2,
  getInternalformatParameter: 3,
  getQuery: 2,
  getQueryParameter: 2,
  getSamplerParameter: 2,
  getSyncParameter: 2,
  getTransformFeedbackVarying: 2,
  getUniformBlockIndex: 2,
  getUniformIndices: 2,
  invalidateFramebuffer: 2,
  invalidateSubFramebuffer: 6,
  isQuery: 1,
  isSampler: 1,
  isSync: 1,
  isTransformFeedback: 1,
  isVertexArray: 1,
  pauseTransformFeedback: 0,
  readBuffer: 1,
  readPixels: 7,
  renderbufferStorageMultisample: 5,
  resumeTransformFeedback: 0,
  samplerParameterf: 3,
  samplerParameteri: 3,
  texImage2D: 6,
  texImage3D: 10,
  texStorage2D: 5,
  texStorage3D: 6,
  texSubImage2D: 7,
  texSubImage3D: 11,
  transformFeedbackVaryings: 3,
  uniform1fv: 2,
  uniform1iv: 2,
  uniform1ui: 2,
  uniform1uiv: 2,
  uniform2fv: 2,
  uniform2iv: 2,
  uniform2ui: 3,
  uniform2uiv: 2,
  uniform3fv: 2,
  uniform3iv: 2,
  uniform3ui: 4,
  uniform3uiv: 2,
  uniform4fv: 2,
  uniform4iv: 2,
  uniform4ui: 5,
  uniform4uiv: 2,
  uniformBlockBinding: 3,
  uniformMatrix2fv: 3,
  uniformMatrix2x3fv: 3,
  uniformMatrix2x4fv: 3,
  uniformMatrix3fv: 3,
  uniformMatrix3x2fv: 3,
  uniformMatrix3x4fv: 3,
  uniformMatrix4fv: 3,
  uniformMatrix4x2fv: 3,
  uniformMatrix4x3fv: 3,
  vertexAttribDivisor: 2,
  vertexAttribI4i: 5,
  vertexAttribI4iv: 2,
  vertexAttribI4ui: 5,
  vertexAttribI4uiv: 2,
  vertexAttribIPointer: 5,
  waitSync: 3,
};

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

  function _zwGlNewObject(ifaceName) {
    return { _zwGlObj: true, _zwGlIface: ifaceName, _label: '' };
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
  var _ZW_EXT_NAMES = ['WEBGL_compressed_texture_etc1', 'WEBGL_compressed_texture_s3tc'];

  // GL 枚举集（错误语义校验用；值同数据表）。
  var _ZW_BIND_TARGETS = { 0x8892: 1, 0x8893: 1 }; // ARRAY_BUFFER / ELEMENT_ARRAY_BUFFER
  var _ZW_TEX_BIND_TARGETS = { 0x0DE1: 1, 0x8513: 1 }; // TEXTURE_2D / TEXTURE_CUBE_MAP
  var _ZW_USAGE = { 0x88E0: 1, 0x88E1: 1, 0x88E2: 1, 0x88E3: 1, 0x88E4: 1 }; // STREAM_/STATIC_/DYNAMIC_{READ,DRAW,COPY}
  var _ZW_BEGIN_MODES = { 0: 1, 1: 1, 2: 1, 3: 1, 4: 1, 5: 1, 6: 1 };
  var _ZW_CLEAR_MASKS = 0x4100; // DEPTH|STENCIL|COLOR_BUFFER_BIT

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
    getError: function (state, args) {
      var err = state._err;
      state._err = 0;
      return err;
    },

    // —— 对象族 ——
    createShader: function (state, args) {
      return _zwGlNewObject('WebGLShader');
    },
    isContextLost: function (state, args) { return false; },
    getContextAttributes: function (state, args) {
      // WebGLContextAttributes 默认面（webgl1.idl dictionary 同构；M2 不消费
      // 选项值——FIXME(M3)：attrs 尊重与 drawing buffer 配置）。
      return {
        alpha: true, depth: true, stencil: false, antialias: true,
        premultipliedAlpha: true, preserveDrawingBuffer: false,
        powerPreference: 'default', failIfMajorPerformanceCaveat: false,
        desynchronized: false
      };
    },
    getSupportedExtensions: function (state, args) {
      return _ZW_EXT_NAMES.slice();
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

    // —— program / uniform（corpus uniformMatrixNfv.html）——
    getUniformLocation: function (state, args) {
      var program = args[0];
      if (!program || !program._zwGlObj || program._zwGlIface !== 'WebGLProgram') {
        throw new TypeError('parameter 1 is not of type WebGLProgram');
      }
      // 未成功链接的 program → INVALID_OPERATION + null（spec 5.14.9；M2 无
      // shader 编译管线——所有 program 恒未链接，uniform* 调用经 null location
      // 静默忽略路径，corpus uniformMatrixNfv 的 no-throw 断言依赖此链）。
      _zwGlSetError(state, 0x0502);
      return null;
    },
    uniformMatrix2fv: _zwGlUniformImpl,
    uniformMatrix3fv: _zwGlUniformImpl,
    uniformMatrix4fv: _zwGlUniformImpl,

    // —— 绘制状态机（corpus 无直接案；draw 状态语义为 M2 Mission 面）——
    drawArrays: function (state, args) {
      var mode = +args[0];
      if (!_ZW_BEGIN_MODES[mode]) { _zwGlSetError(state, 0x0500); return; }
      // 无合法 program → INVALID_OPERATION（GL spec；M2 program 恒未链接）。
      if (!state._program) { _zwGlSetError(state, 0x0502); return; }
      _zwGlSetError(state, 0x0502); // FIXME(M3 draw 簇)：真实绘制管线。
    },
    drawElements: function (state, args) {
      var mode = +args[0];
      if (!_ZW_BEGIN_MODES[mode]) { _zwGlSetError(state, 0x0500); return; }
      var type = +args[2];
      if (type !== 0x1401 && type !== 0x1403) { _zwGlSetError(state, 0x0500); return; } // UNSIGNED_BYTE/SHORT
      if (!state._program) { _zwGlSetError(state, 0x0502); return; }
      _zwGlSetError(state, 0x0502); // FIXME(M3 draw 簇)。
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
    enable: function (state, args) { state._enabled[+args[0]] = true; },
    disable: function (state, args) { state._enabled[+args[0]] = false; },

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

    // —— 参数查询（get/plausible 默认面）——
    getParameter: function (state, args) {
      var pname = args.length > 0 ? +args[0] : 0;
      switch (pname) {
        case 0x0D33: return state._drawingBufferWidth;  // MAX_TEXTURE_SIZE 近似面——FIXME(M3) 真值
        case 0x0BA2: return state._viewport;            // VIEWPORT
        case 0x0C10: return state._scissor;             // SCISSOR_BOX
        case 0x8056: return new Float32Array(state._clearColor); // COLOR_CLEAR_VALUE
        case 0x8894: return state._bindings[0x8892];    // ARRAY_BUFFER_BINDING
        case 0x8895: return state._bindings[0x8893];    // ELEMENT_ARRAY_BUFFER_BINDING
        case 0x8069: return state._texBindings[0x0DE1]; // TEXTURE_BINDING_2D
        case 0x8B8D: return state._program;             // CURRENT_PROGRAM
        case 0x1F01: return 'ZeroWeb';                  // VENDOR
        case 0x1F02: return 'ZeroWeb';                  // RENDERER
        case 0x1F03: return 'WebGL 1.0';                // VERSION
        case 0x8B8C: return 'WebGL GLSL ES 1.0';        // SHADING_LANGUAGE_VERSION
        default:
          _zwGlSetError(state, 0x0500);
          return null;
      }
    },
    pixelStorei: function (state, args) {
      state._pixelStore[String(+args[0])] = _zwGlToInteger(args[1]);
    },

    // —— shader/program 管线（M2 状态记录面；编译链接 FIXME(M3 shader 簇)）——
    shaderSource: function (state, args) {
      var sh = args[0];
      if (!sh || !sh._zwGlObj || sh._zwGlIface !== 'WebGLShader') {
        throw new TypeError('parameter 1 is not of type WebGLShader');
      }
      sh._zwSource = args.length > 1 ? String(args[1]) : '';
    },
    compileShader: function (state, args) { /* FIXME(M3)：GLSL 编译管线缺席——恒不编译。 */ },
    attachShader: function (state, args) { /* FIXME(M3)。 */ },
    detachShader: function (state, args) { /* FIXME(M3)。 */ },
    linkProgram: function (state, args) { /* FIXME(M3)：链接管线缺席——恒未链接。 */ },
    useProgram: function (state, args) {
      var p = args[0];
      if (p !== null && p !== undefined &&
        (!p._zwGlObj || p._zwGlIface !== 'WebGLProgram')) {
        throw new TypeError('parameter 1 is not of type WebGLProgram');
      }
      state._program = p || null;
    },
    getShaderParameter: function (state, args) {
      var pname = args.length > 1 ? +args[1] : 0;
      if (pname === 0x8B81) return false; // COMPILE_STATUS——编译管线缺席恒 false
      if (pname === 0x8B80) return !!(args[0] && args[0]._zwDeleted); // DELETE_STATUS
      _zwGlSetError(state, 0x0500);
      return null;
    },
    getProgramParameter: function (state, args) {
      var pname = args.length > 1 ? +args[1] : 0;
      if (pname === 0x8B82) return false; // LINK_STATUS——链接管线缺席恒 false
      if (pname === 0x8B83) return false; // VALIDATE_STATUS
      if (pname === 0x8B80) return false; // DELETE_STATUS
      _zwGlSetError(state, 0x0500);
      return null;
    },
    getShaderInfoLog: function (state, args) { return ''; },
    getProgramInfoLog: function (state, args) { return ''; },
    getAttribLocation: function (state, args) {
      // 未链接 program → INVALID_OPERATION + -1（getUniformLocation 同语义族）。
      _zwGlSetError(state, 0x0502);
      return -1;
    },
    getActiveAttrib: function (state, args) {
      _zwGlSetError(state, 0x0502);
      return null;
    },
    getActiveUniform: function (state, args) {
      _zwGlSetError(state, 0x0502);
      return null;
    }
  };

  // uniform* 通用实现：location null → 静默忽略不产生错误（spec 5.14.9——
  // corpus uniformMatrixNfv 的 no-throw 断言面）；transpose 按 WebIDL boolean
  // 转换（ToBoolean 不触 valueOf——{valueOf: throw} 对象期望不抛）；value 序列
  // 转换校验 Array/TypedArray。 FIXME(M3 shader 簇)：真实 uniform 上传。
  function _zwGlUniformImpl(state, args) {
    var loc = args[0];
    if (loc !== null && loc !== undefined &&
      (!loc._zwGlObj || loc._zwGlIface !== 'WebGLUniformLocation')) {
      throw new TypeError('parameter 1 is not of type WebGLUniformLocation');
    }
    if (loc === null || loc === undefined) return;
    var v = args[2];
    if (!Array.isArray(v) && !(typeof v === 'object' && v && typeof v.length === 'number')) {
      throw new TypeError('parameter 3 is not of type sequence');
    }
  }

  // create*/delete*/is* 族共享机械。
  (function () {
    var name;
    for (name in _ZW_CREATE_TAGS) {
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
      _scissor: [0, 0, 0, 0],
      _viewport: [0, 0, width, height],
      _enabled: {},           // cap 枚举 → bool
      _pixelStore: {},
      _compressedFormats: {}, // 使能的压缩格式（getExtension 写入）
      _extObjects: {}
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
          var impl = _zwGlOps[op];
          return impl ? impl(this._zwGlState, arguments) : undefined;
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
