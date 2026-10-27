# M2 切片 108：strict 首错聚合（后续脚本照跑）+ runner 短路收窄 + 投递 id 保真

日期：2026-10-27。范围：`crates/webview/src/webview.rs`（strict abort→首错聚合 + 收尾段单一出口 + 1 回归钉）+ `tests/wpt-runner/src/testharness.rs`（id 保真 + declared>0 放行收集 + 3 回归钉）。上轮：[M2 切片 107](2026-10-27-m2s107-json-module-sri.md)。

## 1. 入场定谳

- s108 探针（承 s107 下一步三候选）：**候选 A（css/text 数据模块合成器）证伪**——wpt-data（独立 repo，pinned tag）缺 css-module/resources/basic.css 等五个 CSS 资源与 text-module/file.txt，fetch 恒 404，切片不可达；本地补语料属基准变更（guidelines #31 语料红线）记待决清单，不阻塞其他工作。s107 账本「~30 行」估计据此修正。
- execution-timing 余 46F+1T 按根因分组：①122-126 族 getElementById null（runner 装配丢 id）；②124/144/145 mutation 重触发同步执行缺失（机制面，挂 s109）；③142 write 臂 xlink:href 不识别；④143 语料故意 SyntaxError 被 runner 升级页级 Fail。本轮收 ①④ 与 ④ 的执行面根因，② 留 s109。
- 143 时序细节：async_test 头段声明、onload 的 log 顺序断言在轮询 tick 执行——抛错时刻 pending=1 属正常中间态，runner 放行判据用 declared>0（非 pending==0）。
- 结构性零 P→非P 论证：刀2/刀2b 只影响「有脚本抛错」的页——旧口径下此类页恒短路 Fail（declared==0）或整页 abort，从无 Pass；方向只会 Pass/Timeout/维持 Fail。刀1（id 保真）经终扫行级 diff 实证无 P→非P。

## 2. 三刀

- **刀1（runner 投递 id 保真，testharness.rs）**：`inline_local_scripts` 投递标签保留原标签 `id`（`attr_from_open_tag` 提取 + `&`/`"` 转义，与 nonce/nomodule/type/onload/onerror 同族「内联投递丢属性改语义」保真）。execution-timing 122-126 族以 `document.getElementById("test")` 取回 `type=text/plain` 外链单元做 removeAttribute(type)/re-append 的 mutation 重触发，id 丢失即 null 崩溃、mutation 面不可达。
- **刀2（runner 短路收窄，testharness.rs）**：脚本抛错臂读 `__zw_harness_state()` 的 declared——`declared==0`（无 testharness 引用）维持短路 Fail（s102 防假绿口径不变：脚本抛错不能按「未注册 test()」误判为通过）；`declared>0` 放行轮询收集（WPT 语义：uncaught exception 报 window.onerror、不降级 test status；已完成测试照收 Pass、未完成挂 deadline 判 Timeout）。
- **刀2b（webview strict 首错聚合，webview.rs）**：`run_page_scripts_impl` 的 strict 两处 `return Err`（sentinel 臂 + execution/clear 臂）改 `first_strict_error.get_or_insert_with` + `continue`——真浏览器语义 script 元素相互独立，一个抛错其余照跑（旧行为整页 abort 使其后的 onload 赋值等独立脚本不执行，143 的 onload 断言因此挂 pending）；循环末聚合返回 Err（收尾段 CSP img/violation 派发、mutation 应用已全部执行，Err 仅承载错误信息供 runner 防假绿短路判定）。
  - **收尾段单一出口重构（刀2b 配套）**：旧 `!pending` 分支在聚合判定之前 `return Ok` 早退——无 DOM 变更的纯断言页（execution-timing 主体形态）首错被静默吞掉（新钉首轮红灯根因：三 script 页全走此早退，`__before`/`__after` 照跑、result Ok、sentinel 被后续 wrapper 序幕重置）。改 if/else 汇合后统一 drain + 聚合判定 + 返回。

## 3. 验证

- 单测：zero-webview lib **814/814**（+1 钉 `strict_classic_throw_continues_remaining_scripts_s108`：三 script 页断言 is_err 且 before/after 均执行）；zero-wpt-runner bin **233 passed / 0 failed**（+3 钉：投递 id 保留 / 抛错+declared>0 照收 Pass 且无「page script threw」/ 抛错+declared==0 维持短路 Fail——stateful MINI_HARNESS 变体带 tests 注册表 + test_environment stub，IIFE 形参 `global_scope` 与 completion patch 注入引用一致）；fmt clean；clippy webview + wpt-runner `-D warnings` clean。
- 定向 FILTER=execution-timing（对照 s107 基线）：非 P 页 **45 → 41**（**4 页销账**：067 语法错标 has-run（刀2+刀2b——语料故意埋 `log(This script will never run..` + allow_uncaught_exception）与 104/114/115 defer mutation 族（刀1——三页均 `getElementById("defer-script")`，id 保真后 mutation 在既有重扫面生效）；122 null 崩溃消除进第二层（页级 Fail → 2 子测试断言级 Fail：async IDL 属性 + mutation 重触发 scheduler 臂，挂 s109）；143 崩溃形→断言级 Fail（log 顺序断言真正执行，缺 inline script #3——write 嵌套内联重触发，挂 s109）。
- 全子树终扫（FILTER=scripting-1 口径，与 s107 基线同）：**377P/173F/37T+15N**（s107：372P/175F/35T+15N，**+5P**）；行级归一 diff（/tmp/s108-final-scripting-1.log 存档，基线 /tmp/s107-final-scan.log）：**零 P→非P**（diff 无任何 Pass 行消失）——4 Fail→Pass（067/104/114/115）+1 新增子测试 Pass（evaluation-order-1 的「Module script queueing a microtask then throwing an exception」臂，刀2 直接战果）；Fail −2 净额核账（−4 销账、122/checkpoint-after-window-onerror 各 +1 子测试 Fail、evaluation-order-1/delay-load-event-1 页级 Fail 换形子测试 Fail 相抵）；Timeout +2（evaluation-order-1/delay-load-event-1 页级落 deadline——均原 Fail 页，declared>0 放行后未完成部分挂 deadline 的预测形）。microtasks/moving-between-documents 的三处非 P 形态重排全为原 Fail 页短路→子测试级分解，无新失败页。
- 依赖变更：无。渲染面核对：本轮纯执行/收集面（script 独立性 + 投递保真 + 收集口径），无布局/绘制依赖，reftest/product-smoke 未跑。

## 4. 挂账

- **s109 主候选：mutation 重触发机制面七行**（122 第二层 async IDL+时序、123/124/125/126、144/145——嵌套执行 + 重扫 diff，共享主循环高风险面，需独立切片细设计）；142 write 臂 xlink:href；143 的 eventOrder 完整核验（write 嵌套内联）。
- 063 Timeout（runner 脚本记账，承袭）；slow-loading 外链时序 ~11 行（pipe=trickle 建模，承袭）。
- **待决清单（须用户拍板）**：css/text 数据模块语料资源缺失（basic.css/basic-large.css/bad-import.css/malformed.css/parse-error.css/file.txt 不在 wpt-data pinned tag）——补语料属基准变更（guidelines #31 语料红线），本轮证伪 s107「~30 行」估计。
- 承袭挂账不变：parse-error 族、http-accept-header、referrer-policies、dynamic import() with 透传、classic 外链 SRI、collect 账本打包重构、接口对象基建面、FIXME 组、/tmp/zw-hb*.txt 插桩来源待查。
- 本轮销账：execution-timing 067/104/114/115（4 页）。
