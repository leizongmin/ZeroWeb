# M3-S2 shader 编译管线——GLSL ES 1.0 前端 + program link/uniform/attribute

**日期**: 2026-10-22
**范围**: webgl-compat goal M3 切片②（master.md 切分序②）：GLSL ES 1.0
shader 编译前端（词法/语法/语义校验 + 活跃声明提取）、shader 生命周期
（createShader 类型域校验/compileShader/shaderSource/deleteShader）、program
链接（attachShader/detachShader/linkProgram/bindAttribLocation + 双阶段齐备 +
编译前置 + 冲突检测）、反射面（getActiveAttrib/getActiveUniform/getAttribLocation
/getShaderParameter/getProgramParameter/getShaderInfoLog/getProgramInfoLog）、
uniform 加载族（uniform{1..4}{f,i}{,v} + uniformMatrix{2,3,4}fv + getUniform
+ getUniformLocation）。

**基线**: `make testharness-webgl` **2112 Pass / 3 Fail 不变**（3 Fail 同 M2
上游 IDL 同 mixin 同长重载判据，非本切片面）；引擎常驻测试 part69 新增 5 组
+ part67/68 共 9 例回归全绿（14/14）；`make test` 76/76 suites；clippy
`-D warnings` 零告警；fmt 通过；`make reftest` 714/714。

---

## 1. 实现面

### 1.1 GLSL ES 1.0 前端子集（新文件 part08b.js，573 行）

- **发布单点**：`globalThis._zwGlCompileGlsl(source, isVertex)` →
  `{ ok, infoLog, attributes, uniforms }`（type 为 GL 枚举、size 数组长度或
  1、声明序）。part08 的 compileShader **运行时**调用——无加载期耦合；include
  清单顺序 part08a → part08 → part08b。
- **词法**（GLSL ES 1.0.17 §3）：行/块注释、`.5` 前导点浮点、指数形式、
  f/F 后缀按 §3.6 拒绝、`#` 预处理指令按编译错误拒绝（FIXME 子集边界）、
  2/3 字符运算符最长匹配。数字值手工十进制解析（数组长度位使用，不依赖
  parseInt/parseFloat）。
- **语法 + 语义**（§4）：一遍式下降解析——C 风格优先级表达式、语句层
  （if/for/while/do/break/continue/return/discard/precision/const/声明）、
  作用域栈（函数参数入函数作用域不漏全局）、§7 内建变量预注入
  （gl_Position/gl_PointSize/gl_FragColor/gl_FragData/gl_FrontFacing/
  gl_PointCoord）、`gl_` 前缀用户声明拒绝（§3.7）、attribute 仅顶点着色器
  （§4.3.3）、attribute/varying/uniform 无初始化器（§4.3.3/§4.3.5）、const
  必须初始化（§4.3.2）、void 仅函数返回类型合法、main() 必须存在、用户函数
  先声明后调用（§4.2.1）、break/continue 循环内限定（§4.2.4）、void 函数
  return 不带值（§4.2.5）。
- **活跃性精确记账**：解析期收集标识符使用（`use()` 单点——字段选择/swizzle
  与类型构造器名不记），声明未引用 → 非 active（WebGL §5.14.10 优化剔除）。
- **infoLog 面**：`ERROR: 0:<line>: <msg>` GL 风格行，行号按源码计数。
- **子集边界（诚实记账）**：表达式不做类型推导（`vec4 + float` 等结构合法
  即收）；struct 显式拒绝；数组仅一维整型字面量长度、数组参数拒绝；内建
  函数按名表接受不做 arity/实参类型检查；片段着色器 float 默认精度
  （§4.5.4 默认 NONE）不强制；赋值左值合法性不校验。

### 1.2 part08.js op 面（1234 → 1590 行）

- **createShader**：type 仅 VERTEX_SHADER/FRAGMENT_SHADER 否则 INVALID_ENUM +
  null；shader 对象记录 `_zwType`。**共享机械守卫**：create*/delete*/is*
  循环改为不覆盖特化实现（`if (_zwGlOps[name]) continue`）——原循环在 ops
  表之后无条件覆写，createShader 特化版被通用版抹掉（测试驱动发现，见 §3）。
- **compileShader**：品牌 TypeError；调 `_zwGlCompileGlsl` 落
  COMPILE_STATUS/infoLog/活跃 attribute/uniform 表；编译失败不产生 GL 错误
  （§5.14.9）。
- **attachShader/detachShader**：双品牌检查；重复挂接/未挂接 detach →
  INVALID_OPERATION（GLES2 §2.10.3）。
- **linkProgram**（GLES2 §2.10.5）：恰一顶点 + 恰一片元着色器、双阶段编译
  通过、active uniform 跨阶段合并（同名必同型同长否则失败）、location 声明序
  分配（顶点在前片元独有续后，数组每元素一槽）、active attribute 先应用
  bindAttribLocation（双 active 同槽 → 失败）再自动分配（跳过占用）；链接
  失败**无 GL 错误**——LINK_STATUS false + infoLog 非空；链接使旧 location
  全失效（stamp 递增，§5.14.9 重链失效面）。
- **bindAttribLocation**：index ≥ MAX_VERTEX_ATTRIBS(16) → INVALID_VALUE、
  `gl_` 前缀 → INVALID_OPERATION、绑定于下次 link 生效。
- **反射面**：getActiveAttrib/getActiveUniform 返回 WebGLActiveInfo 实例
  （原型链接自 M3-S1 单点创建器）；数组 uniform name 面 `u[0]` + size；
  越界 index → INVALID_VALUE + null；未链接 → INVALID_OPERATION。
  getAttribLocation 未知名 → -1 无错误。getShaderParameter 增 SHADER_TYPE；
  getProgramParameter 增 ATTACHED_SHADERS/ACTIVE_UNIFORMS/ACTIVE_ATTRIBUTES。
- **getUniformLocation**：未链接 → INVALID_OPERATION + null（M2 面保持）；
  已链接 → 名表查找 + `u[N]` 数组元素寻址（基址 + N），未知名 → null 无
  错误；同 program 同名同 stamp 返回同一对象（§5.14.9 对象同一性面）。
- **uniform 加载族**（GLES2 §2.10.4 + WebGL §6.20）：location null → 静默
  （corpus uniformMatrixNfv no-throw 面保持）；品牌 → TypeError；不属当前
  program/重链失效 → INVALID_OPERATION；transpose ToBoolean（不触
  valueOf——corpus `{valueOf: throw}` 转换面）且 WebGL 1.0 仅接受 false
  （true → INVALID_VALUE）；float 值 NaN/±Inf → INVALID_VALUE（§6.20）；
  类型匹配：标量 setter 仅标量 uniform、向量 setter 按分量数、矩阵 setter
  仅同阶矩阵、采样器仅 uniform1i/1iv；容量：标量恰一元素、数组 ≤ size；
  值按 uniform 类型族存 Float32Array/Int32Array（偏移 = 槽差 × 分量——数组
  部分装载语义）。
- **getUniform**：标量返 number（BOOL → boolean）、向量/矩阵返按类型族
  typed array；未上传返规范初值全 0（§2.10.4）。
- **drawArrays/drawElements**：语义不变（无 program 或管线未落地均
  INVALID_OPERATION），FIXME 注释移交 M3-S4。

## 2. 常驻测试（js_dom_bridge_tests/part69.rs 新增 5 组，part67 沙盒复用）

1. `test_webgl_shader_compile_lifecycle`（14 断言）——createShader 类型域
   校验 + SHADER_TYPE 面 + 真编译 COMPILE_STATUS/infoLog + 五类编译失败
   （未声明标识符/片元 attribute/浮点后缀/缺 main/未编译初值）+ delete 面
   + 编译失败无 GL 错误。
2. `test_webgl_program_link_face`（18 断言）——attach 重复 INVALID_OPERATION
   + ATTACHED_SHADERS + 链接成功/缺阶段/编译前置失败（均无 GL 错误）+
   detach 未挂接 INVALID_OPERATION + detach/重挂/重链 + bindAttribLocation
   域校验（16 → INVALID_VALUE、gl_ → INVALID_OPERATION）+ 绑定生效
   （getAttribLocation === 3）+ 双 active 同槽链接失败 + 双顶点着色器失败。
3. `test_webgl_program_reflection_face`（16 断言）——ACTIVE_UNIFORMS=4/
   ACTIVE_ATTRIBUTES=2 + 声明序 + 跨阶段同名合并（u_common 单条目）+
   活跃剔除（u_dead 不可见）+ 数组 `u_lights[0]`/size=4/type 面 + 越界
   INVALID_VALUE + attrib 序 + 未知名 -1 无错误 + 未链接三查
   INVALID_OPERATION + WebGLActiveInfo 原型。
4. `test_webgl_uniform_op_face`（29 断言）——location 对象同一性 + 数组
   别名同址（`u_arr` 写 `u_arr[0]` 读一致）+ 元素位独立 + 未知名/越界 null
   + getUniform 值/初值面（mat4/vec4/float/int/bool/数组元素）+ 类型失配
   三态（标量于 vec4/分量失配/矩阵阶失配）+ 采样器 1f 拒绝 1i 收 +
   NaN INVALID_VALUE + transpose true INVALID_VALUE + 容量越界 + 错
   program + 重链 stamp 失效（旧 location 拒绝、新对象可写）+ null location
   静默（M2 corpus 面保持）。
5. `test_webgl_glsl_error_info_lines`（9 断言）——infoLog 行号对位
   （`ERROR: 0:4:`）+ 未声明/break 域/void 返回值/`gl_` 保留/重复声明/
   语法错误信息面 + 编译失败无 GL 错误。

## 3. 本轮修正的实现缺陷（测试驱动发现）

| 缺陷 | 根因 | 修复 |
|------|------|------|
| 5 组新测试首轮全红 | create*/delete*/is* 共享机械循环在 ops 表后**无条件覆写** createShader，特化版（类型域校验 + `_zwType` 记录）被通用版抹掉 → shader 无 `_zwType` 按片元编译 → attribute 拒绝 → link 恒失败 | 循环加 `if (_zwGlOps[name]) continue` 守卫（特化优先） |
| 合法顶点着色器按片元报错 | 同上根因链 | 同上 |
| part08b 初稿 6 处语义缺陷 | `vec2` 等类型名未入保留字表（词法归类 id → 类型位识别失败）；全局 attribute/uniform/varying 未注入作用域（main 内引用报未声明）；局部 `const` 声明不可解析；函数参数泄漏全局作用域；`gl_Position` 等内建变量缺失；`.5` 前导点不可词法化 | 全部修正 + node 冒烟 19 案先行收敛（含 kw 类型构造器分支补齐） |
| `getUniform` 精确比较 9.9 失败 | uniform 存储为 Float32Array——9.9 回读 9.899999618530273（真机 GL 同为 32-bit float 存储，行为一致） | 测试改用 Float32 可精确表示值（9.5）；非实现缺陷 |

## 4. 已知边界（记账）

- **表达式无类型推导**：`vec4 + float` 等真实 GLSL 的类型错误不拒绝
  （part69 测试着色器即依赖此宽松面——结构校验为主，FIXME(管线需要时)）。
- **预处理不展开**：`#` 指令一律编译错误（真机 `#define`/`#version` 合法
  ——FIXME 子集缺口，语料着色器含预处理时按 compile false 落 infoLog）。
- **片段 float 默认精度不强制**（§4.5.4 默认 NONE 应报错——宽松接受）。
- **validateProgram 未实现真语义**：VALIDATE_STATUS 恒 false（FIXME(M3-S4)
  随 draw 簇）；ACTIVE_UNIFORM_MAX_LENGTH/ACTIVE_ATTRIBUTE_MAX_LENGTH
  pname 未列（落 INVALID_ENUM）。
- **drawArrays/drawElements** 在已链接 program 下仍 INVALID_OPERATION
  （真实绘制管线属 M3-S4）。
- 浮点 uniform 经 Float32Array 存储（32-bit 精度）——与真机 GL 一致，
  JS 侧精确十进制比较须用 Float32 可表示值（part69 已按此写断言）。
