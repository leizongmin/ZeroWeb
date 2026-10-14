# 嵌入式 WebView API 与宿主集成

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位 `zero_webview::{WebView, WebViewConfig, WebViewBuilder}` 的嵌入契约：加载、回调、增量绘制、resize 和脚本/资源宿主桥。它输出渲染图元，不创建系统窗口或替宿主 present。`apps/webview-demo` 当前演示 HostRuntime 与 render-foundation 的文本呈现，不是调用 WebView 加载页面的消费者。

主要 API：`load_html`/`load_url`/`complete_load`/`fetch_url`/`complete_fetched_page`、`resize`、`render_incremental`、`on_event`/`remove_event_callback`。网络与资源策略见[导航加载](../web-data/network/fetch-and-navigation.md)，脚本 feature 见[运行时选择](../scripting/runtimes/engines-and-sandbox.md)，CPU/GPU 输出见[渲染底座](../graphics/render-foundation/index.md)。

验证导航：[嵌入与自动化地图](../../../zeroweb-verify/features/embedding-automation.md)，现有 `basic.rs`、`builder_nav.rs`、`user_actions.rs` 覆盖不同 API 行为；本轮没有执行测试，地图为工作区新增，不归上述 HEAD。

## Gotchas
- WebViewBuilder::url(...).build() 只进入 load_url 的加载状态并发 LoadStart/UrlChanged，不发起网络请求；宿主必须另行 fetch 或 complete_load 才得到页面图元。 (`crates/webview/src/webview_builder.rs`, `crates/webview/src/webview.rs`, `crates/webview/src/tests/builder_nav.rs`)
- load_html 不自动设置 URL 或 is_loading；相同 HTML 经 load_html 与 load_url→complete_load 的事件/地址状态不同，不能互换这两条入口。 (`crates/webview/src/webview.rs`, `crates/webview/src/tests/basic.rs`, `crates/webview/src/tests/builder_nav.rs`)
- fetch_url 是同步 HTTP 获取并处理 final_url，complete_fetched_page 再完成外部 CSS/图片等页面资源；单纯 complete_load 外部给定 HTML 不等于完整网络加载链。 (`crates/webview/src/webview.rs`)
- render 结果包含 primitives、dirty_rects 和 timings，不是窗口像素或 GPU 已呈现凭据；宿主仍需字体/GPU 资源和 surface/present。 (`crates/webview/src/webview.rs`, `apps/webview-demo/src/main.rs`)
- 事件回调存 Option 槽，删除不移动其余回调 ID，后续注册复用空槽；回调内部是 Rc/RefCell，不能因 external_script 要求 Send+Sync 就声称 WebView 可跨线程共享。 (`crates/webview/src/webview.rs`)
- resize 相同尺寸直接 no-op，否则新建 Pipeline 并回填 document_url、焦点、图片尺寸/比例与字体解析/度量；只改 config 宽高不会等价重建页面布局。 (`crates/webview/src/webview.rs`)
- render_incremental 在没有 cached HTML 时返回 None，而非生成空白帧；宿主需区分“尚未有页面”与“页面渲染结果为空”。 (`crates/webview/src/webview.rs`)
- 各宿主桥的 None 含义不同：script_source_fetcher 跳过外链脚本，image_source_fetcher 回退 HTTP，fetch_handler 不安装 __zw_fetch，ws_host_factory 缺失降级 1006；不能用统一“None=自动联网”策略配置它们。 (`crates/webview/src/webview.rs`)
- script_timeout_ms 默认 0，不截断脚本执行；嵌入方需要有界执行时须显式选择超时，不能从浏览器自动化等待期限推断页面脚本看门狗已开启。 (`crates/webview/src/webview.rs`)
- csp_enforcement 默认 true，mixed_content_enforcement 默认受 ZW_MIXED_CONTENT_ENFORCEMENT 是否等于 0 控制；它们是独立策略开关，启用脚本不意味着资源策略关闭。 (`crates/webview/src/webview.rs`)
- load_url 换代和清焦点/悬停/连击态，但不清旧 last_render；异步加载需要 prepare_document_state 清 HTML/CSS/图片及旧绘制缓存，否则宿主可能发布上一文档帧。 (`crates/webview/src/webview.rs`, `crates/webview/src/tests/builder_nav.rs`)
- 直接调用同 URL 的 load_url 仍发 LoadStart，但 UrlChanged 只在字符串改变时发；回调数量不能充当导航次数，complete_load 的 LoadEnd 使用 current_url 而不从 HTML title 推断地址。 (`crates/webview/src/webview.rs`, `crates/webview/src/tests/builder_nav.rs`)

## Architecture
- WebView 将 DOM/CSS/布局/绘制串到 Pipeline，宿主事件通过 WebView 的用户动作入口影响页面；BrowserApp 与外部嵌入方不应为一次交互直接绕开稳定 API 改底层 Document。 (`crates/webview/src/lib.rs`, `crates/webview/src/webview.rs`, `crates/webview/src/tests/user_actions.rs`)

## Decisions
- native_dom 配置字段与 builder 开关已在双引擎默认开启后删除，原生 DOM 绑定是当前生产路径；旧示例里的 native_dom(false) 与 ZW_NATIVE_DOM 不能作为现行回退方案。 (`crates/webview/src/webview.rs`, `crates/webview/src/webview_builder.rs`, `git:222b2d6c9a58c6d46e00dfe46359f0c4709990a9`)

## Testing Strategy
- 用户动作测试同时断言 beforeinput 可取消、live value 不变、页面脚本只初始化一次和跨文档 PageNodeRef 返回 StaleTarget no-op；这比仅检查 changed 或图元非空更能守住嵌入调用契约。 (`crates/webview/src/tests/user_actions.rs`)
- builder_nav 的完整 load_url→complete_load 用例不联网，断言 LoadStart→UrlChanged→LoadEnd 三事件与 last_render；其“e2e”名称不是网络或系统窗口验收证据。 (`crates/webview/src/tests/builder_nav.rs`, `crates/webview/src/webview_builder.rs`)
