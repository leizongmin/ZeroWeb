# CDP 协议兼容 + Playwright 验证矩阵 — 运行时控制面板（master.md）

**入口文档**: [../cdp-protocol.md](../cdp-protocol.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-12（M4 切片落地：Storage cookie 域 + UA override + Network 事件总线雏形）

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
| P3 | Target/Runtime/Page/Input/DOM/CSS/Network/Emulation 域实现 | ⏳ M1-M4 |
| P4 | Node/Playwright 测试链（pin + E2E 用例集 + make 入口） | 🔶 pin 工程已入库（tests/playwright-matrix/，playwright-core 1.63.0）；E2E 用例与 make 入口随 M1 建 |
| P5 | console 对象化（V8 侧结构化序列化，替换扁平字符串） | ⏳ M4（矩阵 G5） |
| P6 | net 请求事件总线（Network 域 + devtools Network 面板共用脊柱） | ⏳ M4（矩阵 G4） |
| P7 | WS 层 sessionId 多路复用（单连接扁平会话 → per-target session，响应回显 sessionId） | 🔶 M1 切片 2 传输面完成：解析/回显/未附接校验（-32001）+ 附接注册表；per-target 真路由随切片 3 Target 域落地 |
| P8 | `/json/version` 尾斜杠 404（Playwright 请求 `/json/version/`） | ✅ M1 切片 2（normalize_discovery_path 容忍尾斜杠；`/json`、`/json/list` 同步受益） |

## 已完成切片

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

1. **M4 收口余项**：console 对象化（P5，engine 宿主回调扩展——并行流碰头管理，等
   zero-web 流窗口）+ Network 事件字段补全（net 观测点扩展：headers/mimeType/
   dataReceived）
2. **iframe 子帧事件源**：需引擎子帧可见性（渲染流域协调点），frames.access 步骤依赖
3. **M5 矩阵收口预备**：E2E 用例集（playwright-matrix 工程已有 30 步流）建 make 入口
   + 双跑 deterministic 判定 + DC-1~4 逐项盘点
4. **objectId 桥获批后**：renderer/protocol/engine 跨 crate 对象注册表——收口
   evaluate/locator/click/fill 全族（~20 步，占差距大头）

**待用户决策清单**：
- **objectId 句柄桥（深结构，2026-09-12 实测确认）**：Playwright evaluate/locator 全
  管线依赖 `utilityScript` 对象句柄——需 renderer 侧 JS 对象注册表（retain JSValue 跨
  IPC）+ protocol 新消息（EvaluateRetainingHandle/CallFunctionOnHandle/ReleaseHandle）+
  headless Runtime.evaluate(callFunctionOn) returnByValue:false objectId 返回 +
  Runtime.releaseObject。现有 AutomationElementRef 仅覆盖 DOM 节点，不覆盖任意 JS 对象。
  改动面跨 protocol/page-runtime（renderer 自动化域）/engine/browser 四处，属跨 crate
  协议扩展。**M2/M3/M4 域不受阻**（value-only），本 goal 按入口文档「跳过并继续其他域」
  执行，等拍板后作为独立切片落地。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — 传输/发现/Target 基座 + Playwright 首连 | 🚧 连接面全通（S4：connect/attach/newPage ✓）；evaluate 收口卡 objectId 桥（待用户决策） |
| M2 — Page/Input 域 → 点击/填充/键盘/导航流 | 🚧 S5：goto 绿 + Input 域全通 + 导航事件族；locator 类点击/填充仍挂 objectId 桥（value-only 面已尽） |
| M3 — DOM/CSS/Emulation → locator 流 | 🚧 S6：viewport 桥/媒体仿真/截图 clip 绿；DOM 句柄桥挂 objectId 桥；iframe 子帧挂引擎子帧事件面 |
| M4 — Network/cookies/console 对象化（cookie 落点=Storage 域） | 🚧 S7：cookie 域 + UA override + Network 事件雏形绿；console 对象化挂 engine 碰头窗口 |
| M5 — 矩阵收口 | ⏳ |

## 验证基线

- 测试基线：立项时点全绿（`make test` 19,170P/0F，2026-09-12 变基后口径；禁止裸跑
  cargo test，经 test-guard）
- CDP 现状：`Page.navigate` / `Runtime.evaluate` / `Target.getTargets` 3 命令 +
  `/json/version` + `/json` 发现（headless.rs L782-796/L571/L1159）
- **命令矩阵捕获基线（S1，2026-09-12）**：playwright-core 1.63.0 @ Chromium 153.0.8010.12
  （chromium-1243 缓存），全核心流 30 步全绿，395 调用/40 方法/30 事件；
  `evidence/chromium-capture-2026-09-12.md` + `…-summary.json`（生成物，复现命令见账本头）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过；
  Playwright E2E 用例须双跑 deterministic 才计入账本「绿」

**碰撞管理**：碰 `apps/browser` 前先 `git log --since="14 days ago" -- apps/browser/`
核对 android-browser 活跃面。
