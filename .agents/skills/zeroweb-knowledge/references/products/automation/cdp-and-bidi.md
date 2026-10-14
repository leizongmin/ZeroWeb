# CDP、BiDi、会话与远程对象

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位 headless HTTP 发现、WebSocket 多连接、CDP/BiDi 路由、Target/Page/Runtime/DOM/CSS/Input/Storage/Emulation 与远程对象生命周期。`HeadlessServer` 路由命令，`HeadlessSession` 负责页面与 renderer 通信，domain 文件定义各命令的真实兼容面。

关键入口：`dispatch_mode`、`dispatch_with_events`、`automation_request`、`object_id_string`、`HeadlessSecurityConfig`。生产消息见[IPC](../../runtime/protocol/messages-and-lifecycle.md)，句柄代际见[脚本身份](../../scripting/dom-binding/identity/handles-and-generations.md)；DevTools 资源服务与 UI 驱动见[相邻主题](ui-driver-and-devtools.md)。

验证导航：[嵌入与自动化地图](../../../../zeroweb-verify/features/embedding-automation.md)，真实进程钉为 `cdp_input_events.rs`、`cdp_navigate_evaluate.rs`，不能用 headless 单测的进程内 fixture 替代；本轮未执行，地图为工作区新增，不归上述 HEAD。

## Gotchas
- 服务器在连接循环外创建一个 HeadlessSession，HTTP 发现和多个 WebSocket 共享 Shell/renderer，命令串行推进，Network 事件按连接订阅广播；多个 target 不等于多个独立 renderer 会话。 (`apps/browser/src/headless/mod.rs`, `apps/browser/src/headless/session.rs`)
- HeadlessSession 非 test 路径使用 renderer IPC，cfg(test) 路径才有进程内 WebView fixture；单测里的脚本/输入成功不能证明发布进程链成功。 (`apps/browser/src/headless/session.rs`)
- Runtime.evaluate 是表达式 EvaluateRetaining，ExecuteScript 是函数体；把带 return 的函数体直接当 evaluate 表达式会改变返回/报错语义。 (`apps/browser/src/headless/domains/runtime.rs`, `apps/browser/src/headless/session.rs`)
- RemoteObject 的 zw:<handle> 保留 renderer 注册表身份，DOM 对象 subtype 为 node；嵌套值走 JSON，嵌套 Handle 降 null，调用方不能期待任意深度的对象图句柄。 (`apps/browser/src/headless/domains/remote_object.rs`)
- Runtime.evaluate 忽略 contextId，Page.createIsolatedWorld 只产生 ID、worldName 与事件，没有真正 JS 隔离；不同 world ID 不能作为脚本安全隔离依据。 (`apps/browser/src/headless/domains/runtime.rs`, `apps/browser/src/headless/domains/page.rs`)
- Target flat session registry 映射真实 TabId；浏览器层 autoAttach 枚举页面，page 直连不生成子 target 附接事件，waitForDebuggerOnStart 始终 false。 (`apps/browser/src/headless/domains/target.rs`)
- BiDi getTree 返回按计数生成的 context 编号和 about:blank，不是 create 返回的真实 TabId；reload 发 RequestFrame 而非重新 fetch，不能宣称完整浏览上下文语义。 (`apps/browser/src/headless/domains/bidi.rs`)
- Page.navigate 加载失败放在成功响应的 frameId/loaderId/errorText 中，不一定是协议 error；只检查响应没有 error 对象会漏掉导航失败。 (`apps/browser/src/headless/domains/page.rs`)
- Page.getLayoutMetrics 的 contentSize 当前是 viewport 尺寸，childFrames 只有 about:blank 元数据；它们不能证明文档全高或子页面真实 URL。 (`apps/browser/src/headless/domains/page.rs`)
- 鼠标 released 先发 Up，clickCount 为 1 再发 Click、为 2 再发 DblClick；Input.insertText 走 IME，不能用协议返回值推断点击或文本默认动作已落页面。 (`apps/browser/src/headless/domains/input.rs`)
- CDP key 与 text 分开保真，text 只随 Down 传，rawKeyDown 不从可打印 key 推断字符；历史修复针对 keypress 缺失与 rawKeyDown 幻插入，负控制钉验证无 text 时不插入。 (`apps/browser/src/headless/domains/input.rs`, `apps/browser/tests/cdp_input_events.rs`, `git:56de3f923d0052db474a3fabc06c303115a5a023`)
- Storage 与 Network 操作同一个 cookie jar，但 setCookies 的 url 分支提前返回且忽略部分描述字段，getCookies 的 URL 过滤仅做 host 后缀判断，不是完整 cookie 适用性筛选。 (`apps/browser/src/headless/domains/storage.rs`)
- CSS computed 只探测 15 个属性并过滤空值，matched styles 只返回 inline style，没有完整匹配规则；DevTools 侧栏出现数据不表示 CSS 规则内省已实现。 (`apps/browser/src/headless/domains/css.rs`)
- Debugger/Profiler enable、Network.setCacheDisabled、Browser.setDownloadBehavior 等命令只 ack；返回 {} 是握手兼容，不证明暂停、性能采样、禁缓存或下载策略生效。 (`apps/browser/src/headless/domains/mod.rs`)
- 默认安全配置无 token 且空 allowed_origins 允许任意来源；启用 token 时要求首个请求字段匹配，Origin 列表按完整字符串匹配，不能把默认端点宣传成已认证服务。 (`apps/browser/src/headless/security.rs`, `apps/browser/src/headless/mod.rs`)
- HTTP Upgrade 名称和值匹配大小写无关，但 Origin 提取只识别实现中的两种拼写；非标准大小写请求不能仅依据 verify_origin 的单测判断整条握手兼容。 (`apps/browser/src/headless/discovery.rs`, `apps/browser/src/headless/mod.rs`)

## Architecture
- Runtime.releaseObject/Group 与远程句柄保留配对，句柄属于 renderer 文档/运行时代际；导航后的旧 ID 不应被调用方当作普通可重用字符串缓存。 (`apps/browser/src/headless/domains/runtime.rs`, `apps/browser/src/headless/domains/remote_object.rs`, `apps/browser/src/headless/session.rs`)
- Emulation.setEmulatedMedia 把 color scheme 与 media type 分别映射为 SetColorScheme/SetMediaType，UA override 则进入 HTTP 代理；媒体变化不等价于所有 navigator/请求字段都同步仿真。 (`apps/browser/src/headless/domains/emulation.rs`, `apps/browser/src/headless/session.rs`)

## Decisions
- 默认预注入通过 PreDocumentScripts 全量登记，在新文档页面脚本前执行；旧的导航后重放太晚。此保证不是 HTML 解析前注入，ZW_CDP_PREDOC_SCRIPTS=0 仍回退旧路径。 (`apps/browser/src/headless/domains/page.rs`, `apps/browser/src/headless/session.rs`, `git:d3ee678412f251a8c8eca96f33cd4893f9e24916`)
- Page.navigate 的 iframe 探测默认有界等待 250ms，超界放弃本轮元数据，避免单 JS worker 忙时卡住所有连接；迟发事件可能对应旧文档，所以不补发。ZW_CDP_NAV_IFRAME_PROBE_ASYNC=0 恢复阻塞旧行为。 (`apps/browser/src/headless/domains/page.rs`, `apps/browser/src/headless/session.rs`, `apps/browser/tests/cdp_navigate_evaluate.rs`, `git:737edf1cb51a9292b3f12eaaa43380d1559fb984`)

## Testing Strategy
- 输入进程测试分三轴：带 text 的键断言 keydown<keypress<beforeinput<input<keyup 和 trusted/target，insertText 断言 value 与输入事件且无 keydown，rawKeyDown 无 text 断言无插入与编辑事件；三者不能用一个文本插入成功用例代替。 (`apps/browser/tests/cdp_input_events.rs`)
- 忙页导航用 3000ms 同步脚本臂断言 Page.navigate 返回少于1500ms，另有跨连接和开关回退用例；cfg(test) 的 iframe fixture 同步求值不执行生产有界等待，不能用它证明 mux 无阻塞。 (`apps/browser/tests/cdp_navigate_evaluate.rs`, `apps/browser/src/headless/session.rs`)
- preDocument 单测用 LAST_PREDOC_SEND 记录器断言登记时发送全量列表，专门防“脚本登记存在但 IPC 没接”的假绿；记录器不证明真实 renderer 已执行，需结合 renderer 时序与进程测试。 (`apps/browser/src/headless/tests.rs`, `apps/browser/src/headless/session.rs`)
