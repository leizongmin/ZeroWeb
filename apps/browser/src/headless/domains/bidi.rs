//! BiDi 会话/浏览上下文/脚本/截图域 — goal 前既有自动化面。

use std::sync::atomic::Ordering;

use serde_json::Value;
use zero_browser_shell::TabId;
#[cfg(not(test))]
use zero_protocol::message::{IpcMessage, IpcMessageKind};

use crate::headless::HeadlessServer;

use super::remote_object::framebuffer_to_png_base64;
use crate::headless::protocol::ProtocolError;
use crate::headless::session::HeadlessSession;

impl HeadlessServer {
    pub(super) fn cmd_session_status(&self) -> Result<Value, ProtocolError> {
        Ok(serde_json::json!({
            "ready": true,
            "message": "ZeroWeb headless server ready"
        }))
    }

    pub(super) fn cmd_session_new(&self) -> Result<Value, ProtocolError> {
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

    pub(super) fn cmd_browser_close(&self) -> Result<Value, ProtocolError> {
        Ok(serde_json::json!({ "result": "closing" }))
    }

    pub(super) fn cmd_navigate(&self, session: &mut HeadlessSession, params: Value) -> Result<Value, ProtocolError> {
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

    pub(super) fn cmd_script_evaluate(
        &self,
        session: &mut HeadlessSession,
        params: Value,
    ) -> Result<Value, ProtocolError> {
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

    pub(super) fn cmd_capture_screenshot(&self, session: &mut HeadlessSession) -> Result<Value, ProtocolError> {
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

    /// DC-13 line 315：加载内联 HTML（绕过 fetch_url 的 HTTP-only 限制），供 headless 路径
    /// 加载自包含 fixture（如 welcome.html，内联 CSS + data-URI 图标）做截图对比。参数 `html`
    ///（必填）+ `css`（可选）。区别于 `browsingContext.navigate`（需 URL + HTTP/file 抓取）。
    pub(super) fn cmd_load_html(&self, session: &mut HeadlessSession, params: Value) -> Result<Value, ProtocolError> {
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

    pub(super) fn cmd_get_dom_snapshot(&self, session: &mut HeadlessSession) -> Result<Value, ProtocolError> {
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

    /// browsingContext.create — 创建新的浏览上下文（新标签页）。
    pub(super) fn cmd_browsing_context_create(
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
    pub(super) fn cmd_browsing_context_get_tree(&self, session: &mut HeadlessSession) -> Result<Value, ProtocolError> {
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
    pub(super) fn cmd_browsing_context_close(
        &self,
        session: &mut HeadlessSession,
        params: Value,
    ) -> Result<Value, ProtocolError> {
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
    pub(super) fn cmd_browsing_context_reload(&self, session: &mut HeadlessSession) -> Result<Value, ProtocolError> {
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
    pub(super) fn cmd_script_call_function(
        &self,
        session: &mut HeadlessSession,
        params: Value,
    ) -> Result<Value, ProtocolError> {
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
