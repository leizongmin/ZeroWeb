# M3-S4 draw 光栅化管线证据（2026-10-23）

goal：[docs/goal/webgl-compat.md](../../webgl-compat.md) ④ draw 命令族 + GLSL 求值 + 光栅化输出。
本切片把 M2 的"draw 恒 INVALID_OPERATION 桩"升级为完整 CPU 光栅化管线：drawArrays/
drawElements 校验序（WebGL 1.0 §6.24 Range Checking）→ part08b GLSL ES 1.0 前端校验通过
的程序经 part08d 递归下降求值器逐顶点执行 → 图元装配 → 重心光栅（像素中心采样、双环绕、
透视校正 varying 插值）→ scissor/colorMask → host 2d 位图 putImageData 呈现。

## 交付物

- `crates/engine/src/js_dom_shim/part08d.js`（新增，~1390 行）：GLSL 绘制求值器 + 光栅化后端。
  - 消费 part08b 的 `_zwGlslShared` 词法/类型表交接（用后即清），发布
    `_zwGlDrawBackend = { drawArrays, drawElements }` 供 part08 draw op 调用期查表。
  - 求值器：递归下降 parser（语句 block/decl/if/for/return/discard/break/continue/
    表达式；赋值 =、+=、-=、*=、/=；前后缀 ++/--；三元；swizzle 读写（写通道互异
    GLSL ES 1.0 §5.5）；索引读写；构造器；内建函数表 radians…texture2D（texture2D
    采样返回黑——FIXME 纹理采样））。步数预算 4e6（`_ZW_STEP_BUDGET`）+ for 循环
    65536 次上限防失控；求值异常 → 本次 draw 静默跳过（spec 禁止对合法 GLSL 报错）。
  - 值模型：标量 number（bool 0/1）、vec{k,c[]}、mat{k,m[][]} 列主序、uniform 数组
    {__arr,els[]}；类型 k 复用 part08b GLSL 类型枚举。
  - 顶点取数：DataView 小端按 `_ZW_VATTR_TYPE_SIZES`（BYTE/UNSIGNED_BYTE/SHORT/
    UNSIGNED_SHORT/FIXED/FLOAT，GLES2 §2.8 type 域——INT/UINT 不在）读，normalized
    定点归一化，FIXED /65536；禁用属性取 current 值；声明类型与 fetch vec4 失配经
    `_zwCoerceAttrib` 收敛（mat/sampler → 中止 draw）。
  - 光栅：viewport 变换 `x=(ndc*0.5+0.5)*vp[2]`、host `y=H−yGL`；图元装配 7 模式
    （POINTS/LINES/LINE_LOOP/LINE_STRIP/TRIANGLES/TRIANGLE_STRIP/TRIANGLE_FAN）；
    w≤0 整图元丢弃（FIXME 完整裁剪）；像素中心 edge-function 重心 + 透视校正
    `Σ(a_i·λ_i/w_i)/Σ(λ_i/w_i)`；scissor（host rect [s0, H−s1−s3, s2, s3]）+
    colorMask 逐通道；写入为替换（默认 blend ONE/ZERO 等价——FIXME 其他因子）。
  - FS 面内建变量：gl_FragColor/gl_FragCoord/gl_PointCoord，输出 clamp 0..1。
  - FS 已声明但 VS 未写的 varying → 零填充。
- `crates/engine/src/js_dom_shim/part08.js`：drawArrays/drawElements 桩换真校验序 + 后端
  委派；vertexAttrib 操作族（enable/disable/pointer 全校验链/1f-4f/*v current 值/
  getVertexAttrib 7 pname）；readPixels op（域语义见下）；`_zwLinkProgram` 重链时
  失效 `_zwDrawProg` AST 缓存。
- `crates/engine/src/js_dom_shim/part08b.js`：尾部发布 `_zwGlslShared` 交接包。
- `crates/engine/src/js_dom_bridge.rs`：注册 part08d include（必须后于 part08b）。

## 校验序（spec 逐条核自 KhronosGroup/WebGL specs/latest/1.0 源）

drawArrays：mode∉7 模式 → INVALID_ENUM；first<0/count<0 → INVALID_VALUE；程序空/
未链接 → INVALID_OPERATION；enabled+被消费属性数组越顶点范围 → INVALID_OPERATION
（robust 取次 1，§6.24 Range Checking）；enabled 但未被程序消费 → 恒不报错。
drawElements：type 仅 UNSIGNED_BYTE/UNSIGNED_SHORT（UNSIGNED_INT 需
OES_element_index_uint，未启用 → INVALID_ENUM）；count/offset<0 → INVALID_VALUE；
offset 非索引类型大小倍数 → INVALID_OPERATION；count>0 需 ELEMENT_ARRAY_BUFFER
绑定；索引取数越界 → INVALID_OPERATION。
vertexAttribPointer：index≥MAX_VERTEX_ATTRIBS(16)/size∉1..4/offset<0/stride<0/
stride>255 → INVALID_VALUE；type∉GLES2 §2.8 域 → INVALID_ENUM；offset/stride 非
类型大小倍数 → INVALID_OPERATION；无 ARRAY_BUFFER 绑定且 offset≠0 →
INVALID_OPERATION（offset 0 合法）。
readPixels：仅 RGBA/UNSIGNED_BYTE → 否则 INVALID_OPERATION；w/h<0 或 pixels null →
INVALID_VALUE；容量不足/视图类型失配/FBO 绑定（FIXME render-to-texture）→
INVALID_OPERATION；区域越界对应目标区间不动、无错误（§6.16）；行序 GL 底上 →
host 顶上翻转。

## 验证

- 冒烟（node 直跑 5 part 链，/tmp 脚本 69 断言）：像素精确值手算 λ 对照（含缩放、
  mat2、discard、for 循环 i++、drawElements 四边形、readPixels 行翻转/负 x 越界）、
  全部校验序、求值器子集外静默跳过。69/69。
- 常驻测试 `js_dom_bridge_tests/part71.rs`（7 组）：drawArrays/drawElements/
  vertexAttribPointer/readPixels 校验序面 + 真位图像素断言（Rust 侧
  `canvas_context_full_rgba` 回读：三角重心插值 [48,64,0,255]、缩放 [191,42,0,255]、
  四边形 [143,191,0,255]、discard 帧前后逐字节一致、retained buffer 保留、FS for
  循环 i++ alpha=96）。`cargo test -p zero-engine --lib webgl` 24/24。
- testharness：`make testharness-webgl` 基线保持 2112 Pass / 3 Fail（3 Fail 同上游
  IDL 同 mixin 同长重载判据，未新增失败）。
- clippy `-p zero-engine --all-targets -D warnings` 干净；cargo fmt 无差异。
- reftest 标尺：内联孪生 webgl/draw-quad-solid-zw-003（drawElements 全幅四边形实心
  蓝）与 webgl/draw-scissor-zw-004（全屏三角 + scissor 底左 50×25 红），文件孪生
  tests/wpt-runner/local-reftests/webgl/ 同名。`make reftest` 716/716。

## 已知限制（FIXME 汇总于 part08d 头注）

- w≤0 整图元丢弃，无逐像素裁剪；blend 仅默认 ONE/ZERO；纹理采样返回黑；POINTS
  1×1 无 gl_PointSize 面积；LINE 用参数化近似无线宽；无 DITHER/STENCIL/DEPTH 测试
  写入；preserveDrawingBuffer 语义面未建。
