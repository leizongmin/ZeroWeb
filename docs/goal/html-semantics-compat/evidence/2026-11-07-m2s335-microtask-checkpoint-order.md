# M2 切片 335 —— microtasks 检查点序：runner 尾微任务延迟 + strict classic 抛错报告面

日期：2026-11-07。承 s334（canvas 双面）。

## 目标

`html/semantics/scripting-1/the-script-element/microtasks/` 域残红勘根收口：
before 共 8 红（3 Fail 目标面 + 2 Fail R3 面 + 3 文件级 Timeout）。

## 定谳（勘根）

前轮探针 zz-s335-probe.html 因未挂 `evaluation-order-setup.js` 全线 ReferenceError
作废，账面以 corpus 源文件 + 基线日志重推（探针资产核验先于测量，s227 同训）。

1. **R2（runner 尾粘合）**：runner 内联外链 script 的 load/error 派发尾与脚本体
   同块同步粘合——事件先于 chunk 末微任务检查点。spec：run-a-classic-script
   抛错→report→clean-up-after-running-script（检查点排空微任务）→fire load。
   故 corpus 期望 `[body, microtask]`，实测 `[body]`（evaluation-order-1
   classic/module "queueing a microtask" 双假红根因）。
2. **R1a（strict 车道无报告无 load）**：`run_page_scripts_strict`（唯一走
   try-catch 包装的车道，webview 独占）classic 抛错既无 window error 报告亦无
   元素 load——sentinel 静默续跑，checkpoint-after-window-onerror classic t1
   `Promise resolved during #report-the-error` 拿不到 error 事件（文件假红根因）。
   spec 顺：report the exception（window error 同步派发，handler 跑在 catch 域内）
   →检查点→fire load（runtime 抛错照常 load，error 事件仅网络级取回失败面）。
3. **R3（逐 handler 检查点缺失，架构面→s336）**：checkpoint-after-window-onerror
   {,-module} t2 `Promise resolved during event handlers other than error` 残红
   ×2——WebIDL 要求事件派发在逐 listener 间跑 clean-up-after-running-script
   （message handler 期望 `[h1,h1p,h2,h2p]` 实测 `[h1,h2,h1p,h2p]`）。shim 派发
   循环为纯 JS，无法在 listener 间强制 V8 检查点。
4. **语料缺口**：evaluation-order-{2,3,4}.html 引 `resources/evaluation-order-
   {2,3,4}.1.mjs` 三资产缺席→3 文件级 Timeout。根因 = `goals/lib.sh` 两处
   fetch case 白名单漏 `*.mjs`（浅层与深挖双模式）。本轮按钉版直抓补齐 +
   白名单修复 + imported-testharness.txt 记账（M2-S335-asset ×3 行）。

## 施工

- **A（runner）**`tests/wpt-runner/src/testharness.rs`：
  `script_event_dispatch_tail_el/_unit` 元素引用**同步捕获**（currentScript
  微任务窗已清，R3258；unit 走 data-zw-unit 标记扫描），派发包
  `queueMicrotask` 延至 chunk 末检查点后。跨 chunk 先序不变（检查点在下一
  chunk 脚本前排空）。单测
  `script_event_dispatch_tail_is_checkpoint_deferred_s335` 钉捕获<延迟<派发序。
- **B（engine）**`crates/engine/src/js_dom_bridge/script_gen.rs`：新增
  `script_run_classic_page_strict` 变体（`script_run_classic_page_impl` 加
  `report_uncaught` 形参；基线包装字节不变，browser/renderer 车道共用）。
  strict 尾 `STRICT_UNCAUGHT_REPORT_TAIL`：catch 内**同步** reportError 原值
  透传（window error 事件在 catch 域内派发，先于检查点）+ 元素 load 走
  queueMicrotask（检查点后派发）。
- **接线**`crates/webview/src/webview.rs` strict 车道独占换用 strict 变体；
  browser/renderer sentinel→R2940 自报链保持基线包装（互换即双报——报告纪律
  面注释两侧各一）。
- **语料**`tests/wpt-runner/scripts/goals/lib.sh` 两 case 补 `*.mjs`（挂日期
  注释）；`resources/evaluation-order-{2,3,4}.1.mjs` 钉版
  31597693...（WPT_REV 同源）抓取，imported-testharness.txt 追 3 行。

## 验证

| 门 | 结果 |
| --- | --- |
| 契约 engine（part09 `test_classic_strict_wrapper_uncaught_report_and_load_s335`） | 1/1 绿——strict 包装全 shim 沙箱断言序 `body\|error:boom\|mt\|load` + sentinel 含 boom；基线包装静默面 `body\|mt` |
| 契约 runner（`script_event_dispatch_tail_is_checkpoint_deferred_s335`） | 1/1 绿——捕获<延迟<派发 + data-zw-unit 标记保持 |
| microtasks A/B（`s335-microtasks-before.log` vs `s335-microtasks-after.log`） | **3 Fail→Pass 零 P→F**：checkpoint-after-window-onerror t1、evaluation-order-1 classic/module t1；evaluation-order-1 文件级 Timeout 消解（4 子测试全 Pass）；残红 = R3×2 Fail（s336）+ 资产×3 Timeout（本轮补齐，s336 复跑） |
| the-script-element 域 A/B（s329 基线 1123 vs `s335-script-element-after.log` 1122 join） | **零回归**；3 改进同上；data-url Timeout→Fail = 报告面打通暴露保真新面（onerror 零触发→至少一触发，断言细目未达）→s336 挂账；non-html-noscript 缺席 = FILTER=the-script-element 不含 the-noscript-element 域（良性域差） |
| engine lib（test-guard） | 3801P/6F = 已知红集精确（零新增） |
| clippy -D warnings ×3 crate + fmt --check | 净 |

## 挂账（→ s336）

1. **R3 逐 handler 微任务检查点**（checkpoint-after-window-onerror{,-module}
   t2 ×2 Fail）——shim 派发循环纯 JS，需宿主面（engine 派发循环改 Rust 或
   dispatch_event 调用点宿主排空）。
2. **evaluation-order-{2,3,4} 复跑**——语料已补，3 Timeout 待真实结果（模块
   求值序 face）。
3. **data-url 报告保真**——lineno/colno/原始 throw 值断言面（1 Fail）。
4. **execute Err 路径跳过检查点**（probe-D 浏览器道 parse 级抛错）——基线
   包装 Err 早退无排空；WPT 面未触，浏览器道挂账。

## 证据

- `evidence/logs/s335-microtasks-before.log` / `s335-microtasks-after.log`
- `evidence/logs/s335-script-element-after.log`（对 s329-survey-scripting-1.log join）
- `evidence/logs/s335-lib.log`（engine lib 3801P/6R）、`s335-clippy.log`
