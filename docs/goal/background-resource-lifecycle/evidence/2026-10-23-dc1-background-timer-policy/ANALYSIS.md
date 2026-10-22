# DC-1 续：后台 timer 节流策略面（钳制落页面 timer API 入口，IDB 内部任务免钳）

**运行身份**：HEAD b6c6ac515（可见性消息切片）+ 本轮工作区。
**策略锚**：HTML spec 不强制后台节流数值；1s 对齐 Chromium hidden-page timer clamp 同族策略。DC-1 验收「后台策略明确且恢复正确」的页面策略面。

## 策略语义（明确面）

- **钳制**：页面隐藏（`__ZW_HIDDEN === true`，SetVisibility 链注入）时经页面 timer API arm 的定时器 delay < 1000ms 一律抬到 1s——setTimeout / setInterval re-arm / requestIdleCallback 三族都经 shim `_zwClampDelay` 单口（与 nesting 4ms 钳制叠加取 max），在 shim 公共 API 入口判定；长 delay 不动。
- **内部任务免钳**：shim 内部异步机制（IndexedDB 事务推进/完成轮询、请求事件派发 `_zwIDBDispatch`、versionchange/blocked、upgrade 重试、deleteDatabase 收尾）是 event-loop task 而非 timer task，spec 中不受 hidden timer 节流——迁到 `_zwInternalTask`（绕开公共 setTimeout 直接 arm host 0ms，`__zwtask:` 键空间与 timer/interval/rIC 隔离，fire-once 即焚；无 host 回落 `_defer`）。IDB 事务推进被钳会 1s/拍爬行（首轮设计实测，见「设计返工」）。
- **恢复**：re-arm 时点判定（interval 每代 wrapper 重读 `__ZW_HIDDEN`），已入队 entry 不重排——恢复可见后首个 pending entry 至多迟到 1s，随后新 arm 自动回原延迟，无需显式恢复动作。
- **不做**：intensive throttling（长链 5 分钟后 1/分钟）未实现；已入队 entry 的恢复重排未做（迟到 ≤1s 语义）。

## 设计返工（首轮桥级钳制被多进程集成测否决）

首轮把钳制放在 TimerBridge `__zw_setTimeout` 回调（arm 时点单口，`page_hidden` Arc 三方共享）。browser 多进程单测 `cross_renderer_transactions_wait_for_conflicting_scope` 稳定红归因定位：该测试的 `execute_script` 辅助每轮 `poll(Some(当前 tab))`——脚本切到 reader tab 时 poll 差分给 writer 发 `hidden=true`，writer 在隐藏窗内 arm 的 IDB 续体（事务推进经 shim `setTimeout(…, 0)`）被抬到 1s；`wait_for_value` 哨兵（pending/undefined）见陈旧 `holding` 首拍短路即失败。二分证据链：我树 renderer 二进制 4/4 红、HEAD renderer 二进制绿、engine 单独还原绿——根因是「IDB 内部任务与页面 timer 共用 `__zw_setTimeout` 单口」被桥级钳制误伤，非测试编排缺陷。返工：钳制上移 shim 公共 API 入口（`_zwClampDelay`），TimerBridge/js_worker/runtime 全部还原 HEAD（无 Arc 共享），IDB 内部派发整体迁 `_zwInternalTask`（part02 共 13 处）。返工后同测 0.47s 绿。

## 链路

`SetVisibility` IPC（browser poll 前后台差分，上轮切片）→ renderer `handle_set_visibility` → `__ZW_HIDDEN` 宿主全局（跨导航重注入；rAF 泵门控/文档 getter 同源）→ shim `_zwClampDelay` arm 时点读。JS 关闭也翻转（节流不依赖 JS 开关）。

## 证据

1. **shim 单测**（`js_dom_bridge_tests/part01.rs test_background_timer_clamp_hidden_page_dc1`，记录式 host stub）：可见 5ms 原样；隐藏 5ms/0ms/rIC/interval 0ms 全抬 1000、5000ms 不动；恢复后 5ms/0ms 回原值。`_zwInternalTask` 免钳不在本测（闭包内部函数脚本不可直呼），由浏览器多进程集成测端到端钉住。
2. **renderer 集成测**（真 SetVisibility IPC 臂 + 真 shim interval 链，`visibility_tests.rs background_timer_policy_throttles_interval_and_recovers`）：可见 500ms 窗 ≥10 拍（~4ms/拍 nesting 钳制节奏）；隐藏 2200ms 窗 ≤3 拍（1s/拍，翻转后静置 100ms 排空残留 entry 再起测窗）；恢复 2200ms 窗 ≥10 拍（首拍 ≤1s 后回 4ms 节奏）。
3. **浏览器多进程集成测**（`process_backend` owner 测试 `cross_renderer_transactions_wait_for_conflicting_scope`）：writer 跨 tab 隐藏窗持有事务、reader 冲突等待、释放后 complete——内部任务免钳的端到端 oracle（正是它抓出首轮设计缺陷）。
4. **嵌套钳制回归**（同文件两测）：nesting 4ms 链与 interval re-arm 钳制不受本改动影响。

## 已知边界（后续切片）

1. **worker 面不在此策略**（script-sandbox worker timer 无可见性语义）——dedicated worker 是否随属页可见性节流随 worker 可见性语义切片处理。
2. **单进程 in-process 路径不节流**（tab_js_worker 路径无 `__ZW_HIDDEN` 注入，shim 读 undefined 恒可见）——与可见性链同构的多进程-only 边界；GUI 生产恢复腿（M4）补生产证据。
3. **headless 生产腿不可驱动**：headless 会话自有 renderer 驱动面不经 `ProcessTabBackend::poll`（上轮 bg01-background 负结果归因同源），后台节流的生产 CPU 对照归 M4 GUI 腿。
4. **IDB 之外的 shim 内部 setTimeout 用户未迁**（part02 AbortSignal/FileReader/history、SW 轮询，part03–06 事件/媒体等）：隐藏页这些机制的 0ms 派发会 1s/拍爬行（语义上是 task 应免钳），本轮只迁有测试实证危害的 IDB 子系统；全量测试绿表明无其他实证危害，后续按证据逐子系统迁移。
5. **intensive throttling 未做**：隐藏页长链 timer 的分钟级下限（Chromium 5min→1/min）不在本切片。
