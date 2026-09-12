# CDP 命令矩阵账本（DC-1 控制件）

**版本**: v0.1（M1 前置纯资产切片产出）
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
DC-3 基线）。「现状」列以 2026-09-12 `apps/browser/src/headless.rs`（2256 行）为准。

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
| `Browser.getVersion` | 1 | — | ❌ | M1（数据已有，纯形状） |
| `Browser.setDownloadBehavior` | 1 | behavior/downloadPath/eventsEnabled | ❌ | M1 stub |

### Target 域

| 方法 | 捕获调用 | 参数键 | ZeroWeb 现状 | 计划 |
|------|---------|--------|--------------|------|
| `Target.setAutoAttach` | 4 | autoAttach/flatten/waitForDebuggerOnStart | ❌ | M1（连接脊柱） |
| `Target.getTargetInfo` | 1 | — | ❌ | M1 |
| `Target.createTarget` | 2 | url | ❌ | M1（newPage 依赖） |
| `Target.closeTarget` | 1 | targetId | ❌ | M2 |
| `Target.detachFromTarget` | 3 | sessionId | ❌ | M2 |
| `Target.getTargets` | **0（未调用）** | — | ⚠️ 返回 BiDi tree 形状非 CDP `targetInfos` | M1 修形（devtools 面兜底） |
| `Target.attachToTarget` | 0（flatten 路径不需要） | — | ❌ | 不实现（账本注明） |

### Runtime 域

| 方法 | 捕获调用 | 参数键 | ZeroWeb 现状 | 计划 |
|------|---------|--------|--------------|------|
| `Runtime.enable` | 5 | — | ❌ | M1 |
| `Runtime.evaluate` | 8 | contextId/expression | ⚠️ 扁平 `{type:"string",value}`；无 objectId/exceptionDetails/contextId | M1（remoteObject 雏形） |
| `Runtime.callFunctionOn` | **156** | arguments/awaitPromise/functionDeclaration/objectId/returnByValue/userGesture | ❌ | M1 雏形 → M3 全量（objectId 桥） |
| `Runtime.releaseObject` | 51 | objectId | ❌ | M1（objectId 生命周期） |
| `Runtime.runIfWaitingForDebugger` | 8 | — | ❌ | M1 stub |

### Page 域

| 方法 | 捕获调用 | 参数键 | ZeroWeb 现状 | 计划 |
|------|---------|--------|--------------|------|
| `Page.enable` | 13 | — | ❌ | M1 |
| `Page.navigate` | 2 | frameId/url/referrerPolicy | ⚠️ 返回 `{url,title,success}` ≠ `{frameId,loaderId}`；无 frameId/referrer 参数 | M2（形状对齐 + 事件族） |
| `Page.captureScreenshot` | 3 | captureBeyondViewport/clip/format | ⚠️ 无 clip/format/captureBeyondViewport | M2 |
| `Page.getLayoutMetrics` | 3 | — | ❌ | M2 |
| `Page.handleJavaScriptDialog` | 3 | accept/promptText | ❌ | M2 |
| `Page.addScriptToEvaluateOnNewDocument` | 3 | source/worldName | ❌ | M2（utility 脚本，locator 依赖） |
| `Page.createIsolatedWorld` | 3 | frameId/grantUniveralAccess/worldName | ❌ | M2 |
| `Page.getFrameTree` | 3 | — | ❌ | M2 |
| `Page.setLifecycleEventsEnabled` | 3 | enabled | ❌ | M2（导航等待的事件源） |
| `Page.setFontFamilies` | 3 | fontFamilies | ❌ | M2 stub |

### Input 域

| 方法 | 捕获调用 | 参数键 | ZeroWeb 现状 | 计划 |
|------|---------|--------|--------------|------|
| `Input.dispatchMouseEvent` | 35 | button/buttons/clickCount/force/modifiers/type/x/y | ❌ | M2 |
| `Input.dispatchKeyEvent` | 14 | autoRepeat/code/commands/isKeypad/key/location/modifiers/text/type/unmodifiedText/windowsVirtualKeyCode | ❌ | M2 |
| `Input.insertText` | 1 | text | ❌ | M2 |

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
| `Emulation.setEmulatedMedia` | 4 | features/media | ❌ | M3 |
| `Emulation.setFocusEmulationEnabled` | 3 | enabled | ❌ | M3 stub |
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
| `Log.enable` | 3 | — | ❌ | M2 stub（`Log.entryAdded` 可选） |

### 浏览器事件（S2C 无 id，捕获 30 种）

| 事件 | 次数 | ZeroWeb 现状 | 计划 |
|------|------|--------------|------|
| `Target.attachedToTarget` | 8 | ❌ | M1（auto-attach 应答） |
| `Runtime.executionContextCreated` | 14 | ❌ | M1 |
| `Runtime.executionContextsCleared` | 4 | ❌ | M2 |
| `Runtime.executionContextDestroyed` | 2 | ❌ | M2 |
| `Runtime.consoleAPICalled` | 14 | ❌ | M4（console 对象化，P5 缺口） |
| `Page.loadEventFired` | 3 | ⚠️ 有雏形（timestamp 恒 0.0） | M2 |
| `Page.frameNavigated` | 3 | ❌ | M2 |
| `Page.frameStartedLoading` / `frameStoppedLoading` | 3 / 5 | ❌ | M2 |
| `Page.frameStartedNavigating` | 3 | ❌ | M2（低优） |
| `Page.domContentEventFired` | 3 | ❌ | M2 |
| `Page.lifecycleEvent` | 42 | ❌ | M2（setLifecycleEventsEnabled 开关下） |
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
| G3 | `Runtime.evaluate` 扁平字符串结果，无 remoteObject/objectId | 全部 evaluate/locator 流 |
| G4 | 无请求事件总线（net 生命周期无观测点） | Network 域 + devtools 面板（P6） |
| G5 | console 走 `__zw_console_log` 扁平字符串宿主回调 | consoleAPICalled remoteObject 形态（P5） |
| G6 | `headless.rs` 2256 行超 2000 上限 | ✅ 已解（S2 拆分 9 模块） |
