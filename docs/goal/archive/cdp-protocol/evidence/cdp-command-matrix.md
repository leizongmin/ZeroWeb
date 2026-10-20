# CDP 命令矩阵账本（DC-1 控制件）

**版本**: v0.6（S39 子帧元数据面三行——frameAttached/frameDetached ❌→✅、getFrameTree childFrames 扩展；v0.5 S25 补测行/审计节 + S28 组合态复核与复现链闭合；v0.4 S17 漂移刷新——
S12-S17 落地面三态/口径逐行核对，ground truth=
`apps/browser/src/headless/domains/` dispatch 表（S32 起，原单文件 domains.rs）；v0.3 S9、v0.2 S4、v0.1 M1 前置初稿）
**日期**: 2026-09-13
**捕获客户端**: playwright-core **1.63.0**（pin，见 `tests/playwright-matrix/package.json` + lockfile）
**捕获目标**: Chromium 153.0.8010.12（playwright 缓存 chromium-1243，headless=new）
**捕获方式**: `tests/playwright-matrix/scripts/capture-core-flow.mjs` — Playwright
`connectOverCDP` 经捕获代理驱动全核心流（30 步全绿），代理旁路记录全部 HTTP 发现 + WS
CDP 流量。零源码改动。
**复现**: `cd tests/playwright-matrix && npm install && npm run capture:chromium && npm run matrix`
**明细（生成物）**: [chromium-capture-2026-09-12.md](chromium-capture-2026-09-12.md) +
[chromium-capture-summary-2026-09-12.json](chromium-capture-summary-2026-09-12.json)

---

## 汇总（2026-09-12 捕获）

- **395 次命令调用 / 40 个唯一方法**（10 域）+ **187 次事件 / 30 个唯一事件**
- HTTP 发现：Playwright 仅请求 `GET /json/version/`（注意**带尾斜杠**）
- 高频方法：`Runtime.callFunctionOn` 156 次（locator/evaluate 全走此路）、
  `Runtime.releaseObject` 51 次、`Input.dispatchMouseEvent` 35 次、`DOM.scrollIntoViewIfNeeded` 13 次

### 关键契约发现（修正入口文档假设）

1. **Playwright 不调用 `Target.getTargets`** — 连接靠 `Target.setAutoAttach`（flatten）+
   `Target.getTargetInfo`；页面枚举经 auto-attach 事件流。ZeroWeb 现有
   `Target.getTargets`（返回 BiDi tree 形状）对 Playwright 无用，需按 CDP 形状修正或登记
   devtools 面。
2. **cookie 操作走 `Storage` 域**（`Storage.getCookies` / `setCookies` / `clearCookies`），
   不是旧 `Network.getCookies` 族。M4 cookie 接线的 CDP 落点以 Storage 域为准。
3. **locator 流不使用 `DOM.getDocument` / `CSS.*` 域** — 元素交互全靠
   `Runtime.callFunctionOn`（utility world 注入脚本）+ `DOM.resolveNode`（backendNodeId →
   objectId）+ `DOM.getContentQuads`/`getBoxModel`。CSS.getMatchedStylesForNode 保留为
   devtools 前置（goal 扩展面，非 Playwright 矩阵项）。
4. **发现端点尾斜杠**：Playwright 请求 `/json/version/`；ZeroWeb 现为精确匹配
   `"/json/version"` → 404 → connectOverCDP 直接失败。M1 须容忍尾斜杠。
5. **sessionId 多路复用无处不在**：除 Storage/Target 浏览器级命令外全部命令带
   `sessionId`，响应必须回显。ZeroWeb 现为单连接扁平会话（无 sessionId），是 M1 结构性
   前提缺口。
6. **`Runtime.callFunctionOn` 是体量最大的命令**（156 次）——remoteObject objectId 桥
   （V8 对象句柄）是 Playwright 可用性的真正脊柱，印证入口文档「深结构」预警。

## 三态登记

三态：✅ 实现 / ⚠️ 部分（命令被接受但语义/形状不全）/ ❌ 不实现（当前返回 `-32601`，符合
DC-3 基线）。「现状」列 v0.4 起以 S17 时点 dispatch 表（时点文件 `domains.rs`，S32 起为
`domains/` 子模块）为 ground truth 逐行核对；行内（Sx）标记对应 master.md 已完成切片编号。

策略记号：**stub** = 先接受返回 `{}`（解附接摩擦），实义语义后续里程碑补。

### 传输与发现（HTTP）

| 端点 | Playwright 使用 | ZeroWeb 现状 | 差距要点 | 计划 |
|------|----------------|--------------|----------|------|
| `GET /json/version`（含 `/` 尾斜杠） | Y（连接入口） | ✅ 尾斜杠容忍（S3）+ 六字段齐 | — | 完成（M1 S3） |
| `GET /json`、`/json/list` | N（chrome://inspect、devtools 用） | ✅ 按真实标签页枚举（S3，`zeroweb-tab-<n>` + shell url/title） | targetId ↔ 标签页映射由 Target 域消费 | 完成（M1 S3） |
| `PUT /json/new`、`/json/close/<id>` 等 | N | ❌ | 不在账本（Playwright 矩阵不需要） | 不实现 |

### Browser 域

| 方法 | 捕获调用 | 参数键 | ZeroWeb 现状 | 计划 |
|------|---------|--------|--------------|------|
| `Browser.getVersion` | 1 | — | ✅（S4） | 完成（M1 S4） |
| `Browser.setDownloadBehavior` | 1 | behavior/downloadPath/eventsEnabled | ✅ stub 接受（S4） | 完成（M1 S4；下载能力待点名） |

### Target 域

| 方法 | 捕获调用 | 参数键 | ZeroWeb 现状 | 计划 |
|------|---------|--------|--------------|------|
| `Target.setAutoAttach` | 4 | autoAttach/flatten/waitForDebuggerOnStart | ✅（S4：浏览器级附接全部 page target + 会话级仅子 target 语义） | 完成（M1 S4） |
| `Target.getTargetInfo` | 1 | — | ✅（S4：浏览器级 + 按 targetId） | 完成（M1 S4） |
| `Target.createTarget` | 2 | url | ✅（S4：autoAttach 时自动附接发事件） | 完成（M1 S4） |
| `Target.closeTarget` | 1 | targetId | ✅（S4：targetDestroyed + 会话摘除） | 完成（M1 S4） |
| `Target.detachFromTarget` | 3 | sessionId | ✅（S4；S25：命令发起的 detachedFromTarget 事件盖发起会话 sessionId——发起方按 flat 路由收到应答；closeTarget 广播路径保持无盖章不变）。e2e：`target.attachDetach`（attachToTarget → detach → 事件配对） | 完成（M1 S4 + S25） |
| `Target.getTargets` | **0（未调用）** | — | ✅ CDP 形状修正（S4，`targetInfos`）。e2e：`target.getTargets`（S25——PW 高层流不调用，经 CDPSession 直发验证） | 完成（M1 S4 + S25 e2e） |
| `Target.attachToTarget` | 0（**前提修正 S25**：高层流不需要，但 PW `context.newCDPSession(page)` 建会话后经此绑定页面 target——raw-CDP 面真实入口） | targetId/flatten | ✅（S25：校验 targetId（缺参 -32602/未知 -32000）→ 登记 → `{sessionId}`，无事件（PW 按响应配对））。e2e：`target.attachDetach` | 完成（S25） |
| `Target.attachToBrowserTarget` | 0（矩阵外；PW `newBrowserCDPSession`/`newCDPSession` 的建会话入口，S25 探针实测） | — | ✅（S25：分配 sessionId 登记到活跃 target，flat 模型浏览器级/页面级命令同面）。e2e：全部 4 个新步的前置建会话路径 | 完成（S25） |

### Runtime 域

| 方法 | 捕获调用 | 参数键 | ZeroWeb 现状 | 计划 |
|------|---------|--------|--------------|------|
| `Runtime.enable` | 5 | — | ✅（S4：补发 executionContextCreated + auxData 契约） | 完成（M1 S4） |
| `Runtime.evaluate` | 8 | contextId/expression | ✅（S9：returnByValue 双分支统一走 objectId 桥——表达式语义修复（W3C ExecuteScript 函数体语义 ≠ CDP evaluate 表达式形态）；对象结果保留句柄返回 `objectId`） | 完成（M4 S9） |
| `Runtime.callFunctionOn` | **156** | arguments/awaitPromise/functionDeclaration/objectId/returnByValue/userGesture | ✅（S9：objectId 路径——句柄为 `this` 调用 + `arguments[].objectId` 实参顶层还原 + `awaitPromise` 落定轮询（microtask drain + timer 泵）；falsy 实参标记误判已修） | 完成（M4 S9） |
| `Runtime.releaseObject` | 51 | objectId | ✅（S9：renderer 注册表释放，objectId 桥配对） | 完成（M4 S9） |
| `Runtime.releaseObjectGroup` | 0（矩阵外） | objectGroup | ✅（S9：整组释放；goal 扩展面）。e2e：`runtime.releaseObjectGroup`（S25——释放后 callFunctionOn 返回 exceptionDetails 即句柄失效语义） | 完成（M4 S9 + S25 e2e） |
| `Runtime.getProperties` | 0（未调用） | objectId/ownProperties | ❌ | 不实现（矩阵不需要；devtools 前置等点名） |
| `Runtime.runIfWaitingForDebugger` | 8 | — | ✅ stub 接受（S4） | 完成（M1 S4） |

### Page 域

| 方法 | 捕获调用 | 参数键 | ZeroWeb 现状 | 计划 |
|------|---------|--------|--------------|------|
| `Page.enable` | 13 | — | ✅（S4） | 完成（M1 S4） |
| `Page.navigate` | 2 | frameId/url/referrerPolicy | ✅（S5：`{frameId,loaderId,errorText?}` 形状 + 导航事件族 + 注入脚本重放） | 完成（M2 S5） |
| `Page.captureScreenshot` | 3 | captureBeyondViewport/clip/format | ✅（S6：CDP `{data:"<b64>"}` 形状 + clip 原始 fb 裁剪；format 仅 png，jpeg -32601；BiDi 对象形不动） | 完成（M3 S6；captureBeyondViewport 随内容尺寸暴露） |
| `Page.getLayoutMetrics` | 3 | — | ✅（S5：headless 固定视口映射，css* 全字段） | 完成（M2 S5；动态视口随 M3 viewport 桥） |
| `Page.handleJavaScriptDialog` | 3 | accept/promptText | ⚠️ stub 接受（S5）。S10 澄清：dialog 步绿因**引擎无阻塞对话框语义**——shim alert no-op、confirm/prompt 立即返回（无事件、无挂起），步骤「不挂起即过」；真对话框事件面属跨流域立项 | 事件源随引擎对话框能力（不阻 M5 收口） |
| `Page.addScriptToEvaluateOnNewDocument` | 3 | source/worldName | ✅（S5：真执行 + 跨导航重放 + worldName 登记/新文档 world context 重发；单引擎主 world 执行） | 完成（M2 S5；world 隔离随引擎能力） |
| `Page.createIsolatedWorld` | 3 | frameId/grantUniveralAccess/worldName | ⚠️ 返回新 contextId + worldName 事件（S4）；world 不隔离（单引擎） | 记账注记；真隔离随引擎能力 |
| `Page.getFrameTree` | 3 | — | ✅（S4：主 frame id=targetId 硬契约；会话级按 target 归属；S39：childFrames 来自子帧元数据记录） | 完成（M1 S4 + S39 扩展） |
| `Page.setLifecycleEventsEnabled` | 3 | enabled | ⚠️ stub 接受（S4）；lifecycleEvent 事件已产（S5 导航族 + S16 write 落定重发）但**不随本开关门控**（恒发） | 记账：门控语义随域收口（PW 消费面不依赖开关） |
| `Page.setFontFamilies` | 3 | fontFamilies | ❌（PW 容忍缺失，实测未阻流） | M3 stub |

### Input 域

| 方法 | 捕获调用 | 参数键 | ZeroWeb 现状 | 计划 |
|------|---------|--------|--------------|------|
| `Input.dispatchMouseEvent` | 35 | button/buttons/clickCount/force/modifiers/type/x/y | ✅（S5：→renderer MouseEvent/ScrollEvent；released 按 clickCount 合成 Click/DblClick；wheel→ScrollEvent） | 完成（M2 S5） |
| `Input.dispatchKeyEvent` | 14 | autoRepeat/code/commands/isKeypad/key/location/modifiers/text/type/unmodifiedText/windowsVirtualKeyCode | ✅（S5：keyDown/rawKeyDown→Down、keyUp→Up、char→Press(text 优先)、modifiers 位解码。S16：accel(Ctrl/Meta)+A → 全选默认动作——`apply_select_all_at`（shim setSelectionRange 全选），keyboard.type+press 绿） | 完成（M2 S5 + S16） |
| `Input.insertText` | 1 | text | ✅（S5：→ImeEvent Commit） | 完成（M2 S5） |

### DOM 域

| 方法 | 捕获调用 | 参数键 | ZeroWeb 现状 | 计划 |
|------|---------|--------|--------------|------|
| `DOM.scrollIntoViewIfNeeded` | 13 | objectId/rect | ✅（S9：句柄桥求值 + shim scrollIntoView 面；无布局对象 → PW 可识别 notvisible 错误形状） | 完成（M4 S9） |
| `DOM.getContentQuads` | 10 | objectId | ✅（S9：句柄桥求值，rect 来自 shim gBCR/RectBridge 真实布局；flat 8 数 quad——click 坐标来源） | 完成（M4 S9） |
| `DOM.describeNode` | 7 | objectId | ✅（S9：句柄桥求值 + `backendNodeId`=句柄 id） | 完成（M4 S9） |
| `DOM.resolveNode` | 7 | backendNodeId/executionContextId | ✅（S9：backendNodeId（=句柄 id）重保留为新句柄返回 `subtype:"node"`——PW adopt 流程（utility→main world 重析）全通） | 完成（M4 S9） |
| `DOM.getBoxModel` | 4 | objectId | ✅（S9：句柄桥求值，四 quad 同 content（headless 单一面板）） | 完成（M4 S9） |
| `DOM.getFrameOwner` | 1 | frameId | ❌ | 不实现（iframe 面随引擎子帧能力，维持挂起） |

### Emulation 域

| 方法 | 捕获调用 | 参数键 | ZeroWeb 现状 | 计划 |
|------|---------|--------|--------------|------|
| `Emulation.setDeviceMetricsOverride` | 2 | deviceScaleFactor/height/mobile/screenHeight/screenOrientation/screenWidth/width | ✅（S6：→renderer SetViewport + 服务器视口状态联动 getLayoutMetrics/captureScreenshot + frameResized 事件；宽高 0=恢复默认） | 完成（M3 S6） |
| `Emulation.setEmulatedMedia` | 4 | features/media | ✅（S6→S11：prefers-color-scheme→SetColorScheme、media type→SetMediaType；S11 补 matchMedia 求值接线——宿主媒体上下文 cell 注入 `__zw_match_media`，PW 读回真值——emulation.media 绿；reduced-motion 等无 IPC 面暂忽略） | 完成（M3 S6 + M4 S11；余 feature 随引擎能力） |
| `Emulation.setFocusEmulationEnabled` | 3 | enabled | ✅ stub 接受（S4） | 完成（M1 S4） |
| `Emulation.setUserAgentOverride` | 2 | userAgent | ✅（S7：proxy_fetch 注入 User-Agent；accept-language 等附带头暂忽略）。e2e：`emulation.userAgentOverride`（S25——img 子资源经 proxy_fetch 携带 override UA，服务端回读断言）。**语义边界记账（S25 实证）**：注入面 = proxy 子资源路径；renderer 直连 fetch（ResourceLoader 观测路径）不经 override——Chromium 全请求语义的差距，随 renderer fetch 管线统一时收口 | 完成（M4 S7 + S25 e2e；边界记账） |

### Network 域

| 方法 | 捕获调用 | 参数键 | ZeroWeb 现状 | 计划 |
|------|---------|--------|--------------|------|
| `Network.enable` | 5 | — | ✅（S7：真实门控 + headless proxy_fetch 生命周期事件 requestWillBeSent/responseReceived/loadingFinished，session 盖章） | 完成（M4 S7；renderer 侧请求观测随 net 观测点扩展） |

### Storage 域

| 方法 | 捕获调用 | 参数键 | ZeroWeb 现状 | 计划 |
|------|---------|--------|--------------|------|
| `Storage.getCookies` | 2 | — | ✅（S7：session 级 jar 全量枚举，CDP cookie 形状） | 完成（M4 S7） |
| `Storage.setCookies` | 1 | cookies | ✅（S7：url/domain+path 作用域 + expires/secure/httpOnly） | 完成（M4 S7） |
| `Storage.clearCookies` | 1 | — | ✅（S7） | 完成（M4 S7） |

### Log 域

| 方法 | 捕获调用 | 参数键 | ZeroWeb 现状 | 计划 |
|------|---------|--------|--------------|------|
| `Log.enable` | 3 | — | ✅ stub 接受（S4） | 完成（M1 S4；entryAdded 不实现-ok） |

### 浏览器事件（S2C 无 id，捕获 30 种）

| 事件 | 次数 | ZeroWeb 现状 | 计划 |
|------|------|--------------|------|
| `Target.attachedToTarget` | 8 | ✅（S4） | 完成（M1 S4） |
| `Target.detachedFromTarget` / `Target.targetDestroyed` | 4 / — | ✅（S4：closeTarget/detachFromTarget 应答；S25：detachFromTarget 命令路径的事件盖发起会话 sessionId，closeTarget 广播保持无盖章）。e2e：`target.attachDetach`（S25） | 完成（M1 S4 + S25） |
| `Runtime.executionContextCreated` | 14 | ✅（S4：auxData.frameId/isDefault 硬契约） | 完成（M1 S4） |
| `Runtime.executionContextsCleared` | 4 | ✅（S5，导航 commit 时） | 完成（M2 S5） |
| `Runtime.executionContextDestroyed` | 2 | ❌ 不单发——导航换代经 `sandbox.reset_context` 整体失效（= CDP context destroyed 语义），由 `executionContextsCleared` 覆盖（S9 实测 PW 消费面绿） | 记账：不实现-ok |
| `Runtime.consoleAPICalled` | 14 | ✅（S11：renderer ConsoleLog IPC → 会话事件排空盖章；value-only remoteObject args + level→CDP type 映射——console.collect 绿） | 完成（M4 S11） |
| `Page.loadEventFired` | 3 | ✅（S5：导航族真 timestamp；S16：document.write 落定重发——PW setContent console tag 清 lifecycle 后等新 load 的时序契约） | 完成（M2 S5 + S16） |
| `Page.frameNavigated` | 3 | ✅（S5，frame.id=targetId） | 完成（M2 S5） |
| `Page.frameStartedLoading` / `frameStoppedLoading` | 3 / 5 | ✅（S5） | 完成（M2 S5） |
| `Page.frameStartedNavigating` | 3 | ❌ | M2（低优） |
| `Page.domContentEventFired` | 3 | ✅（S5） | 完成（M2 S5） |
| `Page.lifecycleEvent` | 42 | 🔶 S5+S16：DOMContentLoaded/load 两点随导航发出 + document.write 落定重发；细粒度事件未逐一生效 | 记账：逐 lifecycle 对齐随 devtools 面需求 |
| `Page.javascriptDialogOpening` / `javascriptDialogClosed` | 3 / 3 | ❌（S10 澄清：引擎无阻塞对话框语义 → 无事件源；dialog 步骤经 shim 立即返回语义通过） | 随引擎对话框能力（不阻 M5 收口） |
| `Page.frameAttached` / `frameDetached` | 1 / 1 | ✅（S39 子帧元数据面：导航事件族内探测 iframe 元素数 → frameAttached{frameId,parentFrameId}、文档换代 frameDetached；记录按主帧分组防跨 target 串扰。语义边界：子帧无文档加载/渲染/JS realm——url 停留 about:blank、无子帧 frameNavigated） | 完成（M3 S39，frames.access 步验证） |
| `Page.frameResized` | 4 | ✅（S6：尺寸变更时发出） | 完成（M3 S6） |
| `Page.documentOpened` | 1 | ❌ | M3（低优） |
| `Page.frameRequestedNavigation` | 1 | ❌ | M3（低优） |
| `Page.frameSubtreeWillBeDetached` | 1 | ❌ | M3（低优） |
| `Network.requestWillBeSent` / `responseReceived` / `loadingFinished` / `dataReceived` | 9 / 8 / 9 / 10 | ✅（S7+：随 proxy_fetch 发出（Network.enable 门控，headers/mimeType/frameId 齐，失败路径 loadingFailed）；**frameId 为 PW 硬要求——缺省请求被丢弃（实测）**。S17：dataReceived 补齐——proxy 子资源路径 + renderer 观测（JS fetch/XHR）双路径均在 loadingFinished 前发出，body 一次性到达语义（dataLength=encodedDataLength=body 字节）；分块流式随 net 观测点流式化） | 完成（M4 S7 + S17） |
| `Network.requestWillBeSentExtraInfo` / `responseReceivedExtraInfo` | 9 / 9 | ❌ | M4（低优，header 面） |
| `Network.policyUpdated` | 6 | ❌ | 不实现（Chromium 内部策略事件） |
| `Log.entryAdded` | 2 | ❌ | 不实现-ok（console 覆盖） |
| `Inspector.workerScriptLoaded` | 2 | ❌ | 不实现（worker 深域，等点名） |

> 事件「必发 vs 可缺」以 M1 首连实测为准（Playwright 对非关键事件的容忍度按实际行为
> 判定，分歧记账）；上表计划列是工作假设。

### goal 扩展面（非 Playwright 矩阵项，devtools goal 前置）

| 方法 | 来源 | 状态（S20 挂账判定） |
|------|------|----------------------|
| `DOM.getDocument` / `DOM.querySelector` / `DOM.querySelectorAll` | 入口文档 M3 句柄桥 | ❌ 挂账——**devtools 面需求驱动**：nodeId 持久化语义/子树分页/backendNodeId 注册表等形状须按 devtools frontend 真实消费面设计（M3 计划作废——S9 实证 locator 脊柱不用此族，提前实现即推测性开发）；句柄桥（S9）为其实现基座 |
| `CSS.getMatchedStylesForNode` | 入口文档 M3 | ❌ 挂账——需 style-system 查询面（**渲染流域专属 crate**，跨流域协调点）；随 devtools 面需求立项 |
| `Target.getTargets` CDP 形状修正 | 见上 | ✅ 完成（M1 S4） |

---

## ZeroWeb 侧实测捕获（S17，2026-09-13）

**方式**：捕获代理 + `capture-core-flow.mjs`（`CDP_ENDPOINT_URL` 指向代理）对 ZeroWeb
headless 重跑全核心流——真实 Playwright 客户端发送面实测（frames×2 期望失败不影响统计）。
明细（生成物）：[zeroweb-capture-2026-09-13-summary.json](zeroweb-capture-2026-09-13-summary.json)
（复现：`node tests/playwright-matrix/scripts/probe-s17-capture.mjs`——S28 起已入库；
生成物写 `out/`，不入 git）。

| 维度 | Chromium 基线 | ZeroWeb S17 | 差异归因 |
|------|--------------|-------------|----------|
| 命令调用 | 395 / 40 方法 | 429 / 35 方法 | 调用数+34 = frames 步骤失败的 PW 重试放大（dispatchMouseEvent 32 vs 35 基线近似） |
| 事件 | 187 / 30 种 | 81 / 17 种 | 未发事件均为账本 ❌/挂起 项（对话框族/子帧族/extraInfo/executionContextDestroyed 等） |

**方法面差异（chromium-only 5 项，零意外缺口）**：
- `DOM.getFrameOwner` — frames×2 挂起（引擎子帧可见性）的下游，PW 未走到
- `Page.handleJavaScriptDialog` — 无 `javascriptDialogOpening` 事件源 → PW 无从应答（S10 澄清）
- `Page.setFontFamilies` — ❌ -32601，PW 容忍（S4 实测）
- `Target.detachFromTarget` — ZW 面关闭路径差异，PW 未调用（drift 记账，不影响消费面；
  **S25 已以 `target.attachDetach` 步直发验证**）
- `Emulation.setUserAgentOverride` — PW 未对 ZW 调用（Chromium 会话 init 发 2 次）；
  ZW 的 `Browser.getVersion` UA 直读路径使 override 非必需（drift 记账；
  **S25 已以 `emulation.userAgentOverride` 步直发验证**）

**结论**：35 个被调方法全部为账本「实现」态方法，**真实客户端命令面与账本登记零漂移**；
M5 矩阵收口的实测复核通过。

### S28 复现链验证 + 组合态捕获复核（2026-09-13）

复现脚本 `probe-s17-capture.mjs` 入库（原样，保留证据产出溯源），并在当前 tip
（S27，含 S25 raw-CDP 面）实跑验证——**复现链闭合，零未登记漂移**：

| 维度 | S17 时点 | S28 复核（当前 tip） | 漂移归因 |
|------|---------|---------------------|----------|
| 命令调用 | 429 / 35 方法 | 456 / **41** 方法 | +4 方法 = S25 补测步集合（`Target.getTargets`/`attachToTarget`/`attachToBrowserTarget`/`Runtime.releaseObjectGroup`，账本全登记）；调用数增量为新增步 + PW 重试放大 |
| 事件 | 81 / 17 种 | 136 / **17** 种 | 类型零漂移；数量随步数增长 |
| chromium-only 缺口 | 5 项 | **3 项** | S25 补测覆盖 `detachFromTarget`/`setUserAgentOverride`；余 3 项挂账有因（getFrameOwner=frames 挂起下游、handleJavaScriptDialog=无事件源、setFontFamilies=-32601 容忍） |

## ZeroWeb 侧 DC-1 覆盖审计（S25，2026-09-13）

**审计口径**：DC-1 要求「实现」态每命令 ≥1 个 E2E 用例。S17 捕获（35 方法）只证明
「被 PW 高层流调用的命令已验证」——不覆盖「已实现但 PW 高层流不调用」的命令。以
S17 捕获方法集 × 矩阵实现态清单做差集，**缺口 4 项**：`Target.getTargets`、
`Target.detachFromTarget`、`Runtime.releaseObjectGroup`、`Emulation.setUserAgentOverride`。

**S25 收口**：
- 新增 4 个 e2e 步（`target.getTargets` / `target.attachDetach` /
  `runtime.releaseObjectGroup` / `emulation.userAgentOverride`），绿步 28 → **32**，
  expected-green 基线同步扩至 32（全量 34 步，余 frames×2 挂起不变）。
- 补齐 raw-CDP 会话面 2 命令（探针实证 PW 客户端真实依赖）：`Target.attachToBrowserTarget`
  （`newBrowserCDPSession`/`newCDPSession` 建会话入口）、`Target.attachToTarget`
  （newCDPSession(page) 绑定页面 target）——账本由「矩阵外/❌」转 ✅。
- `target.attachDetach` 同时覆盖 attachToTarget/detachFromTarget/detachedFromTarget
  事件三面（事件盖发起会话后 PW 可收、可断言 sessionId 配对）。

**工具坑（learning）**：PW `session.send()` 对桥 miss 语义不 throw——headless 把
automation 错误译为 `exceptionDetails`（200 形响应），断言须查 `exceptionDetails` 而非
异常捕获（`runtime.releaseObjectGroup` 步实证）。

---

## ZeroWeb 结构性缺口（域实现的前提）

| # | 缺口 | 影响 |
|---|------|------|
| G1 | WS 层 sessionId 多路复用 | 🔶 传输面已解（S3：解析/回显/未附接 -32001/附接注册表）；per-target 真路由随 Target 域（切片 3） |
| G2 | `/json/version` 尾斜杠 404 | ✅ 已解（S3） |
| G2b | tungstenite write() 缓冲不落盘 + peek 5s read timeout 未恢复 → 任何 CDP 客户端收不到响应 | ✅ 已解（S3，实测发现；learning 2026-09-12） |
| G3 | `Runtime.evaluate` 扁平字符串结果，无 remoteObject/objectId | ✅ 已解（S9：objectId 全量 remoteObject 桥——注册表/renderer 原语/双路径 evaluate + DOM 域句柄面） |
| G4 | 无请求事件总线（net 生命周期无观测点） | 🔶 雏形已建（S7 proxy_fetch 生命周期 + S14 renderer FetchObserved 观测管线 + S17 dataReceived）；分块流式观测点待 net 窗口流式化（P6） |
| G5 | console 走 `__zw_console_log` 扁平字符串宿主回调 | consoleAPICalled remoteObject 形态（P5） |
| G6 | `headless.rs` 2256 行超 2000 上限 | ✅ 已解（S2 拆分 9 模块） |

---

## M5 定稿口径（2026-09-16，分支 B——挂账剔除定稿，用户批复落账）

**批复**：2026-09-16 用户批复「DC-2 口径 = 分支 B（挂账 + 口径剔除定稿，goal 先行
DONE），按 M5 定稿预案机械执行，附三条件」（入树提交 5af87ab69）。

**绿步基线定稿**：expected-green **33** 步（`frames.access` 在列；全量 35 步面 =
33 基线绿 + 基线外 1 步 + `frames.click+evaluate` 挂账剔除）。`frames.click+evaluate`
**继续跑、不门禁**——挂账非豁免：阻塞方 = 渲染流域子帧能力冻结（子帧文档加载 +
JS realm + child quads 三件套），解冻后一轮回填 34/34 并撤剔除（去处：master.md
子帧解冻清单 + rendering-compat master.md 待用户决策清单 GB-20260916 落账条目）。

**挂账清单终稿（不实现域 + 有因挂起项，随本节定稿）**：

| 项 | 态 | 归因 |
|----|----|------|
| `frames.click+evaluate` | 挂账剔除（继续跑不门禁） | 引擎子帧能力三件套未落地（渲染流域专属 crate 面），非本流单方可解；解冻回填 |
| `Page.handleJavaScriptDialog` | 有因挂起（绿步经语义挂账路径） | 引擎无阻塞对话框语义（shim alert no-op、confirm/prompt 立即返回，无 `javascriptDialogOpening` 事件源）——S10 现状澄清，等引擎对话框语义立项 |
| `DOM.getFrameOwner` | 有因挂起 | frames 挂起下游（PW 未走到）；随子帧能力解冻一并复核 |
| `Page.setFontFamilies` | 不实现（-32601 容忍） | PW 容忍路径（S4 实测），devtools 面需求时再评估 |
| Tracing / Profiler / Debugger 断点深域 | 不实现（-32601） | goal 排除项（入口文档排除清单），等点名 |

**实测复核账（定稿依据）**：S17 真实客户端捕获 35 方法零漂移；S25 DC-1 覆盖审计
缺口 4 项补齐（实现态命令 e2e 全覆盖）；S28 复现链闭合零未登记漂移；门禁
`make cdp-e2e` 自 S8 起每轮防回归（S893/S894/S895 连续 PASS 33 绿 deterministic
双跑 YES，首调红形态连续三百余次零再现）。
