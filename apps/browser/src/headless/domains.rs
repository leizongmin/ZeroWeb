//! 协议命令域实现 — BiDi 会话/浏览上下文/脚本/截图域 + CDP 兼容子集（Phase 3）。

use std::sync::atomic::Ordering;

use serde_json::Value;
use zero_browser_shell::TabId;
use zero_protocol::message::AutomationValue;
#[cfg(not(test))]
use zero_protocol::message::{IpcMessage, IpcMessageKind};
use zero_protocol::message::{KeyboardEventType, MouseEventType};
use zero_render_foundation::cpu::render_full_scene;
use zero_render_foundation::font::cache::GlyphCache;
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::surface::FrameBuffer;

use super::HeadlessServer;

use super::protocol::{ProtocolError, ServerEvent};
use super::session::{HeadlessSession, InjectedScript};

/// R1601：把 RGBA8 FrameBuffer 编码为 base64 PNG 字符串，供 `captureScreenshot`
/// 协议响应携带像素数据（headless 截图用于像素对比，DC-13 line 315）。
fn framebuffer_to_png_base64(fb: &FrameBuffer) -> String {
    use base64::Engine;
    use png::{BitDepth, ColorType, Encoder};
    let mut png_buf: Vec<u8> = Vec::new();
    {
        let mut encoder = Encoder::new(&mut png_buf, fb.width, fb.height);
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);
        let mut writer = encoder.write_header().expect("PNG header encode");
        writer.write_image_data(&fb.data).expect("PNG image data encode");
    }
    base64::engine::general_purpose::STANDARD.encode(&png_buf)
}

/// 解码 PNG 字节为 (width, height, RGBA8 像素)（clip 裁剪前置步骤；Rgb 补 alpha）。
fn decode_png_to_rgba(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), String> {
    use png::ColorType;
    let decoder = png::Decoder::new(bytes);
    let mut reader = decoder.read_info().map_err(|e| format!("png read_info: {e}"))?;
    let info = reader.info().clone();
    let mut buf = vec![0u8; reader.output_buffer_size()];
    reader.next_frame(&mut buf).map_err(|e| format!("png decode: {e}"))?;
    let rgba = if info.color_type == ColorType::Rgb {
        buf.chunks(3).flat_map(|px| [px[0], px[1], px[2], 255u8]).collect()
    } else {
        buf
    };
    Ok((info.width, info.height, rgba))
}

/// 类型化值 → CDP remoteObject（returnByValue 语义，可序列化值直接内联）。
/// 不可序列化对象的 objectId 句柄（V8 桥）尚未实现——见命令矩阵 G3。
fn automation_value_to_remote_object(value: &AutomationValue) -> Value {
    match value {
        AutomationValue::Null => serde_json::json!({ "type": "object", "subtype": "null", "value": null }),
        AutomationValue::Bool(v) => serde_json::json!({ "type": "boolean", "value": v }),
        AutomationValue::Number(v) => serde_json::json!({ "type": "number", "value": v }),
        AutomationValue::String(v) => serde_json::json!({ "type": "string", "value": v }),
        AutomationValue::Array(items) => serde_json::json!({
            "type": "object",
            "subtype": "array",
            "value": items.iter().map(automation_value_to_remote_object_value).collect::<Vec<_>>(),
        }),
        AutomationValue::Object(entries) => serde_json::json!({
            "type": "object",
            "value": entries
                .iter()
                .map(|(k, v)| (k.clone(), automation_value_to_remote_object_value(v)))
                .collect::<serde_json::Map<String, Value>>(),
        }),
    }
}

/// 嵌套值走纯 JSON（remoteObject 嵌套层不重复包 type/value 外壳，Chromium returnByValue 同）。
fn automation_value_to_remote_object_value(value: &AutomationValue) -> Value {
    match value {
        AutomationValue::Null => Value::Null,
        AutomationValue::Bool(v) => serde_json::json!(v),
        AutomationValue::Number(v) => serde_json::json!(v),
        AutomationValue::String(v) => serde_json::json!(v),
        other => automation_value_to_remote_object(other),
    }
}

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
                let result = self.cmd_target_detach_from_target(&params, &mut events);
                (result, events)
            }
            "Target.getTargets" => (self.cmd_target_get_targets(session), events),
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

    // ── 命令实现 ──

    fn cmd_session_status(&self) -> Result<Value, ProtocolError> {
        Ok(serde_json::json!({
            "ready": true,
            "message": "ZeroWeb headless server ready"
        }))
    }

    fn cmd_session_new(&self) -> Result<Value, ProtocolError> {
        let session_id = self.next_session_id.fetch_add(1, Ordering::SeqCst);
        Ok(serde_json::json!({
            "sessionId": session_id,
            "capabilities": {
                "browserName": "ZeroWeb",
                "browserVersion": zero_product_version::VERSION,
                "platformName": std::env::consts::OS,
            }
        }))
    }

    fn cmd_browser_close(&self) -> Result<Value, ProtocolError> {
        Ok(serde_json::json!({ "result": "closing" }))
    }

    fn cmd_navigate(&self, session: &mut HeadlessSession, params: Value) -> Result<Value, ProtocolError> {
        let url = params
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'url' parameter".into(),
            })?;

        session.shell.navigate(url);

        // 发布构建由 renderer 执行页面管线；单元测试保留内存 WebView fixture。
        #[cfg(test)]
        let render_result = session.webview.fetch_url(url).map(|_| ());
        #[cfg(not(test))]
        let render_result = session.navigate_renderer(url);
        let title = match &render_result {
            Ok(_) => url.to_string(),
            Err(_) => "Error loading page".to_string(),
        };

        session.shell.on_page_loaded(&title);

        Ok(serde_json::json!({
            "url": url,
            "title": title,
            "success": render_result.is_ok(),
        }))
    }

    fn cmd_script_evaluate(&self, session: &mut HeadlessSession, params: Value) -> Result<Value, ProtocolError> {
        let expression = params
            .get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'expression' parameter".into(),
            })?;

        #[cfg(test)]
        let result = session.webview.execute_script(expression);
        #[cfg(not(test))]
        let result = session.execute_script_renderer(expression);
        match result {
            Ok(result) => Ok(serde_json::json!({
                "result": {
                    "type": "string",
                    "value": result
                }
            })),
            Err(e) => Ok(serde_json::json!({
                "exceptionDetails": {
                    "text": e.to_string()
                }
            })),
        }
    }

    /// 渲染当前页面帧（GPU 开关路径 → CPU 兜底），供 BiDi 与 CDP 截图共用。
    fn render_page_framebuffer(&self, session: &mut HeadlessSession) -> Result<FrameBuffer, ProtocolError> {
        #[cfg(test)]
        let result = session.webview.render();
        #[cfg(not(test))]
        let result = session.snapshot.last_render.clone().ok_or_else(|| ProtocolError {
            code: -32000,
            message: "No renderer frame available".into(),
        })?;

        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(1024);
        // R1600：用 render_full_scene 渲染**全部 13 种图元**（旧 render_scene_to_framebuffer 仅
        // 渲染 fills+glyphs，静默丢弃 gradients/shadows/images/strokes/paths/transforms/clips/
        // filters/blend_modes 11 种）。headless 截图须反映真实 ZeroBrowser 渲染管线才能用于
        // DC-13 line 315（welcome headless 截图 vs chromium oracle）等像素对比；image_cache
        // 传入以渲染 `<img>` 子资源。
        // R3282（#4）：`ZW_HEADLESS_GPU_SCREENSHOT=1` 时用 GPU 无头渲染（性能开关；
        // 默认 CPU——DC-13 oracle 对比基线稳定）。GPU 支持子集与 CPU 逐像素一致
        //（parity/reftest 验证），未实现特性返回 false 自动回退 CPU。
        let fb = if std::env::var("ZW_HEADLESS_GPU_SCREENSHOT").as_deref() == Ok("1") {
            let (vw, vh) = self.viewport_size();
            let w = vw as u32;
            let h = vh as u32;
            // R3254-G5：设备丢失（真实）后丢弃 renderer——保留带标志的实例会
            // 永不重建、截图永久回退 CPU。
            if session.gpu_renderer.as_ref().is_some_and(|g| g.is_device_lost()) {
                session.gpu_renderer = None;
            }
            if session.gpu_renderer.is_none() {
                session.gpu_renderer = zero_render_foundation::gpu::renderer::GpuRenderer::new_headless(w, h).ok();
            }
            let gpu_ok = session.gpu_renderer.as_mut().is_some_and(|g| {
                !g.is_device_lost()
                    && g.render_full_scene_gpu(
                        &result.primitives,
                        &font_loader,
                        &mut glyph_cache,
                        #[cfg(test)]
                        Some(session.webview.image_cache()),
                        #[cfg(not(test))]
                        Some(&mut session.snapshot.image_cache),
                        &[],
                        &[],
                        &[],
                        &[],
                        1.0,
                    )
            });
            if gpu_ok {
                let pixels = session
                    .gpu_renderer
                    .as_ref()
                    .unwrap()
                    .read_pixels()
                    .expect("GPU read_pixels");
                let mut fb = zero_render_foundation::surface::FrameBuffer::new(w, h);
                fb.data.copy_from_slice(&pixels);
                fb
            } else {
                render_full_scene(
                    w,
                    h,
                    1.0,
                    &result.primitives,
                    &font_loader,
                    &mut glyph_cache,
                    #[cfg(test)]
                    Some(session.webview.image_cache()),
                    #[cfg(not(test))]
                    Some(&mut session.snapshot.image_cache),
                    &[],
                    &[],
                    &[],
                    &[],
                )
            }
        } else {
            let (vw, vh) = self.viewport_size();
            render_full_scene(
                vw as u32,
                vh as u32,
                1.0,
                &result.primitives,
                &font_loader,
                &mut glyph_cache,
                #[cfg(test)]
                Some(session.webview.image_cache()),
                #[cfg(not(test))]
                Some(&mut session.snapshot.image_cache),
                &[],
                &[],
                &[],
                &[],
            )
        };
        Ok(fb)
    }

    fn cmd_capture_screenshot(&self, session: &mut HeadlessSession) -> Result<Value, ProtocolError> {
        let fb = self.render_page_framebuffer(session)?;
        // R1601：返回 base64 PNG 像素数据（旧版仅返回尺寸，headless 截图无法用于像素对比）。
        // 保留 width/height/pixelCount 供 HeadlessClient::parse_screenshot 向后兼容。
        let png_b64 = framebuffer_to_png_base64(&fb);
        Ok(serde_json::json!({
            "data": {
                "width": fb.width,
                "height": fb.height,
                "format": "png-base64",
                "pixelCount": fb.width as usize * fb.height as usize,
                "png": png_b64,
            }
        }))
    }

    /// CDP Page.captureScreenshot — `{"data": "<base64 png>"}` 形状（区别于 BiDi 对象形）；
    /// 支持 clip 裁剪（原始 fb 行级裁剪）。format 仅支持 png（jpeg 编码器未接入）。
    fn cmd_page_capture_screenshot(
        &self,
        session: &mut HeadlessSession,
        params: Value,
    ) -> Result<Value, ProtocolError> {
        if let Some(format) = params.get("format").and_then(|v| v.as_str()) {
            if format != "png" {
                return Err(ProtocolError {
                    code: -32601,
                    message: format!("captureScreenshot format '{format}' not implemented (png only)"),
                });
            }
        }
        let fb = self.render_page_framebuffer(session)?;
        let fb = match params.get("clip") {
            Some(clip) => {
                let fx = clip.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0).max(0.0) as u32;
                let fy = clip.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0).max(0.0) as u32;
                let fw = clip.get("width").and_then(|v| v.as_f64()).unwrap_or(0.0).max(0.0) as u32;
                let fh = clip.get("height").and_then(|v| v.as_f64()).unwrap_or(0.0).max(0.0) as u32;
                if fw == 0 || fh == 0 || fx + fw > fb.width || fy + fh > fb.height {
                    return Err(ProtocolError {
                        code: -32602,
                        message: format!(
                            "clip out of bounds: frame {}x{}, clip {}x{} at ({fx},{fy})",
                            fb.width, fb.height, fw, fh
                        ),
                    });
                }
                let mut cropped = Vec::with_capacity((fw * fh * 4) as usize);
                for row in fy..fy + fh {
                    let start = ((row * fb.width) + fx) as usize * 4;
                    cropped.extend_from_slice(&fb.data[start..start + (fw as usize) * 4]);
                }
                FrameBuffer {
                    data: cropped,
                    width: fw,
                    height: fh,
                }
            }
            None => fb,
        };
        Ok(serde_json::json!({ "data": framebuffer_to_png_base64(&fb) }))
    }

    /// Emulation.setDeviceMetricsOverride — viewport 桥：renderer SetViewport +
    /// 服务器视口状态（getLayoutMetrics/captureScreenshot 联动）+ Page.frameResized 事件。
    fn cmd_emulation_set_device_metrics_override(
        &self,
        session: &mut HeadlessSession,
        cdp_session: Option<&str>,
        params: Value,
    ) -> (Result<Value, ProtocolError>, Vec<ServerEvent>) {
        let width = params.get("width").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let height = params.get("height").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let dsf = params.get("deviceScaleFactor").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
        if width <= 0.0 || height <= 0.0 {
            // Chromium：width/height 0 = 清除覆盖，恢复默认视口
            let (default_w, default_h) = (800.0_f32, 600.0_f32);
            self.set_viewport_size(default_w, default_h);
            let _ = session.send_set_viewport(default_w, default_h, dsf);
        } else {
            let (old_w, old_h) = self.viewport_size();
            self.set_viewport_size(width, height);
            let _ = session.send_set_viewport(width, height, dsf);
            if (old_w - width).abs() > f32::EPSILON || (old_h - height).abs() > f32::EPSILON {
                // Chromium frameResized 无参数体；事件归属当前会话
                return (
                    Ok(serde_json::json!({})),
                    vec![ServerEvent {
                        method: "Page.frameResized".into(),
                        params: serde_json::json!({}),
                        session_id: Some(cdp_session.unwrap_or_default().to_string()),
                    }],
                );
            }
        }
        (Ok(serde_json::json!({})), Vec::new())
    }

    /// Emulation.setEmulatedMedia — prefers-color-scheme → SetColorScheme；
    /// media type → SetMediaType；其余 feature（reduced-motion 等）引擎无 IPC 面暂忽略。
    fn cmd_emulation_set_emulated_media(
        &self,
        session: &mut HeadlessSession,
        params: Value,
    ) -> Result<Value, ProtocolError> {
        if let Some(media) = params.get("media").and_then(|v| v.as_str()) {
            session
                .send_set_media_type(media == "print")
                .map_err(|error| ProtocolError {
                    code: -32000,
                    message: error,
                })?;
        }
        if let Some(features) = params.get("features").and_then(|v| v.as_array()) {
            for feature in features {
                let name = feature.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let value = feature.get("value").and_then(|v| v.as_str()).unwrap_or("");
                if name == "prefers-color-scheme" {
                    session
                        .send_set_color_scheme(value == "dark")
                        .map_err(|error| ProtocolError {
                            code: -32000,
                            message: error,
                        })?;
                }
            }
        }
        Ok(serde_json::json!({}))
    }

    /// DC-13 line 315：加载内联 HTML（绕过 fetch_url 的 HTTP-only 限制），供 headless 路径
    /// 加载自包含 fixture（如 welcome.html，内联 CSS + data-URI 图标）做截图对比。参数 `html`
    ///（必填）+ `css`（可选）。区别于 `browsingContext.navigate`（需 URL + HTTP/file 抓取）。
    fn cmd_load_html(&self, session: &mut HeadlessSession, params: Value) -> Result<Value, ProtocolError> {
        let html = params
            .get("html")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'html' parameter".into(),
            })?;
        let css = params.get("css").and_then(|v| v.as_str());
        session.shell.navigate("about:blank");
        #[cfg(test)]
        let render_result = {
            let _ = session.webview.load_html(html, css);
            Ok::<(), String>(())
        };
        #[cfg(not(test))]
        let render_result = session.load_html_renderer(html, css);
        render_result.map_err(|message| ProtocolError { code: -32000, message })?;
        session.shell.on_page_loaded("headless:loadHtml");
        Ok(serde_json::json!({ "success": true }))
    }

    fn cmd_get_dom_snapshot(&self, session: &mut HeadlessSession) -> Result<Value, ProtocolError> {
        #[cfg(test)]
        let result = session.webview.render();
        #[cfg(not(test))]
        let result = session.snapshot.last_render.clone().ok_or_else(|| ProtocolError {
            code: -32000,
            message: "No renderer frame available".into(),
        })?;

        let fill_count = result.primitives.fills.len();
        let glyph_count = result.primitives.glyphs.len();

        Ok(serde_json::json!({
            "renderPrimitives": {
                "fills": fill_count,
                "glyphs": glyph_count,
                "gradients": result.primitives.gradients.len(),
                "shadows": result.primitives.shadows.len(),
                "images": result.primitives.images.len(),
            }
        }))
    }

    // ── Phase 2 命令实现 ──

    /// browsingContext.create — 创建新的浏览上下文（新标签页）。
    fn cmd_browsing_context_create(
        &self,
        session: &mut HeadlessSession,
        params: Value,
    ) -> Result<Value, ProtocolError> {
        let url = params.get("url").and_then(|v| v.as_str());
        let tab_id = session.shell.new_tab(url);

        Ok(serde_json::json!({
            "context": tab_id.0,
            "url": url.unwrap_or("about:blank"),
        }))
    }

    /// browsingContext.getTree — 获取浏览上下文树（标签页列表）。
    fn cmd_browsing_context_get_tree(&self, session: &mut HeadlessSession) -> Result<Value, ProtocolError> {
        let active_id = session.shell.active_tab_id();
        let tab_count = session.shell.tab_count();

        // 收集所有标签页信息
        let mut children = Vec::new();
        for i in 0..tab_count {
            let tab_id = TabId(i as u64);
            let is_active = active_id == Some(tab_id);
            children.push(serde_json::json!({
                "context": i,
                "url": "about:blank",
                "active": is_active,
            }));
        }

        Ok(serde_json::json!({
            "contexts": children,
        }))
    }

    /// browsingContext.close — 关闭指定浏览上下文（标签页）。
    fn cmd_browsing_context_close(&self, session: &mut HeadlessSession, params: Value) -> Result<Value, ProtocolError> {
        let context = params
            .get("context")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'context' parameter".into(),
            })?;

        session.shell.close_tab(TabId(context));
        Ok(serde_json::json!({ "result": "closed" }))
    }

    /// browsingContext.reload — 重新加载当前页面。
    fn cmd_browsing_context_reload(&self, session: &mut HeadlessSession) -> Result<Value, ProtocolError> {
        // 重新渲染当前缓存内容
        #[cfg(test)]
        let _ = session.webview.render();
        #[cfg(not(test))]
        session
            .renderer
            .send(IpcMessage {
                id: 0,
                kind: IpcMessageKind::RequestFrame,
            })
            .map_err(|error| ProtocolError {
                code: -32000,
                message: error.to_string(),
            })?;
        Ok(serde_json::json!({ "result": "reloaded" }))
    }

    /// script.callFunction — 调用指定的 JS 函数（通过表 达式包装）。
    fn cmd_script_call_function(&self, session: &mut HeadlessSession, params: Value) -> Result<Value, ProtocolError> {
        let function_declaration = params
            .get("functionDeclaration")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'functionDeclaration' parameter".into(),
            })?;

        let args = params.get("arguments").and_then(|v| v.as_array());

        // 将函数调用转换为可执行表达式
        let expression = if let Some(args) = args {
            let args_json: Vec<String> = args
                .iter()
                .filter_map(|a| a.get("value").and_then(|v| serde_json::to_string(v).ok()))
                .collect();
            format!("({function_declaration})({})", args_json.join(", "))
        } else {
            format!("({function_declaration})()")
        };

        #[cfg(test)]
        let result = session.webview.execute_script(&expression);
        #[cfg(not(test))]
        let result = session.execute_script_renderer(&expression);
        match result {
            Ok(result) => Ok(serde_json::json!({
                "result": {
                    "type": "string",
                    "value": result
                }
            })),
            Err(e) => Ok(serde_json::json!({
                "exceptionDetails": {
                    "text": e.to_string()
                }
            })),
        }
    }

    // ── Runtime 域（M1 切片 3 — remoteObject 类型化返回）──

    /// Runtime.evaluate — remoteObject 形状返回（BiDi `script.evaluate` 的扁平字符串
    /// 语义不受影响，走独立实现）。
    fn cmd_runtime_evaluate(&self, session: &mut HeadlessSession, params: Value) -> Result<Value, ProtocolError> {
        let expression = params
            .get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'expression' parameter".into(),
            })?;
        match session.execute_script_typed(expression) {
            Ok(value) => Ok(serde_json::json!({
                "result": automation_value_to_remote_object(&value),
            })),
            Err(e) => Ok(serde_json::json!({
                "result": { "type": "undefined" },
                "exceptionDetails": {
                    "text": e,
                },
            })),
        }
    }

    /// Runtime.callFunctionOn — 无 objectId 路径：函数体包装为表达式在页面上下文执行
    ///（returnByValue 语义）。objectId（远程对象句柄）未实现 → 标准报错，见矩阵 G3。
    fn cmd_runtime_call_function_on(
        &self,
        session: &mut HeadlessSession,
        params: Value,
    ) -> Result<Value, ProtocolError> {
        if params.get("objectId").is_some() {
            return Err(ProtocolError {
                code: -32601,
                message: "Runtime.callFunctionOn with objectId is not implemented".into(),
            });
        }
        let function_declaration = params
            .get("functionDeclaration")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'functionDeclaration' parameter".into(),
            })?;
        let args_json: Vec<String> = params
            .get("arguments")
            .and_then(|v| v.as_array())
            .map(|args| {
                args.iter()
                    .filter_map(|a| a.get("value").and_then(|v| serde_json::to_string(v).ok()))
                    .collect()
            })
            .unwrap_or_default();
        let expression = format!("({function_declaration})({})", args_json.join(", "));
        match session.execute_script_typed(&expression) {
            Ok(value) => Ok(serde_json::json!({
                "result": automation_value_to_remote_object(&value),
            })),
            Err(e) => Ok(serde_json::json!({
                "result": { "type": "undefined" },
                "exceptionDetails": {
                    "text": e,
                },
            })),
        }
    }

    // ── Page 域（M2 — 导航事件族 / 布局面 / 注入脚本）──

    /// Page.addScriptToEvaluateOnNewDocument — 登记并在**当前文档**立即执行；
    /// 新文档加载后由导航路径重放（ZeroWeb 单引擎：主 world 执行，无 world 隔离）。
    fn cmd_page_add_script_to_evaluate_on_new_document(
        &self,
        session: &mut HeadlessSession,
        params: Value,
    ) -> Result<Value, ProtocolError> {
        let source = params.get("source").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let world_name = params.get("worldName").and_then(|v| v.as_str()).map(str::to_string);
        let n = self.next_session_id.fetch_add(1, Ordering::SeqCst);
        let identifier = format!("zw-script-{n}");
        // 当前文档立即执行（Chromium 对已加载文档同样生效）；失败不阻塞注册
        let _ = session.execute_script_typed(&source);
        session.injected_scripts.push(InjectedScript {
            identifier: identifier.clone(),
            source,
            world_name,
        });
        Ok(serde_json::json!({ "identifier": identifier }))
    }

    /// 重放已登记的注入脚本（导航成功后调用；单条失败不阻断其余）。
    fn replay_injected_scripts(&self, session: &mut HeadlessSession) {
        let sources: Vec<String> = session
            .injected_scripts
            .iter()
            .map(|script| script.source.clone())
            .collect();
        for source in sources {
            let _ = session.execute_script_typed(&source);
        }
    }

    /// 主 world 执行上下文事件（Runtime.enable 与导航后新文档共用）。
    fn push_main_world_context_event(
        &self,
        session: &mut HeadlessSession,
        cdp_session: Option<&str>,
        events: &mut Vec<ServerEvent>,
    ) {
        let frame_id = self.session_frame_id(session, cdp_session);
        events.push(ServerEvent {
            method: "Runtime.executionContextCreated".into(),
            params: serde_json::json!({
                "context": {
                    "id": 1,
                    "origin": "://",
                    "name": "main",
                    "auxData": {
                        "frameId": frame_id,
                        "isDefault": true,
                    }
                }
            }),
            session_id: None,
        });
    }

    fn cdp_timestamp_now() -> f64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as f64
    }

    /// CDP Page.navigate — {frameId, loaderId} 形状；成功后补发导航事件族
    ///（Chromium 时序：frameStartedLoading → frameNavigated → executionContextsCleared
    /// → 新文档 context → domContentEventFired → loadEventFired → frameStoppedLoading），
    /// 并重放 addScriptToEvaluateOnNewDocument 注入脚本。
    fn cmd_page_navigate(
        &self,
        session: &mut HeadlessSession,
        cdp_session: Option<&str>,
        params: Value,
        events: &mut Vec<ServerEvent>,
    ) -> Result<Value, ProtocolError> {
        let url = params.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let frame_id = self.session_frame_id(session, cdp_session);
        let loader_id = format!("zw-loader-{}", self.next_session_id.fetch_add(1, Ordering::SeqCst));

        let nav_params = serde_json::json!({ "url": url });
        let result = self.cmd_navigate(session, nav_params);
        let (success, error_text) = match result {
            Ok(val) => (val.get("success").and_then(|v| v.as_bool()).unwrap_or(false), None),
            Err(err) => (false, Some(err.message)),
        };
        if !success {
            return Ok(serde_json::json!({
                "frameId": frame_id,
                "loaderId": loader_id,
                "errorText": error_text.unwrap_or_else(|| "net::ERR_FAILED".into()),
            }));
        }

        self.emit_navigation_event_family(session, cdp_session, &frame_id, &loader_id, &url, events);
        Ok(serde_json::json!({ "frameId": frame_id, "loaderId": loader_id }))
    }

    /// 导航成功后的事件族（Chromium 时序：frameStartedLoading → frameNavigated →
    /// executionContextsCleared → 新文档 context → domContent → load → frameStoppedLoading），
    /// 含注入脚本重放。pub(super) 供单测直测事件序列。
    pub(super) fn emit_navigation_event_family(
        &self,
        session: &mut HeadlessSession,
        cdp_session: Option<&str>,
        frame_id: &str,
        loader_id: &str,
        url: &str,
        events: &mut Vec<ServerEvent>,
    ) {
        let ts = Self::cdp_timestamp_now();
        let frame_event = |method: &str| ServerEvent {
            method: method.into(),
            params: serde_json::json!({ "frameId": frame_id }),
            session_id: None,
        };
        events.push(frame_event("Page.frameStartedLoading"));
        events.push(ServerEvent {
            method: "Page.frameNavigated".into(),
            params: serde_json::json!({
                "frame": {
                    "id": frame_id,
                    "loaderId": loader_id,
                    "url": url,
                    "mimeType": "text/html",
                }
            }),
            session_id: None,
        });
        events.push(ServerEvent {
            method: "Runtime.executionContextsCleared".into(),
            params: serde_json::json!({}),
            session_id: None,
        });
        self.replay_injected_scripts(session);
        self.push_main_world_context_event(session, cdp_session, events);
        // 新文档后重发 world 级 context（如 Playwright utility world——title/evaluate
        // 管线在 utilityContext() 上等待，缺事件会永久挂起，2026-09-12 实测）
        let frame_id_str = frame_id.to_string();
        let worlds: Vec<String> = session
            .injected_scripts
            .iter()
            .filter_map(|script| script.world_name.clone())
            .collect();
        for world_name in worlds {
            let n = self.next_session_id.fetch_add(1, Ordering::SeqCst);
            events.push(ServerEvent {
                method: "Runtime.executionContextCreated".into(),
                params: serde_json::json!({
                    "context": {
                        "id": n as i64,
                        "origin": "://",
                        "name": world_name,
                        "auxData": {
                            "frameId": frame_id_str,
                            "isDefault": false,
                        }
                    }
                }),
                session_id: None,
            });
        }
        events.push(ServerEvent {
            method: "Page.lifecycleEvent".into(),
            params: serde_json::json!({ "frameId": frame_id, "name": "DOMContentLoaded", "timestamp": ts }),
            session_id: None,
        });
        events.push(ServerEvent {
            method: "Page.domContentEventFired".into(),
            params: serde_json::json!({ "timestamp": ts }),
            session_id: None,
        });
        events.push(ServerEvent {
            method: "Page.lifecycleEvent".into(),
            params: serde_json::json!({ "frameId": frame_id, "name": "load", "timestamp": ts }),
            session_id: None,
        });
        events.push(ServerEvent {
            method: "Page.loadEventFired".into(),
            params: serde_json::json!({ "timestamp": ts }),
            session_id: None,
        });
        events.push(frame_event("Page.frameStoppedLoading"));
    }

    /// Page.getLayoutMetrics — 视口映射（启动参数初始化，Emulation 运行时可变）。
    fn cmd_page_get_layout_metrics(&self) -> Value {
        let (vw, vh) = self.viewport_size();
        let w = vw as i64;
        let h = vh as i64;
        serde_json::json!({
            "layoutViewport": { "pageX": 0, "pageY": 0, "clientWidth": w, "clientHeight": h },
            "visualViewport": {
                "offsetX": 0, "offsetY": 0, "pageX": 0, "pageY": 0,
                "clientWidth": w, "clientHeight": h, "scale": 1, "zoom": 1,
            },
            "contentSize": { "x": 0, "y": 0, "width": w, "height": h },
            "cssLayoutViewport": { "pageX": 0, "pageY": 0, "clientWidth": w, "clientHeight": h },
            "cssVisualViewport": {
                "offsetX": 0, "offsetY": 0, "pageX": 0, "pageY": 0,
                "clientWidth": w, "clientHeight": h, "scale": 1, "zoom": 1,
            },
            "cssContentSize": { "x": 0, "y": 0, "width": w, "height": h },
        })
    }

    // ── Input 域（M2 — CDP 输入 → renderer IPC）──

    /// CDP modifiers 位掩码 → KeyboardEventParams 布尔组（Alt=1 Ctrl=2 Meta=4 Shift=8）。
    fn decode_modifiers(mask: i64) -> (bool, bool, bool, bool) {
        (mask & 2 != 0, mask & 8 != 0, mask & 1 != 0, mask & 4 != 0)
    }

    /// Input.dispatchMouseEvent — mousePressed/Released/Moved/Wheel →
    /// renderer MouseEvent/ScrollEvent。click 语义：released 时按 clickCount 合成
    /// Click/DblClick（renderer 侧 `Click` 类型即完整 click DOM 事件合成）。
    fn cmd_input_dispatch_mouse_event(
        &self,
        session: &mut HeadlessSession,
        params: Value,
    ) -> Result<Value, ProtocolError> {
        let event_type = params.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let x = params.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let y = params.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let button = params.get("button").and_then(|v| v.as_str()).unwrap_or("none");
        let button_id = match button {
            "left" => 0u8,
            "middle" => 1,
            "right" => 2,
            _ => 0,
        };
        let click_count = params.get("clickCount").and_then(|v| v.as_i64()).unwrap_or(0);

        let send = |session: &mut HeadlessSession, m: MouseEventType| {
            session
                .send_input_mouse(x, y, button_id, m)
                .map_err(|error| ProtocolError {
                    code: -32000,
                    message: error,
                })
        };
        match event_type {
            "mousePressed" => {
                send(session, MouseEventType::Down)?;
            }
            "mouseReleased" => {
                send(session, MouseEventType::Up)?;
                match click_count {
                    2 => send(session, MouseEventType::DblClick)?,
                    1..=i64::MAX => send(session, MouseEventType::Click)?,
                    _ => {}
                }
            }
            "mouseMoved" => send(session, MouseEventType::Move)?,
            "mouseWheel" => {
                let delta_x = params.get("deltaX").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let delta_y = params.get("deltaY").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                session
                    .send_input_scroll(delta_x, delta_y, x, y)
                    .map_err(|error| ProtocolError {
                        code: -32000,
                        message: error,
                    })?;
            }
            other => {
                return Err(ProtocolError {
                    code: -32602,
                    message: format!("Unknown mouse event type: {other}"),
                });
            }
        }
        Ok(serde_json::json!({}))
    }

    /// Input.dispatchKeyEvent — keyDown/rawKeyDown→Down、keyUp→Up、char→Press
    ///（Press 优先用 text 作为输入内容）。
    fn cmd_input_dispatch_key_event(
        &self,
        session: &mut HeadlessSession,
        params: Value,
    ) -> Result<Value, ProtocolError> {
        let event_type = params.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let key = params.get("key").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let code = params.get("code").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let modifiers = params.get("modifiers").and_then(|v| v.as_i64()).unwrap_or(0);
        let (ctrl, shift, alt, meta) = Self::decode_modifiers(modifiers);
        let key_type = match event_type {
            "keyDown" | "rawKeyDown" => KeyboardEventType::Down,
            "keyUp" => KeyboardEventType::Up,
            "char" => KeyboardEventType::Press,
            other => {
                return Err(ProtocolError {
                    code: -32602,
                    message: format!("Unknown key event type: {other}"),
                });
            }
        };
        let key_text = params
            .get("text")
            .and_then(|v| v.as_str())
            .filter(|t| !t.is_empty())
            .unwrap_or(&key)
            .to_string();
        session
            .send_input_key(key_text, code, ctrl, shift, alt, meta, key_type)
            .map_err(|error| ProtocolError {
                code: -32000,
                message: error,
            })?;
        Ok(serde_json::json!({}))
    }

    /// Input.insertText — ImeEvent Commit（合成文本提交）。
    fn cmd_input_insert_text(&self, session: &mut HeadlessSession, params: Value) -> Result<Value, ProtocolError> {
        let text = params
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'text' parameter".into(),
            })?
            .to_string();
        session.send_input_ime_commit(text).map_err(|error| ProtocolError {
            code: -32000,
            message: error,
        })?;
        Ok(serde_json::json!({}))
    }

    // ── Storage / Network 域（M4 — cookie jar 接线 + Network 门控事件）──

    /// Cookie struct → CDP cookie 描述形状。
    fn cookie_to_cdp(cookie: &zero_net::cookie::Cookie) -> Value {
        let expires = cookie
            .expires
            .map(|e| serde_json::json!(e as f64))
            .unwrap_or_else(|| serde_json::json!(-1.0));
        serde_json::json!({
            "name": cookie.name,
            "value": cookie.value,
            "domain": cookie.domain.clone().unwrap_or_default(),
            "path": cookie.path.clone().unwrap_or_else(|| "/".to_string()),
            "expires": expires,
            "size": cookie.name.len() + cookie.value.len(),
            "httpOnly": cookie.http_only,
            "secure": cookie.secure,
            "session": cookie.expires.is_none(),
            "priority": "Medium",
        })
    }

    /// Storage.getCookies — 浏览器级全量枚举（PW 按 URL 客户端侧过滤）。
    fn cmd_storage_get_cookies(&self, session: &mut HeadlessSession) -> Value {
        let cookies: Vec<Value> = session
            .cookie_store
            .all()
            .iter()
            .map(|c| Self::cookie_to_cdp(c))
            .collect();
        serde_json::json!({ "cookies": cookies })
    }

    /// Storage.setCookies — CDP cookie 描述 → jar（url 或 domain+path 定位作用域）。
    fn cmd_storage_set_cookies(&self, session: &mut HeadlessSession, params: Value) -> Result<Value, ProtocolError> {
        let cookies = params
            .get("cookies")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'cookies' parameter".into(),
            })?;
        for descriptor in cookies {
            let name = descriptor.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let value = descriptor.get("value").and_then(|v| v.as_str()).unwrap_or("");
            if name.is_empty() {
                continue;
            }
            let mut builder = zero_net::cookie::Cookie {
                name: name.to_string(),
                value: value.to_string(),
                domain: None,
                host_only: true,
                path: None,
                expires: None,
                secure: false,
                http_only: false,
                same_site: zero_net::cookie::SameSite::Lax,
                creation_time: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };
            if let Some(url) = descriptor.get("url").and_then(|v| v.as_str()) {
                if let Ok(parsed) = zero_net::parse_url(url) {
                    builder.domain = parsed.host.clone();
                    builder.secure = parsed.scheme == "https";
                    if let Some(path) = descriptor.get("path").and_then(|v| v.as_str()) {
                        builder.path = Some(path.to_string());
                    } else {
                        builder.path = Some(parsed.path.trim_end_matches('/').to_string());
                    }
                    session.cookie_store.add(builder);
                    continue;
                }
            }
            if let Some(domain) = descriptor.get("domain").and_then(|v| v.as_str()) {
                builder.domain = Some(domain.trim_start_matches('.').to_string());
                builder.host_only = false;
            }
            if let Some(path) = descriptor.get("path").and_then(|v| v.as_str()) {
                builder.path = Some(path.to_string());
            }
            if let Some(secure) = descriptor.get("secure").and_then(|v| v.as_bool()) {
                builder.secure = secure;
            }
            if let Some(http_only) = descriptor.get("httpOnly").and_then(|v| v.as_bool()) {
                builder.http_only = http_only;
            }
            if let Some(expires) = descriptor.get("expires").and_then(|v| v.as_f64()) {
                builder.expires = if expires < 0.0 { None } else { Some(expires as u64) };
            }
            session.cookie_store.add(builder);
        }
        Ok(serde_json::json!({}))
    }

    /// Storage.clearCookies — 清空 jar。
    fn cmd_storage_clear_cookies(&self, session: &mut HeadlessSession) -> Value {
        session.cookie_store.clear();
        serde_json::json!({})
    }

    /// Emulation.setUserAgentOverride — proxy_fetch 注入 User-Agent 请求头。
    fn cmd_emulation_set_user_agent_override(
        &self,
        session: &mut HeadlessSession,
        params: Value,
    ) -> Result<Value, ProtocolError> {
        let ua = params
            .get("userAgent")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'userAgent' parameter".into(),
            })?
            .to_string();
        session.user_agent_override = Some(ua);
        Ok(serde_json::json!({}))
    }

    // ── Target 域（M1 切片 3 — Playwright connectOverCDP 连接脊柱）──

    /// 解析 targetId（`zeroweb-tab-<n>`，与 HTTP 发现枚举一致）→ TabId。
    fn parse_target_id(target_id: &str) -> Option<TabId> {
        target_id
            .strip_prefix("zeroweb-tab-")
            .and_then(|n| n.parse::<u64>().ok())
            .map(TabId)
    }

    /// TabInfo 的 CDP targetInfo 形状（Target 域与 attachedToTarget 事件共用）。
    fn target_info_json(tab: &zero_browser_shell::Tab) -> Value {
        serde_json::json!({
            "targetId": format!("zeroweb-tab-{}", tab.id().0),
            "type": "page",
            "title": tab.title().unwrap_or("ZeroWeb"),
            "url": tab.url().unwrap_or("about:blank"),
            "attached": true,
            "canAccessOpener": false,
            "browserContextId": "zeroweb-default",
        })
    }

    /// Target.setAutoAttach — flatten 模式：对现存每个 page target 分配 sessionId、
    /// 登记注册表并逐个发 `Target.attachedToTarget`；此后新建 target（createTarget）
    /// 自动附接。Playwright 连接的第一条 Target 命令。
    fn cmd_target_set_auto_attach(
        &self,
        session: &mut HeadlessSession,
        params: &Value,
        events: &mut Vec<ServerEvent>,
    ) -> Result<Value, ProtocolError> {
        let enable = params.get("autoAttach").and_then(|v| v.as_bool()).unwrap_or(false);
        self.set_auto_attach(enable);
        // waitForDebuggerOnStart：ZeroWeb 无 debugger 暂停语义，恒 false 生效。
        if !enable {
            return Ok(serde_json::json!({}));
        }
        for tab in session.shell.tabs().collect::<Vec<_>>() {
            let sid = self.next_cdp_session();
            let info = Self::target_info_json(tab);
            self.attach_session(&sid, info["targetId"].as_str().unwrap_or_default());
            events.push(ServerEvent {
                method: "Target.attachedToTarget".into(),
                params: serde_json::json!({
                    "sessionId": sid,
                    "targetInfo": info,
                    "waitingForDebugger": false,
                }),
                session_id: None,
            });
        }
        Ok(serde_json::json!({}))
    }

    /// Target.createTarget — 新建标签页；autoAttach 开启时自动附接并发事件。
    fn cmd_target_create_target(
        &self,
        session: &mut HeadlessSession,
        params: &Value,
        events: &mut Vec<ServerEvent>,
    ) -> Result<Value, ProtocolError> {
        let url = params.get("url").and_then(|v| v.as_str());
        let tab_id = session.shell.new_tab(url);
        let target_id = format!("zeroweb-tab-{}", tab_id.0);
        if self.auto_attach_enabled() {
            let sid = self.next_cdp_session();
            self.attach_session(&sid, &target_id);
            events.push(ServerEvent {
                method: "Target.attachedToTarget".into(),
                params: serde_json::json!({
                    "sessionId": sid,
                    "targetInfo": {
                        "targetId": target_id,
                        "type": "page",
                        "title": url.unwrap_or("about:blank"),
                        "url": url.unwrap_or("about:blank"),
                        "attached": true,
                        "canAccessOpener": false,
                        "browserContextId": "zeroweb-default",
                    },
                    "waitingForDebugger": false,
                }),
                session_id: None,
            });
        }
        Ok(serde_json::json!({ "targetId": target_id }))
    }

    /// Target.closeTarget — 关闭标签页、摘除其全部附接会话，发 `Target.targetDestroyed`。
    fn cmd_target_close_target(
        &self,
        session: &mut HeadlessSession,
        params: &Value,
        events: &mut Vec<ServerEvent>,
    ) -> Result<Value, ProtocolError> {
        let target_id = params
            .get("targetId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'targetId' parameter".into(),
            })?;
        if let Some(tab_id) = Self::parse_target_id(target_id) {
            session.shell.close_tab(tab_id);
        }
        for sid in self.detach_sessions_for_target(target_id) {
            events.push(ServerEvent {
                method: "Target.detachedFromTarget".into(),
                params: serde_json::json!({ "sessionId": sid, "targetId": target_id }),
                session_id: None,
            });
        }
        events.push(ServerEvent {
            method: "Target.targetDestroyed".into(),
            params: serde_json::json!({ "targetId": target_id }),
            session_id: None,
        });
        // Chromium 153 对 closeTarget 的响应体（捕获实测）
        Ok(serde_json::json!({ "success": true }))
    }

    /// Target.detachFromTarget — 客户端主动解除附接，发 `Target.detachedFromTarget`。
    fn cmd_target_detach_from_target(
        &self,
        params: &Value,
        events: &mut Vec<ServerEvent>,
    ) -> Result<Value, ProtocolError> {
        let sid = params
            .get("sessionId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'sessionId' parameter".into(),
            })?;
        let target_id = self.session_target(sid);
        if self.detach_session(sid) {
            events.push(ServerEvent {
                method: "Target.detachedFromTarget".into(),
                params: serde_json::json!({
                    "sessionId": sid,
                    "targetId": target_id.unwrap_or_default(),
                }),
                session_id: None,
            });
        }
        Ok(serde_json::json!({}))
    }

    /// Target.getTargetInfo — 无 targetId = 浏览器级 target；带 targetId = 查标签页。
    fn cmd_target_get_target_info(&self, session: &mut HeadlessSession, params: Value) -> Result<Value, ProtocolError> {
        if let Some(target_id) = params.get("targetId").and_then(|v| v.as_str()) {
            let want = Self::parse_target_id(target_id);
            for tab in session.shell.tabs() {
                if Some(tab.id()) == want {
                    return Ok(serde_json::json!({ "targetInfo": Self::target_info_json(tab) }));
                }
            }
            return Err(ProtocolError {
                code: -32602,
                message: format!("Target not found: {target_id}"),
            });
        }
        // 浏览器级 target（连接握手时 Playwright 查询）
        Ok(serde_json::json!({
            "targetInfo": {
                "targetId": "zeroweb-browser",
                "type": "browser",
                "title": "",
                "url": "",
                "attached": true,
                "canAccessOpener": false,
            }
        }))
    }

    /// Target.getTargets — CDP 形状 `{targetInfos: [...]}`（旧实现误用 BiDi tree 形状）。
    fn cmd_target_get_targets(&self, session: &mut HeadlessSession) -> Result<Value, ProtocolError> {
        let infos: Vec<Value> = session.shell.tabs().map(Self::target_info_json).collect();
        Ok(serde_json::json!({ "targetInfos": infos }))
    }

    /// Page.getFrameTree — 主 frame 树（ZeroWeb 无子 frame 语义，childFrames 恒空）。
    /// frame id 取 `zeroweb-frame-<TabId>`，后续 Page.frameNavigated 等事件须对齐同 id。
    /// 请求来自某个附接会话时，frame 归属该会话的 target（而非全局活跃 tab）。
    fn cmd_page_get_frame_tree(
        &self,
        session: &mut HeadlessSession,
        cdp_session: Option<&str>,
    ) -> Result<Value, ProtocolError> {
        self.session_tab(session, cdp_session)
            .ok_or_else(|| ProtocolError {
                code: -32000,
                message: "No active tab".into(),
            })
            .map(|tab| {
                let frame = serde_json::json!({
                    "id": Self::frame_id_for_tab(tab.id()),
                    "loaderId": "",
                    "url": tab.url().unwrap_or("about:blank"),
                    "mimeType": "text/html",
                });
                serde_json::json!({ "frameTree": { "frame": frame, "childFrames": [] } })
            })
    }

    /// 主 frame id 与 targetId 同值（CDP 契约：page target 的根 frame 复用 targetId，
    /// Playwright 按 targetId 索引主 frame 会话，帧 id 另起命名会报 Frame detached）。
    fn frame_id_for_tab(tab_id: TabId) -> String {
        format!("zeroweb-tab-{}", tab_id.0)
    }

    /// Page.createIsolatedWorld — 隔离执行 world（ZeroWeb 单引擎，world 仅记账不隔离）。
    /// 返回新 executionContextId；事件侧以 worldName 供 Playwright 识别 utility world。
    fn cmd_page_create_isolated_world(
        &self,
        session: &mut HeadlessSession,
        cdp_session: Option<&str>,
        params: &Value,
        events: &mut Vec<ServerEvent>,
    ) -> Result<Value, ProtocolError> {
        let n = self.next_session_id.fetch_add(1, Ordering::SeqCst);
        let context_id = n as i64;
        let world_name = params.get("worldName").and_then(|v| v.as_str()).unwrap_or("");
        let frame_id = self.session_frame_id(session, cdp_session);
        events.push(ServerEvent {
            method: "Runtime.executionContextCreated".into(),
            params: serde_json::json!({
                "context": {
                    "id": context_id,
                    "origin": "://",
                    "name": world_name,
                    "auxData": {
                        "frameId": frame_id,
                        "isDefault": false,
                    }
                }
            }),
            session_id: None,
        });
        Ok(serde_json::json!({ "executionContextId": context_id }))
    }

    /// 解析请求归属的标签页：附接会话 → 其 target 的标签页；浏览器级 → 活跃标签页。
    fn session_tab<'a>(
        &self,
        session: &'a mut HeadlessSession,
        cdp_session: Option<&str>,
    ) -> Option<&'a zero_browser_shell::Tab> {
        let want = cdp_session
            .and_then(|sid| self.session_target(sid))
            .and_then(|tid| Self::parse_target_id(&tid));
        session
            .shell
            .tabs()
            .find(|tab| Some(tab.id()) == want)
            .or_else(|| session.shell.active_tab())
    }

    /// 请求归属的 frame id（Runtime.executionContextCreated 的 auxData 用）。
    fn session_frame_id(&self, session: &mut HeadlessSession, cdp_session: Option<&str>) -> String {
        self.session_tab(session, cdp_session)
            .map(|tab| Self::frame_id_for_tab(tab.id()))
            .unwrap_or_else(|| "zeroweb-tab-0".into())
    }
}
