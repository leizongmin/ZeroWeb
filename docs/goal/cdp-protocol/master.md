# CDP 协议兼容 + Playwright 验证矩阵 — 运行时控制面板（master.md）

**入口文档**: [../cdp-protocol.md](../cdp-protocol.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-13（S21：DC-2 连接生命周期健壮性实证（顺序重连×3/异常断开×3/断后重连全通）+ 并发第二客户端单连接限制记账；绿步维持 28/30）

---

## 当前状态

**专项定位**：把 `apps/browser/src/headless.rs` 的 CDP 雏形（3 命令）收敛到 Playwright
（pin 版本）`connectOverCDP` 可用——命令矩阵账本为验收标尺，Playwright E2E 全绿收口。
本 goal 是 devtools goal（Chrome DevTools frontend 复用）的协议基座（下游门控）。

**与兄弟 goal 的边界**：
- android-browser — `apps/browser` 共享活跃并行流，碰前 `git log --since="14 days ago"`
  核对
- devtools — 下游消费方：只消费本 goal CDP 面；改 CDP 域实现须本 goal 收口或碰头
- rendering-compat — crate 零重叠

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | Playwright 命令矩阵账本（pin 版空跑导出命令全集 + 三态登记） | ✅ 初稿落地（evidence/cdp-command-matrix.md；随域更新三态） |
| P2 | headless.rs 职责拆分（2256 行超 2000 上限；transport/discovery/domains/session） | ✅ M1 切片 1（headless/ 9 模块，纯搬移零语义变化，make test 19,170P/0F 与基线一致） |
| P3 | Target/Runtime/Page/Input/DOM/CSS/Network/Emulation 域实现 | 🚧 S16 后余 frames 面：locator/evaluate/editing/viewport/媒体全通；唯余 iframe 子帧事件源（frames.access/click+evaluate 2 步，挂 engine 子帧可见性——渲染流域协调） |
| P4 | Node/Playwright 测试链（pin + E2E 用例集 + make 入口） | ✅ S8：`make cdp-e2e`（test-guard 包裹，deterministic 双跑 + expected-green 回归门）；用例集=30 步全核心流 |
| P5 | console 对象化（V8 侧结构化序列化，替换扁平字符串） | ✅ S11 value-only 面落地（consoleAPICalled 绿——shim 逐参值序列化 + `__zw_console_log` 三参 + headless 转事件，PW 消费面 msg.type()/text() 全通）；完整对象句柄化（remoteObject preview/objectId）挂账随 devtools 面需求 |
| P6 | net 请求事件总线（Network 域 + devtools Network 面板共用脊柱） | 🔶 雏形已建（S7 proxy_fetch 三事件 + S14 renderer FetchObserved + S17 dataReceived 双路径）；分块流式观测点待 net 窗口流式化——**net 近 14 天无外部流占用，窗口已开**（2026-09-12 实测） |
| P7 | WS 层 sessionId 多路复用（单连接扁平会话 → per-target session，响应回显 sessionId） | ✅ S4 收口：解析/回显/未附接校验（-32001）+ 附接注册表 + Target 域 per-target 会话（ServerEvent sessionId 盖章路由，Target 宣告事件除外）——实测复核 35 方法零漂移佐证 |
| P8 | `/json/version` 尾斜杠 404（Playwright 请求 `/json/version/`） | ✅ M1 切片 2（normalize_discovery_path 容忍尾斜杠；`/json`、`/json/list` 同步受益） |

## 已完成切片

- **S21（2026-09-13）DC-2 连接生命周期健壮性实证（验证切片，绿步维持 28）**：
  **实证**（真实 PW 客户端 + 裸 WS 探针）：① 顺序重连 ×3（connect → newPage →
  setContent → locator → close 循环）全通、无状态残留；② 异常断开 ×3（裸 WS 发一条
  命令后不发 close 帧 RST 直断）全部被吸收——transport read error 分支 break 内循环 →
  外循环接受下一连接，无进程崩溃、无句柄悬挂；③ 断后新连接完全可用 + `/json/version`
  存活。**DC-2「连接生命周期健壮（重复连接、异常断开）」就此验证**。
  **限制记账（非缺陷，结构注记）**：transport 为单连接 serve loop（一次服务一个 WS
  连接）——第一客户端存活期间第二并发客户端在 TCP backlog 等待（不报错不泄漏，仅
  不可用）；Chromium 支持多并发 CDP 客户端。多客户端多路复用需 HeadlessSession 共享
  化（Arc<Mutex> 重构）= 结构变更，按需立项（Playwright 典型用法单客户端；e2e 门不受
  影响）。
- **S18（2026-09-13）M5 定稿预案 + 子帧缺口实证（纯文档/验证切片，绿步维持 28）**：
  **子帧缺口探针实证**（挂起理由从假设升级为实测）：iframe 元素存在但
  `contentDocument`=null（引擎不加载子帧文档，无子帧 DOM）、`contentWindow`=object
  （stub）、`page.frames()`=1（无 frameAttached 事件源）——frames×2 需引擎子帧文档
  加载 + 子帧渲染面（子文档布局/iframe 区域绘制/child quads 坐标，属 layout-engine/
  paint 渲染流域专属 crate）+ 子帧 JS realm——三件套均跨流域，本流不可单方解。
  **M5 定稿预案**（双分支机械执行清单，见下一步计划 #2）：分支 A（等 30/30，维持
  门禁防回归）／分支 B（挂账剔除定稿，四步全 docs 一个提交）——DC-2 口径一决即执行。
  **全量基线刷新**：make test 全套经 test-guard（结果见验证基线）。
  **引擎碰撞核对**：`git log --since="14 days ago" -- crates/engine/` = 渲染流 paint
  域修复（R4285-R4296 filter/svg/bleed），无子帧相关工作——维持挂起不变。
  **跨流红灯跟踪**：renderer lib form fixture 2 失败仍在（S16 时点归因不变）。
- **S17（2026-09-13）Network dataReceived + 矩阵账本 v0.4 漂移刷新（绿步维持 28）**：
  **dataReceived**：protocol `FetchObservedParams` 增 `data_length`（末位追加）；renderer
  fetch 观测记录扩为六元组（loadingFinished 阶段带 body 字节数——`body_bytes` 原始字节
  优先、文本回退）；headless 双路径在 loadingFinished 前发 `Network.dataReceived`——
  proxy 子资源路径（session proxy_fetch，body 同步在握）+ renderer 观测路径（JS
  fetch/XHR）。**body 一次性到达语义**（dataLength=encodedDataLength=body 字节），
  分块流式随 net 观测点流式化（记账）。两类流量本就分路（子资源=proxy、页面
  fetch=ResourceLoader 直连观测），无重复发射。
  **账本 v0.4**：evidence/cdp-command-matrix.md 逐行以 domains.rs dispatch 表为 ground
  truth 核对——loadEventFired（雏形→✅ S5+S16）、lifecycleEvent（补 S16 重发）、
  setLifecycleEventsEnabled（门控语义记账）、executionContextDestroyed（❌→不实现-ok，
  contextsCleared 覆盖）、dispatchKeyEvent（S16 accel+A 注记）、dialog 行（S10 澄清）、
  dataReceived 行（✅ S17）、G4 请求事件总线（雏形已建）。
  **验证**：cdp-e2e 28 绿 deterministic 双跑一致；browser bin 449P/0F；integration
  781P/0F；renderer lib 161P+2 已知跨流失败（无新增）；clippy -D warnings + fmt 全过。
  **M5 实测复核（同切片）**：ZeroWeb 侧经捕获代理重跑全核心流——429 调用/35 方法 vs
  Chromium 基线 395/40（+34 调用=frames 失败重试放大）；35 个被调方法全部为账本「实现」态，
  **命令面与账本登记零漂移**（5 个 chromium-only 方法全部挂账有因：getFrameOwner=frames
  挂起下游、handleJavaScriptDialog=无事件源、setFontFamilies=-32601 容忍、
  detachFromTarget/setUserAgentOverride=drift 记账）；证据
  evidence/zeroweb-capture-2026-09-13-summary.json。工具坑：execFileSync 冻结父进程事件
  循环致父内嵌代理 × 子进程消费双向死锁——异步 spawn 解（learning 2026-09-13）。
- **S16（2026-09-13）keyboard Ctrl+A 编辑面 + document.open/write/close（绿步 26→28）**：
  **keyboard.type+press**：`Control+a` 此前被当普通可打印键注入 `'a'`（实测值
  'abca'）。修复：`apply_keydown_default` 增 `accel` 形参（CDP dispatchKeyEvent 路径传
  `ctrl||meta`；DispatchDomEventParams 路径无修饰键字段保持 `false`，协议不动）——
  accel+A 命中可打印分支时改走 `apply_select_all_at`（复用指针选区路径
  `set_pointer_text_selection` → shim setSelectionRange，UTF-16 偏移口径；非文本控件
  no-op）。
  **page.setContent（四层落点）**：① shim part06 `document` 补 `open/write/writeln/close`
  三连（PW setContent 在 utility world 执行 `open(); console.debug(tag); write(html);
  close();`——三函数此前缺失 → TypeError）。简化语义：open 清 body + 起缓冲、write 缓冲、
  close 把缓冲作 body innerHTML 一次性应用（live host 解析+重排版，探针验证查询/读回
  可达）；head/title 剥离、unload、隐式 open 未建模（FIXME 记档）。② **console tag 时序**：
  PW 在 tag console 消息到达时 `_onClearLifecycle()` 清 `_firedLifecycleEvents` 再等新
  'load'——headless 逐命令排空此前 network 队列先于 console 队列，load 族先到被清 → 挂起。
  修复：排空序改 console → network/Page（与空闲期 drain 一致）。③ **load 生命周期重发**：
  spec close() 解析结束触发 load（软导航语义）——新增 protocol `DocumentWriteSettled`
  （renderer → headless 单向事件，末位追加；shim close() 经 `__zw_document_write_settled`
  回调 → js_worker 共享队列 → runtime 尾 drain）→ headless 重发
  `Page.lifecycleEvent{DOMContentLoaded,load}` + `domContentEventFired`/`loadEventFired`
  （不发 frameNavigated/contextsCleared：文档对象与 JS context 未换代）。④ 清理 S14 残留
  诊断（headless 两处 println + runtime tick 内 /tmp 文件写——println 污染即 S12 事故根因类）。
  **验证**：绿步 26→28；deterministic 双跑一致；expected-green 基线扩至 28；
  integration 781P/0F（全仓一轮中 network_loading 单测并行负载下偶发 1 失败、隔离与整包
  重跑均绿，非本切片回归）；余 2 步 = frames×2（挂 engine 子帧可见性）。
- **S12（2026-09-13）hit-test 溢出剪枝修复 + CDP 空闲期 renderer 通道 drain**：
  **根因定位（插桩 PW coreBundle 注入诊断 + 点阵探测）**：`#btn-fetch` 点击失败的真因是
  **引擎 hit-test 溢出剪枝**——`deepest_node_at`/`collect_nodes_at` 对「祖先盒不含点」整棵
  剪枝，而裸页 body 盒高仅 6px（gBCR 实测 [8,8,784,6]）容不下 24.6px 的按钮 → 按钮在自身
  中心 `elementFromPoint` 返 html 兜底（点阵探测：按钮盒内仅 y∈[8,11] 命中，其余全 html）
  → PW `setupHitTargetInterceptor` 的 preliminary check 返回 `<html>` description → 无限
  重试。**修复**：hit-test 走树不再按包含剪枝（下探全树、仅记录含点的盒）——溢出内容
  （overflow:visible）可命中，与真浏览器绘制盒命中语义对齐；overflow:hidden 裁剪语义
  未建模（FIXME 记档）。**验证**：点击已真实落地（btn-fetch handler 的 fetch 触达测试
  服务器 API，apiHits=1）。
  **第二层（新发现，未解）**：点击落地后 PW click action 仍不完成——**host-dispatched
  listener 内的 fetch promise 不落定**（handler 内 `fetch()` 的 `.then` 链不执行，
  `__fetched` 恒 null；独立 evaluate 的 fetch 正常）——疑 FetchBridge 在宿主派发事件
  的 execute 内同步 resolve 的**重入死锁**（嵌套 sandbox.execute）。**第三层**：CDP 空闲
  期 renderer 通道无人消费（fetch 的 FetchRequest/console IPC 饿死）——已修：transport
  WS read 改 120ms 轮询 + `drain_renderer_channel`（fetch 代理 + console/network 事件
  即时推送，600s 空闲 deadline 语义保持）。
  **第四层（S13 定位+修复）**：fetch settle 路径 `__zwServiceWorkerFetchSettled →
  ensureDocument → __zw_sw_controller` 走 SW IPC client 同步等待（20s 超时），headless
  从不应答 `ServiceWorkerRequest` → JS worker 挂 20s、PW click 10s 超时。修复：headless
  `handle_renderer_message` 应答 SW 请求（Controller→无 controller、GetRegistrations→空、
  StateChanges→空、写类→NotFound——headless 无 SW 支持=正确语义）。
  **fetch 观测管线（FetchObserved IPC + renderer 观测 handler）已实现后回退**：队列 Arc
  双实例错接 + 诊断期 println 污染 IPC 帧流导致 renderer 通道崩溃（12 步回退事故）；
  已全部回退至 S12 等效状态，观测管线待独立切片以正确队列所有权重做。
  绿步维持 25（无回退）；deterministic 双跑一致。
- **S11（2026-09-13）console value-only 小切片 + emulation.media 接线**：
  **console.collect（P5 降级方案落地）**：shim `_zwConsoleEmit` 增逐参值序列化
  `_zwSerializeConsoleValue`（string/number/boolean 原样、undefined 标记串、对象 JSON
  round-trip）→ `__zw_console_log(level, text, args_json)` 三参（tracing 面不变）；
  renderer js_worker 后注册覆盖引擎回调（last-wins）推共享队列 → runtime 主循环 +
  脚本执行尾 drain → IPC `ConsoleLog`（protocol 末位追加）→ headless session
  `pending_console_events` → transport 逐命令盖章 `Runtime.consoleAPICalled`
  （value-only remoteObject args、executionContextId=1、level→CDP type 映射）。
  **时序要点**：console 事件须先于 AutomationResponse 转发（run_page_context_script 尾
  drain），否则 headless 在响应后才收到、要等下一条命令才排空（实测单命令消费面失效）。
  **emulation.media**：engine `match_media_to_json_ctx`（MediaContext 用户偏好注入）+
  renderer `MediaBridge` 重注册 `__zw_match_media`（共享 cell——SetColorScheme/
  SetMediaType 更新 prefers_color_scheme/media_type）→ matchMedia 读回真值。
  **绿步 23→25**（console.collect + emulation.media 翻绿）；deterministic 双跑一致；
  expected-green 基线扩至 25。
- **S10（2026-09-13）click hit-target 修复 — 合成输入事件面 + 视口真值**：
  S9 后 click 族卡「PW hit-target 拦截器判 `<html> intercepts pointer events`」，三层实测定位：
  ① PW `_hitTargetInterceptor` 读 `event.clientX/clientY` 复核命中点——宿主合成鼠标事件走
  `_makeEvent` 泛型面无坐标（undefined → elementFromPoint(undefined) → null → documentElement
  兜底）；② renderer `handle_mouse_event` 对 mousemove 直接跳过派发——拦截器挂 document
  mousemove 捕获收不到事件。修复：`DomEventDetail` 增 `client_x/client_y`（engine script_gen）
  + shim `__zw_dispatch_event` 新增鼠标类型分支（`new MouseEvent` 带 coords/click detail，
  UI Events §MouseEventInit）+ renderer 坐标随事件注入 + mousemove 照常派发（未命中目标时
  dispatch_dom_at 内部 no-op）。**click 事件保持泛型 Event 不入鼠标分支**——R108 pre-click
  activation/取消回滚协议与宿主激活事务（execute_shared_action）的 checked 翻转/取消语义按
  旧路径协作（实测：click 改 MouseEvent 会双重翻转 checked 且破坏三宿主 conformance）。
  ③ `screenshot.fullPage`：shim innerWidth/
  innerHeight 缺省 1280x800 与真实视口失配（PW `_fullPageSize` 以 scrollWidth 族测量）——
  js_worker 增 `SetViewportHint`（renderer 启动/SetViewport 时注入，快照换代后幂等校正）。
  **Playwright 绿步 17→23**（click.button/dblclick/withPosition + dialog.accept/confirm+prompt
  + screenshot.fullPage 翻绿）；deterministic 双跑一致；expected-green 基线同步扩至 23。
  诊断资产：tests/playwright-matrix/scripts/{raw-min,debug-zw-pw}.mjs（不入 git 调试脚本：
  局部复现 + 捕获代理 + 事件字段探测）。
- **S9（2026-09-13）objectId 全量 remoteObject 桥 — Runtime/DOM 域句柄面（用户拍板全量面）**：
  protocol `AutomationValue::Handle(AutomationHandleRef{id,node})` + 四操作
  `EvaluateRetaining/CallFunctionOnHandle/ReleaseHandle/ReleaseObjectGroup`（含 serde 契约
  测试）；renderer 侧 JS 句柄注册表（页面 context 全局单例、65536 上限、objectGroup 分组、
  primitive 按值/对象保留双尾；**导航换代经 `sandbox.reset_context` 整体失效 = CDP context
  destroyed 语义**，无需显式清理）+ `awaitPromise` 有界轮询（execute 边界 microtask drain +
  宿主 timer 泵，8s 超时）；headless Runtime.evaluate 双分支统一走桥（**表达式语义修复**：
  W3C ExecuteScript 是函数体语义、CDP evaluate 是表达式形态——裸表达式旧恒 undefined，
  PW 全管线的真实根因）+ callFunctionOn objectId（`arguments[].objectId` 实参顶层还原 +
  falsy 实参标记误判修复）+ `releaseObject/releaseObjectGroup` + **DOM 域 objectId 面**
  （scrollIntoViewIfNeeded/getContentQuads/getBoxModel/describeNode/resolveNode——经句柄桥
  对保留元素求值，rect 来自 shim gBCR/RectBridge 真实布局；`backendNodeId`=句柄 id，
  resolveNode 重保留新句柄支撑 PW adopt 流程）+ shim has-trap 白名单补
  nodeName/nodeType/tagName/validity 族/value（PW queryEngine `"nodeName" in element` 断言面）
  + 嵌套值纯 JSON 保真（remoteObject 嵌套不再包 type/value 外壳——PW `{o:[...]}` 线格式）。
  remoteObject 句柄形态带 `subtype:"node"`（PW ElementHandle 分叉点）。
  **Playwright 绿步 6→17**（evaluate 全族 5 步 + title + fill + locator.boundingBox +
  setContent 面前移 + screenshot.element + page.second.lifecycle + viewport.verified 翻绿）；
  deterministic 双跑一致；make test 19,238P/0F；workspace clippy -D warnings 全过。
  余 13 步根因已定位（见下一步计划）。
- **S8（2026-09-12）M5 收口预备 — cdp-e2e 门 + DC 盘点**：
  `make cdp-e2e` 入口落地（test-guard 包裹，spawn 独立 headless 双跑全核心流）：
  **deterministic 双跑一致**（两次入口运行均 YES）+ **expected-green 回归门**（6 步基线
  `expected-green.json`：context.default/page.new/setViewportSize/goto/cookies.roundtrip/
  screenshot.viewport，任一回退即门禁失败）。DC-1~4 盘点（见下）。CI 可行性记账：
  node 20.19 本机在位、playwright 缓存 chromium-1243 命中（npm install 仅装
  playwright-core+ws 两个包，lockfile 离线可复现）、CI 需 pre-step `npm ci` +
  `cargo build -p zero-browser`；CI 集成等 goal 收口时随 M5 定稿评估。
  **DC 盘点**：DC-1 ✅（账本 40 方法三态全登记 + goal 扩展面）；DC-2 ⏳（绿步 6/30，
  双跑 deterministic ✅ 已门禁化，全绿挂 objectId 桥决策）；DC-3 ✅（-32601/-32700/
  -32602/loopback/token-origin 语义保持；**超大 payload 实测**：100MB 消息触发
  tungstenite 16MB 帧上限干净拒绝（`Message too long` + 连接断开），服务器存活、
  后续连接正常——安全拒绝语义成立）；DC-4 ✅（make test 全绿 + clippy/fmt +
  cdp-e2e 门 + BiDi 既有面零回归）。
- **S7（2026-09-12）M4 — Storage cookie 域 + UA override + Network 事件总线雏形**：
  session 级 `CookieStore`（net 既有 jar 复用，goal 支持包络「net 只加观测点」——新增
  只读 `CookieStore::all()`）；`Storage.getCookies/setCookies/clearCookies` 实义
  （url/domain 作用域 + expires/secure/httpOnly，CDP cookie 形状）+ proxy_fetch 双向
  接线（Set-Cookie 捕获 + Cookie 请求头注入）→ **Playwright cookies.roundtrip 绿**；
  `Emulation.setUserAgentOverride`（proxy_fetch 注入 User-Agent）；`Network.enable/
  disable` 真实门控 + proxy_fetch 生命周期事件（requestWillBeSent/responseReceived/
  loadingFinished，session 盖章排空）——P6 net 观测点雏形。**Playwright 绿步 5→6**。
  console 对象化（P5）挂起：需 engine 宿主回调签名扩展（engine 为并行流活跃面，
  碰头管理延后）。make test 全绿（+6 M4 单测）。
- **S6（2026-09-12）M3 — viewport 桥 + 媒体仿真 + CDP 截图形状**：
  `Emulation.setDeviceMetricsOverride` 实义（→renderer SetViewport IPC + 服务器视口状态
  联动 getLayoutMetrics/captureScreenshot + Page.frameResized 事件；宽高 0=恢复默认；
  实测 page.setViewportSize 绿）；`setEmulatedMedia` 实义（prefers-color-scheme→
  SetColorScheme、media type→SetMediaType）；`Page.captureScreenshot` CDP 形状修正
  （`{data:"<b64>"}` 字符串形——此前对象形致 PW screenshot 直接报错，BiDi 对象形不动）
  + clip 原始 fb 行级裁剪（实测 screenshot.viewport 绿）。**Playwright 绿步 4→5**。
  挂起记档：iframe 子帧事件源需引擎子帧可见性（渲染流域协调），frames.access 步骤
  随引擎能力。make test 全绿（+5 M3 单测）。
- **S5（2026-09-12）M2 — 导航事件族 + Input 域 + getLayoutMetrics**：
  `Page.navigate` 实义化（`{frameId,loaderId,errorText?}` 形状 + Chromium 时序导航事件族
  frameStarted/StoppedLoading→frameNavigated→executionContextsCleared→新文档 context→
  domContent→load）；`addScriptToEvaluateOnNewDocument` 真执行 + 跨导航重放 + worldName
  登记/新文档 world context 重发（**实测修复 title/evaluate 在导航后永久挂起**——PW 的
  `utilityContext()` 等待 world context 事件）；`Page.getLayoutMetrics`（固定视口映射）；
  `handleJavaScriptDialog` stub；**Input 域全接**：dispatchMouseEvent（→renderer
  MouseEvent/ScrollEvent，released 按 clickCount 合成 Click/DblClick）、dispatchKeyEvent
  （Down/Up/Press + modifiers 位解码）、insertText（→ImeEvent Commit）——裸 API 实测
  keyboard.press/type、mouse click/wheel 全通。**Playwright goto 核心流绿**（47ms）；
  全流 30 步无挂起（此前 title/evaluate 挂起根因即 world context 缺失）。make test
  19,194P/0F（+10 M2 单测）。
- **S4（2026-09-12）M1 切片 3 — Target/Browser/Runtime 域 + Playwright 首连**：
  `Target.setAutoAttach`（flatten，浏览器级附接全部 page target + attachedToTarget 事件，
  会话级正确语义=仅子 target→无事件）/ `getTargetInfo`（浏览器级+按 targetId）/
  `createTarget`（autoAttach 自动附接）/ `closeTarget`（targetDestroyed+会话摘除）/
  `detachFromTarget` / `getTargets` CDP 形状修形（targetInfos）；`Browser.getVersion`/
  `setDownloadBehavior` stub；`Runtime.enable`（executionContextCreated + auxData 契约）/
  `Runtime.evaluate`+`callFunctionOn`（无 objectId 路径，renderer 类型化 AutomationValue →
  remoteObject returnByValue）/ `runIfWaitingForDebugger`；`Page.enable`/`getFrameTree`
  （**主 frame id = targetId**，CDP 硬契约）/`createIsolatedWorld`（utility world 记账）/
  init 命令族 stub；**ServerEvent 增 sessionId 盖章**（session 级事件客户端路由必需，
  Target 宣告事件除外）。**Playwright connectOverCDP 首连成功**：connect/attach/
  context.newPage 全通；evaluate 执行到 utilityScript 句柄处暴露 objectId 桥缺口
  （→ 待用户决策）。实测确认三条 CDP 硬契约：主 frame id=targetId、session 级事件必带
  sessionId、executionContextCreated 必带 auxData.frameId/isDefault。
- **S3（2026-09-12）M1 切片 2 — 传输层 sessionId 复用 + 发现端点修正**：`ClientRequest/
  ServerResponse` 增 `sessionId`（camelCase rename，回显 + 未附接 `-32001`）；`/json/version`
  尾斜杠容忍（P8/G2 收口）；`/json`、`/json/list` 按真实标签页枚举（`zeroweb-tab-<n>`，
  url/title 取自 shell 模型）；会话提升为服务器级（target 跨连接持续，CDP 语义）；**实测
  拦截两个传输层存量 bug**——tungstenite 0.29 `write()` 小消息不落盘（响应滞留缓冲）+
  peek 阶段 5s read timeout 未恢复（空闲误杀连接），已修并沉淀 learning（2026-09-12
  tungstenite-write-buffer-stale-read-timeout）。Playwright 直连 smoke：WS 往返已通，
  connect 推进至 `Browser.getVersion` -32601（切片 3 范围）。`make test` 19,174P/0F
  （基线 19,170 + 新增 4 传输层单测）。
- **S2（2026-09-12）M1 切片 1 — headless.rs 职责拆分**（P2/G6 收口）：`apps/browser/src/headless.rs`
  （2256 行超限）→ `headless/` 9 模块（mod=transport / protocol / session / security /
  discovery / domains / client / tests / gpu_screenshot_tests），纯搬移零语义变化，
  `pub(super)` 子树内可见，测试代码零改动；`make test` 19,170P/0F 与拆分前基线一致，
  workspace clippy `-D warnings` 全过。
- **S1（2026-09-12）M1 前置纯资产切片**：`tests/playwright-matrix/` pin 工程
  （playwright-core 1.63.0 + lockfile）+ CDP 捕获代理 + 全核心流空跑脚本（30 步全绿
  @ Chromium 153.0.8010.12）→ 命令全集 395 调用/40 方法/30 事件 →
  `evidence/cdp-command-matrix.md` 初稿（三态登记 + 6 条关键契约发现 + G1-G6 结构缺口）。
  关键修正：cookie 走 **Storage 域**（非 Network.getCookies 族）；Playwright 不调
  `Target.getTargets`（连接靠 setAutoAttach flatten）；locator 流不用 DOM.getDocument/
  CSS.*，脊柱是 Runtime.callFunctionOn（156 次）→ objectId 桥。

## 下一步计划

1. **M5 收口评估（绿步 28/30，余 2 步全挂同一协调点）**：`frames.access`/
   `frames.click+evaluate` 依赖 engine 子帧可见性（iframe 子帧 DOM/事件面）——渲染流域
   真协调。DC-2 口径决策：等子帧能力解冻后 30/30 收口，or 以「挂账 + 口径剔除」先定稿
   （见待用户决策）。**实测复核已过**（S17：35 被调方法零漂移，见矩阵账本 ZeroWeb 侧
   实测捕获节）——DC-2 口径一决即可定稿。
2. **M5 定稿（口径确定后）**：expected-green 基线定稿 → cdp-e2e 即 DC-2 门；挂账清单
   （不实现域）终稿；CI 集成可行性随收口评估（S8 记账：node 20.19 + lockfile 离线可复现）。
   **定稿预案（S18 预备，双分支机械执行）**：
   - **分支 A（等 30/30）**：goal 维持 Active；每轮门禁防回归；渲染流域子帧能力落地后
     解 frames×2 → 基线扩 30 → DC-2 ✅ → M5 定稿。挂账清单不豁免 frames 项。
   - **分支 B（挂账剔除定稿）**：① 矩阵账本「ZeroWeb 侧实测捕获」节加口径注记（frames×2
     记「挂账：随引擎子帧能力，M5 定稿时点不阻收口」）；② master.md 里程碑 M3/M5 改
     ✅（口径挂账注记）；③ goal 入口文档 DC-2 行加挂账口径注记（不改判定语义原文，仅
     注记）；④ expected-green 基线维持 28 不动（frames 步骤继续跑、不门禁）；⑤ CI 集成
     评估出结论记账。四步全 docs，一个提交。
3. **持续推进**：每轮 pull → cdp-e2e 门（基线 28 步）+ make test 防回归，余项按窗口逐个解冻。

**待用户决策清单**：
- **DC-2 收口口径（2026-09-13 新入，维持）**：余 2 步（frames.access/frames.click+evaluate）
  挂 engine 子帧可见性——S18 探针实证：`iframe.contentDocument` 为 null（引擎不加载子帧
  文档）、`page.frames()`=1（无 frameAttached 事件源）；且子帧**渲染面**（子文档布局/
  iframe 区域绘制/child quads 坐标）属 layout-engine/paint——渲染流域专属 crate，本流
  不可单方解。「等子帧能力后 30/30 收口」vs「挂账剔除先定稿」。口径不清则 M5 无法判定
  完成（**M5 定稿预案见下，口径一决机械执行**）。
- ~~dialog 事件源~~ **绿步已过、语义挂账（S10 现状澄清 2026-09-13）**：dialog.accept/
  dialog.confirm+prompt 绿因**引擎无阻塞对话框语义**——shim alert no-op、confirm/prompt
  立即返回（无 javascriptDialogOpening 事件、无挂起）、`Page.handleJavaScriptDialog` 为
  stub——步骤「不挂起即过」。真对话框事件面（引擎阻塞语义 + 事件源）仍属跨流域立项，
  不阻 M5 收口（PW 消费面绿）。
- ~~objectId 句柄桥~~ **已拍板（2026-09-12）：全量 remoteObject 桥**——✅ S9 落地。

**维持挂起**：iframe 子帧事件面（frames.access/frames.click+evaluate 2 步）——渲染流域
真协调（engine 子帧可见性），rendering 流 R41xx-R42xx 高频活跃，维持挂起合理。

**跨流红灯记录（S16 时点归因，非本流）**：`cargo test -p zero-renderer --lib` 2 失败
（page_scripts::tests::form_interaction_fixture_complete_sequence /
…_dispatches_idless_reset_and_submit_buttons——`apply_reset_on_click` 断言）。stash 验证
干净树同败（S16 变更无关），疑似 `514f07b29`（tick_observers per-task 重构）或渲染流
4fed099dc 组合态引入——归 event-loop-spec/渲染流修，本流不碰 page_scripts.rs 工作面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — 传输/发现/Target 基座 + Playwright 首连 | ✅ S9 收口：连接面 + evaluate 全族（literal/function/withArgs/object/async）+ releaseObject(Group) 全通 |
| M2 — Page/Input 域 → 点击/填充/键盘/导航流 | ✅ S16 收口：goto/title/fill/click 全族/dialog/键盘 type+press（Ctrl+A 全选）/导航事件族全绿 |
| M3 — DOM/CSS/Emulation → locator 流 | 🚧 S16：locator.boundingBox/viewport/媒体/截图 clip+element+fullPage 绿；iframe 面维持挂起（frames×2） |
| M4 — Network/cookies/console 对象化（cookie 落点=Storage 域） | ✅ S17 收口：cookie 域 + UA override + Network 事件族（含 dataReceived，S17）+ consoleAPICalled（value-only）绿；分块流式观测记账 |
| M5 — 矩阵收口 | 🚧 绿步 28/30（expected-green 基线同步扩至 28）；余 2 步全挂 engine 子帧可见性（DC-2 口径待决策） |

## 验证基线

- 测试基线：立项时点全绿（`make test` 19,170P/0F，2026-09-12 变基后口径；S9 后
  19,238P/0F；**S18 全量刷新 19,251P/0F EXIT=0**（2026-09-13；并行流计数会漂移，
  以当轮实跑为准）；禁止裸跑 cargo test，经 test-guard。注：make test 的 workspace 腿
  exclude zero-renderer——renderer lib 单测不在全量门内，跨流红灯（form fixture×2）
  经显式 `-p zero-renderer --lib` 跟踪）
- **CDP E2E 基线（S16，2026-09-13）**：绿步 28/30，deterministic 双跑一致，
  expected-green 基线 28 步（余 frames.access/frames.click+evaluate 挂 engine 子帧可见性）
- CDP 现状：`Page.navigate` / `Runtime.evaluate` / `Target.getTargets` 3 命令 +
  `/json/version` + `/json` 发现（headless.rs L782-796/L571/L1159）——历史基线，现行面
  见缺口清单 P3/P4 与切片记录
- **命令矩阵捕获基线（S1，2026-09-12）**：playwright-core 1.63.0 @ Chromium 153.0.8010.12
  （chromium-1243 缓存），全核心流 30 步全绿，395 调用/40 方法/30 事件；
  `evidence/chromium-capture-2026-09-12.md` + `…-summary.json`（生成物，复现命令见账本头）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过；
  Playwright E2E 用例须双跑 deterministic 才计入账本「绿」

**碰撞管理**：碰 `apps/browser` 前先 `git log --since="14 days ago" -- apps/browser/`
核对 android-browser 活跃面。
