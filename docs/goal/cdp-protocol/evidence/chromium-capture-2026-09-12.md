# Chromium CDP 捕获明细（生成物，勿手改）

- 源：`tests/playwright-matrix/out/capture.jsonl`（playwright-core 1.63.0 经捕获代理 connectOverCDP）
- 汇总：395 次调用 / 40 个唯一方法；187 次事件 / 30 个唯一事件
- HTTP 发现端点：`GET /json/version/`

## Browser

| 方法 | 调用 | ok | err | sessionId | 参数键 | 结果样例（截断） | 错误样例 |
|------|------|----|-----|-----------|--------|------------------|----------|
| `Browser.getVersion` | 1 | 1 | 0 |  |  | `{"protocolVersion":"1.3","product":"Chrome/153.0.8010.12","revision":"@971a7443b0c9b0a9b2860529b33331b76077ec62","userAgent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) HeadlessChrome/153.0.0.0 Safari/537.36","jsVersion":"15.3.76.4"}` |  |
| `Browser.setDownloadBehavior` | 1 | 1 | 0 |  | `behavior` `downloadPath` `eventsEnabled` | `{}` |  |

## DOM

| 方法 | 调用 | ok | err | sessionId | 参数键 | 结果样例（截断） | 错误样例 |
|------|------|----|-----|-----------|--------|------------------|----------|
| `DOM.describeNode` | 7 | 7 | 0 | Y | `objectId` | `{"node":{"nodeId":0,"backendNodeId":9,"nodeType":1,"nodeName":"DIV","localName":"div","nodeValue":"","childNodeCount":1,"attributes":["id","clicked-1"]}}` |  |
| `DOM.getBoxModel` | 4 | 4 | 0 | Y | `objectId` | `{"model":{"content":[205,82.875,253.875,82.875,253.875,97.875,205,97.875],"padding":[199,81.875,259.875,81.875,259.875,98.875,199,98.875],"border":[197,79.875,261.875,79.875,261.875,100.875,197,100.875],"margin":[197,79.875,261.875,79.875,261.875,100.875,197,100.875],"width":65,"height":21}}` |  |
| `DOM.getContentQuads` | 10 | 10 | 0 | Y | `objectId` | `{"quads":[[197,79.875,261.875,79.875,261.875,100.875,197,100.875]]}` |  |
| `DOM.getFrameOwner` | 1 | 1 | 0 | Y | `frameId` | `{"backendNodeId":8}` |  |
| `DOM.resolveNode` | 7 | 7 | 0 | Y | `backendNodeId` `executionContextId` | `{"object":{"type":"object","subtype":"node","className":"HTMLDivElement","description":"div#clicked-1","objectId":"-9199741618047099485.3.5"}}` |  |
| `DOM.scrollIntoViewIfNeeded` | 13 | 13 | 0 | Y | `objectId` `rect` | `{}` |  |

## Emulation

| 方法 | 调用 | ok | err | sessionId | 参数键 | 结果样例（截断） | 错误样例 |
|------|------|----|-----|-----------|--------|------------------|----------|
| `Emulation.setDeviceMetricsOverride` | 2 | 2 | 0 | Y | `deviceScaleFactor` `height` `mobile` `screenHeight` `screenOrientation` `screenWidth` `width` | `{}` |  |
| `Emulation.setEmulatedMedia` | 4 | 4 | 0 | Y | `features` `media` | `{}` |  |
| `Emulation.setFocusEmulationEnabled` | 3 | 3 | 0 | Y | `enabled` | `{}` |  |
| `Emulation.setUserAgentOverride` | 2 | 2 | 0 | Y | `userAgent` | `{}` |  |

## Input

| 方法 | 调用 | ok | err | sessionId | 参数键 | 结果样例（截断） | 错误样例 |
|------|------|----|-----|-----------|--------|------------------|----------|
| `Input.dispatchKeyEvent` | 14 | 14 | 0 | Y | `autoRepeat` `code` `commands` `isKeypad` `key` `location` `modifiers` `text` `type` `unmodifiedText` `windowsVirtualKeyCode` | `{}` |  |
| `Input.dispatchMouseEvent` | 35 | 35 | 0 | Y | `button` `buttons` `clickCount` `force` `modifiers` `type` `x` `y` | `{}` |  |
| `Input.insertText` | 1 | 1 | 0 | Y | `text` | `{}` |  |

## Log

| 方法 | 调用 | ok | err | sessionId | 参数键 | 结果样例（截断） | 错误样例 |
|------|------|----|-----|-----------|--------|------------------|----------|
| `Log.enable` | 3 | 3 | 0 | Y |  | `{}` |  |

## Network

| 方法 | 调用 | ok | err | sessionId | 参数键 | 结果样例（截断） | 错误样例 |
|------|------|----|-----|-----------|--------|------------------|----------|
| `Network.enable` | 5 | 5 | 0 | Y |  | `{}` |  |

## Page

| 方法 | 调用 | ok | err | sessionId | 参数键 | 结果样例（截断） | 错误样例 |
|------|------|----|-----|-----------|--------|------------------|----------|
| `Page.addScriptToEvaluateOnNewDocument` | 3 | 3 | 0 | Y | `source` `worldName` | `{"identifier":"1"}` |  |
| `Page.captureScreenshot` | 3 | 3 | 0 | Y | `captureBeyondViewport` `clip` `format` | `{"data":"iVBORw0KGgoAAAANSUhEUgAAAyAAAAJYCAIAAAAVFBUnAAAQAElEQVR4nOzde7xWY/4//hURksM4nxWfGJRTyPk4ZZAwJVT6TOSQPiahlIzjkGnwmQZFzikSapyNUgg55zgiUkREoiJEv/dnr9/cj/u7d3vb7X3J3vV8/nE/1r3u6173tdZ9rbVe67rWvnfdhQsXZgAApLNcBgBAUgIWAEBiAhYAQGICFgBAYgIWAEBiAhYAQGICFgBAYgIWAEBiAhYAQGICFgBAYgIWAEBiAhYAQGICFgBAYgIWAEBiAhYAQGICFgBAYgIWAEBiAhYAQGICFgBAYgIWAEBiAhYAQGICFgBAYgIWAEBiAhYAQGICFgBAYgIWAEBiAhYAQGLLYsCaOXNm586dt99++7XXXrtBgwZbbbXVn/70pzr/rxVXXDEDAKiSZS5gPf3009ttt93NN9/82muvffPNN40bN548efL777//6quv/v73v88AAKpt2QpY8+bN++///u/PPvssfxph66WXXurWrVtMN23adMcdd8wAAKqtbrYsueGGG6K/Kp+OkcE8UZ144onRm5UBACSybAWs4iC…` |  |
| `Page.createIsolatedWorld` | 3 | 3 | 0 | Y | `frameId` `grantUniveralAccess` `worldName` | `{"executionContextId":2}` |  |
| `Page.enable` | 13 | 13 | 0 | Y |  | `{}` |  |
| `Page.getFrameTree` | 3 | 3 | 0 | Y |  | `{"frameTree":{"frame":{"id":"EA06B199540F05605223812A6370E570","loaderId":"E7C8EBC4D432074B7CD5BE0FCF9927CA","url":"about:blank","domainAndRegistry":"","securityOrigin":"://","securityOriginDetails":{"isLocalhost":false},"mimeType":"text/html","adFrameStatus":{"adFrameType":"none"},"secureContextType":"InsecureScheme","crossOriginIsolatedContextType":"NotIsolated","gatedAPIFeatures":[]}}}` |  |
| `Page.getLayoutMetrics` | 3 | 3 | 0 | Y |  | `{"layoutViewport":{"pageX":0,"pageY":0,"clientWidth":800,"clientHeight":600},"visualViewport":{"offsetX":0,"offsetY":0,"pageX":0,"pageY":0,"clientWidth":800,"clientHeight":600,"scale":1,"zoom":1},"contentSize":{"x":0,"y":0,"width":800,"height":600},"cssLayoutViewport":{"pageX":0,"pageY":0,"clientWidth":800,"clientHeight":600},"cssVisualViewport":{"offsetX":0,"offsetY":0,"pageX":0,"pageY":0,"clientWidth":800,"clientHeight":600,"scale":1,"zoom":1},"cssContentSize":{"x":0,"y":0,"width":800,"height":600}}` |  |
| `Page.handleJavaScriptDialog` | 3 | 3 | 0 | Y | `accept` `promptText` | `{}` |  |
| `Page.navigate` | 2 | 2 | 0 | Y | `frameId` `referrerPolicy` `url` | `{"frameId":"623A961F257026804E504BB613B03239","loaderId":"7357B5D8EEC4B45D3744305599DAC4C0","isDownload":false}` |  |
| `Page.setFontFamilies` | 3 | 3 | 0 | Y | `fontFamilies` | `{}` |  |
| `Page.setLifecycleEventsEnabled` | 3 | 3 | 0 | Y | `enabled` | `{}` |  |

## Runtime

| 方法 | 调用 | ok | err | sessionId | 参数键 | 结果样例（截断） | 错误样例 |
|------|------|----|-----|-----------|--------|------------------|----------|
| `Runtime.callFunctionOn` | 156 | 156 | 0 | Y | `arguments` `awaitPromise` `functionDeclaration` `objectId` `returnByValue` `userGesture` | `{"result":{"type":"string","value":"matrix-main"}}` |  |
| `Runtime.enable` | 5 | 5 | 0 | Y |  | `{}` |  |
| `Runtime.evaluate` | 8 | 8 | 0 | Y | `contextId` `expression` | `{"result":{"type":"object","className":"UtilityScript","description":"UtilityScript","objectId":"-9199741618047099485.4.1"}}` |  |
| `Runtime.releaseObject` | 51 | 51 | 0 | Y | `objectId` | `{}` |  |
| `Runtime.runIfWaitingForDebugger` | 8 | 8 | 0 | Y |  | `{}` |  |

## Storage

| 方法 | 调用 | ok | err | sessionId | 参数键 | 结果样例（截断） | 错误样例 |
|------|------|----|-----|-----------|--------|------------------|----------|
| `Storage.clearCookies` | 1 | 1 | 0 |  |  | `{}` |  |
| `Storage.getCookies` | 2 | 2 | 0 |  |  | `{"cookies":[{"name":"zw","value":"1","domain":"127.0.0.1","path":"/","expires":-1,"size":3,"httpOnly":false,"secure":false,"session":true,"priority":"Medium","sourceScheme":"NonSecure","sourcePort":80}]}` |  |
| `Storage.setCookies` | 1 | 1 | 0 |  | `cookies` | `{}` |  |

## Target

| 方法 | 调用 | ok | err | sessionId | 参数键 | 结果样例（截断） | 错误样例 |
|------|------|----|-----|-----------|--------|------------------|----------|
| `Target.closeTarget` | 1 | 1 | 0 |  | `targetId` | `{"success":true}` |  |
| `Target.createTarget` | 2 | 2 | 0 |  | `url` | `{"targetId":"623A961F257026804E504BB613B03239"}` |  |
| `Target.detachFromTarget` | 3 | 3 | 0 |  | `sessionId` | `{}` |  |
| `Target.getTargetInfo` | 1 | 1 | 0 |  |  | `{"targetInfo":{"targetId":"d083a748-5aa6-49f9-98a6-e36c8534a8a3","type":"browser","title":"","url":"","attached":true,"canAccessOpener":false}}` |  |
| `Target.setAutoAttach` | 4 | 4 | 0 | Y | `autoAttach` `flatten` `waitForDebuggerOnStart` | `{}` |  |

## 浏览器事件（S2C，无 id）

| 事件 | 次数 |
|------|------|
| `Inspector.workerScriptLoaded` | 2 |
| `Log.entryAdded` | 2 |
| `Network.dataReceived` | 10 |
| `Network.loadingFinished` | 9 |
| `Network.policyUpdated` | 6 |
| `Network.requestWillBeSent` | 9 |
| `Network.requestWillBeSentExtraInfo` | 9 |
| `Network.responseReceived` | 8 |
| `Network.responseReceivedExtraInfo` | 9 |
| `Page.documentOpened` | 1 |
| `Page.domContentEventFired` | 3 |
| `Page.frameAttached` | 1 |
| `Page.frameDetached` | 1 |
| `Page.frameNavigated` | 3 |
| `Page.frameRequestedNavigation` | 1 |
| `Page.frameResized` | 4 |
| `Page.frameStartedLoading` | 3 |
| `Page.frameStartedNavigating` | 3 |
| `Page.frameStoppedLoading` | 5 |
| `Page.frameSubtreeWillBeDetached` | 1 |
| `Page.javascriptDialogClosed` | 3 |
| `Page.javascriptDialogOpening` | 3 |
| `Page.lifecycleEvent` | 42 |
| `Page.loadEventFired` | 3 |
| `Runtime.consoleAPICalled` | 14 |
| `Runtime.executionContextCreated` | 14 |
| `Runtime.executionContextDestroyed` | 2 |
| `Runtime.executionContextsCleared` | 4 |
| `Target.attachedToTarget` | 8 |
| `Target.detachedFromTarget` | 4 |
