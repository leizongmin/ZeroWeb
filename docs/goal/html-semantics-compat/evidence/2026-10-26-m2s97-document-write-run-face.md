# M2 切片 97：document.write 写入脚本执行面（阻塞链 + 嵌套求值窗）

日期：2026-10-26。范围：`html/semantics/scripting-1/the-script-element/execution-timing/` document.write 族全量（005-012/018/020/026-027/032-033/036-037/041/048/051/053/068-069/096/102/116/118/141/149）+ part04 insertBefore 动态脚本触发。上轮：[M2 切片 96](2026-10-26-m2s96-script-src-prepare-and-forev-gate.md)。

## 1. 入场核账

- s96 基线（/tmp/s96-final-scan.txt，scripting-1 口径）：288P/251F/17T + 18 NotRun。
- 目标簇：document.write 族（005-012 内联/外链/混合、018 DOM added + doc.write、102 写入 harness 全家桶、069 外链不阻塞解析）+ 018 测试暴露的 insertBefore 动态脚本缺口。
- spec 判定链：#document-write-steps（写入标记交解析器插入点）→ #prepare-the-script-element（内联同步、外链按文档序阻塞写流内其后脚本、嵌套写入落在当前求值窗内）。本引擎回放走 innerHTML/insertAdjacentHTML（inert），写入脚本须显式执行面。

## 2. 执行面结构（part06 `__zwDocWriteRunFace`，write 入口惰性安装）

- **缓冲段收集**：`__zwDocWriteBaseChildCount` = 首 write 轮 `body.innerHTML = base` **往返后**的 childNodes.length（dbg 探针实证序列化/重解析对空白文本节点有损耗：快照 9 → 往返 7，写前快照使 bufTop 落负、收集恒空）；bufTop = 现长 − 基数，只收集写入尾段子树（文档序，4096 节点预算）——页面管线自有脚本不入面（018 双执行根除）。
- **内联槽位**：doneSlots 按槽位防重；求值窗 `__zwDocWriteEvalSlot`（part04 prepare then 内设置/清除）内嵌套 write 的面走**即时 pass**——从求值槽位下一起只执行新增内联、src 槽位留给续流链、不动水位线/挂链记账（009「external before/dw/after」序）。
- **外链阻塞链**：src 槽位占位（`__zwDocWriteExecWatermark` 停在本槽位 = 重入锚点，`__zwDocWritePendingSlot` 挂链）→ plain wrapper 重建 handle 域副本（**全属性透传**——012 include-N onload 载荷）→ `_zwPrepareSrcScript(copy, cont)` → 续流回调清挂链（条件清）、水位线推进、续跑面（102 写入 harness 全家桶依序语义；不阻塞外层页脚本，007）。
- **close 流接管**（无宿主 settle 的 runner 环境）：open 已清 body → 基点 0 → 全部缓冲脚本入面走同一阻塞链。
- **part04 insertBefore 镜像**：handle 形 SCRIPT 带位插入统一出口挂 R387 同口径门（连接态 → nomodule → for/event → run-once → 内联 eval / `_zwPrepareSrcScript`）——018 testlib.addScript firstInParent 形态依赖。

## 3. 调试记录（探针定谳）

- 探针页（临时，验后全删）：zw-s97-dbg（状态转储）、zw-s97-102b/102c（close 流链）、zw-s97-a/min/probe/fetch/018/102。
- 假因一：写前快照 childNodes.length——innerHTML 往返丢空白文本节点，bufTop = 8−9 落负钳 0，收集恒空（dbg 转储 `kids=8 base=9` 实证）。修：快照取首轮往返后口径（对同一 base 字符串确定）。
- 假因二：src 槽位 prepare 时水位线未占位——close 流续流回调条件推进（`wm === slot`）永不成立 → 重入重复 prepare 同一 fresh 副本 → 无限 fetch（102b/102c 90s 执行超时实证）。修：占位推进（wm 停本槽位作重入锚点，pendingSlot 阻塞重入，续流推进到槽位后）。
- 定谳插桩（temp trace，已拆）：close 流链全序 `close-takeover → face:wm=0 → slot0 内联 → slot1 src → prepare → then:len=26 → cont → face:wm=2 → slot2 内联 → face-exit:wm=3` ——引擎链正确；102b/102c 残留 Fail 系探针自身缺 testlib.js（include-1.js 内 `log` 未定义 → ReferenceError，已知作者缺陷），非引擎回归。

## 4. 验证

- execution-timing 全目录（corpus-only，探针删除后）：s96 83P/73F/3T → **97P/51F/2T**（/tmp/s96-exec-timing.txt vs /tmp/s97-exec-timing-final.txt 逐行 diff）。
- 全切片终扫（scripting-1 口径）：s96 288P/251F/17T + 18 NotRun → **311P/229F/16T + 18 NotRun**，净 +23 Pass，**零 P→F/P→T**（comm 校验空；/tmp/s97-s1.diff）。
- 翻绿面：005-012（write 内联/外链/混合序）、018、020、027、032、036、037、048、051、053、069（Timeout→Pass）、118、141、149×3。
- zero-engine lib：3530 passed / 0 failed（test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900）。
- shim 拼接语法：13 part 真实顺序 node --check = SYNTAX-OK。

## 5. 范围与挂账

- 102 Fail→Timeout：页脚本全执行（写入 harness 全家桶依序跑通），但 write 注入的 testharness 未接 `__zw_mark_harness_loaded` 完成探测（hook=undefined）→ runner 等满超时。**runner 面挂账**（非写流面缺陷）。
- 116 Timeout→Fail：frameset 文档 `document.body` 报 BODY 非 FRAMESET——引擎 frameset 文档语义既有缺口，页完成度提升后显形（原 Timeout 挂起）。
- FIXME(spec-gap) 留档：写流进行中页面脚本结构变更使 bufTop 漂移；嵌套 write 内含外链近似文档序（严格序不保证）；s95 children-changed 其余变异入口（append()/textContent= 已连接面）仍未挂触发。
- load-error-events 族：脱离树副本上的元素 load/error 事件保真仍挂账（语料 execution-timing 只断言执行序）。
- 063 Timeout（hook=function，pending=1）承袭 s96 未动。
