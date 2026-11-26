# 刀㊼ 判读（s10k48）——module 子系统 R1-a：s250 动态内联 module 臂延迟机制 setTimeout(0)→microtask（TT 语料 +6P/−6F，module 执行面六连收，真实池 29→23F）

日期：2026-11-27 · 域：trusted-types module 子系统（census bb1d0aea0 §1 R1 簇的执行面半区）· 树锚：刀体 patch md5 `0ebbfde03cdc4d1e253d1b966c2594cd`（发射 TREE-MD5-OK＋REAPPLY-MD5-OK 双断言；patch 常驻 `/tmp/s10k48/knife48-part03.patch`）· 文件域＝`part03.js` 单文件（17+/8−）。

## 目标簇（module 执行面，6F）

| 页 | 面 | 根因链 |
|---|---|---|
| script-enforcement-006 | `Untrusted HTMLScriptElement of module type uses the source text returned by the default policy.` | s250 臂延迟求值 setTimeout(0)：runner rAF 同步 stub（part01 requestAnimationFrame OFF 路径——回调预算内当场执行）使 script-messages.js `script_messages_for` 双 rAF→DONE 脚本在同一同步续体关窗 resolve，module 体定时器后到时 `window.log_message` 已被 DONE 置 null → 体抛 TypeError 丢失（got 恰 `[CREATE_SCRIPT, sink]` 缺 MODULE） |
| script-enforcement-007 | 同面（SVGScriptElement 变体） | 同上（SVG 内联 script 走同一 s109 prepare→s250 臂） |
| script-enforcement-008.https | 同面＋inline CSP check 变体 | 同上（CSP 门以变换后文本判，允许后入 module 臂） |
| script-enforcement-009.https | 同面（SVG＋inline CSP） | 同上 |
| script-enforcement-010 | `Changing script's type from module to classic in the default policy works.`（名反向；判据体＝起 classic 终 module——policy `setAttribute('type','module')`，strict 体 `undefined=0` 抛→MODULE＝module 执行直证） | 门后 type 重读（刀㊷）已正确入臂，败因同延迟序 |
| script-enforcement-011 | 同面（SVGScriptElement 变体） | 同上 |

TT 门本体（刀㊷ s10k42）无涉：门在 module 臂前、type 无关调 `createScript`、变换后文本替换——solo 探针实证 CREATE_SCRIPT＋sink 名两消息在位，唯 MODULE 缺＝执行序问题非管道问题。

## 根因定谳（三探针链）

- **probe1**（非执法页裸 module/classic 双面）：双绿——s250 臂＋宿主 `__zw_compile_inline_module` 五态钩子在 runner **已活**（runner 配 `script_source_fetcher` ⇒ webview.rs s250 注册块生效），裸 module 动态插入照常执行。`__zwHostDynamicScripts` 仅桥测试置位，runner 无涉。
- **probe2**（执法页＋default policy，006 module 臂最小复刻）：固定 300ms 等窗下全链通（CREATE_SCRIPT→变换→module 体求值→MODULE）＝管道完好，败因收敛到「DONE 关窗早于求值」的时序窗。
- **probe3**（block-text-node 页形复刻＋事件序面包屑）：序＝`pre-append→policy→post-append→raf1→raf2→post:done→post:count→recv:done→recv:count`——rAF 同步链内 done-post 先入队，微task求值的 count-post 后入队（主窗 postMessage 派发＝part06 setTimeout(0) 任务）→ done 消息任务先 fire 时 count=0。

修复＝s251 注释本意（「microtask 近似」）落地：臂外层 `setTimeout(0)`→`Promise.resolve().then`。microtask 于当前任务尾排水，先于 rAF 同步链（同步链在更晚的续体），恢复上游序（上游 module 求值＝插入任务后排队任务，先于后续 rAF 帧）；s251 accessor 导出序语义保持（accessor 装于插入体 eval 尾，微task检查点在其后——text-module/load-error-events inline,200 动态臂根因不复现）。

## 修复（part03.js 单点）

s250 臂编译+求值整体延迟外壳换形（内层事件派发 setTimeout 保持——空态 error/态②③④ load 派发仍在任务队列）；s251 注释块扩记刀㊼ 根因与语料锚（006-009 module 面＋010/011 module 终局面 got 恰缺 MODULE 链）。

spec：https://html.spec.whatwg.org/multipage/scripting.html#prepare-the-script-element （dynamic inline module evaluation 排插入完成之后）

## solo 页级（post 树，电池前先行）

全 TT 腿 solo：2243P/43FT/19T vs k47-tt-post 2237P/49FT——GONE 恰六目标面、NEW=0、P-GONE=0。

## 电池六腿（solo 串行，2026-11-26 22:28:17→23:01:01 UTC，墙钟 32m44s，无 ABORT）

| 腿 | 计数 | 判 |
|---|---|---|
| sanitizer-api | 713P/65F | F/T SET-IDENTICAL vs k47 |
| domparsing | 1314P/222F | SET-IDENTICAL |
| web-components | 17657P/940F | F/T SET-IDENTICAL；P +1132＝gethtml.html 单页（2264→3396，已知巨页截断抖动族，F 集零差排除） |
| dom | 57513P/374F | F/T SET-IDENTICAL；P +77＝Range-mutations 双页（dataChange 869→918／replaceData 867→895，已知截断抖动族） |
| TT base（scoped checkout＝恰 part03 回 HEAD） | 2237P/49FT/19T | vs k47-tt-post **多重集恒等**（gone=0 new=0）＝字节形状锚：base 树＝HEAD(双㊻ committed)＋part02 与写者席 k47 post 树同构 |
| TT post（reapply md5 断言过） | **2243P/43FT/19T** | **post − base ＝本刀效应 +6P/−6F**：GONE 恰 module 执行面六面（多重集各 1）；F NEW=0；T 恒（19） |

判读行提取 LC_ALL=C awk＋wc 对账＋awk 多重集（/tmp/s10k45/mdiff.awk）；P 漂移页级 join 归因仅已知噪声族三页。四腿 rc=2 均为既有基线失败集（rc 只留痕）。

## 残余与边界（module 子系统 16F → 10F）

- **block-text-node-insertion-into-(svg-)?script-element module 臂 2F 不在本刀**：postMessage 计数面竞速——上游 module 求值（插入后任务）先于 rAF 帧，runner rAF 同步 stub 使 done-post 抢入同步链；微task求值的 count-post 任务恒后入队（任务 FIFO）→ done 先 fire count=0。sync-at-prepare 可赢但破 s251 accessor 导出序（text-module 面根因）。**归 runner rAF-stub 任务序张力挂账**（runner 架构项：帧驱动 rAF 模式 `__ZW_RAF_FRAME_DRIVEN` 或主窗 postMessage 派发机制级别改动，非 TT sink 门域）。
- **importmap 面 8F**（006-009 importmap＋two-importmaps）＝R1-b 独立链：pre-gate classify 判 'importmap' blocked 提前返（政策未调，got `[UNMAPPED]` 零 CREATE_SCRIPT）＋importmap 注册表＋动态 import() 决议链。runner 动态 import() 本体已活（probe 面模块取回执行实证），缺 map 注册与决议接入。

## 归属与计账边界

- **本刀计账：TT 语料 +6P/−6F（module 执行面六连收），真实池 29→23F（module 16→10F：block-text-node 2＋importmap 8），套件 F 30→24，累计 84,274→84,280（距 ≥99,947 差 15,667）**。基线含写者席 s10k47 已推字节（96e706979＋00e5e751d，local==origin 0/0 核验）。
- part02 navigate 五面（+5P/−5F，在池外计账）＝写者链字节（diff md5 `32985cd4` 全腿保树不 checkout，coord 纪律），归其轮收编。
- 刀号注记：两席标题「刀㊻」并存（本席 s10k46／写者席 s10k47），s10kNN 编号为唯一键；本刀＝刀㊼。

## 结论

刀㊼ 净增 **+6**（module 执行面六连收），TT 真实池 **29→23F**，Timeout 19 恒，**累计 84,274→84,280**。池余：module 10F（importmap 8＝R1-b 独立链＋block-text-node 2＝rAF-stub 挂账）＋散布 8F（S1 直接 eval 间接化 1＋S3 2＋S4 5）＋navigate 5F（写者链域）。下一刀位：R1-b importmap 链（classify 'importmap' 臂＋注册表＋决议接入，文件域 part03＋webview.rs）；block-text-node 2F 依 runner 帧驱动 rAF 项另案。
