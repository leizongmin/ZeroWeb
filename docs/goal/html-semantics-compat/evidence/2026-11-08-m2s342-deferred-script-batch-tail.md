# M2-S342：deferred 脚本 batch 尾调度——module/classic defer 文档序交错面（module/ 簇 1F→P，承 s341 域长尾勘向）

日期：2026-11-08。控制文档：`docs/goal/html-semantics-compat.md`（L5 链 s342 段）。
本切片收口 s341 挂账 3（the-script-element 域长尾 module/ 簇）的执行时序面：
`module/module-vs-script-1.html`（1P+1F→2P）。spec 依据：
- prepare-the-script-element：parser-inserted module 无 `async` = deferred 列表
  （文档解析完后按文档序执行；`defer` 属性对 module 是 no-op）；classic 带
  `defer` 仅外链生效（parser-inserted 无 src classic 一律 parse 位立即执行）；
  module 带 `async` = 尽快就绪即执行（本片未建，s343 面）
  https://html.spec.whatwg.org/multipage/scripting.html#prepare-the-script-element

## 定谳（本切片新增事实）

1. **引擎 batch 全文档序 parse 位执行，外链 module 无 deferred 语义**（mvs-1
   根因）：extract 输出的全部页面脚本（classic inline/external、module
   inline/external）按文档序在 batch 内 parse 位串行执行——spec 要求文档解析
   完后执行的 deferred 列表（外链 module、外链 defer classic）被提前。失败链：
   markup 先 `<script type=module src=this.js>` 后 `<script src=this.js>`，
   spec 期望 classic 先执行（module 顶层 `this`=undefined，classic 顶层
   `this`=window → `log=[window, undefined]`），引擎旧序 module 先 →
   `log=[undefined, window]` → `assert_array_equals` 翻转。
2. **分类与重排缺一不可**（probe5 双轮同态定谳）：extract 侧单独加调度分类
   不改变执行序——probe5-order/probe5-after observed 同为 `c1,m3,d2,d4`
   （module 在 defer classic 之间 parse 位执行）；webview batch 重排落地后
   全域 mvs-1 即翻绿（domain-after 07:00 已 2P）。
3. **charset-01 红面为负载暂态非本面缺口**（solo A/B 定谳）：domain-after
   07:00 charset-01 UTF-8 三案 3F → solo 复跑 main binary（stash）6P +
   WIP binary 6P，两 binary 零确定性 delta；domain-final 回归 6P。外链 module
   charset 取回在 batch 内阻塞串行（取回发起随执行位后移）为观测事实，但
   charset-01 案面不依赖发起时点。
4. **async module ASAP 执行面未建**（probe6/6b/6c 三轮持续红定谳）：引擎无
   fetch 完成驱动的调度臂，async module 现按 Blocking 槽 parse 位执行——
   execorder 簇 3F（Interlaced async-ordered + inline-async 两案）稳定在册，
   本片不动（s343 面）。
5. **script_index 副作用表与执行序解耦**：CSP/referrer/credentials sidecar
   表按 extract 序号取（序号在提取时定，与 batch 执行序无关），重排不影响
   对位。

## 施工

| 面 | 位置 | 内容 |
|---|---|---|
| 调度分类 | engine `pipeline/extract.rs` | `ScriptSched`（Blocking/Deferred/AsyncModule）：module 无 `async`→Deferred；module `async`→AsyncModule；classic `defer`+非空 src→Deferred；余 Blocking。`extract_page_scripts_sched` 新出口，`extract_page_scripts_indexed` 变 2-tuple wrapper（browser/renderer 调用方 API 不变） |
| batch 重排 | webview `webview.rs`（testharness 装载臂） | blocking/deferred 两桶稳定分桶（组内文档序保持），`blocking.extend(deferred)`；script_index 原样透传 sidecar 表 |
| 分类单测 | engine `extract.rs` 测试模块 | `extract_page_scripts_sched_classification_s342` 七分支矩阵（module 三态 + classic defer 外链/内联/空 src/plain） |

## 验证

- solo（FILTER=module-vs-script，WIP binary）：mvs-1 + mvs-2 各 1P——目标案
  翻绿，mvs-2 护栏原绿。
- charset-01 A/B（stash 法，test-guard + RAF 齐备）：main 6P + WIP 6P——
  零确定性 delta，暂态定谳（定谳 3）。
- execorder 前后同态：5P/3F == 5P/3F（async 3F 稳定，非本片回归）。
- 090 / inline-defer-onload 护栏：各 1P 前后同态（inline defer no-op /
  defer+load 护栏）。
- 单测（test-guard 包裹 `cargo test -p zero-engine`）：七分支断言过。
- clippy：`make guarded-clippy` workspace 全量 `-D warnings` 净（exit 0）。
- 全域 key-state join（子测试级 `(key,state)` sort -u，LC_ALL=C）：
  - the-script-element 子域：1072P/41F/9T → **1073P/40F/9T**（+1P/−1F =
    mvs-1 逐案吻合）。
  - 全域（九子域）：8090P/323F/47T/11NR → **8070P/320F/48T/11NR**——
    headline −20P 分解：逐键 **P→F = 0**（全域零确定性回归）、F→P = 1
    （目标案）、3 flake 文件 partial-run（见下）+ csp/link 簇 T↔P 双向
    churn 净额 ±1。
  - flake 归因 solo 三连（WIP binary，独立窗口）：focus-dynamic-type-change-
    on-blur **80P/0F/0T**（逐键 = s341 基线同态；全域 run 62P+文件级 T 为
    并行 page-timeout 边界）；select-home-end 双文件 **4F+4F**（= s341 稳态，
    detailed 4F 即 s341 挂账 2 在册 flake；全域 run 1F+3 键未注册同因）；
    link-multiple-load-events **6P/0F/0T**（= s341 全 P 同态；全域 run
    文件级 T）。

## 挂账

1. **async module ASAP 执行面**（s343 候选）：execorder
   （Interlaced async-ordered）+ inline-async-execorder +
   inline-async-inserted-execorder 3F 稳定；需 fetch 完成驱动的调度臂
   （ASAP 即 fetch 就绪执行，非 parse 位、非 deferred 文档序）。
2. **取回发起时点**：deferred 桶取回随执行位后移（spec prepare 时发起）；
   现无案面依赖（charset-01 A/B 零 delta 佐证），案面出现再勘。
3. **renderer 产品路径 deferred parity**（本片新记）：
   `apps/renderer/src/page_scripts.rs` 同取 `extract_page_scripts_indexed`
   文档序 batch——重排只达 webview 路径（runner 面）；IPC renderer 产品路径
   parity 待案面出现另切片。
4. s341 挂账原样承继：iframe 子文档脚本 currentScript、select optional
   并行跑 page-timeout 边界 flake、域长尾（the-script-element 剩余
   40F/9T：single-evaluation/evaluation-error/late-*-request 等）、
   s340 credentials 三项（主文档外链模块静态图/动态 src prepare/
   setTimeout 变体）、window.onerror per-fire window.event。

## 证据日志

- `s342-probe5-order.log` / `s342-probe5-after.log`：defer 定序探针双轮
  （observed `c1,m3,d2,d4` 同态——分类落地、重排未落地阶段）；探针验后删。
- `s342-probe6.log` / `s342-probe6b.log` / `s342-probe6c.log`：async 面探针
  三轮持续红（`P6:a,b,m`）——s343 面定谳；探针验后删。
- `s342-charset01-solo.log` / `s342-charset01-baseline.log` /
  `s342-charset01-fix.log`：charset-01 solo 3F → A/B 双 binary 6P（暂态）。
- `s342-mvs-fix.log`：目标案 solo 2P。
- `s342-execorder.log` / `s342-execorder-b.log`、`s342-090.log` /
  `s342-090b.log`、`s342-inline-defer-onload.log` / `s342-inline-defer-b.log`：
  护栏 solo 前后同态。
- `s342-domain-after.log`（8008P/325F/47T，重排落地首跑）/
  `s342-domain-final.log`（8070P/320F/48T，终态）。
- `s342-flake-focus-dynamic.log` / `s342-flake-select-home-end.log` /
  `s342-flake-link-multi.log`：flake 归因 solo 三连。
- `s342-unittest-extract.log`：分类单测（test-guard 包裹）。
