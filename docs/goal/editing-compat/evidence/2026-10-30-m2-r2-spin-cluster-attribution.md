# editing-compat M2 次轮 — 自旋簇归因 + R5340（2026-10-30）

M2 首轮挂账的「自旋簇」（run 面 ?3001-last 变体 90s 脚本中止 + R5331 记录的
8 大案 420s 墙钟转 Timeout + other 面 list/inline delete 循环）本轮归因
完成：**不是一个根因，是三个独立事实**——一个性能簇（未修，挂账）、一个
真自旋（已修 R5340）、一个 runner 完成接线缺陷（归因闭合未修，挂账）。

## 簇 A：run 面墙钟杀 = cloneNode 前置成本（非自旋，性能簇，挂账）

30 行计时探针（临时已删，形态见本节）：`browserTests` 前 30 行逐行计时
`reset（innerHTML+querySelector）/ setupDiv / cloneNode / queryCommand 三连 /
execCommand`，结果 **syncTotal=1726ms，reset=52 setup=38 clone=1611（93%）
query=4 exec10=20**，单行克隆 ~54ms。

对照 tests.js `runConformanceTest`：每行数据一次
`document.documentElement.cloneNode(true)`（5582 行）。bold.html（213 行 ×
4 变体）当前 HEAD 四变体全部完成（3048 结果）——R5331 记录的 8 大案 420s
墙钟 = ~54ms 深克隆物化 × 数据行数 × 变体页数（每变体独立 page load 都跑
全量 forEach 前置）的线性放大，**不是死循环**。delete（672 行）/
forwarddelete（649）/ multitest（628）同推。

处置：性能议题（cloneNode 深克隆物化路径），与 CASE_WALL_S 墙钟调优一并
挂账性能簇；不阻塞语义收敛轮。

## 簇 B：真自旋 = removeChild 对 pending 解析子静默穿透（已修 R5340）

归因方法：放弃手工复刻（复刻 utils 与真实文件行为分歧不可靠），改为
editor-test-utils.js 机械副本 + 8 处循环计数埋点（500 步上限
`throw "CAP <label>"`）直跑真实路径——**CAP ranges-loop** 一击指认
`EditorTestUtils.setupEditingHost` 的 `getNextRangeAndDeleteMarker`
外层循环。

机制链：marker 删除循环里 `node.remove()` → R238 generic → parent.removeChild；
marker 是 innerHTML 同 turn 解析产物（`_zwMText`/`_zwMEl` 视图，无 handle、
无 sel、`__zwIsText` 印缺失——part05 `_wrapNodeEntry` 与 part06
`_zwRegisterTextEl` 打印，part03 `_zwMText` 不打）→ removeChild 分支网四
既有分支全不命中（slice24 门=`__zwIsText`；R195 门=handle 父 registry；
R125 门=`child.__zwSelector`；handle 子门）→ 末尾静默 `return child` →
父融合视图仍含该子 → 循环重扫 firstChild 又见同一 `{` 文本节点 → 无限重删
→ 90s 脚本看门狗整变体中止（`declared tests: 0` unreported 形态）。

**R5340**（part04.js +65 行，removeChild 分支链 slice24 与 handle 子分支之间
插入）：sel 父 + 无 handle/sel 子新分支——融合视图（`_childNodeList`）
identity 定位 → host `__zw_remove_child_at(sel, idx)`（apply 序在
SetInnerHTML 后，queue 时融合序=apply 时落位，多子连续删除序正确）→ 桶
retract（`_zwPendBucket` added splice + removed push）→ 迭代器/range 通知 +
挂槽清除 + 断链 + `_mo_notify` childList record + `_zwLiveNLSync`；
identity miss 维持静默穿透（R117 lenient 同语义，不新增限制）。
规范：dom-spec §dom-node-removechild / §concept-node-pre-remove。

验证：

- createlink-with-selecting-img（原 90s 自旋案）：修复后完整跑完，产出真实
  语义断言 Fail（`createLink should wrap the <img style="display:inline">`
  ——inline 选区包裹缺口，真实语义簇，非本环目标）。
- delete-in-last 家族（definition-list-item + list-item-when-parent-list-
  is-editing-host 两案）：修复前 90s 脚本中止；修复后 44 断言全出真实语义
  Fail、**0 Timeout**。该家族是 load handler + `promise_test` 形态（异步
  promise 结算晚于全部声明，不受缺陷 #2 打击，见簇 C），44 行即完整账。
- clippy `-p zero-engine --all-targets -D warnings` 过；`cargo fmt --check` 过。
- `make test`：3601 passed / 9 failed——9 红与 M2 首轮记录的
  fetch/Request/Response/ServiceWorker 族完全同集（HEAD 既有，非本轮引入，
  留并行流域归因），js_dom_shim 相关套件全绿。
- 跨域口径（对照首轮）：本轮 R5340 是单窄分支（激活条件=sel 父 + pending
  解析子 + 融合视图 identity 命中，编辑工具链形态），非首轮四笔共享路径
  全环改动，故未重跑 html-syntax 全量通道，以 lib 套件 + editing 语料
  spot 覆盖；html-syntax 99.90% 收口态基线沿用首轮账。

## 簇 C（缺陷 #2）：runner 完成接线过早置 all_loaded → load handler 内同步 test() 提前完成套件（归因闭合，未修，挂账）

**形态**：window load handler 内同步声明 `test()` 的页面，首个测试 done 即
整页提前完成，后续声明全部丢失（只报 ≤1 测试，或 0）。最小复现 =
[evidence/2026-10-30-load-handler-sync-test-probe.html](2026-10-30-load-handler-sync-test-probe.html)
（load+focus+2 trivial tests → 只出 M1-a；干净 HEAD 同行为=既有缺陷，
非本轮引入）。

**排除项**：uncaught error（P 探针 600ms error/unhandledrejection 捕获空）；
timer-per-task 泵模式（`ZW_TESTHARNESS_TIMER_PER_TASK=0` 行为不变）。

**机制链**（testharness.js 证据）：

1. `test()` 声明即**同步执行**（`test_obj.step(func, test_obj, test_obj)`
   直跑，629-640 行）——非队列异步泵。
2. 首测 done → `Tests.prototype.result`（num_pending--）→ `notify_result`
   尾部 `all_done()` 全真即 `complete()`（3901-3919 行）。
3. `all_done()` 条件（3875 行）：`tests.length>0 && all_loaded &&
   num_pending===0 && !wait_for_finish && ...`；num_pending 只计声明未报账
   （`push` ++ / `result` --）。
4. `Tests.prototype.push` 对 `phase === COMPLETE` **静默丢弃**后续声明
   （3856-3866 行）——提前 complete 后同 handler 及后续 listener 的全部
   `test()` 调用无声消失。
5. 真实浏览器：testharness 原生 load listener（94-100 行）在 load 事件的
   `setTimeout(0)` 里才置 `test_environment.all_loaded`——**严格晚于全部
   load handler 执行完毕**，首测 done 时 num_pending 已计满，提前完成
   不可能。
6. 本仓：runner `apply_harness_completion_patch`（testharness.rs 9908）注入
   `__zw_mark_harness_loaded`（`all_loaded=true; if (tests.all_done())
   tests.complete()`），经 `HARNESS_LOADED_TAIL_CALL` 在**页 load listener
   跑完之前**调用（fetch 服务源 eval 尾 / 静态生命周期脚本尾）→ 首测 done
   时四条件全真 → 提前 complete。

**来源约束**：尾调由 270896e25（write 注入 harness 完成探测接线——102 伪
Timeout 翻绿）引入，依赖它处理 `document.open` 后 load 事件不重发、
all_loaded 恒 false 使末测 done 后永不 complete 的 write 注入路径——**不能
简单删除**。

**修复方向（下轮入口）**：把调用时机改为「load 已派发完成」门（镜像原生
load+setTimeout(0) 时序），或 readyState 门（需先核 document.open 后
readyState 是否被重置为 loading——若是则 readyState 门会打回 102 伪
Timeout）。改后验证：本探针双 Pass + scripting-1 对照 270896e25 的 102 行级
diff 不回退 + editing other 面全量对照。

**影响面**：other 面 29/96 案 `addEventListener("load"` 声明形态；其中
handler 内同步 `test()` 的子集受打击（当前只报 ≤1 测试，通过率分母被低
估），`promise_test` 子集天然免疫（promise 结算晚于声明期）；修后这些案
才出完整账。run 面 runConformanceTest 是 initial script 声明形态，不受影响。

## 临时探针处置

zz-probe-run-cost.html、zz-probe-clwsi.html（run 面）、zz-capped-utils.js
（include 面）已删——探针位于 runner 扫描面内，残留会污染全量基线。结果
已录本文；缺陷 #2 验收探针归档于
[2026-10-30-load-handler-sync-test-probe.html](2026-10-30-load-handler-sync-test-probe.html)。

## 复验入口

- `make testharness-editing-other-one FILTER=delete-in-last`（家族解锁）
- `make testharness-editing-run-one FILTER=createlink-with-selecting-img`
  （原自旋案出真实断言）
- 缺陷 #2 修后：探针文件经 runner 直跑比对双 Pass
