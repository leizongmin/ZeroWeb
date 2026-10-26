# M2 切片 96：script.src 属性变更步骤 prepare 触发 + R387 for/event 门

日期：2026-10-26。范围：`html/semantics/scripting-1/the-script-element/execution-timing/` 晚设 src 家族与 149 for/event dom 臂。上轮：[M2 切片 95](2026-10-26-m2s95-script-text-and-children-changed.md)。

## 1. 入场核账

- s95 基线（/tmp/s95-final-scan.txt，scripting-1 口径）：281P/256F/19T + 18 NotRun。
- 目标簇：022/024（append 空脚本后连接态设 src，external script 不执行）、149.html 3 个 dom_inserted for/event 臂。
- 语料只读核读：022（eventOrder 含 external script #1）、024（第二次 src set 须忽略）、149（window/onload()、window/onload、" WINdow\t\n"/"ONload\t\n" 执行，dom 插入的 window/load、window/onpageshow、document/onload 不执行）、script-for-event.html（17 臂全绿基线）。

## 2. spec 判定链

- prepare-the-script-element 的 for/event 门（whatwg/html source #script-processing-for）：for/event 双属性在场，且非「for 剥白 ASCII 小写 == window 且 event 剥白 == onload 或 onload()」即 return 中止；任一属性缺失照跑。下载源文 grep 锚点核实，与 s94 extract 门极性一致。
- script attribute change steps（commit c22fb1b32c patch 经 gh api 证实）：src 属性变更且 value 非 null 且 connected → post-connection steps → prepare。parser-inserted 首步 return（静态 script[0].src= 写入不受扰）。
- script-for-event.html 17 臂全绿 + spec 原文互证：s94 门极性正确；149 parser 臂失败全因 document.write 通路（挂账不动）。

## 3. 两刀

- 刀①（part04）：`_zwScriptForevBlocked(el)` helper——双属性在场 + ASCII 剥白 CI 门；R377 fallback、内联臂、R387b 三执行路径统一接入。规范链接 #script-processing-for。
- 刀②（part04+part05）：R387b fetch/执行体原样提升为 `_zwPrepareSrcScript(child)`（run-once/nomodule/for-event 门 → src 读 → 宿主管线让位 → Resource Timing initiator → fetch → s91 base URL/s92 nonce swap → eval → load/error 事件）；part05 set trap 新增 SCRIPT.src 专用分支（`_reflectedUrlAttr` URL 反射分支加 SCRIPT/src 排除，沿用 IFRAME.src/IMG.src 先例）——反射写属性 + connected 时调 prepare。规范链接 #script-attribute-change-steps + #dom-script-src。

## 4. 调试记录（探针定谳）

- 探针页（临时，验后删）四臂：A 内联动态、B append 后连接态设 src（100–2000ms 阶梯）、C 先设 src 再 append、D/D2 for=event 矩阵。
- 首轮误报教训：B 臂 2s 轮询判 Pass，换语料同款 onload+100ms 时点后失败——首轮 teardown 竞态侥幸，非真执行。改阶梯时点 + fetch 插桩后定谳 B 从不 fetch。
- 假因排除二：`_reflectedUrlAttr` 分支拦截（修后仍不 fetch）；handle-only isConnected 恒 false（TRAP 插桩显示 conn=true nodeContains=true）。
- 真因：`_zwPrepareSrcScript` 定义于 part04 appendChild trap 闭包，part05 set trap 是另一函数作用域——`typeof` 恒 undefined（DIAG `hasFn=undefined` 实证）。修：`globalThis._zwPrepareSrcScript` 导出（_zwNodeContains part03 导出先例）。
- 修后 DIAG：B 臂 TRAP-PREPARE-CALLED → fetch include-1.js → 执行于 load 前（B@1797 load@1827）；C 臂 set 时 conn=false 正确不触发，走 R387b append 路径。

## 5. 验证

- 探针页 9/9 全绿（清理插桩后复跑确认）。
- execution-timing 全目录回归：022 Fail→Pass、023 Timeout→Pass（意外收益，同晚设 src 形态）、024 Fail→Pass，目录内零回归（/tmp/s96-exec-timing.txt vs s95 基线逐文件 diff）。
- script-for-event 两文件（classic + module）34/34 全 Pass。
- zero-engine lib：3530 passed / 0 failed（test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1500）。
- shim 拼接语法：13 part 按真实顺序拼接 node --check = SYNTAX-OK-FULL。
- 全切片终扫（scripting-1 口径）+ 基线 diff：见控制面本轮状态块。

## 6. 范围与挂账

- 不动：149 parser 臂与 005-012/018/026/027 document.write 族；020（data: URL fetch 通道未证）；048（sel 形 script[0].src= 写入——post-connection parser-inserted 首步 return 语义上不受刀②影响，自身另有缺口）；module 尾片、script-charset、svg-script-fragment-clone、nomodule-async（承袭）。
- FIXME 留档：setAttribute('src') 通路（get/set trap 之外的属性写入面）、type 门（非 JS type 的 prepare 中止）、children-changed 其余变异入口。
- `FIXME(spec-gap)` 注释留在 `_zwPrepareSrcScript`：setAttribute 通路未挂、type 门未补。
