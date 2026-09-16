# M1/M2 — frontend 附接 ZeroWeb：S1 路由 + S3a/S3b + S1.5 并发 + M2-N2 事件路由 + M2-N3 cookie 面

**日期**: 2026-09-16（M1/M2 多轮推进，M0 收口后）
**上游状态**: cdp-protocol goal Done（M5 守成态，`make cdp-e2e` 33 绿 = 本 goal 防回归门）

---

## 0-e. M2-N3 ✅ Application cookie 面板（DC-2 cookie 判据全通，2026-09-16）

**新增（消费侧，`domains/storage.rs` + 路由）**：`Network.getCookies`（urls host
后缀过滤）/ `Network.setCookie`（单 cookie 写入，复用 Storage.setCookies 管道）/
`Network.clearBrowserCookies`（清空 jar）——与 Storage 域同 jar；Application 面板的
CookiesModel 走 Network 域三件。

**验收（attach-zeroweb-probe.mjs 全绿 exit=0，截图存证）**：
- 种 cookie（Storage.setCookies）→ 被调试页停到 example.com →
  `&panel=resources` 打开 Application 面板 → Cookies 树展开出
  **`https://example.com`** origin → 点击 → **cookie 表渲染
  `zw_devtools_probe` 行**（`attach-zeroweb-application.png`）。
- **编辑回写**：`Network.setCookie` 改值 → `Network.getCookies` 反映
  `edited-v2`（协议级断言）。

**面板 ID 注记**：Application 面板的注册 id 是 **`resources`**（历史名）——
`&panel=application` 不存在该面板（静默回落 Elements）。

**附带发现与修复（transport 反压，M2-N3 轮实测根因）**：probe 未排空 ZeroWeb 的
stdout 管道——周期性 tracing（120ms/连接的 idle tick）填满 64KB 管道缓冲后**阻塞
整个服务进程**（mux 循环冻住 → 新连接饿死，隔离复测不现因复测脚本已重定向 stdout）。
双层修复：①probe 持续排空双管道（嵌入方契约注记）；②`[S13] idle drain tick`
降级 `trace!`（info 级周期刷日志本就是噪音）。probe stdout 排空后 application 腿
立即恢复可达。

---

## 0-d. M2-N2 ✅ Network 域事件多客户端路由（订阅制广播，2026-09-16）

**机制**：连接状态机增 `wants_network` 订阅态（Text 消息解析 method 嗅探
`Network.enable`/`Network.disable` 更新）；Network 域事件不写本连接，汇入主循环
staged 列表，tick 末**广播到所有已订阅连接**（`drain_renderer_channel` 重构为
`drain_renderer_events` 返回事件，调用方分流）。Console/Runtime 事件保持既有
"本连接排空即得"语义（单 frontend 页场景不变）。

**验收（双客户端协议级实测）**：ws1（page-direct，Network.enable）+ ws2（浏览器级
Page.navigate 触发导航）→ ws1 收到完整事件序列
`requestWillBeSent → responseReceived → dataReceived → loadingFinished`。
门禁：465P/0F + clippy clean + cdp-e2e 33 绿（network.events 单客户端语义保持）。

**记账（请求行 E2E 演示流）**：面板行级断言暂为诊断项——REPL 驱动 `location.href`
的打字流可靠性待稳（REPL 焦点/时序），且 demo 需被调试页停在真实 http 页
（about:blank reload 无网络请求，语义正确）。数据面已通（协议级实测），
演示流脚本化随 M2 收口。

---

## 0-c. M1-S3b ✅ 样式侧栏数据面（Computed 侧栏全链路打通，2026-09-16）

**消费侧实现（零 engine/style-system 改动，`headless/domains/css.rs` 新模块）**：

1. **nodeId → 节点解析**：`DOM.getDocument` 探测随树捕获元素 shim `__zwSelector`
   （`q` 字段）→ `devtools_node_selectors` 注册表（nodeId → selector）；CSS 域按
   nodeId 反查（`selector_for_devtools_node`，无记录 → `-32602 No node with given id`）。
2. **`CSS.getComputedStyle`**（+ 旧名 `getComputedStyleForNode` 兼容臂）：renderer
   JS 探测按固定属性清单（15 项：display/position/visibility/opacity/颜色族等）
   逐项调用既有 host 回调 `__zw_get_computed_style(sel, prop)`（style-system 计算值桥，
   per-snapshot 缓存），空值滤除 → `computedStyle: [{name, value}]`。
3. **`CSS.getMatchedStylesForNode`**：`inlineStyle`（元素 `style` 属性解析）+
   `matchedCSSRules: []`——规则级内省无 IPC 消费面（style-system 匹配结果未暴露过
   协议，记账；Styles 侧栏现阶段呈现 inline style，Computed 侧栏为真实计算值）。
4. 节点选中链路 ack：`DOM.setInspectedNode`/`DOM.highlightNode`/`Overlay.hideHighlight`。

**验收（attach-zeroweb-probe.mjs 15/15 全绿 exit=0 + frontend UI 实测）**：
- `cdp-css-computed-style` ✅ 15 props 真实计算值（`display=block`、
  `background-color=rgb(238,238,238)`——example.com 样式表经 ZeroWeb 引擎级联生效）。
- `cdp-css-matched-styles-shape` ✅ CDP 形状完整（inlineStyle.cssProperties + 空表族）。
- **frontend UI 实测**：Elements 树点选 BODY → Computed 子标签 → 盒模型图（margin/
  border/padding/content 尺寸）+ `display: block` 计算值渲染
  （`attach-zeroweb-computed-sidebar.png`）——数据链路 = ZeroWeb 引擎 → style-system →
  host 桥 → shim → CDP → frontend UI 全通。
- S1.5 并发再证：probe 的 CSS 验证 WS 客户端与 frontend 面板连接并存互不阻塞。

**记账（余项）**：Styles 侧栏 matched rules 需 style-system 匹配结果的协议暴露
（engine 侧新面，碰头协调）；overlay 高亮渲染（Overlay.enable 已 ack，无高亮绘制）。

门禁：`cargo test -p zero-browser` 465P/0F（+2：inline 解析/转换注册表）、
clippy -D warnings clean、`make cdp-e2e` 33 绿、`make test` 全绿。

---

## 0-b. M1-S1.5 ✅ 单线程多路复用 + M2-N1 ✅ Network 面板渲染（2026-09-16）

**S3a 轮遗留的「&panel=network 不渲染」定谳并修复**——根因三层：

1. **串行 accept 循环饿死 lazy import**：frontend 面板模块经 `import()` 动态导入，
   该 HTTP 请求发生在 WS 建立之后——旧单连接串行模型下它永远排在长活 WS 后面
   （accept 不回来），模块 import 永不 resolve → 面板空白、零报错。
   **修复 = S1.5 并发连接**。设计取舍：thread-per-connection 因 `HeadlessSession`
   内含 `Rc`（zero-dom Document / shell 表）非 `Send` 而不可行（跨线程需深改
   zero-dom/shell，属共享面）；落地为**单线程 socket 多路复用**——listener 与全部
   连接非阻塞，主循环 3ms tick 轮询推进连接状态机
   （`ConnState`/`Phase::Peek→Phase::Ws`），`HeadlessSession` 保持主线程独占（零锁、
   命令语义天然串行）。连接形态（page-direct）从全局 AtomicBool 改为随连接状态机
   传递（`handle_message_with_events_mode` → `dispatch_with_events_for`/`dispatch`
   增 `page_direct` 参数）。
2. **`Page.getNavigationHistory` 字段名错**：CDP 契约是 `currentIndex` 非 `index`——
   字段名错导致 frontend `ScreencastView.requestNavigationHistory` 读
   `entries[undefined].url` 崩溃（pageerror 栈已存 §0）。
3. **`Network.emulateNetworkConditionsByRule` 缺 `ruleIds`**：frontend 读
   `response.ruleIds.length` —— 空 ack 改为 `{ruleIds: []}`。

另修 `Page.startScreencast`/`Page.stopScreencast`/`Page.screencastFrameAck`/
`Overlay.setShowContainerQueryOverlays` ack 族（ScreencastView 初始化链）。

**验收（attach-zeroweb-probe.mjs 13/13 全绿，exit=0）**：
- `frontend-network-panel` ✅ —— Network 面板全 UI 渲染（录制条/过滤器/瀑布图，
  `attach-zeroweb-network.png`）；M2-N1「面板渲染排查」就此清账。
- Elements 活 DOM + Console REPL 双绿保持。
- 请求行诊断项：面板开着时经面板自带 "Reload page" 触发导航，请求行仍空——
  **Network 域事件多客户端路由**（事件被任一连接的 drain tick 排空，未保证路由到
  面板所在连接）= 新记账，归 M2（cdp-e2e network.events 单客户端语义未受影响，
  33 绿实测）。

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
2. ~~M1-S1.5 并发连接~~ ✅（见 §0-b，单线程多路复用形态）
3. ~~M2-N1 Network 面板渲染排查~~ ✅（见 §0-b——根因即 S1.5 缺位 + 两处返回形状）
4. ~~M1-S3b 样式侧栏数据面~~ ✅（见 §0-c——Computed 全链路；Styles=inline 起步）
5. **M2 Network 域事件多客户端路由**：事件 drain 归属保证（面板所在连接优先），
   解锁 Network 请求行/详情演示流；随后 Application cookie 面板（`&panel=application`）。
6. **Styles 侧栏 matched rules**（碰头协调项）：style-system 匹配结果的协议暴露
   （engine 侧新面，非本 goal 单方消费可及）；overlay 高亮渲染同记。
