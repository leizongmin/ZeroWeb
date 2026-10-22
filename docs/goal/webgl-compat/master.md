# WebGL 兼容 — 运行时控制面板（master.md）

**入口文档**: [../webgl-compat.md](../webgl-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-22（M3-S3 对象真存储落地——基线保持 2112/2115）

## 当前状态

**M3-S3 对象真存储已落地（2026-10-22）**：part08.js（+270/−59）buffer 真字节
存储（bufferData 三形/重分配弃旧/无 store INVALID_OPERATION/getBufferParameter）
+ 纹理参数面（texParameteri/f 精确匹配 + getTexParameter Table 6.9 初始值 +
跨 target 改绑拒绝）+ texImage2D/texSubImage2D 校验链与 level 元数据/像素
存储（Table 3.2 组合 + internalformat≡format + 容量域 + 区域字节写）+
framebuffer/renderbuffer 附件骨架（completeness 四态 + 附件参数四 pname +
删除解挂当前绑定 FBO）。**M2 usage 枚举表缺陷修正**（幻值 0x88E3/漏
DYNAMIC_DRAW——详见 evidence §3）。基线 **2112 Pass / 3 Fail 不变**；part70
常驻 3 组 74 断言 + part66-69 回归 14 例全绿；`make test` exit 0、clippy/fmt
零告警、`make reftest` 714/714。详见
[evidence/2026-10-22-m3-s3-object-storage.md](evidence/2026-10-22-m3-s3-object-storage.md)。

**M3-S2 shader 编译管线已落地（2026-10-22）**：GLSL ES 1.0 前端子集
（新 part08b.js 573 行——词法/语法/语义校验 + §7 内建变量 + `gl_` 保留 +
活跃声明精确记账 + infoLog 行号面；发布单点 `globalThis._zwGlCompileGlsl`，
compileShader 运行时调用无加载期耦合）+ part08.js op 面（+356 行）：
createShader 类型域校验（共享机械循环改特化优先守卫）、attach/detach
INVALID_OPERATION 域、linkProgram（双阶段齐备 + 编译前置 + active uniform
跨阶段合并 + location 声明序分配 + bindAttribLocation 融合/冲突检测 + 重链
stamp 失效）、反射面（getActiveAttrib/getActiveUniform WebGLActiveInfo +
数组 `u[0]` 命名 + ACTIVE_* 计数）、uniform 加载族全量（16 setter +
getUniformLocation 对象同一性 + getUniform 值/初值面 + 类型失配/采样器
1i-only/NaN/transpose false-only/容量域校验）。基线 **2112 Pass / 3 Fail
不变**（3 Fail 同 M2 上游 IDL 判据）；引擎常驻测试 part69 新增 5 组
（86 断言）+ part67/68 回归 9 例全绿；`make test` 76/76、clippy/fmt 零告警、
`make reftest` 714/714。详见
[evidence/2026-10-22-m3-s2-shader-pipeline.md](evidence/2026-10-22-m3-s2-shader-pipeline.md)。

**M3-S1 查询与事件面（2026-10-22）**：getParameter 全表
（字符串身份/位面/最小上限/状态初值/压缩格式表/WebGL 专有像素参数）+
getContextAttributes + enable/disable/isEnabled cap 校验 + 状态设置族
（域校验/clamp/NaN 语义）+ getShaderPrecisionFormat（GLES2 §6.2 最低档）+
WEBGL_lose_context（可取消 webglcontextlost → preventDefault 抑制 →
restoreContext 链 + 丢失态方法旁路 + CONTEXT_LOST_WEBGL 首查）+
**GL 对象实例原型链接**（共享路径修复——_zwGlNewObject 单点接接口原型，
buf.label 等属性访问器修通）。基线 **2112 Pass / 3 Fail 不变**（3 Fail 同
M2 上游 IDL 判据）；引擎常驻测试 part68 新增 4 例 + part67 M2 5 例回归全绿；
clippy/fmt 零告警。详见
[evidence/2026-10-22-m3-s1-query-events.md](evidence/2026-10-22-m3-s1-query-events.md)。

**M2 全量切片（2026-10-22 已收）**：runner 通道 `make testharness-webgl`
（idlharness 资产补拉，裁决③）+ 上下文对象面（engine js_dom_shim/part08.js
——18 接口 WebIDL 断言面对齐 + getContext 'webgl' 实例 + getError 状态机 +
buffer/texture 错误语义）+ clear() 光栅化（经既有 host 2d 位图，scissor/
colorMask 生效）+ 本地 reftest 标尺（inline 2 例 + local-reftests 孪生 4 文件，
裁决② 标尺①）；引擎常驻 part66 5 例（后续因远端 R4600 撞名改 part67）。
详见 [evidence/2026-10-22-m2-context-slice.md](evidence/2026-10-22-m2-context-slice.md)。

**跨流发现（本轮 A/B 归因）**：reftest-upstream 929→930（+1）——上游案
canvas-webgl-translate-in-animation 由空对空假绿转真红，暴露**既有**
canvas 内容桥 × CSS transform 缺口（2d canvas 同样复现，非 webgl 面）；
按 run-rules §9 记入缺口清单 P6 交渲染流，不跨流硬修。归因与复现组见
M2 evidence §7。

**下一里程碑 M3（授权内自主）**：切片边界自划——候选簇按依赖序：
① ~~getContextAttributes/getParameter 参数查询补全 + 上下文丢失/恢复事件面~~
   （✅ 2026-10-22）；
② ~~GLSL ES 1.0 shader 编译管线 + program link/uniform/attribute~~
   （✅ 2026-10-22）；
③ ~~buffer/texture/framebuffer 对象真存储 + texImage2D 上传链路~~
   （✅ 2026-10-22）；
④ drawArrays/drawElements 真光栅化（开工前置 ✅ 2026-10-22：part08 已按职责
   拆分——GL 对象存储簇独立 part08c（592 行，_zwGlStorageHandoff 单点交接），
   part08 留状态机/查询/program 簇 1443 行；基线 2112/3 与 reftest 714/714
   不变）。
每簇落账 + 常驻测试 + 飞书通知，DC-4 门禁不放松。

**历史**：M1 导入盘点（2026-10-22）——pin rev `webgl/` 为 10 文件平铺目录；
用户四项裁决（2026-10-22 当面）——门控解除全自主至 DONE（批次 B 全量一次收 /
标尺 ①+② / idlharness 补拉 / DC-3 边界自划），原文见
[evidence/2026-10-22-m1-import-inventory.md](evidence/2026-10-22-m1-import-inventory.md)
与 git 历史版本本文件。Rally park→重启背景见 git 历史（2026-10-22
门控等待态契约 BLOCK 化）。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | webgl/ corpus 导入 + 用例分布盘点 | ✅ 2026-10-22（M1） |
| P2 | 首批收敛子集建议书提交用户 | ✅ 2026-10-22（裁决：批次 B） |
| P3 | WebGL 1.0 最小上下文（创建/clear/draw） | ✅ 2026-10-22（M2，基线 2112/2115） |
| P4 | 缓冲/纹理/shader 程序管线 | ▶ M3（S1 查询/事件面 ✅、S2 shader 管线 ✅、S3 对象真存储 ✅ 2026-10-22；余：draw 光栅） |
| P5 | WebGL2/WebGPU 挂账定稿 | ▶ M4（挂账文档交用户签收） |
| P6 | canvas 内容桥 × CSS transform（位图不随元素 transform，2d 同复现） | 移交渲染流（2026-10-22 A/B 归因，M2 evidence §7；非本 goal 面） |

## 已完成切片

- **M1 导入盘点**（2026-10-22）：`99-webgl-compat.sh` 补 `GOAL_PULL_ANY_JS=1`
  后拉齐语料（wpt-data/webgl/ 平铺 10 文件）；盘点 + 建议书 +
  suites CSV note 回填。evidence：
  [2026-10-22-m1-import-inventory.md](evidence/2026-10-22-m1-import-inventory.md)。
- **M2 全量切片**（2026-10-22）：通道（testharness-webgl + fetch 脚本
  idlharness 资产块）+ 接口面（part08.js 18 接口 + 559 常量/220 方法数据表，
  idlharness WebIDL 口径）+ 实例面（part04/05 getContext 单点对接，类型冲突
  语义）+ 错误语义（批 B 15 subtests 全过）+ clear() 光栅化 + reftest 标尺
  （inline `webgl/clear-{solid-zw-001,scissor-zw-002}` + local-reftests 孪生）。
  基线 2112/2115（3 Fail 上游 IDL 重载判据，M4 观察）；part66 常驻 5 例；
  make reftest 714/714。evidence：
  [2026-10-22-m2-context-slice.md](evidence/2026-10-22-m2-context-slice.md)。
- **M3-S1 查询与事件面**（2026-10-22）：getParameter 全表 + getContextAttributes +
  cap/状态设置族域校验 + getShaderPrecisionFormat + WEBGL_lose_context
  （可取消丢失事件/抑制恢复/丢失态旁路）+ GL 对象实例原型链接（共享路径修复）。
  基线 2112/2115 不变；part68 常驻 4 例（part67 沙盒加第二 canvas）。evidence：
  [2026-10-22-m3-s1-query-events.md](evidence/2026-10-22-m3-s1-query-events.md)。
- **M3-S2 shader 编译管线**（2026-10-22）：GLSL ES 1.0 前端子集（part08b.js
  发布单点 `_zwGlCompileGlsl`）+ compileShader 真编译 + linkProgram（双阶段/
  合并/冲突/stamp 失效）+ 反射面（ActiveInfo/数组命名/ACTIVE_* 计数）+
  uniform 加载族 16 setter + getUniformLocation/getUniform。
  基线 2112/2115 不变；part69 常驻 5 组 86 断言。evidence：
  [2026-10-22-m3-s2-shader-pipeline.md](evidence/2026-10-22-m3-s2-shader-pipeline.md)。
- **M3-S3 对象真存储**（2026-10-22）：buffer 字节存储三形 + 纹理参数/
  上传校验链（Table 3.2 组合 + internalformat≡format + 区域写）+
  framebuffer/renderbuffer 附件骨架（completeness 四态 + 删除解挂）。
  M2 usage 表缺陷修正；基线 2112/2115 不变；part70 常驻 3 组 74 断言。
  evidence：[2026-10-22-m3-s3-object-storage.md](evidence/2026-10-22-m3-s3-object-storage.md)。

## 下一步计划

1. **M3 逐簇（自主）**：按当前状态节切分序推进，逐簇 evidence + 常驻测试 +
   suites CSV 回填 + 飞书通知；性能关键路径变更跑 bench-gate（规则 12）。
2. **M4 收口（自主）**：WebGL2/WebGPU 挂账定稿文档 → 交用户签收。

**已裁决清单（2026-10-22 用户当面敲定）**：
- [x] M2 启动授权 + 批次 → 批次 B 全量一次收（已完成，见 M2 evidence）；
- [x] clear/draw 验收标尺 → ①+② 都要（已完成：本地 reftest 常驻 + corpus 通道）；
- [x] idlharness 资产 → 补拉（已完成：fetch 脚本资产块）；
- [x] 自主授权范围 → 至 DONE 全自主（DC-3 边界 Agent 自划，M4 挂账交签收）。
