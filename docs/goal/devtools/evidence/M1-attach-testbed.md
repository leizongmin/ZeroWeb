# M1 — frontend 附接 ZeroWeb：S1 路由 + S2 试验台 + S3 账本 + S3a 最小域集

**日期**: 2026-09-16（M1 首轮 + S3a 轮，M0 收口后）
**上游状态**: cdp-protocol goal Done（M5 守成态，`make cdp-e2e` 33 绿 = 本 goal 防回归门）

---

## 0. M1-S3a ✅ 最小域集（Elements 活 DOM + Console REPL 双绿，2026-09-16）

S3 账本定谳的核心缺口按消费侧实现（`headless/domains/`）：

1. **`DOM.getDocument`**：renderer 页面上下文 JS 探测（`EvaluateRetaining` +
   return_by_value，零 protocol-crate 改动）把 shim DOM 序列化为紧凑 JSON →
   headless 侧 `convert_cdp_node` 转 CDP Node 形状（nodeId 顺序分配，doctype/元素/
   文本/注释 + depth 截断桩语义）。**这是 frontend Elements 面板唯一数据源**。
2. **`DOM.enable` / `DOM.requestChildNodes` / `CSS.enable` / `Overlay.enable`** ack。
3. **`Page.getResourceTree`**：复用 getFrameTree 树形 + resources 空表。
4. **enable 型 no-op ack 族扩面**（frontend 初始化二波：Overlay.setShow*Overlays、
   Debugger.set*、DOMDebugger.set*、Network.set*、Page.set* 等 40+ 方法）+
   带返回形状三件：`Runtime.getIsolateId`、`Storage.getStorageKey`、
   `Page.getNavigationHistory`（index/entries）。
5. **`Target.getTargetInfo` 无参分类修复**：page-direct 连接（`/devtools/page/<id>`）
   返回活跃页 `type:"page"` targetInfo（此前恒返 browser 占位——frontend 据此误判
   连接形态装载 ScreencastView 浏览器调试 UI）。`page_direct_connection` 形态登记
   随 accept 循环维护（单连接串行模型下 = 当前连接形态）。

**验收（attach-zeroweb-probe.mjs 实测，截图存证）**：
- `frontend-elements-live-dom` ✅ —— Elements 面板渲染被调试页活 DOM 树
  （`<!DOCTYPE html><HTML lang=…><HEAD>…<BODY>…` + 面包屑 + `$0` 选中态），
  截图 `attach-zeroweb-elements.png`。
- **`frontend-console-repl-evaluate` ✅** —— Console 面板 REPL 键入 `1+1` →
  `Runtime.evaluate` → ZeroWeb V8 → 回显表达式 + 结果 `2`（DC-2 Console 判据的
  最小演示流打通），截图 `attach-zeroweb-console.png`。
- frontend 初始化 -32601 归零（账本 §3 清单全部消解）。

**遗留（M2 面）**：`&panel=network` 的 NetworkLogView 在 ZeroWeb 附接下不渲染——
frontend 未捕获异常（pageerror）定位：`ScreencastView.requestNavigationHistory`
（panels/screencast/screencast.js:3893，读 `entries[i].url` undefined）+
`sdk.js:36029` Promise.all 链 `reading 'length'`。ScreencastView 为何在 inspector
入口被实例化待查（`new ScreencastView` 仅 ScreencastApp；疑似共享 bundle 初始化链）。
Elements/Console 两面板不受影响。

单测 +1（getDocument 探测 JSON → CDP Node 转换：doctype/属性/文本递归 id/depth 桩）。
门禁：`cargo test -p zero-browser` 464P/0F、clippy -D warnings clean、
`make cdp-e2e` 33 绿（守成面零漂移）、`make test` 全绿。

---

## 1. M1-S1 ✅ per-page WS 路径路由（前一轮落地）

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

1. ~~M1-S3a 最小域集~~ ✅（见 §0）
2. **M1-S1.5 并发连接**：transport 线程化（HeadlessSession 共享化边界先探），
   解锁 Console/Network 多步流与多客户端（附接试验台的两次 goto 竞态即源于此）。
3. **M2-N1 Network 面板渲染排查**：ScreencastView 在 inspector 入口的实例化链 +
   sdk.js:36029 Promise.all 崩溃定位（见 §0 遗留）；Network 域事件流在
   Network.enable 后由 REPL fetch 触发即有数据（管线已在，单连接下触发时序受限）。
4. **M1-S3b CSS.getMatchedStylesForNode**：样式侧栏数据源（style-system 消费面；
   现状 = Elements 树可选中，样式栏 "No matching selector or style"）。
