# DC-2 切片 5 刀③③b 证据：Function 构造器编译门（2026-11-23）

## 机制（双后端宿主跳板——V8 主通道教训吸收）

③③b 首轮把 `__zwTtMakeCompileTrampoline` 工厂只落在 quickjs_runtime.rs，探针
6F 全灭——**engine 默认 feature = `v8`，runner/lib 测试全走 V8Sandbox 主通道**，
QuickJS-only 宿主桥在主通道静默失效（已沉淀 memory + learnings TODO）。终版双
后端各落同契约工厂：

- **V8**（v8_runtime.rs）：`tt_trampoline_factory_invoke`（每 idx 建
  FunctionTemplate，`.data(idx)` 携键，可构造位由 FunctionTemplate 天然获得）
  + `tt_trampoline_invoke`（data 取 idx → global `__zwTtTrampReg[idx]` 取
  {orig,gate,sink} → 组参 Array → `gate.call(orig, [orig, args, sink])`，
  None＝异常挂起留 rv 不设即传播）。execute_with_catch 装配尾**无条件**安装
  （工厂缺位安全：part14 安装臂 typeof 守卫跳过）。
- **QuickJS**（quickjs_runtime.rs）：`install_tt_compile_trampoline`
  （`Function::new` + `.with_constructor(true)`〔JS_SetConstructorBit，
  infallible〕；闭包 `Ctx` + `Rest<Value>` 形态保 TT 实例品牌不提前
  ToString）+ `tt_trampoline_call`（registry 查表 + `Array::set` 组参 +
  `Error::throw` 传播）。install_callbacks 尾调用。
- **registry 模式**：rquickjs 0.7 无持久句柄，宿主闭包不持 JS 值——
  orig/gate/sink 存 JS 侧 `__zwTtTrampReg[idx]`，跳板逐调用经 ctx 取。

## 交付面（part14 §4 重写 + 三件运行期 reroute）

`_zwTtCompileGate(orig, args, sinkName)`（spec EnsureCSPDoesNotBlockString-
Compilation + GetTrustedTypeCompliantString）：

- 非 enforced 原参透传；TT 实例经 `__zwTtData` 非枚举 own 属性直读数据串
  （**零 toString 调用**——forged-toString 断言 ground truth），其余
  String 化；全 TT → 原参照常编译。
- **装配文本恒全形 V8 制**（`function anonymous(<params>\n) {\n<body>\n}`，
  n=0 空形）：ground truth 藏在 default-policy-callback-arguments 页——
  1 参（仅 body）时 policy 收到的也是全形；基线该页 P 是「门缺席空洞通过」
  （断言根本没执行），装门后才真验（现 5P/0F 真过）。
- default policy 管道：无政策/回调抛错/返 undefined → EvalError；**输出 ≠
  装配文本 → EvalError**（mutation 规则，default policy 不可改写编译串——
  mutate/Window-block/applying 页 ground truth）；恒等 → 原参编译。
- **编译域错误恒 EvalError**（属性/HTML sink 域 TypeError 管道不适用）；
  四构造器 sink 名恒 `'Function'`（support/block-eval-function-constructor.js
  断言）。

`_zwTtWrapCtor`（idx 键控注册 + 跳板实例）+ `_installCompileSinks`：wrap
globalThis.Function（descriptor 保持）+ `Function.prototype.constructor` 与
三派生构造器（async/generator/async-generator 的 %X.prototype%.constructor）
重定义；identity 面闭合：wrapper.name/length 照抄、wrapper.prototype =
orig.prototype（instanceof 恒真）。

**17 处 shim 运行期编译站点 reroute `new __zwNativeFunction`**（part01×5、
part03×4、part05×8）：`__zwNativeFunction` 是 part01 顶部装配期原生引用快照
（先于 part14 安装）——脚本本体照常执行；**s90 定时器串不再二次进门**（已在
setTimeout/setInterval 入口过 ③③a 管道一次，再进 Function 门会对同一串二调
default policy，sink 名断言二现崩）。SVG/接口工厂等装配期站点无需 reroute
（安装晚于装配）。

**eval 域整体挂账**（头部注记）：直接 eval 是编译器特判拦不到；替换
globalThis.eval 破坏所有页面直接 eval 的局部作用域语义（dom 语料 Range 家族
18 文件实证）；间接 eval 单独拦截不可行（同 binding）。

## 探针（zz-knife33b-*.html 两件，临时件已删不入库）

- enforced 14 测全绿：工厂安装、undefined 臂 EvalError、TT/混合/全 TT 编译、
  forged toString 零调用、mutation EvalError + sink='Function' +
  type='TrustedScript'、恒等输出编译、三派生构造器（async 值异步录、
  generator 同步 .next()）、wrapper identity（instanceof/constructor/
  name/length/Function.prototype.constructor）、页本体照常、string timer
  单管道（③③a 入口一次 + s90 原生编译）。
- 非 enforced 6 测全绿：plain/eval/派生构造器/调形语义全原生。

## 语料对账

- **TT 语料全量：1420P/665F/64T → 1507P/578F/64T（净 +87P/-87F，T 恒等）**。
  剔 zz 探针行后逐行 comm（五字段键、双侧 2149 行对账）零未知漂移：
  **88 F→P 真翻绿 + 1 P→F（p11 探针时序 flake，见下）**。翻绿逐文件：
  eval-function-constructor 64（masks/forged 主面）、applying 8、
  no-default-policy 6、mutate 4、Window-block-eval 4、throwing 1、
  **eval-with-permissive-csp 1（计划外收益：`Function constructor` 子测
  F→P，solo 复跑 5P/4F 确认真过，余 4F 全为 eval 挂账域）**。
- default-policy-callback-arguments：5P/0F（空洞过转真过，行面无翻转）。
- **san 整套件恒等：323P/328F/1T ＝ ③③a 记录值精确恒等** ✓（F 面构成
  sethtml-with-trustedtypes 129F 大头＝knife2 后形态，与编译门零交涉）。
- ③b① 哨兵页（set-attributes…default-policy / no-default-policy 264P×2、
  no-require 202P）：comm 双侧零行出现＝持位实证 ✓。
- zero-engine lib unittests **4252P/0F** ＝ ③③a 记录值精确恒等 ✓（归因/
  worker/iframe 面零扰）。

## 探针 p11 flake 修复（常驻件 zz-knife33-probe.html）

全量跑单点回落 `p11-interval-tt-instance-fires` P→F：10ms 间隔 × 60ms 窗
需 ~50ms 内 2 tick，全量语料态裕量不足（单文件 3/3 过）。窗口 60→200ms，
断言语义不变（≥2 tick）；修后 solo 复跑 10P/0F。机制核清：timer 串走 s90
原生引用不进新门，本刀与该路径零交涉。

## 门禁

- fmt `--all -- --check` RC=0；clippy `-p zero-script-sandbox --all-targets
  -D warnings`（test-guard 包裹 --per-proc-mem 6 --total-mem 12）RC=0。
- 引擎编译兼容由 lib 4252P/0F 编译面覆盖（本刀仅新增方法，无既有签名变化）。

## 挂账（顺延）

- eval 域（直接/间接）——见头部注记，随引擎宿主桥臂独立评估。
- `.headers` sidecar 形态 enforcement（17 页 runner 无 CSP 头通道）——
  runner 预处理微刀（刀③④ 位）。
- worker 影子域 script sink；report-only violation 事件面
  （SecurityPolicyViolationEvent，reporting 页族 TO 预存）。
