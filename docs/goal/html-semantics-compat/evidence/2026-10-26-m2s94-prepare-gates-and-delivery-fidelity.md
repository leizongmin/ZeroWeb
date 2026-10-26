# M2-s94：classic 脚本 prepare 双门 + 投递保真扩口 + module 缓存键分离（23F 翻绿零附带）

日期：2026-10-26。归属：html-semantics-compat / s86 切片序③ the-script-element 基面（承接 s93，HEAD 43f10a3e7）。

## 1. 面与基线

s93 HEAD 基线（/tmp/s93-final-scan.txt，FILTER=scripting-1 口径 540 行 211P/282F/25T + 22 NotRun）。s94 入场探针自 282F 聚类选 runner 可达 classic 属性面两簇：

- **nomodule face 7F**：nomodule-set-on-inline-classic-scripts 2F + nomodule-set-on-synchronously-loaded-classic-scripts 2F + nomodule-set-on-async-classic-script 2F + module/nomodule-attribute 1F。
- **script-for-event face 13F**：17 例中 13 例红（Script 0-11、14）。

## 2. spec 依据与刀位

- **nomodule**（webReader 取回 prepare-the-script-element 原文）：元素带 `nomodule` 内容属性且类型 classic → 中止不执行；module 不受影响（module/nomodule-attribute 以 this.js `log.push(this)` 顶层 `this` 区分执行形态——module 求值 this=undefined，classic=window）。
- **for/event**：classic 脚本**同时**带 `for` 与 `event` 时，两值各剥首尾 ASCII whitespace 后 `for` 须 ASCII case-insensitive 匹配 `window`、`event` 须匹配 `onload` 或 `onload()`，否则中止；任一属性缺失照常执行（语料 Script 15/16 单属性臂 true 定谳；NBSP 沾染臂 false——非 ASCII 空白不剥）。

### 刀①：extract 双门（engine extract.rs）

`script_suppressed_by_nomodule` + `script_blocked_by_for_event` 两 helper（spec 链接 + 单测），提取循环 `is_module` 判定后 `continue` 跳过；`this_idx` 过滤前递增，与 shim `getElementsByTagName('script')` 序号对齐不变（沿 s93 单测风格 +2 测）。

### 刀②：shim R387 插入钩子 nomodule 门（engine part04.js）

动态插入执行钩子三执行分支（R377 registry 回落 / 内联 eval / R387b src fetch）条件加 `!_r387nomod` 前缀；`nomodule` 按内容属性在场 + type 非 module（ASCII 剥白后比较，与 extract 同口径）判定；**不置 run-once 标记**——spec 中止发生在 already-started 置位前，摘除属性再触发钩子应重新走门。语料覆盖：`noModule = false` IDL 反射摘除属性 → 门不触发（绿臂保持）；`noModule = true` → 门触发（inline 动态臂翻绿）。

### 刀③：runner inline_local_scripts 投递保真扩口（wpt-runner testharness.rs）

- `nomodule`：新 `open_tag_has_attr`（engine extract，bool 属性裸形/空值形/任意值形一律在场——与 `attr_from_open_tag` 的「nonce 裸形视缺失」契约分工，+1 单测六形态）；语料 `<script nomodule src=…>` 正是裸形。
- `type`：s92 挂账「投递不保留 type="module"」兑现——module 形外链脚本内联后按 module 编译执行（module/nomodule-attribute 翻绿的必要条件）。

## 3. 附带损伤与根因返工（本轮主发现）

首轮全子树复扫 17F 目标全数翻绿 + module-vs-script-2 1F 红利，但 **currentScript-null.html 基线 subtest Fail → file Timeout**。stash 实验定谳本刀 type 保真所致（stash 后 subtest Fail 复现）→ 最小探针二分（probe-a「src module + 测试 module」红 / probe-b「单测试 module」绿 / probe-c「src module + classic 测试」绿 / probe-d「双纯内联 module」红）锁定**任意双 module script 页挂起**——潜伏缺陷被投递保真首次暴露（语料此前无双内联 module 页）。

根因：R5182 `__moduleCache` 文档级记忆化键 = `compile_module_script` 的 `url` 参数，而 webview 宿主循环把**页面 URL** 一并传入——同文档所有 module script（内联/外链皆然）共享同键，第二枚起被首枚记忆化命中短路，模块体零次求值（runner 诊断行 `completed without reporting registered tests (tests:0)` = 测试注册缺失）。修复：**键参数分离** `compile_module_script(source, url, cache_key, registry)`——`url` 保留相对 import 解析基准 + `import.meta.url` 发射语义；`cache_key` 内联 = 页面 URL 挂 ordinal 片段（`#!zw-inline-module-{n}`，仍绝对 URL，URL 构造剥旧 fragment 解析不受影响），外链 = src 绝对化（同 src 两元素命中同键恰好一次，spec module map 语义）。生产调用方三处（webview 脚本循环 / EsModuleSandbox / service_worker 编译）均传自身 URL 语义不变。+1 单测（双内联各自求值 + 同键恰好一次双形态）；learning 沉淀 [bugs/2026-10/2026-10-26-module-cache-key-page-url-collision.md](../../../learnings/bugs/2026-10/2026-10-26-module-cache-key-page-url-collision.md)。

## 4. 验证

- **定向**：nomodule FILTER 17P/4F（4F→P：inline 双臂 + sync 臂 2 + module-attr）；script-for-event 主页 18 行全绿（13F→P）；探针 4 页修复后全绿；currentScript-null 全绿（**连带基线 Fail→Pass**——module 求值 currentScript=null 语义成立）。
- **全子树终扫**（FILTER=scripting-1，test-guard 包裹，同 ZW_TESTHARNESS_RAF_FRAME_DRIVEN）：/tmp/s94-final-scan2.txt 574 行 **263P/270F/23T + 18 NotRun**。diff vs s93 基线（name 键）：**F→P 23 行、P→非P 0 行**。23 行归因：for-event 13 + nomodule 4 + currentScript-null 1 + module-vs-script-2 1 + module/load-error-events{,-inline} 3（页内 `type="module"` 标签在，缓存键解锁；非 classic load-error 族）。新增 49 / 消失 16 = module 族页此前 file 级 Fail/Timeout（多 module 挂起）终扫首次以 **subtest 粒度**完成——type.html 4 例、imports.html 5 例、integrity inline 1 例、module/script-for-event 18 例、empty-attributes-clause 1 例等翻绿；charset-01/03、integrity 外链、single-evaluation-1/2、instantiation-6/7、specifier-error、evaluation-error-2、fetch-error-2、import-attributes 2 页的 Fail 行 = **真 module 面缺口显形**（此前 file 粒度掩盖，入挂账）。域账自此按 **264P/270F/23T** 记（raw 263P + 惯例 +1）。**新基线 /tmp/s94-final-scan2.txt**。
- **单测七腿**（test-guard `--per-proc-mem 4 --total-mem 8 --time-limit 1500`）：zero-engine **3530/3530**（+3）、zero-wpt-runner **226**（+76 ignored）、zero-webview **800/800**、zero-script-sandbox v8 **365/365**（+1 缓存键）、quickjs **260/260**（+1 同测）。
- **fmt / clippy**：engine/sandbox/webview/runner 四 crate `--all-targets -- -D warnings` clean（ulimit + timeout shell 兜底，同 s91-s93）。
- reftest 未跑：无 render/layout 面（prepare 执行门 + 投递属性保真 + 模块编译键；影响脚本执行与否/次数语义），沿 s91-s93 影响面口径。

## 5. 挂账残面（本轮不动）

- **nomodule-set-on-async-classic-script 2F**：两臂 waitForLoadEvent（`window.onload`）promise 门——window load 派发/时序 face（与 nomodule 门无涉：true 臂的门已就位，promise 不 resolve）。
- **sync 臂 1**（无 nomodule 外链须跑 + `onload="loaded = true"` 元素事件断言）：投递丢 onload 属性 / 元素 load 事件 face（同族 load-error-events-1/2/3 既有账）。
- **nomodule-set-on-external-module-script file Fail**：外链 module 装载面（module 族）。
- **module 族新显形 Fail 行**（charset/integrity-external/single-evaluation/instantiation-6/7/specifier-error/evaluation-error-2/fetch-error-2/import-attributes 部分）：module 尾片真缺口首次 subtest 显形——.py 执行器前置项不变；后续切片按 subtest 账重排 module 面优先级。
- **execution-timing 80F / 顶层 65F / 完整接口对象基建面**（承袭）。
