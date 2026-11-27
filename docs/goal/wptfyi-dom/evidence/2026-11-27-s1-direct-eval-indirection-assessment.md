# S1 独立评估——enforced 页直接 eval 间接化：可立项，形状＝arg-transform 轻刀（非 parser 级工程）

日期：2026-11-27 · 触发：控制面下一步①「S1 余 1F 直接 eval 间接化族需 parser/管线级直接 eval 保形工程，独立评估后再立项」· 本文档为该立项评估结论。

## 目标面与根因链

目标面（TT 池散布 8F 之一，eval 家族唯一红面）：

| 页 | 面 |
|---|---|
| trusted-types-reporting-for-Window-eval | `No violation reported for eval with TrustedScript.`（同页 plain-string violation 面已绿） |

判据体：`eval(policy.createScript('beacon="i ran"'))` 于 async_test 函数体内断言**函数局部** `beacon` 被赋值（`assert_equals(beacon, 'i ran')`）＝要求真·直接 eval 语义（调用方作用域链）。

根因（现行门形，刀㉙ s10k29，part14-trusted-sinks.js:695 `_installEvalGate`）：enforced 页装配期**持久替换 `globalThis.eval`** 为包装函数（TT enforce face＋CSP face＋RO face 三面收敛），native 尾调经 `__zwNativeEval` 快照。页内裸 `eval(...)` 解析到包装函数即退化为普通调用→native 尾调间接化→**声明落点/闭包链丢失**（注释 687-692 行自证此边界并挂 FIXME；「WPT Range 框架 strict 直接 eval 闭包取 paras」即早期全装版被推翻的实证）。阻断/上报面（String→violation+EvalError）不受作用域影响故已绿，透传面（TrustedScript→执行）踩作用域丢失故红。

## 机制论证：保形无需解析器

直接性（direct eval）由**语法 callee 是否为 `eval` 标识符**决定（ECMA-262 PerformEval 入口判定），与 callee 绑定值无关。由此：

- 任何「替换 eval 绑定」形（包装函数/local shadow/Proxy）都必然破坏直接性——包装内调 native eval 永远相对包装作用域＝间接。realm 技巧同死路。
- **arg-transform 形**：页源码 `eval(expr)` → `eval(__zwTtEvalArg(expr))`——语法 callee 保持 `eval`（原生绑定不动），门作**实参变换器**：非串透传（PerformEval 非串保恒语义）、TrustedScript→data 串（WebIDL 边界转换，现门 face ② 同口径）、String→TT enforce/RO faces→CSP face→返串。作用域链零损伤，阻断/上报面在 native eval 进入前抛/派发，语义与现门逐面同形。

结论：控制面预估的「parser/管线级保形工程」高估——机制本体是实参变换技巧＋调用点检测，不需要 JS 解析器。parser 仅用于升级调用点检测的守卫集（对任意内容的正确性），属可选深化非立项前提。

## 现行管线承载力（实证）

- 文本级变换载体已有：part03 s109 prepare 的 `__zwTtScriptExecGate`（5723-5724，el+text→变换后文本，TT 门同款链位）——eval 调用点检测挂同载体（enforced fresh-parse 条件同现行安装条件：TT enforced／meta-RO／meta script-src）。
- 管线无 JS AST 解析器（script-sandbox/engine grep 零命中）→ 调用点检测为文本级守卫集：排除 `.eval(`／标识符后缀/前缀边界；字符串/注释内误报风险对语料为零（语料全为裸 `eval(` 形），对任意内容记正确性债边界＋FIXME。
- harness 安全：`resources/testharness.js` 零 eval（grep 零命中）——变换不触 harness。
- 语料零间接 eval 形：`window.eval(`/`(0,eval)`/`window["eval"]` 全语料零命中→「全局绑定保持原生」后间接面失控的风险对语料为零。

## 落码形状（立项建议）

1. part14：新 `_zwTtEvalArg` 门函数（三面迁移表与现包装 687-747 行逐面同形：①非串透传 ②TS→data 串 ③String→TT enforce（四 block＋violation 派发）→RO 臂→CSP face→返串）；`_installEvalGate` 退役 `globalThis.eval` 替换（全局绑定恒原生），安装点改为页面级 enforced 旗标＋导出文本变换入口 `__zwEvalArgTransform(text)`；`__zwUninstallEvalGate` 随持久替换消亡（无源可逆注记：已编译脚本不可逆变，跨页旗标泄漏面随全局替换消亡闭合——script_gen.rs FIXME 同根闭合候选）；薄装臂（`__zwEvalThinInstalled`，840-846）同迁移。
2. part03：prepare 链 TT exec gate 之后、eval-gate install 位（5775 邻域）对 enforced 页文本调用 `__zwEvalArgTransform`。
3. 窗口.eval 间接面：语料零命中，留边界注记不落码（如未来语料出现，`Object.defineProperty` 单属性门＋间接语义全局作用域无损可补）。

## 边界与债（明示不扩）

- 别名 eval（`var e=eval; e(x)`）源级不可检＝失控面，记边界（spec 语义上仍是 TT sink；语料零形）。
- 动态嵌套代码（Function 体串内/eval 返串内 eval）不经 prepare 变换＝失控面，记边界；可选深化＝`_zwTtEvalArg` 返串前对串内容同规则变换（递归一层，语料无需）。
- 文本守卫集对任意内容的字符串/注释误报债（AST 升级路径挂账，以管线引入 parser 为前提，不随本刀）。

## 回归面与判据（电池设计）

- **eval 家族 19 单元全绿现状**（eval-csp-* 族/tt-block-eval/RO 两页/Function-ctor 族；k49-tt-post 实证唯一红面＝目标面）——重形必须严格加法：目标面 GONE＝1、NEW=0、T 19 恒。
- TT A/B 同树 scoped checkout（k49 协议复用）：base 2245P/33FT 线上推进；四 corpus 腿 SET-IDENTICAL（dom Range 直接 eval 框架页非 enforced＝零触碰实证面）。
- solo 扫：目标页双测＋eval-csp-* 全家族页绿。
- 风险面收敛点：eval'd 片段的**作用域假设翻转**（原间接＝全局；新直接＝调用方）——全绿页若存在「依赖全局落点且调用方有同名局部遮蔽」的片段才会翻红；语料 eval'd 片段全为短字面（'beacon=…'/throw 串），电池判读按页归档。

## 工时形态与刀位建议

文件域 part14＋part03 两文件（门重形＋prepare 钩），无 Rust 面；工时与刀㊷/刀㉙ 级门刀同型（实现＋TT A/B 电池＋solo 家族扫一轮）。**建议立项**：S1 余 1F 收敛的正路，且顺带闭合 `__zwEvalGateInstalled` 跨页泄漏 FIXME（script_gen.rs 在册）与刀㉙ 遗留直接 eval 边界两个旧账。非 enforced 页零触碰（全局绑定不动）＝dom 语料零风险域。
