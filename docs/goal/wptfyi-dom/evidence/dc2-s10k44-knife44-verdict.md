# 刀㊹ 判读（s10k44）——worker 面 TT 门三臂＝Worker 构造 sink 门＋SharedWorker 构造落地＋SW register 门与宿主桥缺席回落（真实池 43→38F）

日期：2026-11-27 · 域：trusted-types（worker/serviceworker sink 族）· 树锚：刀体 patch md5 `9bdac9803fe942d4b39c415799c7a9b0`（发射 TREE-MD5-OK＋TT 重放 REAPPLY-MD5-OK 双断言）；**落库码刀 `48534843f` patch md5 恒等 `9bdac980`——电池域=落库域零差**。落库字节 TT 全量已含在电池 post 腿（同一 bytes）。

## 目标簇（worker 面 4F，reporting 族）

三页六臂（`trusted-types-reporting-for-Window-*`，violation 测试助手 `trusted_type_violation_for` / `no_trusted_type_violation_for`）：

| 页 | 臂 | 刀前形态（k43-tt-post 权威） | 根因 |
|---|---|---|---|
| DedicatedWorker-constructor | TT 实例不违规／plain string 违规＋TypeError | 2P／**1F** | Worker 构造器无 TT sink 门——plain string 臂静默构造成功（无违规无异常） |
| SharedWorker-constructor | 同构两臂 | **2F**（ReferenceError 双臂） | SharedWorker 全局整体缺失（part05 只有 Worker） |
| ServiceWorkerContainer-register.https | TT 实例不违规／plain string 违规＋TypeError | 1P／**1F** | register 逻辑在 support .js（页不被枚举跳过）；`navigator.serviceWorker.register` 对 TT 实例 `typeof !== 'string'` 拒绝且 plain string 臂无门（无违规） |

## 修复（两文件三臂）

- **part05 Worker 构造门**：`function Worker(url,…)` 参数检查后、URL 解析前插 `__zwTtSinkValue('TrustedScriptURL', url, 'Worker constructor')`——enforcement 下非 TT 实例→default policy（有则转换）或 TypeError＋违规派发（part14 管道，sample `Worker constructor|<clip40>`，setTimeout-0 任务 blockedURI `trusted-types-sink`）；TT 实例与非 enforcement 页经管道 String 化＝零行为差。
- **part05 SharedWorker 构造器落地**：参数校验（0 参 TypeError）→同款 TT 门（sink 名 `'SharedWorker constructor'`）→`__zwTtBypassNextSink()`（刀③b 同款单次透传旗标，part14:1152 导出）防内核 `new Worker(转换后字符串)` 二次 gated（二次 default policy 调用/二次违规派发）→port 配对（pagePort/workerPort `_other` 互指）＋core message/error 转发＋onerror defineProperty。FIXME：真共享语义（同源多文档聚合＋connect 事件）未落——语料只钉构造门与单文档消息面。
- **part02 SW register 门＋桥缺席回落**：`_container.register` 头部 option 解析前移，scriptURL 经 `__zwTtSinkValue('TrustedScriptURL', …, 'ServiceWorkerContainer register')` 门；`typeof __zw_sw_register !== 'function'` 时回落新助手 `_zwSwRegisterNoBridge43`——scriptURL/scope 同源校验（SecurityError reject）→已有 registration 精确命中复用→`upsertSnapshot` installing 快照→`scheduleClientMessagePoll`＋`setTimeout-0` 一次性 `updatefound` 派发（pollRegistration `_updateFoundPending` 臂同形，不走自续环——桥缺席域 poll 无宿主 activation 永续自调度）。FIXME：真安装/激活链待宿主桥全覆盖收口。

runner 无 `__zw_sw_*` 宿主桥（11 函数仅 webview.rs:7062 提供）——回落使 SW 臂无需新 runner 子系统即可全绿。

## 枚举跳过面（零门回归实证）

`gap_batch1_case_skipped`（testharness.rs:4181）跳过 SOURCE 含 `new Worker(`/`SharedWorker(`/`navigator.serviceWorker` 的页——SharedWorker-constructor.https.html（内联逻辑版）在 TT 套件内本为枚举跳过（刀前刀后均零输出，直跑静默复现＝非 bug）；套件内回归面收缩为三 reporting 页＋嵌套 worker 域（enforcement 标志不可达，门透明）。plain-string Worker 语料页（构造即过门 String 化）零门回归面。

## solo 页级（post 树，终版字节，电池前先行）

| 页 | base（k43-tt-post 权威） | post | 判 |
|---|---|---|---|
| SharedWorker-constructor.html（reporting 版） | 0P/2F | **2P/0F** | 两臂全翻 |
| DedicatedWorker-constructor | 2P/1F | **2P/0F** | plain string 臂翻 |
| ServiceWorkerContainer-register.https | 1P/1F | **2P/0F** | plain string 臂翻 |

## 电池四腿＋TT 同树 A/B（solo 串行，2026-11-27 18:13:21→18:46:33 UTC，墙钟 33m12s，无 ABORT）

| 腿 | 计数 | 判 |
|---|---|---|
| sanitizer-api | 713P/65F/0T | F/T 多重集 SET-IDENTICAL vs k43 |
| domparsing | 1314P/221F/1T | SET-IDENTICAL |
| web-components | 17811P/920F/20T | F/T SET-IDENTICAL；P +595＝gethtml.html 单页（2955→3550，已知巨页截断抖动族） |
| dom | 57544P/362F/12T | Fail 多重集恒等；P +160＝Range-mutations 双页（+107/+53，已知截断抖动族）；T 形变＝replaceChildren 单页（页级 T→2 子测 T，同族 solo 时序伪影，k43 反向） |
| TT base（checkout＝HEAD 含刀㊸） | 2213P/50F/20T | vs k43-tt-post **F/T SET-IDENTICAL**（对锚正确性自证） |
| TT post（重放 md5 断言过） | **2218P/45F/20T** | **+5P/−5F；GONE 恰＝三页五条 worker 臂失败行；F NEW=0；T IDENTICAL** |

四腿 rc=2 均为既有基线失败集（rc 只留痕）。页行 dump：三页六臂全 Pass（SharedWorker TT/string＋SW register TT/string＋DedicatedWorker TT/string）。逐页 P census 对账：wc/dom P 漂移全部落在已知截断噪声页（F 集零差排除），零真实漂移。

## 结论

刀㊹ 净增 **+5**，TT 真实池 **43→38F**（套件 F 50→45−7 zz 探针页），**累计 84,260（距 ≥99,947 差 15,687）**。worker 面 4F 全部出池；散布池余键：其余散布归簇＋挂账刀㊸b（about:blank/blank.html write-exec 链＋每文档 CSP 不继承）。module 子系统切片 16F 立项评估待做（master 在案）。
