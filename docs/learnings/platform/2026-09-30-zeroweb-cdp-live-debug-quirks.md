---
date: 2026-09-30
modules: apps/renderer,apps/browser,crates/protocol
---

# ZeroWeb CDP live 调试坑点（/json/new、Page.reload、renderer stderr 环、探针退化）

## 问题描述

用 CDP 探针对运行中的 ZeroBrowser 做 live 诊断（页面世界注入、console/网络采集）时，
多个 Chrome 惯用端点与惯例不可用或语义不同，且 renderer 日志默认不可达，排查
「取回成功但页面收到 error 事件」一类跨进程问题时极易空转。

## 根因与分析

1. **`/json/new` 不可用**：GET 返回非 JSON 体（早期版本），PUT 直连被 RST——
   ZeroWeb CDP 未实现建 tab 端点。诊断脚本须从 `/json` 列表复用既有 page tab。
2. **`Page.reload` 未实现**（`Unknown method`）：重新加载用
   `Page.navigate`（同 URL + cache-busting query）替代。
3. **renderer stderr 不落浏览器日志**：`zero-protocol::process` 用
   `Stdio::piped()` + 16KB 环形缓冲（`STDERR_TAIL_LIMIT`）只保留尾部，仅在
   renderer **退出时**随 crash 报告输出（`format_renderer_exit`）。renderer 的
   `tracing` 全量走 stderr（无 EnvFilter，main.rs 直接 fmt+stderr init），因此：
   - 平时**无法**看到 renderer 日志；
   - 想看某段日志，须在目标事件发生后**尽快 kill renderer**（`kill -9`，浏览器
     会 respawn 并把 stderr 尾巴打进 browser 日志）。但 baidu 等重绘刷屏约
     7KB/s，16KB 环只够 ~2 秒历史——**用本地静态页（几乎零日志）复现**才能保住
     目标行。
4. **`addScriptToEvaluateOnNewDocument` 跨导航持久 + 同 URL 匹配全元素**：注入的
   探针在之后**每次**导航都会再跑（含重定向的中间文档）；元素级 load/error 事件按
   src 绝对 URL 匹配**所有**同 src 元素——多轮探针叠加时计数互相污染。探针须带
   runId + `location.href` 逐事件上报，不要用聚合计数器。
5. **renderer respawn 后 CDP 事件流退化**：kill renderer 后旧 ws 会话的
   `Runtime.enable`/`Network.enable` 可能收不到任何事件（仅 executionContextCreated）。
   诊断会话应「一次浏览器生命周期内完成」，跨 respawn 后重新附着不一定恢复。
6. **页面 fetch 与脚本资源通路不同源**：页面 `fetch()` 走 renderer 进程内
   `ResourceLoader::shared()`（线程池），脚本/文档/图片走 IPC fetch_proxy（browser
   进程 zero_net）。CDP Network 事件只反映 browser 侧通路——「Network 200 但页面
   收 error」时先分清响应走了哪条通路。
7. **pkill 自匹配**：复合命令里出现明文进程名（如 `zero-browser`）会杀掉自己的
   bash 包装器（exit 144）。用字符类技巧 `zero-browse[r]` 或按精确 PID 杀。

## 解决方案

诊断脚本模板：复用 `/json` 既有 tab → `Page.navigate`（非 reload）→
`addScriptToEvaluateOnNewDocument` 注入带 runId/href 的逐事件探针 →
`Runtime.consoleAPICalled` 采集 → 单次生命周期内完成。需要 renderer 日志时在
静默页面上复现后 kill renderer 读 stderr 尾巴。

## 如何避免

- 对 ZeroWeb 写 CDP 诊断前，先按本条核对端点可用性，勿照搬 Chrome 惯例。
- 跨进程「网络成功但页面失败」问题，先按第 6 条分流通路，再找响应路由 bug。
- 无人值守杀进程一律 `pkill -f 'xxx[r]'` 形式或按 PID。
