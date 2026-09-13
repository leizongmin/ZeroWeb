//! CDP Page 域 — 截图/注入脚本/导航/布局度量/frame 树/隔离 world + 导航事件族 helper。

use std::sync::atomic::Ordering;

use serde_json::Value;
use zero_browser_shell::TabId;
use zero_render_foundation::cpu::render_full_scene;
use zero_render_foundation::font::cache::GlyphCache;
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::surface::FrameBuffer;

use crate::headless::HeadlessServer;

use super::remote_object::framebuffer_to_png_base64;
use crate::headless::protocol::{ProtocolError, ServerEvent};
use crate::headless::session::{HeadlessSession, InjectedScript};

impl HeadlessServer {
    /// CDP Page.captureScreenshot — `{"data": "<base64 png>"}` 形状（区别于 BiDi 对象形）；
    /// 支持 clip 裁剪（原始 fb 行级裁剪）。format 仅支持 png（jpeg 编码器未接入）。
    pub(super) fn cmd_page_capture_screenshot(
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

    /// Page.addScriptToEvaluateOnNewDocument — 登记并在**当前文档**立即执行；
    /// 新文档加载后由导航路径重放（ZeroWeb 单引擎：主 world 执行，无 world 隔离）。
    pub(super) fn cmd_page_add_script_to_evaluate_on_new_document(
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

    /// CDP Page.navigate — {frameId, loaderId} 形状；成功后补发导航事件族
    ///（Chromium 时序：frameStartedLoading → frameNavigated → executionContextsCleared
    /// → 新文档 context → domContentEventFired → loadEventFired → frameStoppedLoading），
    /// 并重放 addScriptToEvaluateOnNewDocument 注入脚本。
    pub(super) fn cmd_page_navigate(
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

    /// Page.getLayoutMetrics — 视口映射（启动参数初始化，Emulation 运行时可变）。
    pub(super) fn cmd_page_get_layout_metrics(&self) -> Value {
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

    /// Page.getFrameTree — 主 frame 树（ZeroWeb 无子 frame 语义，childFrames 恒空）。
    /// frame id 取 `zeroweb-frame-<TabId>`，后续 Page.frameNavigated 等事件须对齐同 id。
    /// 请求来自某个附接会话时，frame 归属该会话的 target（而非全局活跃 tab）。
    pub(super) fn cmd_page_get_frame_tree(
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

    /// Page.createIsolatedWorld — 隔离执行 world（ZeroWeb 单引擎，world 仅记账不隔离）。
    /// 返回新 executionContextId；事件侧以 worldName 供 Playwright 识别 utility world。
    pub(super) fn cmd_page_create_isolated_world(
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

    /// 渲染当前页面帧（GPU 开关路径 → CPU 兜底），供 BiDi 与 CDP 截图共用。
    pub(super) fn render_page_framebuffer(&self, session: &mut HeadlessSession) -> Result<FrameBuffer, ProtocolError> {
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

    /// 重放已登记的注入脚本（导航成功后调用；单条失败不阻断其余）。
    pub(super) fn replay_injected_scripts(&self, session: &mut HeadlessSession) {
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
    pub(super) fn push_main_world_context_event(
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

    pub(super) fn cdp_timestamp_now() -> f64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as f64
    }

    /// 导航成功后的事件族（Chromium 时序：frameStartedLoading → frameNavigated →
    /// executionContextsCleared → 新文档 context → domContent → load → frameStoppedLoading），
    /// 含注入脚本重放。pub(super) 供单测直测事件序列。
    pub(in crate::headless) fn emit_navigation_event_family(
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

    /// 主 frame id 与 targetId 同值（CDP 契约：page target 的根 frame 复用 targetId，
    /// Playwright 按 targetId 索引主 frame 会话，帧 id 另起命名会报 Frame detached）。
    pub(super) fn frame_id_for_tab(tab_id: TabId) -> String {
        format!("zeroweb-tab-{}", tab_id.0)
    }

    /// 解析请求归属的标签页：附接会话 → 其 target 的标签页；浏览器级 → 活跃标签页。
    pub(super) fn session_tab<'a>(
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
    pub(super) fn session_frame_id(&self, session: &mut HeadlessSession, cdp_session: Option<&str>) -> String {
        self.session_tab(session, cdp_session)
            .map(|tab| Self::frame_id_for_tab(tab.id()))
            .unwrap_or_else(|| "zeroweb-tab-0".into())
    }
}
