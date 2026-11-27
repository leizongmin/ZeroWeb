# 刀㊾ 判读（s10k50）——散布簇 S1：eval arg-transform 门形（直接 eval 保形，TT 语料 +1P/−1F，真实池 13→12F）

日期：2026-11-27 · 域：trusted-types eval 子系统（S1 直接 eval 间接化，评估文 46f0e9b0f 立项兑现）· 树锚：三运行时文件刀体 patch md5 `cd67357ba3b768fc6b27fb546d6d9de3`（24,544 B，发射 TREE-MD5-OK＋REAPPLY-MD5-OK 三文件断言；patch 常驻 `/tmp/s10k50/knife50.patch`）· 文件域＝`part03.js`（11+/0−）＋`part14-trusted-sinks.js`（大头：门重形＋§5.6 扫描器）＋`webview.rs`（30+/0−），另 `part177_s10k29_eval_gate_pins.rs`（钉测适配，随刀同提交，不属运行时 patch）。

## 目标面（S1，1F）

| 页 | 面 | 根因链 |
|---|---|---|
| trusted-types-reporting-for-Window-eval | `No violation reported for eval with TrustedScript.` | 刀㉙ 门形＝运行时替换 `globalThis.eval` 全包装——**任何绑定替换降级为间接 eval**（ECMA-262 directness＝语法 callee 为解析到 %eval% 的 IdentifierReference），TS 面经包装失去直接性（编译落点位移，本地 beacon 不发）；姊妹面「Violation report for eval with plain string」不受影响（间接域同判） |

立项依据：S1 独立评估（docs/goal/wptfyi-dom/evidence/2026-11-27-s1-direct-eval-indirection-assessment.md，已推 46f0e9b0f）——控制面预估「parser/管线级工程」高估；管线无 AST 解析器但语料全裸 `eval(` 形（18 处/9 文件；`eval?.(` 可选调用 18 处同查），arg-transform 轻刀即可。红利：`__zwEvalGateInstalled` 跨页泄漏 FIXME＋刀㉙ 直接 eval 边界两旧账随全包装退役闭合。

## 门重形（双径装置＋文本变换）

- **part14 §5.5 双径门**：meta 三臂路径＝旗标形（`__zwEvalGateInstalled`＋`__zwEvalArgArmed` 置位＋薄门中和退役——薄门原形只拦非字符串，arg-transform 形下薄门自身会成为拦截面故卸出原生）；宿主 header-face 路径（webview eval-blocked 分支 6026）＝**运行时包装形保形**（`_zwEvalWrapInstall`：包装 over `__zwTtEvalArg`＋原生间接尾）——装置在 exec 时点、变换在 prepare 时点之前，armed 迟到车道必须保留包装形。卸装＝无条件双旗清＋仅 `__zwEvalWrapInstalled` 时恢复 eval（6152 checkpoint 重装置依赖 installed=false）。
- **门体 `__zwTtEvalArg`**：刀㉙ wrapper 门体 61 行逐字节迁移（diff 核对仅三处预期差：头重构＋face① `return x`＋尾 `return s`）。三 face：① 非 String 非 TS 透传（PerformEval 恒等）；② TS → isScript → ToString → 免 TT face → CSP face；③ String → TT enforce（default policy 四 block → violation(sink 'eval'，sample＝原输入)＋EvalError）→ RO 臂 → CSP face（宿主回调/meta script-src 兜底）。**fresh parse 分派**（`_zwTtParseCsp()` 直调＋指纹缓存，零旗标读取）＝armed 与语义解耦。
- **§5.6 文本扫描器 `__zwEvalArgTransform`**：无 AST 管线下的文本级变换（fast-exit 无 'eval' 字样；行/块注释＋单双引号串（转义）＋模板串（`${}` 花括号深度）＋正则字面量（前显著字符启发式）四守卫；`eval(` 与 `eval?.(`（含 ws）双形态匹配括号；已包裹点跳过；深度帽 8；异常一律返原文＝fail-safe）。不变式：输出除确认调用点插入外逐字节恒等——误扫最多漏点或触发 fail-safe，永不破坏语法。
- **part03 prepare 钩**（5819-5829，k28 inline CSP 门后、刀㊽ importmap 注册臂前＝TT 变换先于注册，与 k49 已记「CSP 白名单以变换后 JSON sha256 计」语义一致；importmap JSON 无 eval 字样 fast-exit 零触碰）。
- **webview.rs runner 车道**（+30）：交付体求值前装置（单独 execute，非 armed 页零传串）→ armed 页才经沙箱回环 `__zwEvalArgTransform(escape_js_string(code))`。**runner 车道必要性＝本刀首跑教训**：runner 从原始 HTML 抽脚本直执行不经 part03 prepare（刀㉙返修四钉已实证），首版只落 shim 车道致 solo 全语料裸奔（0P/2F＋15F）——运行时全局包装车道无关、文本变换车道特定，逐车道落点沉淀 learning（docs/learnings/bugs/2026-11/2026-11-27-eval-gate-lane-specific-transform.md）。
- **k29 钉测适配**（part177 四行为钉）：钉测直驱沙箱无两变换点，片段先经 `__zwEvalArgTransform` 再执行（语料字节流镜像；门体 fresh parse＝未 armed 页透传原生语义，late-meta 钉阶段一保真）。六钉 guard 入口全绿（PINS-RC=0）。

spec：https://w3c.github.io/webappsec-trusted-types/dist/spec/#eval-dom ；directness：https://tc39.es/ecma262/#sec-direct-eval-functions ；`eval?.(...)`＝OptionalCall 非 IdentifierReference callee＝间接域（TS 语料仅 `eval?.5:0.3` 条件十进制假形，扫描器正确不匹配）。

## solo 页级（`?.` 扫描器扩展后终版，电池前先行）

target 2P/0F（**S1 面绿**：TS 面原生直接 eval＋本地 beacon；String 面缺回调 violation＋throw）、evalcsp 50P/0F、evalwith 10P/0F、evalnocsp 17P/0F、ttblock 1P/0F、ttreport 6P/0F、cspblock 2P/0F、funcctor 85P/0F、blockfunc 6P/0F（Window 页）、dpcallback 5P/0F、se006 4P/0F——eval 家族语料零残红。workereval 0 行定谳＝runner 车道不执行 worker eval 页（k49 全语料腿同零行，刀前刀后恒等，非本刀回归）；该五组 rc=2＝worker 页零输出行的既有门信号（rc 只留痕）。

## 电池（2026-11-27 01:40 → 02:18 UTC；一发 CLEAN——k49 协议静默窗门生效，零 ABORT 零 NOTE）

静默窗 01:43 过（写者字节恒：part02=32985cd4、part04/05/testharness 全 clean；无 cargo/rustc/runner）。四 corpus 腿 vs k49 腿日志：

| 腿 | 计数 | 判 |
|---|---|---|
| sanitizer-api | 713P/65F | F/T SET-IDENTICAL＋P 恒等（零漂移） |
| domparsing | 1314P/222F | SET-IDENTICAL＋P 恒等（零漂移） |
| web-components | 17064P/940F | F/T SET-IDENTICAL；P −421 恰 gethtml.html 单页（3224→2803，已知巨页截断抖动族） |
| dom | 57204P/374F(T含) | F 真实新＝0；GONE 222 分解＝114 t48-trace（写席插桩自移除，k49 已记 +222 同族组成）＋108 Timeout（k49 并发争抢族，solo 窗不复现）；排除两族后 F 集 374=374 恒等；P −228 census＝插桩页各 −1＋Range-mutations 截断噪声族（dataChange 878→781 等，已知族） |

TT A/B 双腿（02:00→02:18，mid-gate 三 sibling 锚＋testharness 复核全过，REAPPLY-MD5-OK）：

| 腿 | 计数 | 判 |
|---|---|---|
| TT base（scoped checkout＝恰我方三运行时文件回 HEAD） | 2253P/33FT/19T | vs k49-tt-post：**零漂移**（2253/33 全等＝k49 后写者域零 TT 面落码，归因域全净） |
| TT post（reapply 三文件 md5 断言过） | **2254P/32FT/19T** | **post − base＝本刀效应 +1P/−1F**：GONE 恰 S1 目标面（TrustedScript 免违规）；NEW=0；T 恒（19）；ΔP=−ΔF 闭合 |

判读行提取 LC_ALL=C awk＋wc 对账＋awk 多重集（/tmp/s10k45/mdiff.awk）；P 漂移页级 join（k49 judge 的 `-F' :: '` census 恒空 bug 本轮修正：页键 substr($1,6)）。rc=2 均为既有基线失败集（rc 只留痕）。

## 残余与边界

- **worker eval 页域**（Dedicated/Service/SharedWorker-eval 等 6 页＋block 构造器 3 页）：runner 车道不执行（零输出行恒等），本刀零触碰；worker.rs/service_worker.rs 无装置点——runner worker 车道启用时按本刀 learning 逐车道落点补齐。
- **别名/嵌套动态代码边界**（`var e=eval`、`Function("eval(...)")` 体串、eval 返串嵌套）：不经变换（语料零形实证）；Function ctor 面＝Function Proxy 半边（§4 宿主跳板）承载不变，funcctor 85P/0F 恒等实证。
- **混合页**（meta armed＋宿主 header-face 同页）：装置幂等（旗标形先到则包装形短路；反之包装形置双旗＋installed），门体不调 globalThis.eval 无双派发。
- **webview.rs :4909 runtime_import_map fmt 漂移**＝刀㊽ HEAD 既有（本刀域外，rule 4 只提及不修）。

## 归属与计账边界

- **本刀计账：TT 语料 +1P/−1F（S1 面收），TT 真实池 13→12F（套件 F 14→13：zz 探针 6＋散布 7 恒持），累计 84,290→84,291（距 ≥99,947 差 15,656）**。
- part02 navigate 五面＝写者链字节（diff md5 `32985cd4` 全程保树不 checkout，coord 纪律），归其轮收编。
- 旧账闭合：刀㉙「直接 eval 失真」边界＋`__zwEvalGateInstalled` 跨页泄漏 FIXME（旗标形页载重置＋卸装无条件双旗清）随全包装退役了结；script_gen.rs FIXME 域（γ1 装置语义注记）保持挂账不动。

## 结论

刀㊾ 净增 **+1**（S1 直接 eval 保形面），TT 真实池 **13→12F**，Timeout 19 恒，**累计 84,290→84,291**。池余：散布 7F（S3 2＋S4 5）＋navigate 5F（写者域）＋zz 探针 6。下一刀位：S3/S4 与写者域协商（本席评估 S3 两面同域可续）；若写者链先发则承接核验。
