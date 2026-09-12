# CDP 命令矩阵账本（DC-1 控制件）

**版本**: v0.2（S4 后三态推进；v0.1 为 M1 前置纯资产切片初稿）
**日期**: 2026-09-12
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
DC-3 基线）。「现状」列以 2026-09-12 `apps/browser/src/headless/`（M1 拆分后模块树，S4
切片状态）为准；行内（Sx）标记对应 master.md 已完成切片编号。

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
| `Target.detachFromTarget` | 3 | sessionId | ✅（S4） | 完成（M1 S4） |
| `Target.getTargets` | **0（未调用）** | — | ✅ CDP 形状修正（S4，`targetInfos`） | 完成（M1 S4） |
| `Target.attachToTarget` | 0（flatten 路径不需要） | — | ❌ | 不实现（账本注明） |

### Runtime 域

| 方法 | 捕获调用 | 参数键 | ZeroWeb 现状 | 计划 |
|------|---------|--------|--------------|------|
| `Runtime.enable` | 5 | — | ✅（S4：补发 executionContextCreated + auxData 契约） | 完成（M1 S4） |
| `Runtime.evaluate` | 8 | contextId/expression | ⚠️ 类型化 remoteObject（S4，returnByValue）+ exceptionDetails；**缺 objectId 句柄（utilityScript 深结构，待用户决策）** | M1 收口挂 objectId 桥 |
| `Runtime.callFunctionOn` | **156** | arguments/awaitPromise/functionDeclaration/objectId/returnByValue/userGesture | ⚠️ 无 objectId 路径已实现（S4：value 参数 + 表达式包装 + 类型化返回）；objectId 路径 -32601 | 收口挂 objectId 桥（待用户决策） |
| `Runtime.releaseObject` | 51 | objectId | ❌（与 objectId 桥配对，挂起） | objectId 桥获批后 |
| `Runtime.runIfWaitingForDebugger` | 8 | — | ✅ stub 接受（S4） | 完成（M1 S4） |

### Page 域

| 方法 | 捕获调用 | 参数键 | ZeroWeb 现状 | 计划 |
|------|---------|--------|--------------|------|
| `Page.enable` | 13 | — | ✅（S4） | 完成（M1 S4） |
| `Page.navigate` | 2 | frameId/url/referrerPolicy | ✅（S5：`{frameId,loaderId,errorText?}` 形状 + 导航事件族 + 注入脚本重放） | 完成（M2 S5） |
| `Page.captureScreenshot` | 3 | captureBeyondViewport/clip/format | ⚠️ 无 clip/format/captureBeyondViewport | M2 |
| `Page.getLayoutMetrics` | 3 | — | ✅（S5：headless 固定视口映射，css* 全字段） | 完成（M2 S5；动态视口随 M3 viewport 桥） |
| `Page.handleJavaScriptDialog` | 3 | accept/promptText | ⚠️ stub 接受（S5）；引擎无阻塞式对话框语义 → 无 javascriptDialogOpening 事件源 | 事件源随引擎对话框能力 |
| `Page.addScriptToEvaluateOnNewDocument` | 3 | source/worldName | ✅（S5：真执行 + 跨导航重放 + worldName 登记/新文档 world context 重发；单引擎主 world 执行） | 完成（M2 S5；world 隔离随引擎能力） |
| `Page.createIsolatedWorld` | 3 | frameId/grantUniveralAccess/worldName | ⚠️ 返回新 contextId + worldName 事件（S4）；world 不隔离（单引擎） | 记账注记；真隔离随引擎能力 |
| `Page.getFrameTree` | 3 | — | ✅（S4：主 frame id=targetId 硬契约；会话级按 target 归属） | 完成（M1 S4） |
| `Page.setLifecycleEventsEnabled` | 3 | enabled | ⚠️ stub 接受（S4）；lifecycleEvent 事件未产 | M2 实义 |
| `Page.setFontFamilies` | 3 | fontFamilies | ❌（PW 容忍缺失，实测未阻流） | M3 stub |

### Input 域

| 方法 | 捕获调用 | 参数键 | ZeroWeb 现状 | 计划 |
|------|---------|--------|--------------|------|
| `Input.dispatchMouseEvent` | 35 | button/buttons/clickCount/force/modifiers/type/x/y | ✅（S5：→renderer MouseEvent/ScrollEvent；released 按 clickCount 合成 Click/DblClick；wheel→ScrollEvent） | 完成（M2 S5） |
| `Input.dispatchKeyEvent` | 14 | autoRepeat/code/commands/isKeypad/key/location/modifiers/text/type/unmodifiedText/windowsVirtualKeyCode | ✅（S5：keyDown/rawKeyDown→Down、keyUp→Up、char→Press(text 优先)、modifiers 位解码） | 完成（M2 S5） |
| `Input.insertText` | 1 | text | ✅（S5：→ImeEvent Commit） | 完成（M2 S5） |

### DOM 域

| 方法 | 捕获调用 | 参数键 | ZeroWeb 现状 | 计划 |
|------|---------|--------|--------------|------|
| `DOM.scrollIntoViewIfNeeded` | 13 | objectId/rect | ❌ | M3 |
| `DOM.getContentQuads` | 10 | objectId | ❌ | M3（click 坐标来源） |
| `DOM.describeNode` | 7 | objectId | ❌ | M3 |
| `DOM.resolveNode` | 7 | backendNodeId/executionContextId | ❌ | M3（NodeId↔objectId 桥） |
| `DOM.getBoxModel` | 4 | objectId | ❌ | M3 |
| `DOM.getFrameOwner` | 1 | frameId | ❌ | M3 |

### Emulation 域

| 方法 | 捕获调用 | 参数键 | ZeroWeb 现状 | 计划 |
|------|---------|--------|--------------|------|
| `Emulation.setDeviceMetricsOverride` | 2 | deviceScaleFactor/height/mobile/screenHeight/screenOrientation/screenWidth/width | ❌ | M3（viewport 桥） |
| `Emulation.setEmulatedMedia` | 4 | features/media | ⚠️ stub 接受（S4）；媒体仿真未生效 | M3 实义 |
| `Emulation.setFocusEmulationEnabled` | 3 | enabled | ✅ stub 接受（S4） | 完成（M1 S4） |
| `Emulation.setUserAgentOverride` | 2 | userAgent | ❌ | M3 stub → M4 实义（net UA 接线） |

### Network 域

| 方法 | 捕获调用 | 参数键 | ZeroWeb 现状 | 计划 |
|------|---------|--------|--------------|------|
| `Network.enable` | 5 | — | ⚠️ stub 接受不产事件 | M4（请求事件总线，P6 缺口） |

### Storage 域

| 方法 | 捕获调用 | 参数键 | ZeroWeb 现状 | 计划 |
|------|---------|--------|--------------|------|
| `Storage.getCookies` | 2 | — | ❌ | M4（cookie jar；域落点修正见发现 #2） |
| `Storage.setCookies` | 1 | cookies | ❌ | M4 |
| `Storage.clearCookies` | 1 | — | ❌ | M4 |

### Log 域

| 方法 | 捕获调用 | 参数键 | ZeroWeb 现状 | 计划 |
|------|---------|--------|--------------|------|
| `Log.enable` | 3 | — | ✅ stub 接受（S4） | 完成（M1 S4；entryAdded 不实现-ok） |

### 浏览器事件（S2C 无 id，捕获 30 种）

| 事件 | 次数 | ZeroWeb 现状 | 计划 |
|------|------|--------------|------|
| `Target.attachedToTarget` | 8 | ✅（S4） | 完成（M1 S4） |
| `Target.detachedFromTarget` / `Target.targetDestroyed` | 4 / — | ✅（S4：closeTarget/detachFromTarget 应答） | 完成（M1 S4） |
| `Runtime.executionContextCreated` | 14 | ✅（S4：auxData.frameId/isDefault 硬契约） | 完成（M1 S4） |
| `Runtime.executionContextsCleared` | 4 | ✅（S5，导航 commit 时） | 完成（M2 S5） |
| `Runtime.executionContextDestroyed` | 2 | ❌ | M2 |
| `Runtime.consoleAPICalled` | 14 | ❌ | M4（console 对象化，P5 缺口） |
| `Page.loadEventFired` | 3 | ⚠️ 有雏形（timestamp 恒 0.0） | M2 |
| `Page.frameNavigated` | 3 | ✅（S5，frame.id=targetId） | 完成（M2 S5） |
| `Page.frameStartedLoading` / `frameStoppedLoading` | 3 / 5 | ✅（S5） | 完成（M2 S5） |
| `Page.frameStartedNavigating` | 3 | ❌ | M2（低优） |
| `Page.domContentEventFired` | 3 | ✅（S5） | 完成（M2 S5） |
| `Page.lifecycleEvent` | 42 | 🔶 S5：DOMContentLoaded/load 两点随导航发出；细粒度事件未逐一生效 | M4 补齐（逐 lifecycle 对齐） |
| `Page.javascriptDialogOpening` / `javascriptDialogClosed` | 3 / 3 | ❌ | M2 |
| `Page.frameAttached` / `frameDetached` | 1 / 1 | ❌ | M2 |
| `Page.frameResized` | 4 | ❌ | M3（viewport 变更时） |
| `Page.documentOpened` | 1 | ❌ | M3（低优） |
| `Page.frameRequestedNavigation` | 1 | ❌ | M3（低优） |
| `Page.frameSubtreeWillBeDetached` | 1 | ❌ | M3（低优） |
| `Network.requestWillBeSent` / `responseReceived` / `loadingFinished` / `dataReceived` | 9 / 8 / 9 / 10 | ❌ | M4 |
| `Network.requestWillBeSentExtraInfo` / `responseReceivedExtraInfo` | 9 / 9 | ❌ | M4（低优，header 面） |
| `Network.policyUpdated` | 6 | ❌ | 不实现（Chromium 内部策略事件） |
| `Log.entryAdded` | 2 | ❌ | 不实现-ok（console 覆盖） |
| `Inspector.workerScriptLoaded` | 2 | ❌ | 不实现（worker 深域，等点名） |

> 事件「必发 vs 可缺」以 M1 首连实测为准（Playwright 对非关键事件的容忍度按实际行为
> 判定，分歧记账）；上表计划列是工作假设。

### goal 扩展面（非 Playwright 矩阵项，devtools goal 前置）

| 方法 | 来源 | 计划 |
|------|------|------|
| `DOM.getDocument` / `DOM.querySelector` / `DOM.querySelectorAll` | 入口文档 M3 句柄桥 | M3 |
| `CSS.getMatchedStylesForNode` | 入口文档 M3 | M3 |
| `Target.getTargets` CDP 形状修正 | 见上 | M1 |

---

## ZeroWeb 结构性缺口（域实现的前提）

| # | 缺口 | 影响 |
|---|------|------|
| G1 | WS 层 sessionId 多路复用 | 🔶 传输面已解（S3：解析/回显/未附接 -32001/附接注册表）；per-target 真路由随 Target 域（切片 3） |
| G2 | `/json/version` 尾斜杠 404 | ✅ 已解（S3） |
| G2b | tungstenite write() 缓冲不落盘 + peek 5s read timeout 未恢复 → 任何 CDP 客户端收不到响应 | ✅ 已解（S3，实测发现；learning 2026-09-12） |
| G3 | `Runtime.evaluate` 扁平字符串结果，无 remoteObject/objectId | 🔶 类型化 returnByValue 已解（S4）；objectId 句柄待用户决策（utilityScript 深结构） |
| G4 | 无请求事件总线（net 生命周期无观测点） | Network 域 + devtools 面板（P6） |
| G5 | console 走 `__zw_console_log` 扁平字符串宿主回调 | consoleAPICalled remoteObject 形态（P5） |
| G6 | `headless.rs` 2256 行超 2000 上限 | ✅ 已解（S2 拆分 9 模块） |
