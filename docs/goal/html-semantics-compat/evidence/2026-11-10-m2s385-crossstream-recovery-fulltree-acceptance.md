# M2 切片 385（s385）——跨流回归面复验销案 + 渲染流 R5491-R5493 后全树验收轮（零源码改动）

日期：2026-11-10（承 s379/s383 挂账：跨流面复验——渲染流消化 1bcfaac6d 归因账后全域 join）
域：interactive-elements（复验主面）+ 全语料十子域（验收面）
性质：**跨流归因复核轮（零代码改动）——closedby 族 20 行跨流回归全部恢复销案 + 全树 8505 键验收零意外位移**

## 案面

s377 归因在册：interactive-elements 域 20 真案行（dialog-popover-closedby-complex 16 + dialog-popover-closedby-simple 3 + auto-expand-details-element-fragment 1）翻红窗口 = 1bcfaac6d（渲染流 R5491 inline fill 上溯修复，click 分类依赖盒几何），按 run-rules #9/#10 记账不修单列口径。s379 挂账：渲染流消化归因账后全域 join 复验。窗口内渲染流落地三修复：R5491（1bcfaac6d，fill 上溯）、R5492（5f12927d6，strut/0 二分）、R5493（6d19f3661，nested float band 游标分带链偏移 + BFC strut 下限）。

## 二进制核算（复验前置）

本 clone runner 二进制 mtime 14:36，早于本 clone pull 入 R5493 的时刻（17:08，pre-pull HEAD 31a4e02a4 不含 6d19f3661——pull diff 新增 r5493_* 测试文件实证）——旧二进制不含渲染流修复，直接复验会假红。经 make 目标重建（编译阶段无阈值 + test-guard 包裹执行）后跑测。

## 复验结果

1. **solo closedby-complex**（FILTER=dialog-popover-closedby-complex）：**40P/0F**（s375b 干净基线同形；s377-s384 期间 16F）——exit 0。
2. **全域 interactive-elements sweep**（FILTER=interactive-elements）：**467P / 16 Fail/Timeout 行**（s378 干净口径 447P/37 行 → +20P/−21 行）。join vs s375b 22 行基线（LC_ALL=C，file::subtest 键）：**cur 16 行全部 ⊆ s375b 22 行，零新增红零改名**；s375b→cur 差集恰 = 探针 3 行（s377 时已删）+ overlay 2 + t5 1（既有翻绿在册）。
3. **16 行构成**（对账恰合在册归属）：R-s31 UA dialog 定位簇 14（abspos 6 / bounds-clicking 2 / top-layer-position 1 / non-modal 1 / multiple-centered 1 / scroll-height 1 / ancestor-is-inert 1 / showModal 1）+ corner-cases 1（上游 Chrome 同败单列）+ closed-details-layout-apis 1（出域渲染流）。
4. **定谳：跨流回归面 20 行全部恢复，销案**。closedby-simple/auto-expand 未入 fail 集 = 全 Pass。

## 恢复归因（记账注记）

红窗口由 1bcfaac6d 引入（s377 stash 法归因在册）；绿窗口含 R5491/R5492/R5493 全部三修复。本轮未逐修复 bisect 定位「恢复修复」——恢复即销案，三修复均已入渲染流控制面（R5492/R5493 收口账 in rendering-compat）；如需精确归因由渲染流自证。跨流回归面 20 行从本流挂账清单移除。

## 全树验收（s377 教训执行：渲染流大改动后全域 join 必要验收）

无 FILTER 全语料 sweep（TIME_LIMIT=14400，Makefile 设计参数；8 job，约 6 分钟）：**8198P / 270F / 37T = 8505 键**。

- **probe 单列 28F**：s325-probe-options-move 13 + s325-probe2-corpus-replay 9 + s327-probe-identity 2 + s327-probe2-decompose 3 + s327-probe3-tree 1（自建诊断探针恒红记账面，s343 方法论单列，不入 corpus 口径）。corpus 真红 = **242F + 37T**。
- **分域对账零意外位移**（域粒度；键级全树 join 无基线在案——s356 意外全树跑未入账，本轮 8505 键为首个在档全树口径）：
  - tabular-data **130P/4F** 恰合收口定格（4F 全 processing-model-1 表格布局渲染流在册）；the-script-element **1150P/1F** 恰合 s383 口径（1F = dynamic-imports-script-error 平行流漂移在册）；interactive-elements 16 行恰合上述干净口径。
  - 其余 307 行全部落在在册红族域：the-select-element 176（customizable-select corpus 117 + s325 探针 22 + filterable-select/select-popover-position 等邻位）、the-img-element 20+1T、the-embed-element 13+5T、customizable-combobox 12、the-object-element 12+6T、the-style-element 10+9T、the-input-element 6+1T、form-submission-target 6+6T、links-created-by-a-and-area 4+4T、form-control-infrastructure 4、the-video-element 4+1T、the-textarea-element 3、the-label-element 2、the-canvas-element 2+1T、interactions-of-styling-and-scripting 2、the-output-element 1、the-form-element 1、the-button-element 1T、the-link-element 1。
  - grouping-content / text-level-semantics / microdata / document-metadata 余量等域零红。
- **本流零入侵实证**：interactive-elements/scripting-1 两活跃域计数与 s378/s383 在册口径逐键恰合——渲染流三修复对本流已收口语料零位移。

## 口径

- interactive-elements 干净口径定格 **467P/16 行**（真案 15 = R-s31 簇 14 + 上游同败 1，另出域 closed-details 1）；跨流回归面销案后本流在册挂账余：R-s31 簇 14（渲染流）+ customizable-select A2 簇 + image input replaced sizing（渲染流）+ closed-details/inert hit-testing（无钉面）+ muted-errors skip 排除形（规范排除）。
- 全树基线：8505 键（8198P/270F/37T，probe 28F 单列）——下轮渲染流大改动后全域 join 以此为键级基线。

## 下一步候选

**customizable-select A2 簇分诊轮**：corpus 117F + 邻位 filterable-select/select-popover-position 等构成 the-select-element 域 154 corpus F+2T（全语义面最大余红块）。s377 已定性 select-dialog-mode-focus 2F + picker open 态注入通道 39 涉案文件归渲染簇（appearance:base-select 解析缺席 + ::picker 仅选择器解析放行 + is_open_state_element 无 select）；分诊目标 = 剩余面三分（渲染簇依赖 / 本流 shim 可施工子集（IDL/属性/键盘语义面）/ 上游 tentative 未实现面（appearance:base-select UA 全规则）），产出归属挂账与可施工切片清单。

## 验证

- 两轮 sweep 均走 make 目标（fetch 幂等 + target-disk-guard + ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1 + test-guard --per-proc-mem 4 --total-mem 8 --time-limit 5400/14400）；join LC_ALL=C + sort/comm 键级。
- 零源码改动 → lib/fmt/clippy/product-smoke 全不适用（s376/s379 零代码轮先例）。
- 日志 /tmp/s385/（closedby-complex.log/.exit、sweep-ie.log/.exit、sweep-full.log/.exit、cur-fail.keys、join.sh）。
