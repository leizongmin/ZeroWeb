# M2-S343：async module 调度面（runner 侧注册 + 投递 async 保真 + 动态内联 module 任务化）——module/ 簇 4F→P

日期：2026-11-08。控制文档：`docs/goal/html-semantics-compat.md`（L5 链 s343 段）。
本切片收口 s342 挂账 1（async module ASAP 执行面）的 runner 可达面：execorder
簇 3F（Interlaced async-ordered + inline-async 两案）+ `non-external-no-import`
1F → 4 案全绿。spec 依据：
- prepare-the-script-element 步骤 22：async module = 尽快就绪即执行（ASAP 集）；
  trickle 查询模拟的取回延迟须在执行序中兑现
  https://html.spec.whatwg.org/multipage/scripting.html#prepare-the-script-element
- 动态插入内联 module 的 evaluate 排队任务（插入语句内不执行，whatwg/html#9864）
  https://github.com/whatwg/html/issues/9864

## 定谳（本切片新增事实）

1. **trickled async module 的取回延迟不经投递面兑现即破 fast-before-slow 判据**
   （inline-async-execorder 失败链）：runner 内联投递本地取回即时完成——src 形
   trickled async module（`slow-module.js?pipe=trickle(d2)`）原走 s112 注册（延迟
   兑现），**无 src 内联 async module**（体内静态 import 携 trickle）原走 resolved
   None 整段透传 → 引擎 InlineModule 批内即刻求值，slow import 即刻完成 →
   `loaded=[fast,fast,slow,slow,slow,fast]` 中 slow 抢跑。async 无 trickle（延迟
   0）与绑定形/动态 import 体不落注册臂（既有透传面不变）。
2. **外链 async module 投递 shell 剥 async 属性 → extract 误分类 Deferred**
   （inline-async-execorder 残红根因，本切片修复）：投递改写只保真
   nonce/nomodule/type/id/onload/onerror（s94/s100/s108 族），async 丢失使
   extract 把 async module 分类成 Deferred（module 无 async 隐式 defer）→ 单元
   从 parse 位挪到 deferred 桶尾。probe13 时序 dump 定谳相位：
   `[fast@1,inline-fast@1,slow@2,slow@3,inline-slow@3,fast@4]`——慢速 async 单元
   注册面的调度 drain 在批内虚拟时（RAF frame-driven）兑现后，fast 单元才在桶尾
   执行。async 保真归位 AsyncModule → 引擎 Blocking 槽（s342 口径）；**注册面
   消费的单元不保真**——注册 shell 带 async 会提前到 Blocking 槽，注册序先于同
   批 async 单元执行，drain 窗定时器先兑现反转文档序。
3. **plain defer classic 条件归属（本切片返工定谳）**：WIP 首版 plain defer
   整段透传入引擎桶全域生效 → 全域 join 3P→F（087 序翻转/103 interactive 相位
   断言/114 src 变更 mutation 冻结）——s112 调度器 drain 相位（interactive 后）
   与注册 shell 无 src 形是这三案判据的承载。透传仅当页面存在 deferred module
   （execorder parsedOrdered：defer classic 与内联 module 文档序交错需同桶）。
   C5 条件化后 A/B 两轮全域 join 逐页取已验证绿行为：det P→F=0、F→P=4。
4. **动态插入内联 module 同步 classic-eval 吞任务化 prepare**（part04 R387/s95
   + part05 S4P 三臂）：M2 建臂时只建模 classic 内联（textContent 非空即
   `(0,eval)` 同步求值），`document.head.append(module)` 体在插入语句内跑完
   （probe10 实证 start|body|after-append 序），违背 spec 排队任务（#9864）。
   委派 part03 `_zwScriptMaybeStartDynamic` module 分支（already-started/
   connected/for-event 门重检幂等 + run-once 章同步置 + setTimeout 编译求值 +
   四态事件契约）。
5. **外链 classic 成功取回执行后元素 fire load 缺席**（load-error-events-1
   defer onload 臂 → 页面 Timeout）：s102 只补了失败 error 面，成功 load 面缺
   失——webview 批循环补 `script_dispatch_script_event(abs, "load")`（执行体抛
   错仍 fire load：spec report the exception 与 load 解耦）。runner 投递内联单
   元（Inline 形）不经本臂（s100 体尾 load 投递无双发）；动态 src 通道（shim
   prepare 家族）另派。
6. **probe9 预期数组按新相位重校准**（自建诊断 fixture，wpt-data 本地）：async
   保真后外链 async module 归位 parse 位，序 `[c0,a1,a3,c5,m2,d4]` =
   blocking 文档序 + deferred 桶尾，确定性成立。

## 施工

| 面 | 位置 | 内容 |
|---|---|---|
| 内联 async module 注册 | runner `testharness.rs` | 无 src + async + 体内静态 import 携 trickle → `transform_inline_module_async_sched`：裸副作用形 `import<ws>"spec"` 改写 `await import()`（绑定形/再导出/动态调用返 None 维持透传），延迟 = 体内 spec max trickle，注册 `__zwSchedRegister("async", delay)` + load 投递尾 |
| 外链 async module 注册 | runner 同上（resolved Some 臂） | module async + tail load → 体改装 + src/body max trickle 并入注册；延迟 0 不注册 |
| 投递 async 保真 | runner 同上 | 非注册消费的外链 module 单元投递 shell 保真 `async` 属性（extract AsyncModule → Blocking 槽） |
| plain defer 条件透传 | runner 同上 + `page_has_deferred_module` | 页面存在 deferred module（type=module 无 async，扫描与主循环同构——体不透明）时 plain defer（`("defer",0)`）整段透传入引擎桶；否则维持 s112 注册面。trickle defer 恒注册 |
| 动态 module 任务化 | engine shim part04 R387/s95 臂 + part05 S4P 臂 | `_zwScriptTypeClassify` 分类一次捕获，module 型委派 `_zwScriptMaybeStartDynamic`（不回落 classic eval） |
| 外链 classic load 派发 | webview `webview.rs` 批循环 | 成功取回+执行 → `script_dispatch_script_event(abs,"load")`（s102 error 面对偶） |
| 单测 | runner 测试模块 | `transform_inline_module_async_sched_bare_import_rewrite_s343`（改装形矩阵）+ `inline_async_module_srcless_trickle_registers_s343` + `plain_defer_route_conditional_on_deferred_module_s343` + s112 分类测试回填（无 module 页 4 注册） |

## 验证

- solo（final binary，test-guard + RAF_FRAME_DRIVEN）：inline-async-execorder
  1P（目标翻绿，probe13 dump `[fast@2,fast@5,inline-fast@5,slow@6,slow@8,
  inline-slow@8]` 三 fast 全先于三 slow）+ probe8/9/10 + execorder 8P +
  inline-async-inserted 1P + non-external-no-import 1P + mvs 2P + lee1 11P +
  lee2/ido/script-defer/090 各 1P + charset-01 12P——087/103/114 回归修复后
  同绿。
- 单测（test-guard 包裹 `cargo test -p zero-wpt-runner`）：**294P/0F**（新增
  三 s343 测试 + s112 回填）。
- clippy：`make guarded-clippy -p zero-wpt-runner` 净；rustfmt 净。
- 全域 key-state join（`grep -a` 全文本态——含控制字节子测试名会被 GNU grep
  二进制判定截断；子测试级 `(key,state)` sort -u 去重后按 stable-P/stable-F
  确定性集求交，双态/Timeout 沾染键不入集）：
  - the-script-element 子域：1073P/40F/9T → **1080P/38F/9T**（+7P/−2F）——F 集
    差逐键 = −4 目标案 + 2 自建 probe dump 恒红案（probe12/13，wpt-data 本地非
    语料），语料口径 **40F→36F** 全数目标面。
  - 全域：8072P/320F/48T → 8099P/322F/46T——det P→F=**0**，det F→P=**4** 逐案
    吻合；headline 漂移归因 = checkbox `checkable-active-onblur-with-click`
    :active 案（B2 全域红、solo×3 同 binary 全绿 + B1 全域绿 → 并行负载时序
    flake，s292 口径）+ select-home-end-pagedown-pageup-detailed.optional
    （s342 在册 page-timeout 边界 flake，A 侧未上报键）+ probe 自建案 + T 边界
    churn（48→46）。

## 挂账（承继 + 新记）

- 引擎 fetch 完成驱动 ASAP 调度臂未建（s342 ④ 原样承继）：本切片以 runner
  注册面近似，引擎侧 async module 仍 Blocking 槽 parse 位——取回发起时点偏差
  （prepare 时发起 vs 执行位发起）原样承继。
- renderer 产品路径 deferred parity（s342 新记原样承继）：renderer 面沿用
  `extract_page_scripts_indexed` 2-tuple wrapper，调度面未同步。
- s341 既有挂账原样承继：iframe 子文档脚本不设 currentScript；window.onerror
  per-fire window.event；credentials setTimeout 变体（C3 同族）。
- 域长尾勘向：the-script-element 语料口径 36F（execution-timing 041/056/063/
  068 等文档写入面 + module 簇 single-evaluation/evaluation-error/late-*-request
  + fetch-src empty 双案）+ 9T（credentials setTimeout 变体/fetch-src empty/
  load-error-events-3/script-crossorigin-network.sub）。
