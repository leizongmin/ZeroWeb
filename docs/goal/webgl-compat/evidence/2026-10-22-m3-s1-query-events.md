# M3-S1 查询与事件面——getParameter 全表 + 状态设置族 + WEBGL_lose_context

**日期**: 2026-10-22
**范围**: webgl-compat goal M3 切片①（master.md 切分序①）：getContextAttributes /
getParameter 参数查询补全（WebGL 1.0 §5.14.5/§5.14.6 + GLES2 Tables 6.9/6.10/
6.18 + WebGL §6.4 最小上限）、enable/disable/isEnabled cap 校验（§5.14.3）、
状态设置族往返与域校验（depthMask/frontFace/cullFace/depthFunc/depthRange/
lineWidth/sampleCoverage/blendColor/blendFunc/blendEquation/stencilFunc/
pixelStorei）、getShaderPrecisionFormat（§5.14.5 + GLES2 §6.2 Table 6.6）、
WEBGL_lose_context 上下文丢失/恢复事件链（§5.15）。

**基线**: `make testharness-webgl` **2112 Pass / 3 Fail 不变**（3 Fail 同 M2
§4.1 上游 IDL 同 mixin 同长重载判据，非本切片面）；引擎常驻测试 part68 新增
4 例全绿 + part67 M2 5 例回归全绿；clippy `-D warnings` 零告警；fmt 通过。

---

## 1. 实现面（crates/engine/src/js_dom_shim/part08.js，~1870 行）

### 1.1 getParameter 全表（§5.14.6）

- **字符串身份**：VENDOR/RENDERER `'ZeroWeb'`、VERSION `'WebGL 1.0'`、
  SHADING_LANGUAGE_VERSION `'WebGL GLSL ES 1.0'`。
- **绘图缓冲构成**（Table 6.18）：SUBPIXEL_BITS 4、RGB/ALPHA 位面（ALPHA_BITS
  随 attrs.alpha）、DEPTH_BITS/STENCIL_BITS 0（FIXME(M3)：无 depth attachment）、
  SAMPLES/SAMPLE_BUFFERS 0（无 MSAA 绘图缓冲）。
- **最小上限**（WebGL §6.4 Table）：MAX_TEXTURE_SIZE/MAX_CUBE_MAP_TEXTURE_SIZE/
  MAX_RENDERBUFFER_SIZE 4096、MAX_VERTEX_ATTRIBS 16、MAX_VERTEX_UNIFORM_VECTORS
  128、MAX_FRAGMENT_UNIFORM_VECTORS/MAX_VARYING_VECTORS 16、各纹理单元 16/16/32、
  MAX_VIEWPORT_DIMS Int32Array [16384,16384]、ALIASED_POINT_SIZE_RANGE [1,1024]、
  ALIASED_LINE_WIDTH_RANGE [1,1]（§6.22 恒 1）。
- **状态查询**：viewport/scissor（初值对齐绘图缓冲 [0,0,w,h]，GLES2 §2.8.1）/
  depthRange Float32Array、lineWidth、cull/frontFace/depthFunc/depthMask、
  stencil 族（func/ref/valueMask/writemask/clearValue；fail 族 0x1E00 KEEP）、
  blend 族（color/equation RGB+alpha/src+dst RGB+alpha）、COLOR_CLEAR_VALUE、
  COLOR_WRITEMASK、binding 族（ARRAY/ELEMENT_ARRAY_BUFFER、TEXTURE_2D/CUBE_MAP、
  CURRENT_PROGRAM）、WebGL 专有 unpack 参数（FLIP_Y/PREMULTIPLY_ALPHA/
  COLORSPACE_CONVERSION，初值 false/false/0x9244 BROWSER_DEFAULT_WEBGL）。
- **COMPRESSED_TEXTURE_FORMATS**（pin rev IDL 0x86A3）：Uint32Array，内容随
  per-context 压缩扩展使能动态增长（M2 语义面复用）。
- **IMPLEMENTATION_COLOR_READ_FORMAT/TYPE**：RGBA/UNSIGNED_BYTE。
- 未知 pname → INVALID_ENUM + null。

### 1.2 状态设置族（域校验 + Table 6.9/6.10 初值）

- 工厂状态补齐 Table 6.9/6.10 缺省：frontFace CCW、cullFace BACK、depthFunc
  LESS、depthMask true、depthRange [0,1]、lineWidth 1、sampleCoverage (1,false)、
  stencil (0x0207, 0, 0xFFFFFFFF, 0xFFFFFFFF)、blendColor [0,0,0,0]、blendFunc
  (ONE, ZERO) ×RGB/alpha、blendEquation FUNC_ADD ×RGB/alpha。
- 校验：enable/disable/isEnabled 对照 `_ZW_CAPS`（9 cap）否则 INVALID_ENUM；
  frontFace/cullFace/depthFunc/blendFunc 因子/blendEquation（核心 3 值）枚举
  校验；depthRange clamp [0,1]（GLES2 §2.13）；lineWidth ≤0 INVALID_VALUE、
  支持域 clamp [1,1]（WebGL §6.22）；blendColor NaN→0 无错误（§6.20）；
  stencilFunc mask `isNaN → 0xFFFFFFFF`，`m >>> 0`（mask=0 合法）；
  pixelStorei UNPACK_ALIGNMENT 仅 1/2/4/8（GLES2 §3.6.1）否则 INVALID_VALUE。

### 1.3 getShaderPrecisionFormat（§5.14.5）

- shaderType ∈ {VERTEX_SHADER 0x8B31, FRAGMENT_SHADER 0x8B30}、precisionType
  六值域校验，非法 → INVALID_ENUM + null。返回值取 GLES2 §6.2 最低保证档：
  LOW/MEDIUM_FLOAT [15,15,10]、HIGH_FLOAT [127,127,23]、LOW/MEDIUM/HIGH_INT
  [31,30,0]（整型取上限档，符合 conformance ≥最低值口径）。返回
  WebGLShaderPrecisionFormat 品牌实例。

### 1.4 WEBGL_lose_context（§5.15）

- `_ZW_EXT_NAMES` 增列；getExtension 返 lose/restore 双方法扩展对象（幂等稳定，
  同 M2 扩展语义）。
- `loseContext()`：置 `_lost` + `_lostErrPending`、清错误旗标、派发
  **可取消** `webglcontextlost`（WebGLContextEvent 于 canvas）；listener
  preventDefault → `_restoreSuppressed`。
- `restoreContext()`：未丢失或被抑制 → no-op（§5.15.3——抑制后永不恢复）；
  否则清三态标志 + 派发 `webglcontextrestored`。
- **丢失态方法旁路**（§5.15.1）：wrapper 层 `st._lost && !_ZW_LOST_ALLOW[op]`
  直接返回 undefined——无错误、无状态副作用；白名单 getError/isContextLost/
  getContextAttributes/getSupportedExtensions/getExtension。FIXME(M4)：按
  §5.15.1 逐项核对白名单完整性（当前 5 项为查询/扩展族）。
- **丢失态 getError**：`_lostErrPending` → 返回 CONTEXT_LOST_WEBGL（0x9242）
  一次后归零；getContextAttributes 丢失态返 null。

### 1.5 实例原型链接（共享路径修复，本次新发现缺口）

`_zwGlNewObject` 创建的 GL 对象族（buffer/texture/shader/…/ShaderPrecisionFormat）
此前为裸对象、未链接接口原型——原型上的属性访问器（WebGLObject.label、
WebGLShaderPrecisionFormat.rangeMin/precision 等）对实例不可达。修复：单点
创建器按接口注册表 `Object.setPrototypeOf`（WebIDL 对象创建步）；未注册名退
Object.prototype。原型面零变化（idlharness 断言面不受影响——2112/3 基线保持
即为回归证据）；顺带修通 `buf.label` 读写与未来 `getActiveInfo()` 的属性面。

### 1.6 文件拆分（2000 行准则）

M3-S1 落地后 part08.js 达 2054 行超限（仓库准则 7）——按职责拆分：IDL 派生
**数据表**（四表 821 行，纯数据零闭包依赖）移入 `part08a.js`（前置加载，
`globalThis._zwWglIdlTables` 单点交接，part08 IIFE 顶部消费后置 null；
include 清单顺序 part08a → part08）；part08.js 保留全部逻辑（1242 行）。
验证：node --check 双文件语法过 + idlharness 2099 子测（遍历全部搬移表项的
常量值/方法 length 断言）2112/3 基线不变 + 引擎 scoped 9 例全绿——搬移
字节级等价。

## 2. 常驻测试（js_dom_bridge_tests/part68.rs 新增 4 例，part67 沙盒复用）

1. `test_webgl_context_attributes_face`——九项默认值 + 返回副本隔离（改返回
   对象不影响内部态）+ 未丢失。
2. `test_webgl_parameter_table_face`——19 断言：字符串身份/位面/上限最小值/
   typed array 形（Int32Array/Float32Array/Uint32Array）/状态初值/压缩格式表
   动态性/WebGL 专有像素参数/未知 pname INVALID_ENUM。
3. `test_webgl_state_setters_parameter_roundtrip`——25 断言：设置族→getParameter
   往返 + 域校验（INVALID_ENUM/INVALID_VALUE）+ clamp（depthRange/lineWidth）+
   cap 校验 + UNPACK_ALIGNMENT + 精度格式表 + 非法参数路径。
4. `test_webgl_context_lost_restored_face`——扩展对象幂等、可取消
   WebGLContextEvent（constructor/cancelable/statusMessage）、CONTEXT_LOST_WEBGL
   首查、丢失态旁路（clear 无错误无副作用）、loseContext 幂等、preventDefault
   抑制恢复（restoreContext no-op + restored 事件 0 次）；第二 canvas 新上下文
   走完整 lost→restored 链 + 抑制态上下文保持丢失。

part67 沙盒 HTML 增第二个 `<canvas id='cv2'>`（同型每元素一上下文——抑制恢复
后原上下文按 §5.15.3 永不复原，恢复链路需第二上下文验证）。

## 3. 本轮修正的实现缺陷（测试驱动发现）

| 缺陷 | 根因 | 修复 |
|------|------|------|
| VENDOR/VERSION 查询落 INVALID_ENUM | getParameter 字符串 case 错位（0x1F01→VENDOR、0x1F03→VERSION；classic GL 实为 0x1F00/0x1F02） | case 重排对齐 pin IDL |
| DEPTH_CLEAR_VALUE 形状错误 | `Float32Array([0,1])`——DEPTH_RANGE 的形状误套；规范为标量初值 1 | `return 1`（Table 6.9） |
| `case 0x0B78` 幽灵标签 | 桌面 GL DEPTH_CLEAR_VALUE 残留；pin IDL 无此常量 | 删除（落 default INVALID_ENUM） |
| GL 对象实例属性访问器不可达 | `_zwGlNewObject` 未链接接口原型（§1.5） | 单点创建器接原型 |

## 4. 已知边界（记账）

- **part05 createElement 元素无 dispatchEvent**（既有 DOM 面缺口，非本切片
  引入）：createElement 建的 detached canvas 上 webglcontextlost/restored
  事件不派发（`_zwGlFireContextEvent` 静默跳过）。产品页路径走 part04
  getElementById 代理（有 dispatchEvent），事件链完整。恢复路径测试改用
  沙盒预置第二 canvas（part04 代理路径）。FIXME 记 DOM 流缺口，不跨流硬修
  （run-rules §9 同判据）。
- **getShaderPrecisionFormat 与编译管线解耦**：值取最低保证档，不依赖具体
  shader 对象；编译管线落地（切片②）后若真机精度档更高再评估对齐。
- DEPTH_BITS/STENCIL_BITS 0 与 depth/stencil attachment 缺席一致（FIXME(M3)）。
