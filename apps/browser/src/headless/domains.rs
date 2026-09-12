//! 协议命令域实现 — BiDi 会话/浏览上下文/脚本/截图域 + CDP 兼容子集（Phase 3）。

use std::sync::atomic::Ordering;

use serde_json::Value;
use zero_browser_shell::TabId;
#[cfg(not(test))]
use zero_protocol::message::{IpcMessage, IpcMessageKind};
use zero_render_foundation::cpu::render_full_scene;
use zero_render_foundation::font::cache::GlyphCache;
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::surface::FrameBuffer;

use super::HeadlessServer;

use super::protocol::{ProtocolError, ServerEvent};
use super::session::HeadlessSession;

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

            // ── 未知命令 ──
            _ => Err(ProtocolError {
                code: -32601,
                message: format!("Unknown method: {method}"),
            }),
        }
    }

    /// 带事件生成的命令路由。
    pub(super) fn dispatch_with_events(
        &self,
        session: &mut HeadlessSession,
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
                    });
                }
                (result, events)
            }
            // CDP 兼容域（Phase 3）
            "Page.navigate" => {
                let url = params.get("url").and_then(|v| v.as_str()).unwrap_or("");
                let nav_params = serde_json::json!({ "url": url });
                let result = self.cmd_navigate(session, nav_params);
                if result.is_ok() {
                    events.push(ServerEvent {
                        method: "Page.loadEventFired".into(),
                        params: serde_json::json!({ "timestamp": 0.0 }),
                    });
                }
                (result, events)
            }
            "Page.captureScreenshot" => (self.cmd_capture_screenshot(session), events),
            "Runtime.evaluate" => (self.cmd_script_evaluate(session, params), events),
            "Target.getTargets" => (self.cmd_browsing_context_get_tree(session), events),
            "Network.enable" => {
                // 启用网络事件追踪（桩：接受命令但不产生事件）
                (Ok(serde_json::json!({ "result": "enabled" })), events)
            }
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

    fn cmd_capture_screenshot(&self, session: &mut HeadlessSession) -> Result<Value, ProtocolError> {
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
            let w = self.viewport_width as u32;
            let h = self.viewport_height as u32;
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
            render_full_scene(
                self.viewport_width as u32,
                self.viewport_height as u32,
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
}
