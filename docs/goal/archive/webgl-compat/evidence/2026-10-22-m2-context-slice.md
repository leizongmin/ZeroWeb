# M2 上下文切片——WebGL 接口面 + runner 通道 + clear/draw 标尺

**日期**: 2026-10-22
**范围**: webgl-compat goal M2 全量切片（2026-10-22 用户四项裁决授权）：
① runner 通道（`testharness-webgl`）+ idlharness 资产补拉（裁决③）；
② 上下文对象面 + WebIDL 校验 + buffer/texture 错误语义 + getError 状态机
（批次 B 全量一次收，裁决①）；
③ clear/draw 本地 reftest 标尺（常驻 local-reftests，裁决② 标尺①）。
**基线**: 2112 Pass / 3 Fail（fail 全部为上游 IDL 重载特性，见 §4）

---

## 1. 资产与通道（裁决③ 落地）

- `99-webgl-compat.sh` 补 M2 资产块：`resources/idlharness.js`、
  `resources/webidl2/lib/webidl2.js`（落到部署路径名 `resources/WebIDLParser.js`
  ——repo 内无 build 期生成物，webpack UMD dist 同字节 125228，暴露
  `globalThis.WebIDL2`，runner META script 链零特判）、`interfaces/{webgl1,webgl2,dom}.idl`。
- runner 通道 `run_webgl_cases`（`tests/wpt-runner/src/testharness.rs`）：
  `.html` 直接作 testharness 页；`idlharness.any.js` 经 `any_js_window_wrapper`
  包窗（META scripts 磁盘解析）。**无 skip 记账排除**（idlharness 全量收，
  与 clipboard/compression 的资产缺席退路不同——本 goal 有裁决补拉）。
- `idl_test` 运行期 `fetch('/interfaces/*.idl')` 由 page fetch 静态文件回退供给。
- Makefile：`fetch-wpt-webgl` + `testharness-webgl`（test-guard 4/8/900，
  FILTER 透传）。

## 2. 上下文对象面（crates/engine/src/js_dom_shim/part08.js，新文件 1700 行）

- **接口面数据表**（脚本派生，python 解析 pin rev webgl1.idl / webgl2.idl）：
  常量 webgl1=297 + webgl2=262（零交集、值无冲突）；方法 webgl1=137 +
  webgl2=106（WebIDL minOverloadLength 口径——首 optional/default 前必参数，
  多重载取最小）。注入 part08.js（`_ZW_WGL1C/_ZW_WGL2C/_ZW_WGL1M/_ZW_WGL2M`）。
  初版生成器漏掉非 void 返回类型的 operation（getExtension 族 14 条）与
  跨重载 min 口径，重写解析后修复（idlharness operation 断言全绿即回归证据）。
- **18 个接口**（idlharness WebIDL 口径）：接口对象 {w:true,e:false,c:true}、
  基接口接口对象派生 [[Prototype]]（WebGLBuffer→WebGLObject、
  WebGLContextEvent→Event）、prototype {w:false,e:false,c:false} +
  constructor {w:true,e:false,c:false→true}、常量双落 {w:false,e:true,c:false}、
  方法 {w:true,e:true,c:true} + .length/.name、属性访问器命名 `get X`/`set X`
  （idlharness 断言面，基线首跑发现修复）、this=null/{} 品牌 TypeError、
  必参 TypeError（minOverloadLength，corpus compressedTexImage2D 缺参案）。
- **WebGL2 无基接口**（pin rev webgl2.idl：`WebGL2RenderingContext` 仅 mixin
  includes，无 `: WebGLRenderingContext` 继承）——接口对象 [[Prototype]] =
  Function.prototype、prototype = Object.prototype；webgl1 成员经合并表落
  **自有属性**（idlharness assert_own_property 面拒绝原型链继承）。
  `getContext('webgl2')` 返 null（实例面 M4 挂账），接口对象存在。
- **实例面**（`globalThis._zwMakeCtxWebGL`，part04/part05 getContext 单点对接，
  part01 `globalThis._zwGetInnerText` 发布先例）：'webgl'/'experimental-webgl'
  别名复用同一上下文；类型冲突（已持 2d → null，HTML spec canvas.getContext）；
  `drawingBufferWidth/Height` 对齐 host 位图（MAX_CANVAS_DIM 16384 截断对齐）。
- **错误语义**（getError 单旗标 sticky；WebGL §6.20 NaN 落 0 无错误）：
  - bufferSubData NaN offset → ToInteger 落 0 → NO_ERROR（corpus 机制面）；
  - 压缩纹理：格式使能集 per-context（getExtension('WEBGL_compressed_texture_s3tc'/'…_etc1')
    写入）——未使能格式 INVALID_ENUM（corpus 4 案）、使能后无 base image
    INVALID_OPERATION（corpus 1 案）、扩展对象幂等稳定；
  - texImage2D/texSubImage2D：重载分派按参数个数（6/7 source 形、9 pixels 形），
    TexImageSource union 品牌判定不触源对象 getter（corpus {width getter 抛} 案）；
  - uniformMatrix*fv：null location 静默、transpose 按 ToBoolean（不触 valueOf）；
  - 未链接 program（M2 无编译管线恒未链接）：getUniformLocation → null +
    INVALID_OPERATION、drawArrays/drawElements → INVALID_OPERATION。

## 3. clear() 光栅化（裁决② 标尺①：常驻 reftest）

- 经**既有 host 2d 上下文**位图（不另起 wgpu 实例；GPU 依托 render-foundation
  约束）：`setCompositeOperation('source-over')` + `setFillStyle(rgba)` +
  `fillRect`。不用 'copy' 合成——host path 光栅对 source-exclusive 合成附带
  uncovered 清除，会擦掉 scissor 矩形外既有像素（违反 GL clear 只写 scissor
  矩形语义，引擎单测实证后改 source-over；不透明色与替换语义逐位一致）。
- scissor：GL 底左原点 → host 顶左 `y' = h - y - sh` 换算；越界交截。
- colorMask 全 false / DEPTH 位清除：静默无错误。
- FIXME(M3 独立 GL 呈现面)：a<1 的 premultiply/替换约定差；depth/stencil
  缺席；纹理/shader/draw 真实管线。
- 常驻标尺：`reftest_data/webgl.rs` 2 例 inline（`webgl/clear-solid-zw-001`、
  `webgl/clear-scissor-zw-002`，ReftestCategory::Layout）+
  `local-reftests/webgl/` 4 文件孪生（reftest-upstream 域）。

## 4. 基线（pin 31597693）

`make testharness-webgl`（test-guard 4/8/900）：

| 用例 | subtests | 结果 |
|------|----------|------|
| webgl/idlharness.any.js | 2099 | 2096 Pass / 3 Fail（§4.1） |
| webgl/bufferSubData.html | 1 | Pass（NaN offset → NO_ERROR） |
| webgl/compressedTexImage2D.html | 4 | Pass（未使能格式 INVALID_ENUM ×2 + 缺参 TypeError ×2） |
| webgl/compressedTexSubImage2D.html | 4 | Pass（同上形态 ×4） |
| webgl/compressed-tex-sub-image-2d-without-base-image.html | 1 | Pass（s3tc 使能后无 base → INVALID_OPERATION） |
| webgl/texImage2D.html | 1 | Pass（非法 TexImageSource → TypeError，getter 不触） |
| webgl/texSubImage2D.html | 1 | Pass（同上） |
| webgl/uniformMatrixNfv.html | 3 | Pass（2/3/4 阶 null location no-throw） |
| webgl/basic-unused-context-crash.html | 1 | Pass（无断言 crash 页，跑完即过） |
| **合计** | **2115** | **2112 Pass / 3 Fail** |

M1 盘点估算（7 案 15 subtests）与实测一致（15 corpus subtests + crash 页 1）。

### 4.1 已知 Fail（3，上游 IDL 重载特性——不可本侧修复）

`WebGLRenderingContext includes WebGLRenderingContextOverloads`、
`WebGL2RenderingContext includes WebGL2RenderingContextBase`、
`WebGL2RenderingContext includes WebGL2RenderingContextOverloads` 的
"member names are unique"：pin rev IDL 在**同一 mixin 内**声明同名同参数个数的
operation 对（bufferData 3 参 ×2、texImage3D 10 参 ×3——GLsizeiptr/BufferSource、
pboOffset/TexImageSource/srcData 形态长度巧合相同）。idlharness.js
`are_duplicate_members`（L906-914）只按「同长操作=可重载」豁免——同长即判重复。
该判据与引擎实现无关，任何引擎跑此 pin 语料同判；改语料（拆分/补 optional 参数）
破坏 corpus 保真，不在本 goal 授权内。挂账 M4 收口时随上游 pin 前移观察。

## 5. 引擎常驻测试（js_dom_bridge_tests/part66.rs，5 例）

1. `test_webgl_context_creation_face`——创建/别名复用/webgl2 null/双向类型冲突/canvas 回引/drawingBuffer 尺寸；
2. `test_webgl_interface_objects_webidl_face`——23 项接口面断言（链/常量双落/.length/.name/品牌/必参 TypeError）；
3. `test_webgl_corpus_batch_b_error_semantics`——批 B 15 subtest 路径逐条（含首错 sticky/清除）；
4. `test_webgl_clear_raster_full`——clear 全幅红，Rust 侧 `canvas_context_full_rgba` 逐像素回读；
5. `test_webgl_clear_raster_scissor_mask`——scissor 左下 2×2 换算 + colorMask 关闭 + DEPTH 位（逐像素回读）。

既有断言同步（行为变更点）：`part05.rs` R2795 的 fresh-canvas
`getContext('webgl') === null` → 冲突语义断言（2d 先持 + webgl2 仍 null）；
`part14.rs` 两处注释（断言本身仍过——已持 2d 后 webgl → null 即冲突路径）。

## 6. DC-4 记账（CPU 路径）

- 本切片零 Rust 渲染改动：clear() 光栅走既有 CPU 2d 位图（zero-canvas
  pixel_buffer），无 GPU 路径分叉——GPU/CPU 双路径差异记账面在 M3 接入
  render-foundation 时展开（本切片无新增差异面）。
- 产品渲染链路（painter 桥）零改动：data-zw-canvas-ctx 复用 2d 同一桥。

## 7. reftest-upstream 翻红归因（929→930，+1 假绿转真红——非本切片缺陷）

A/B（同机同 pin，唯一变量=本切片 diff）：

| 树 | reftest-upstream | 失败清单差 |
|----|------------------|------------|
| HEAD（无本切片） | 16088 总 / **929** 失败 | — |
| HEAD+本切片 | 16088 总 / **930** 失败 | +`css/css-transforms/animation/canvas-webgl-translate-in-animation.html`（4.69%，恰 150×150=一块 canvas 像素差） |

**归因**：该上游案 test/ref 双臂都调 `getContext('webgl')` 绿清屏。pin 基线上
webgl 上下文恒 null → 双臂全空白 → **空对空假绿**。本切片给了真上下文与真内容，
暴露**既有** canvas 内容桥 × CSS transform 缺口：`<canvas>` 位图内容不随元素
transform 平移（`paint_canvas_element` 图元用未变换绝对坐标，
crates/engine/src/paint/painter/mod.rs:937——R3268/R57 桥路，先于本切片存在）。

**2d 复现（与 webgl 无关、非本切片代码路径）**：同树静态页
`<canvas>`（2d fillRect 绿）+ `transform:translate(150px)` vs `div` margin-left:150px
→ 同样 4.69% 失败；对照组全过：div+translate vs div+margin（transform 本身正确）、
canvas+webgl 绿清屏无 transform vs div（内容桥像素级 0.00%）、canvas+margin
（布局位移正确）。即：内容对、布局对、transform 对，三者合一路径缺 transform 施加。

**处置**：paint/transform 相位机（App E step 8、apply_transform_post 全场景重采样）
= rendering-compat 流活跃工作面（R5176 刚落地）——按 run-rules §9 不跨流硬修，
记入缺口清单交渲染流排期（修复判据现成：上游案翻回绿 + 2d 复现页翻绿 +
div/canvas 对照组保持全过；本切片 inline reftest 714/714 全绿不受影响）。
不回退本切片：维持 webgl null 保假绿 = 双白假绿掩盖真实渲染缺口（R3990 同判据）。
