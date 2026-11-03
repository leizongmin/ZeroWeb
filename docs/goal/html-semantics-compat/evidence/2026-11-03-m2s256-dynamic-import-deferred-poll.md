# M2 s256 — 动态 import 延迟决议（deferred contract + 到期 poll 泵；1 键 F→P）

日期：2026-11-03。切片号 s256（承 s255 候选 #1）。目标键全 F：
delay-load-event.html（native 通道 classic 面动态 import，trickle 慢模块
时序族）。红态 solo：`got ["import start", "slow"] length 2`
（evidence/logs/s256-baseline-delayload.log）。

## 勘根（含 s253 判读勘误）

1. **s253 挂账机制描述勘误**：s253 evidence 记「slow-module
   `?pipe=trickle(d2)` 冻结 JS 回合 2 秒」——**不成立**。runner 脚本
   fetcher（`wpt_data_script_fetcher`）忽略 trickle 管道照常即时返回
   （testharness.rs 注册处注释自证「其余 pipe 指令（trickle 等）不涉
   MIME 照旧忽略」），slow-module.js 取回无延迟。真实机制：native
   通道 `dynamic_import_callback`（s88，`set_host_import_module_
   dynamically_callback`）在脚本回合内**同步**调宿主 resolver 取回+
   编译+求值并 resolve——`import().then` 断言微任务在 execute 末
   checkpoint 即排空，早于 window load 事件与 1s `step_timeout`，
   loaded 序列只积到 `['import start','slow']`。spec 判据：动态
   import promise 在 module script 取回完成前保持 pending、取回不阻
   塞 load 事件链——断言要求 import 完成时刻晚于 load 事件与 +1s
   定时器（慢源 2s > 1s > 0s 序）。
   https://html.spec.whatwg.org/multipage/webappapis.html#hostimportmoduledynamically-retrieve-promise
2. **通道归属**：delay-load 页为 classic inline `<script>`——
   `rewrite_dynamic_imports` 仅作用于 module 管线 transform，classic
   面不可达；import() 走 V8 原生回调通道（shim `__zw_dynamic_import`
   helper 通道不涉）。修复面在 native resolver 契约层。

## 施工（4 刀 + 1 钉面）

script-sandbox（v8_runtime.rs）：
1. **A** `PendingDynamicImport`（sandbox_key/due/spec/contract/resolver
   Global）+ `PENDING_DYNAMIC_IMPORTS` thread_local 挂起队列——
   `v8::Global` 非 Send 不能进 `Arc<Mutex<DynamicImportState>>`，存放
   形态与 PENDING_REJECTS/HOST_CALLBACKS 同款；sandbox_key =
   `Arc::as_ptr::<DynamicImportState>`（isolate slot 与 sandbox 字段
   共享同一 Arc 分配，同线程多 sandbox 存活期重叠时 poll 只取本
   sandbox 条目）。
2. **B** 回调延迟包裹臂：宿主 resolver 契约形 `1\x1f<delay_ms>\x1f<inner>`
   （inner 与四态契约同形）时挂起条目 + 返回未决议 promise。前缀判定
   无歧义（未包裹契约首段是宿主解析 URL，不会以 `1\x1f` 起头，同
   s252 `3\x1f` 判据口径）；锁毒化条目丢失 → promise 恒 pending
   （取回挂死同形，不放大）。
3. **C** settle 体抽出共享：四态解析 + s124 module map 记忆化 + IIFE
   编译求值 + promise 决议 → `settle_dynamic_import`（回调同步臂与
   到期 poll 臂共用；v8 150.2 API 恒取 `&PinScope`，TryCatch deref
   到 PinScope，单签名覆盖两调用点）。
4. **D** `V8Sandbox::poll_pending_dynamic_imports() -> bool`：取走本
   sandbox 到期条目（队空零成本早退——execute 头部每跳调用热路径不碰
   isolate），IsolateEnterGuard 栈式 enter/exit + 逐条 WatchdogGuard
   Arm/Disarm + settle + 每条后 `perform_microtask_checkpoint`
   （t-p103 D2-1 同款逐跳语义——`.then` 链整链排空后才下一跳）；
   超时终止臂 `cancel_terminate_execution` + reset、promise 保持
   pending 不毒化后续 execute。返回 true = 队列仍有未到期条目。
   `V8Sandbox::Drop` 按键清空本 sandbox 残条（PENDING_REJECTS
   drop-即清保险同款；键过滤不伤并存 sandbox）。
5. **E** trait 面：`Sandbox::poll_pending_dynamic_imports` 默认 no-op
   （QuickJS 后端维持不支持态）+ V8 委托。

webview（webview.rs）：
6. **F** `WebViewConfig::dynamic_import_fetch_delay`（`DynamicImportFetch
   Delay = Arc<dyn Fn(&str) -> u64>`，输入解析 URL；None = 无延迟）+
   native resolver closure 在 CSP 检查后同步算好契约按延迟值包裹（CSP
   阻止面维持同步——阻止即拒绝不延迟；取回失败契约同样延迟，慢源失败
   照慢语义）+ 完成泵挂三探针点：execute_script / execute_script_with_dom
   头部（`drain_next_async_callback_if_pending` 前）与
   drain_pending_timers_until_idle 循环头。

runner（testharness.rs）：
7. **G** WebViewConfig 注册 `dynamic_import_fetch_delay` =
   `trickle_delay_ms`（s112 解析器复用：`pipe=trickle((d)N)` 秒×1000、
   30s 封顶）——trickle 管道建模为真实网络秒级延迟，慢模块时序族从此
   可观察。

钉面（v8_runtime tests）：
8. **H** `test_dynamic_import_deferred_contract_pending_until_poll_s256`
   （persistent 配置镜像生产页面 sandbox）：40ms 包裹 → execute 返回
   promise 未决议（红态此处被同步 reject SyntaxError——包裹契约按普通
   split 解析失败）→ 未到期 poll 返 true 不 settle → 到期 poll 返
   false、resolve namespace、模块体恰好一次求值（`__s256==1`）。
   红态实证：neutralize 包裹前缀臂后首断言挂（旧版本因目标行为失败）。

## 验证

- **单测**：script-sandbox **385 全绿**（384 基线 + 1 新钉，红态实证
  在账）；webview **834 过 + 1 失败 = HEAD 既有**（image_decoder
  proxy_mode_decodes_png_correctly——stash A/B 定谳 HEAD 同挂，非
  s256 回归）；runner 单测 **281 全绿**（--test-threads=2；8 线程
  默认并行跑 corpus 单测进程峰值 4.2GB 撞 test-guard 4GB 单进程上限
  被杀——既有负载形态非本切片引入，降并行后干净全绿）。fmt/clippy
  三 crate（script-sandbox/webview/runner）零告警（含 useless_format
  钉面返修一刀）；quickjs-only clippy 干净。
- **solo 终态**（evidence/logs/s256-solo-delayload.log）：delay-load-
  event **F→P**（"Dynamic imports don't delay the load event."）。
- **家族邻接面**（s256-solo-module-dir.log，module 目录全量 271 行）：
  dynamic-imports-fetch-error 8 子测试全 Pass、credentials/shared-
  dependency/TLA 家族维持基线 F/T 态（在账挂账不动）。
- **全树 A/B**（s256-ab.log vs s255-ab.log，窗口独占）：行级
  7793P/440F/77T → **7794P/439F/77T**；键级 (key,state) 对
  **8307→8307，LOST 1 / GAINED 1，迁移恰 1 键 = 目标键 F→P 全等**
  （delay-load-event），零其他位移。

## 已知简化（在账）

- 同 spec 并发动态 import 的挂起条目不 dedup（各自挂条、各自 settle
  求值——s124 缓存只在同步臂命中面生效；corpus 单 import 面，多并发
  慢 import 是后续挂账）。
- shim `__zw_compile_module` 通道不包裹延迟（无语料目标；shim fetch
  通道 JS 侧 setTimeout trickle 已有 s112 先例，module 面未建模）。
- 非持久 context 的 poll settle 落在 poll 时刻 resolve_context 所得
  context（非持久模式跨 execute 本就换 context，页面面恒 persistent）。

## 挂账更新

- **credentials 面**（承前不动）：dynamic-imports-credentials ×12
  + setTimeout 变体——跨域 fetch 计数/credentials 语义，独立切片。
- **shared-dependency 取回计数面**（承前不动）：concurrent-failed-
  imports（失败模块恰一次取回）/shared-dependency-repeated-import。
- **microtask checkpoint 排空时序族**（承 s255 不动）。
- **并发慢 import 挂起条目 dedup**（新记档——见已知简化）。

## 下一步候选

1. renderer/tab 宿主 import.meta.resolve 接线（小刀，补多进程面对
   称；s254 起 3 轮在账）。
2. data-zw-harness 投递标记保真面（s255 候选 #3 承前）。
3. import maps 剩余语义面（多张 map / 动态插入 / Blocked 降级，
   s254 起在账）。
