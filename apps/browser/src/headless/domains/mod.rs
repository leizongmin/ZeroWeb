//! 协议命令域实现 — BiDi 会话/浏览上下文/脚本/截图域 + CDP 兼容子集（Phase 3）。
//!
//! S32 起按域拆分子模块（CLAUDE.md §5 文件大小约束）：本文件仅保留命令路由
//! （dispatch / dispatch_with_events 族），各域 handler 见同名子模块；转换助手见
//! `remote_object`。均为 S2→S25 落地代码的纯搬移，零语义变化。

mod bidi;
mod dom;
mod emulation;
mod input;
mod page;
mod remote_object;
mod runtime;
mod storage;
mod target;
// tests.rs 既有 `use super::domains::{object_id_string, parse_object_id}` 路径保持可用（S32）
#[cfg(test)]
pub(super) use dom::convert_cdp_node;
#[cfg(test)]
pub(super) use remote_object::{object_id_string, parse_object_id};

// tests.rs 既有 `use super::domains::{object_id_string, parse_object_id}` 路径保持可用

use serde_json::Value;

use crate::headless::HeadlessServer;

use crate::headless::protocol::{ProtocolError, ServerEvent};
use crate::headless::session::HeadlessSession;

impl HeadlessServer {
    /// 命令路由。
    pub(super) fn dispatch(
        &self,
        session: &mut HeadlessSession,
        method: &str,
        params: Value,
    ) -> Result<Value, ProtocolError> {
        match method {
            // ── 会话管理 ──
            "session.status" => self.cmd_session_status(),
            "session.new" => self.cmd_session_new(),
            "session.end" => Err(ProtocolError {
                code: -32000,
                message: "Session ended by client".into(),
            }),

            // ── 浏览器控制 ──
            "browser.close" => self.cmd_browser_close(),

            // ── 浏览上下文（Phase 2）──
            "browsingContext.create" => self.cmd_browsing_context_create(session, params),
            "browsingContext.getTree" => self.cmd_browsing_context_get_tree(session),
            "browsingContext.close" => self.cmd_browsing_context_close(session, params),
            "browsingContext.reload" => self.cmd_browsing_context_reload(session),

            // ── 导航 ──
            "browsingContext.navigate" => self.cmd_navigate(session, params),
            // DC-13 line 315：加载内联 HTML（绕过 fetch_url HTTP-only），供 headless 截图自包含 fixture。
            "browsingContext.loadHtml" => self.cmd_load_html(session, params),

            // ── 脚本执行 ──
            "script.evaluate" => self.cmd_script_evaluate(session, params),
            "script.callFunction" => self.cmd_script_call_function(session, params),

            // ── 截图 ──
            "browsingContext.captureScreenshot" => self.cmd_capture_screenshot(session),

            // ── 页面内容 ──
            "browsingContext.getDOMSnapshot" => self.cmd_get_dom_snapshot(session),

            // ── CDP 域（无事件命令，M1 切片 3）──
            "Browser.getVersion" => Ok(serde_json::json!({
                "protocolVersion": "1.3",
                "product": format!("ZeroWeb/{}", zero_product_version::VERSION),
                "revision": "",
                "userAgent": zero_net::HttpClient::default_user_agent(),
                "jsVersion": "12.0",
            })),
            // 连接握手即发（Playwright connectOverCDP）；下载能力未实现，stub 接受
            "Browser.setDownloadBehavior" => Ok(serde_json::json!({})),
            "Target.getTargetInfo" => self.cmd_target_get_target_info(session, params),
            "Runtime.callFunctionOn" => self.cmd_runtime_call_function_on(session, params),
            // objectId 桥配对命令（M4+）：释放 renderer 保留句柄
            "Runtime.releaseObject" => self.cmd_runtime_release_object(session, params),
            "Runtime.releaseObjectGroup" => self.cmd_runtime_release_object_group(session, params),
            // DOM 域 objectId 面（M4+）：geometry/身份经句柄桥求值
            "DOM.scrollIntoViewIfNeeded" => self.cmd_dom_scroll_into_view_if_needed(session, params),
            "DOM.getContentQuads" => self.cmd_dom_get_content_quads(session, params),
            "DOM.getBoxModel" => self.cmd_dom_get_box_model(session, params),
            "DOM.describeNode" => self.cmd_dom_describe_node(session, params),
            // adopt 流程另一半：backendNodeId → objectId（utility → main world 重析）
            "DOM.resolveNode" => self.cmd_dom_resolve_node(session, params),
            // DOM 域 DevTools frontend 面（devtools goal M1-S3a）：getDocument 全树
            // 序列化是 Elements 面板唯一数据源；enable/requestChildNodes 只需 ack
            //（getDocument 恒返全树，子树展开无增流）
            "DOM.enable" => Ok(serde_json::json!({})),
            "DOM.getDocument" => self.cmd_dom_get_document(session, &params),
            "DOM.requestChildNodes" => Ok(serde_json::json!({})),
            // Page.getResourceTree：frontend frame 树枚举（复用 getFrameTree 树形 +
            // resources 空表；S3a 面板资源清单无消费面）
            "Page.getResourceTree" => {
                let mut tree = self.cmd_page_get_frame_tree(session, None)?;
                if let Some(frame_tree) = tree.get_mut("frameTree") {
                    frame_tree["resources"] = serde_json::json!([]);
                }
                Ok(tree)
            }
            "Runtime.runIfWaitingForDebugger" => Ok(serde_json::json!({})),
            // Playwright page 初始化命令族：stub 接受解附接摩擦，实义语义随 M2
            "Log.enable" => Ok(serde_json::json!({})),
            "Page.setLifecycleEventsEnabled" => Ok(serde_json::json!({})),
            "Page.addScriptToEvaluateOnNewDocument" => {
                self.cmd_page_add_script_to_evaluate_on_new_document(session, params)
            }
            "Emulation.setFocusEmulationEnabled" => Ok(serde_json::json!({})),
            // M3 媒体仿真：prefers-color-scheme → SetColorScheme；media type → SetMediaType
            "Emulation.setEmulatedMedia" => self.cmd_emulation_set_emulated_media(session, params),
            // Page 布局面（M2）：viewport 来自 headless 启动参数（固定视口）
            "Page.getLayoutMetrics" => Ok(self.cmd_page_get_layout_metrics()),
            // 对话框：引擎暂无阻塞式 JS 对话语义 → 接受（无 javascriptDialogOpening 事件源）
            "Page.handleJavaScriptDialog" => Ok(serde_json::json!({})),
            // Input 域（M2）：CDP 输入 → renderer IPC（MouseEvent/KeyboardEvent/ScrollEvent/ImeEvent）
            "Input.dispatchMouseEvent" => self.cmd_input_dispatch_mouse_event(session, params),
            "Input.dispatchKeyEvent" => self.cmd_input_dispatch_key_event(session, params),
            "Input.insertText" => self.cmd_input_insert_text(session, params),
            // Storage 域（M4）：cookie jar 接线（发现 #2：PW cookie 走 Storage 域）
            "Storage.getCookies" => Ok(self.cmd_storage_get_cookies(session)),
            "Storage.setCookies" => self.cmd_storage_set_cookies(session, params),
            "Storage.clearCookies" => Ok(self.cmd_storage_clear_cookies(session)),
            // Emulation UA override（M4）：proxy_fetch 注入 User-Agent
            "Emulation.setUserAgentOverride" => self.cmd_emulation_set_user_agent_override(session, params),
            // Network 域（M4）：enable/disable 门控（事件在 proxy_fetch 生命周期产出）
            "Network.enable" => {
                session.network_enabled = true;
                Ok(serde_json::json!({}))
            }
            "Network.disable" => {
                session.network_enabled = false;
                Ok(serde_json::json!({}))
            }
            // DevTools frontend 初始化命令族（devtools goal M1-S3a，evidence/M1-attach-testbed.md
            // §3 账本 enable 型）：frontend 只需 ack 才继续初始化流；无事件源/无状态语义，
            // 实义随各面板消费面落地再补
            "CSS.enable"
            | "Overlay.enable"
            | "Profiler.enable"
            | "Debugger.enable"
            | "Debugger.setPauseOnExceptions"
            | "Debugger.setAsyncCallStackDepth"
            | "Debugger.setBlackboxPatterns"
            | "Debugger.setVariableValue"
            | "Log.startViolationsReport"
            | "Overlay.setShowViewportSizeOnResize"
            | "Overlay.setShowGridOverlays"
            | "Overlay.setShowFlexOverlays"
            | "Overlay.setShowScrollSnapOverlays"
            | "Overlay.setShowHingeOverlay"
            | "Overlay.setShowContainerOverlays"
            | "Overlay.setShowIsolatedElements"
            | "Emulation.setEmulatedVisionDeficiency"
            | "Emulation.setAutoDarkModeOverride"
            | "Accessibility.enable"
            | "Animation.enable"
            | "Autofill.enable"
            | "Autofill.setAddresses"
            | "Audits.enable"
            | "ServiceWorker.enable"
            | "Inspector.enable"
            | "Runtime.addBinding"
            | "Runtime.terminateExecution"
            | "Network.setAttachDebugStack"
            | "Network.setBlockedURLs"
            | "Network.emulateNetworkConditionsByRule"
            | "Network.overrideNetworkState"
            | "Network.setCacheDisabled"
            | "Network.setBypassServiceWorker"
            | "Network.setUserAgentOverride"
            | "Network.setExtraHTTPHeaders"
            | "Network.setRequestInterception"
            | "DOMDebugger.setBreakOnCSPViolation"
            | "DOMDebugger.setEventListenerBreakpoint"
            | "DOMDebugger.removeEventListenerBreakpoint"
            | "DOMDebugger.setDOMBreakpoint"
            | "DOMDebugger.setXHRBreakpoint"
            | "Page.setAdBlockingEnabled"
            | "Page.setBypassCSP"
            | "Page.setWebLifecycleState"
            | "CSS.trackComputedStyleUpdates"
            | "CSS.takeComputedStyleUpdates"
            | "Target.setDiscoverTargets"
            | "Target.setRemoteLocations"
            | "Target.addTargetToTarget"
            | "Emulation.setCPUThrottlingRate"
            | "Performance.enable" => Ok(serde_json::json!({})),
            // 带返回形状的 frontend 初始化命令（面板初始化路径依赖返回值继续）
            "Runtime.getIsolateId" => Ok(serde_json::json!({ "id": "zw-isolate" })),
            "Storage.getStorageKey" => Ok(serde_json::json!({ "storageKey": "zeroweb-active-tab" })),
            "Page.getNavigationHistory" => Ok(serde_json::json!({
                "index": 0,
                "entries": [{
                    "id": 1,
                    "url": session.shell.tabs().next().and_then(|t| t.url().map(str::to_string)).unwrap_or_else(|| "about:blank".into()),
                    "userVerifier": "",
                    "title": session.shell.tabs().next().and_then(|t| t.title().map(str::to_string)).unwrap_or_default(),
                }],
            })),
            // ── 未知命令 ──
            _ => Err(ProtocolError {
                code: -32601,
                message: format!("Unknown method: {method}"),
            }),
        }
    }

    /// 带事件生成的命令路由（测试辅助入口：浏览器级，无 CDP 会话语义）。
    pub(super) fn dispatch_with_events(
        &self,
        session: &mut HeadlessSession,
        method: &str,
        params: Value,
    ) -> (Result<Value, ProtocolError>, Vec<ServerEvent>) {
        self.dispatch_with_events_for(session, None, method, params)
    }

    /// 带事件生成的命令路由，感知请求的 CDP 会话层级。
    ///
    /// `cdp_session` = `None` 为浏览器级命令；`Some(sid)` 为附接目标上的命令
    ///（如 Target.setAutoAttach 会话级语义只挂子 target，ZeroWeb 无子 target → 无事件）。
    pub(super) fn dispatch_with_events_for(
        &self,
        session: &mut HeadlessSession,
        cdp_session: Option<&str>,
        method: &str,
        params: Value,
    ) -> (Result<Value, ProtocolError>, Vec<ServerEvent>) {
        let mut events = Vec::new();

        match method {
            "browsingContext.navigate" => {
                let result = self.cmd_navigate(session, params);
                if let Ok(ref val) = result {
                    let url_val = val.get("url").cloned();
                    events.push(ServerEvent {
                        method: "browsingContext.load".into(),
                        params: serde_json::json!({
                            "url": url_val,
                            "success": true,
                        }),
                        session_id: None,
                    });
                    events.push(ServerEvent {
                        method: "log.entryAdded".into(),
                        params: serde_json::json!({
                            "level": "info",
                            "text": format!("Page loaded: {:?}", url_val),
                            "timestamp": std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis(),
                        }),
                        session_id: None,
                    });
                }
                (result, events)
            }
            "browsingContext.reload" => {
                let result = self.cmd_browsing_context_reload(session);
                if result.is_ok() {
                    events.push(ServerEvent {
                        method: "browsingContext.load".into(),
                        params: serde_json::json!({ "success": true }),
                        session_id: None,
                    });
                }
                (result, events)
            }
            "browsingContext.create" => {
                let result = self.cmd_browsing_context_create(session, params.clone());
                if let Ok(ref val) = result {
                    events.push(ServerEvent {
                        method: "browsingContext.contextCreated".into(),
                        params: serde_json::json!({
                            "context": val.get("context"),
                        }),
                        session_id: None,
                    });
                }
                (result, events)
            }
            "browsingContext.close" => {
                let ctx = params.get("context").cloned();
                let result = self.cmd_browsing_context_close(session, params);
                if result.is_ok() {
                    events.push(ServerEvent {
                        method: "browsingContext.contextDestroyed".into(),
                        params: serde_json::json!({ "context": ctx }),
                        session_id: None,
                    });
                }
                (result, events)
            }
            // CDP Page.navigate（M2）：{frameId,loaderId} 形状 + 导航事件族
            //（frameStarted/StoppedLoading、frameNavigated、lifecycle、load/domContent、
            // executionContextsCleared + 新文档上下文、注入脚本重放）
            "Page.navigate" => {
                let result = self.cmd_page_navigate(session, cdp_session, params, &mut events);
                (result, events)
            }
            // CDP 形状：{data: "<base64>"}（BiDi browsingContext.captureScreenshot 走对象形）
            "Page.captureScreenshot" => (self.cmd_page_capture_screenshot(session, params), events),
            "Page.enable" => (Ok(serde_json::json!({})), events),
            // CDP Runtime 域：remoteObject 类型化形状（BiDi script.evaluate 走旧实现）
            "Runtime.evaluate" => (self.cmd_runtime_evaluate(session, params), events),
            // Runtime.enable：响应后补发当前执行上下文（Playwright 依赖 contextId
            // 才能在该上下文里 evaluate）。auxData.frameId/isDefault 为硬要求——
            // PW 的 _onExecutionContextCreated 无 auxData 时直接丢弃上下文，页面
            // 初始化将永不完成（2026-09-12 首连实测）。
            "Runtime.enable" => {
                self.push_main_world_context_event(session, cdp_session, &mut events);
                (Ok(serde_json::json!({})), events)
            }
            // Target 域（M1 切片 3，Playwright connectOverCDP 连接脊柱）
            "Target.setAutoAttach" => {
                // 会话级 setAutoAttach 只自动附接该 page 的子 target（worker/iframe）；
                // ZeroWeb 无子 target → 接受无事件。浏览器级才附接全部 page target。
                if cdp_session.is_some() {
                    (Ok(serde_json::json!({})), events)
                } else {
                    let result = self.cmd_target_set_auto_attach(session, &params, &mut events);
                    (result, events)
                }
            }
            "Target.createTarget" => {
                let result = self.cmd_target_create_target(session, &params, &mut events);
                (result, events)
            }
            "Target.closeTarget" => {
                let result = self.cmd_target_close_target(session, &params, &mut events);
                (result, events)
            }
            "Target.detachFromTarget" => {
                let result = self.cmd_target_detach_from_target(&params, cdp_session, &mut events);
                (result, events)
            }
            "Target.getTargets" => (self.cmd_target_get_targets(session), events),
            // Target.attachToBrowserTarget：PW `newBrowserCDPSession`/`newCDPSession`
            // 的建会话入口（2026-09-13 探针实测）。flat 模型：分配 sessionId 登记到
            // 活跃 target，后续命令照常按方法路由（单页面模型，浏览器级/页面级同面）
            "Target.attachToBrowserTarget" => (self.cmd_target_attach_to_browser_target(session), events),
            // Target.attachToTarget：PW newCDPSession(page) 建会话后绑定页面 target
            //（同经探针实测）；响应携带 sessionId，无事件（PW 按响应配对）
            "Target.attachToTarget" => {
                let result = self.cmd_target_attach_to_target(session, params);
                (result, events)
            }
            // PW evaluate 管线需要隔离 world（utility script 宿主）：返回新
            // executionContextId 并补发 executionContextCreated（worldName 对齐）
            "Page.createIsolatedWorld" => {
                let result = self.cmd_page_create_isolated_world(session, cdp_session, &params, &mut events);
                (result, events)
            }
            // M3 viewport 桥：renderer SetViewport + 服务器视口状态 + frameResized 事件
            "Emulation.setDeviceMetricsOverride" => {
                let (result, resize_events) =
                    self.cmd_emulation_set_device_metrics_override(session, cdp_session, params);
                events.extend(resize_events);
                (result, events)
            }
            // PW CRPage 初始化需要 frame 树确定主 frame id（后续 frameNavigated 同 id 对齐）
            "Page.getFrameTree" => (self.cmd_page_get_frame_tree(session, cdp_session), events),
            // 默认：无事件
            _ => (self.dispatch(session, method, params), events),
        }
    }
}
