# M1 导入与盘点 — webgl/ corpus（2026-10-22）

## 导入方式

- `bash tests/wpt-runner/scripts/goals/99-webgl-compat.sh`（WPT_REV `315976933`
  pin，与全仓各 goal 同 pin）；本轮在脚本内补 `GOAL_PULL_ANY_JS=1`
  （idlharness.any.js 为 WebGL 接口面唯一 .any.js 形态，lib.sh opt-in 约定）。
- 语料落 `tests/wpt-runner/wpt-data/webgl/`（gitignored），平铺 10 文件：
  8 `.html` 用例 + `common.js`（helper）+ `idlharness.any.js`；2 个 `.yml`
  元数据在扩展名 filter 外不拉。
- 上游体量实测：git trees API 列 `webgl/` 子树（pin rev）＝ **12 blob、无子目录**
  （8 html + common.js + idlharness.any.js + META.yml + WEB_FEATURES.yml）。

### 立项基线假设修正

立项时假设「Khronos conformance 移植、体量大、dehydrated 形态 + 渲染断言」，
**对 pin rev 不成立**：WPT 仓的 `webgl/` 从未 vendor Khronos 全量一致性套件
（套件在 KhronosGroup/WebGL 仓独立发布），pin 处只有少量聚焦回归用例。
dehydrated（deqp）形态 **0 案**，渲染断言（像素/draw 比对）**0 案**。

## 用例分布盘点

| 形态 | 数量 | 用例 |
|---|---|---|
| testharness 功能语义（错误/异常断言） | 7 | bufferSubData、compressedTexImage2D、compressedTexSubImage2D、compressed-tex-sub-image-2d-without-base-image、texImage2D、texSubImage2D、uniformMatrixNfv |
| manual crash 回归页（无 harness 断言，通过判据＝不崩） | 1 | basic-unused-context-crash（servo#41082 回归） |
| idlharness（接口面） | 1 | idlharness.any.js（webgl1 + webgl2 两 idl） |
| helper | 1 | common.js（`getGl()` 用 **`experimental-webgl`** 别名） |

子测试静态估算（逐 `test()`/`shouldGenerateGLError` 计数，非实测；
实测分母以 runner 通道首跑为准）：7 页合计 **15 subtests**
（bufferSubData 1、compressedTexImage2D 4、compressedTexSubImage2D 4、
compressed-tex-sub-image-2d-without-base-image 1、texImage2D 1、
texSubImage2D 1、uniformMatrixNfv 3）。

### 依赖面甄别（上下文创建之外各案所需）

| 面 | 用例 | 说明 |
|---|---|---|
| 上下文创建（`webgl` + `experimental-webgl` 别名） | 全部 8 html | engine 现仅 2d 上下文（js_dom_shim part01/part04），webgl 请求面缺失 |
| WebGLRenderingContext 常量（NO_ERROR/INVALID_ENUM/INVALID_OPERATION） | 4 | 校验语义断言的判据 |
| getError | 3 | 错误状态机 |
| buffer 对象（createBuffer/bindBuffer/bufferData/bufferSubData） | 1 | NaN offset 不产生错误 |
| texture 对象 + compressed{,Sub}TexImage2D 校验语义 | 2 | 断言 INVALID_ENUM——**不需扩展真实存在** |
| getExtension（s3tc / etc1） | 1 | 条件依赖：两扩展都缺 → 该案 Fail（归因记账） |
| texImage2D/texSubImage2D WebIDL TypeError 转换 | 2 | 方法存在即可断言（invalid source 抛 TypeError） |
| program/uniform 面（createProgram/getUniformLocation/uniformMatrix{2,3,4}fv） | 1 | 空 program（无 shader）；transpose 为 boolean 型 → ToBoolean 不调 valueOf（不抛） |
| idl 资产（`/interfaces/webgl1.idl`、`webgl2.idl`、`resources/idlharness.js`、`WebIDLParser.js`） | 1 | 均不在仓——runner 通道时按 `idlharness.*` skip 先例（clipboard/timing/net-api/compression）记账排除，或届时补拉资产（M2 裁决） |

## 首批收敛子集建议书（提交用户，供 M2 门控裁决）

分母极小（7 案 15 subtests），「收敛子集」实为**批次划分**问题。两个选项：

- **选项 A（推荐）：逐批收敛**——
  - 批 1「上下文创建 + WebIDL 校验闭环」（4 案 5 subtests）：
    basic-unused-context-crash、texImage2D、texSubImage2D、uniformMatrixNfv。
    只需上下文对象面 + 方法存在 + 参数转换语义，GL 内部可 stub，
    是上下文创建语义的最小验收闭环。
  - 批 2「buffer/texture 错误语义」（3 案 9 subtests）：
    bufferSubData、compressedTexImage2D、compressedTexSubImage2D。
    需 buffer/texture 对象模型 + getError 状态机 + INVALID_ENUM/NO_ERROR 语义。
  - 批 3「扩展条件案」（1 案 1 subtest）：
    compressed-tex-sub-image-2d-without-base-image。依赖压缩纹理扩展暴露，
    建议挂账至扩展决策（先 FAIL 归因记账）。
- **选项 B：全量一次收**——分母小，批 1+2+3 一个切片做完亦可；
  差别只在切片评审粒度。

**M2 附带项**（随所选批次落地）：runner 通道（`testharness_webgl` +
Makefile target + `idlharness.*` skip 规则，test-guard 包裹先例同 compression）+
基线首跑（M1 不跑基线：上下文缺失时全部 Fail，无信息量）。

### 标尺缺口（需用户裁决）

Mission 写「最小上下文（创建/clear/draw 基础状态机）」，但 **pin 处 corpus
没有 clear/draw 断言**（0 渲染断言）。clear/draw 的验收标尺两条路：
① 本地 reftest 补标尺（rendering-compat 通道先例，常驻 `local-reftests`）；
② 接受 corpus 覆盖面（创建 + 错误语义）即 M2 验收面，clear/draw 挂账至
M3 纹理/shader 切片时再定。建议 ②，理由：与 corpus 验收标准原则一致。

## suites CSV 回填

`docs/compat/trends/wpt-suites.csv` webgl 行维持 `status=planned`
（无 runner 通道、无基线实测，total/passed 按规则 2 留空），note 更新为
盘点结论并指向本文件；evidence 列改指本文件。
