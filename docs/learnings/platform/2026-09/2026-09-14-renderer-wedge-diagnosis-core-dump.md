---
date: 2026-09-14
modules: apps/renderer, apps/browser, crates/engine, crates/paint-convert
---

# renderer 死锁诊断：ptrace 受限 + test-guard 禁 core 下的取证方法，与 baidu 主循环冻结的根因链

## 问题描述

headless ZeroWeb（`zero-browser --headless` + Playwright connectOverCDP）导航到
`https://www.baidu.com/` 后：主文档到达（title/DOM 正确、readyState=complete），
但自动化 `evaluate` 间歇→全部超时，子资源观测缺失（resource timing=0、console=0、
Network 域无事件），`goto` 的 load 等待永不返回。gdb attach 报
"ptrace: 不允许的操作"，renderer 子进程日志零输出，现象极易误判为「站点触发的
引擎死循环」或「测试环境故障」。

## 根因分析

### 为什么日志静默：renderer stderr 根本不落地

`crates/protocol/src/process.rs` 的 `spawn_stderr_reader` 把 renderer stderr
截进 **16KB 环形 tail**（`STDERR_TAIL_LIMIT`，供崩溃报告），不转发到 browser 日志。
因此任何「看 renderer 日志排查」的尝试在 headless 多进程模式下都是徒劳——
不是没有日志，是日志进了一个只在崩溃时可见的环形缓冲。

### 线程级取证链（本轮实际路径）

1. `/proc/<pid>/task/*/comm + wchan + stat`（无需 ptrace）：区分各线程状态。
   卡死样本：`renderer-runtim`（主循环）`__futex_wait`、`renderer-js-1`
   futex/空闲交替、`reqwest-interna` `epoll_wait`、`renderer-ipc-in`
   `anon_pipe_read`。瞬时 CPU 0% ⇒ **阻塞等待**而非死循环。
2. 文件探针插桩（临时）：renderer 的 stderr 探针同样进 tail——探针必须
   **直写文件**（`OpenOptions::append` + env 指定路径）。browser 进程无此问题
   （stderr 直接继承到启动方日志）。
3. 时间线交叉（browser 探针 + renderer MARK/IPC 探针写同一文件）：
   - `automation_request enter → TIMEOUT(10s)` 成对出现 ⇒ browser 会话层健康
     （有 10s deadline），是 renderer 侧不回响应。
   - renderer 侧 `IPC-IN got AutomationRequest → ROUTER fwd` 存在，
     但主循环 `recv_next_or_timeout` 后的 `dispatch` 标记缺失 ⇒ 消息已入
     主循环通道并被 dispatch，卡在 **dispatch 处理内部**。
4. gdb 离线 core 分析（本文核心技巧，见下）：拿到全线程用户态栈。

### 最终栈证据（gdb core.\<pid\>）

- 主循环线程（renderer-runtim）：
  `run() → run_page_scripts(after_page_html_loaded_with_cache) →
  RendererJsWorker::spawn_with_handlers::{closure#1} → mpmc recv_timeout`
  ——主循环同步等页面脚本执行结果。
- JS 线程（renderer-js-1）：
  `v8 host_callback_invoke → js_dom_bridge::callbacks::register_dom_callbacks
  ::{closure#N} → QUERY_DOC_CACHE（with_query_doc 全文档 parse_html）`
  ——JS 线程在查询缓存的**全文档重解析**内做原生工作。

### 根因链（P1）

baidu 页面 JS 以「mutation + query 交替」的高频模式运行；
`QUERY_DOC_CACHE`（1 条目、以 html 全文为键）在每个 mutation 后必然 miss，
每次查询触发**整文档 `parse_html`**（每次还要 clone 整份 html 作键）。
查询风暴 × 全量重解析 ⇒ 单个脚本执行拖到分钟级。V8 watchdog
（`timeout_ms=15s`，`terminate_execution`）**无法打断宿主回调内的原生代码**——
terminate 只能在回到 JS 边界时生效，而解析正是原生代码。主循环
`run_page_scripts` 同步等待该脚本 ⇒ 整条 IPC 管线（自动化响应、子资源推进、
生命周期事件、绘制发布）全部停摆。`readyState=complete` 与
「管线冻结」并存即由此而来。

## 解决方案

**已验证有效的诊断方法（可复用）**：

1. ptrace_scope=1 + 非 root：gdb attach 不可行，但
   **`ulimit -c unlimited` 裸启动（绕过 test-guard 的 `prlimit --core=0`）
   + `kill -ABRT <renderer>`（SIGSEGV 会被 V8 的 wasm-trap handler 吞掉，
   SIGABRT 不会）+ `core_uses_pid=1` 生成 `core.<pid>`** +
   `gdb -batch -ex "thread apply all bt" 二进制 core` 离线解析，绕开全部限制。
   release 二进制默认未 strip，函数名可读。
2. renderer 探针必须直写文件（env 指定路径），写 stderr 无效（见上）。
3. 双进程探针写同一日志文件，按写入序交叉读时间线，一次定位「消息断在哪一层」。

**修复方向（结构性，待立项）**：

- 查询缓存重设计：mutation+query 交替模式下消除全文档重解析
  （live doc 命中率、增量应用、或带过期语义的快照——需规范权衡拍板）；
- 宿主回调可中断/限预算：watchdog 对 `__zw_*` 同步回调内的原生工作无能为力，
  需要回调级预算或可取消解析；
- 兜底：`run_page_scripts` 脚本阶段总墙钟预算（超时跳过剩余脚本、降级完成）。
  注意：只对「多脚本各自完成但总量大」的页面有效；对本例「单脚本原生回调内
  无限阻塞」无效（已实测）。

## 如何避免

- headless/多进程排查的**第一步**是确认日志去向（renderer stderr=环形 tail），
  不要把「日志静默」当成「没有活动」。
- 「阻塞 vs 自旋」先用 `/proc/*/task/*/wchan + stat` 一次性判明，再决定
  插桩还是采栈。
- 阻塞型同步等待（`mpsc recv()` 无超时）出现在「主循环同步等 JS」或
  「JS 同步等宿主桥」时，都是 learning #24/#15 死锁族的变体；
  新增此类等待必须同时回答「被等待方由谁推进」。
