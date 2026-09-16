# M1 — frontend 附接 ZeroWeb：S1 路由切片 + S2 附接试验台 + S3 域缺口账本

**日期**: 2026-09-16（M1 首轮，M0 收口后）
**上游状态**: cdp-protocol goal Done（M5 守成态，`make cdp-e2e` 33 绿 = 本 goal 防回归门）

---

## 1. M1-S1 ✅ per-page WS 路径路由（本切片落地）

Chrome 形态 `devtoolsFrontendUrl` 携带 per-page ws 路径，frontend 在该 socket 上说
**非包裹的 page 域协议**。本轮 headless transport 面三件：

1. **升级路径捕获**：WS 升级请求行路径提取（`discovery::extract_request_path`），
   `/devtools/page/<targetId>` 前缀 = DevTools frontend 页面直连形态登记
   （`devtools_serve::PAGE_WS_PREFIX`）。当前单会话模型下 flat 命令本就路由到全局
   会话，page-direct 暂为形态登记 + 日志（target 级隔离随多标签模型补）。
2. **per-tab devtoolsFrontendUrl**：bundle 已配置时 `/json` 逐 tab 输出
   `/devtools/inspector.html?ws=<addr>/devtools/page/zeroweb-tab-<n>`（此前是全体
   同一 browser 级 URL）。
3. **idle 计时重置修复**：`idle_since` 从未在消息到达时重置（注释声称重置但代码缺失，
   推断为历史重构丢失）——任何 600s 无消息的连接在下一个轮询 tick 被杀。DevTools UI
   交互间隔可远超 600s，长活调试会话必被误杀；现 Text/Ping 到达即重置。

单测 +3（升级路径提取含 query 剥离 / per-tab URL 形态与 127.0.0.1 host 断言 /
既有 `/json` 占位形态回归）。门禁：`cargo test -p zero-browser` 463P/0F、
clippy -D warnings 零告警、`make cdp-e2e` 33 绿 deterministic YES（守成面零漂移）。

## 2. M1-S2 ⏳ 附接试验台（`attach-zeroweb-probe.mjs`，可重放）

对真实 frontend + 真实 ZeroWeb CDP（零 mock）：ZeroWeb headless（serve + per-page
ws）← driver Chromium 打开 frontend → 断言逐面板。**已实证**：

- per-page URL 发现 → frontend 加载 → 附接 ZeroWeb 页面 → Elements 面板树骨架渲染
  （`frontend-boot` / `frontend-elements-panel` 绿）。
- **Elements 树空**（无被调试页 `<html>` 节点）——根因见 §3 账本：frontend 的
  `DOM.getDocument` 被拒，树无数据源。
- Console/Network 演示流被 **单连接串行 accept 循环** 卡住：driver 第二次 goto 时
  前一连接未完全释放即撞队列（结构性限制见 §4）。

试验台过程性发现（供后续轮次避坑）：
- ZeroWeb net 栈不收 `data:` URL 导航（`net::ERR_FAILED`）——被调试页用真实 http 页。
- headless chromium `/json` 会列 `browser_ui`（webui-toolbar/omnibox）伪 page target，
  附接实验必须按 URL 过滤。
- Playwright locator 天然穿透 open shadow root；手工 `evaluate` + shadow 递归遍历在
  frontend 页面上有 stale-context 竞态（返回空数组），断言一律用 locator。

## 3. M1-S3 域缺口账本（frontend console -32601 清单，2026-09-16 实测）

frontend 附接 ZeroWeb 后 console 全量 `Request X failed -32601` 清单（39 条去重族）：

**核心缺口（Elements 树空直接根因，M1 下一切片靶点）**：
- `DOM.enable` / **`DOM.getDocument`**（frontend 建树唯一数据源）
- `CSS.enable`（+ `CSS.getMatchedStylesForNode` 族，样式侧栏数据源）
- `Page.getResourceTree`（frontend frame 树枚举）
- `Overlay.enable` / `Overlay.setShowViewportSizeOnResize`（高亮/视口框）

**enable 型 no-op 即可满足（frontend 只需 ack）**：
Profiler.enable / Debugger.enable / Debugger.setPauseOnExceptions /
Debugger.setAsyncCallStackDepth / Log.startViolationsReport /
Overlay.setShowViewportSizeOnResize / Emulation.setEmulatedVisionDeficiency /
Accessibility.enable / Animation.enable / Autofill.enable / Autofill.setAddresses /
Audits.enable / ServiceWorker.enable / Inspector.enable / Runtime.addBinding /
Network.setAttachDebugStack / Network.setBlockedURLs / Network.emulateNetworkConditionsByRule /
Network.overrideNetworkState / CSS.trackComputedStyleUpdates / CSS.takeComputedStyleUpdates

**Target 域缺口**：Target.setDiscoverTargets / Target.setRemoteLocations（frontend
对 per-page 连接也发——no-op ack 可满足）。

**结构判断**：ZeroWeb 现有 CDP 面是 **Playwright 形状**（backendNodeId 盒模型流），
DevTools frontend 是 **DOM-nodeId 树流**（enable → getDocument 建树 → requestId 逐节点
操作）。结构性缺口 = `DOM.getDocument` 全树序列化（nodeId 分配 + child/shape 契约）+
`CSS.getMatchedStylesForNode`（style-system 集成）。前者量级可控（zero-dom 树 API 在
位），后者是 M2 之前最大单件。回流策略：按 goal 协议在本 goal 消费侧实现
（`headless/domains/dom.rs` 扩展），不硬啃 style-system 内部。

## 4. 结构性限制记账

- **单连接串行 accept 循环**：一次只服务一个连接（HTTP 快进快出除外）。第二个 WS
  客户端（PW connectOverCDP / 第二次 frontend 加载）在 backlog 饿死。M1 Elements
  单连接流可先行；Console REPL / Network 演示流与多客户端场景（frontend + PW 并存）
  需 **并发连接切片**（thread-per-connection + HeadlessSession 共享化）。归本 goal
  transport 面，非上游 cdp-protocol 缺口（其矩阵面单客户端语义未被破坏，cdp-e2e 绿）。

## 5. 下一切片建议（按序）

1. **M1-S3a 最小域集**：`DOM.enable`/`DOM.getDocument`（zero-dom 全树序列化，
   nodeId 分配器）+ `Page.getResourceTree` + `CSS.enable`/`Overlay.enable`/§3 no-op
   族 ack → Elements 树渲染被调试页活 DOM（单连接流即可闭环验证）。
2. **M1-S1.5 并发连接**：transport 线程化（HeadlessSession 共享化边界先探），
   解锁 Console/Network 演示流与多客户端。
3. **M1-S3b CSS.getMatchedStylesForNode**：样式侧栏数据源（style-system 消费面）。
