//! CDP Emulation 域 — 视口/媒体/UA 覆写。

use serde_json::Value;

use crate::headless::HeadlessServer;

use crate::headless::protocol::{ProtocolError, ServerEvent};
use crate::headless::session::HeadlessSession;

impl HeadlessServer {
    /// Emulation.setDeviceMetricsOverride — viewport 桥：renderer SetViewport +
    /// 服务器视口状态（getLayoutMetrics/captureScreenshot 联动）+ Page.frameResized 事件。
    pub(super) fn cmd_emulation_set_device_metrics_override(
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
    pub(super) fn cmd_emulation_set_emulated_media(
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

    /// Emulation.setUserAgentOverride — proxy_fetch 注入 User-Agent 请求头。
    pub(super) fn cmd_emulation_set_user_agent_override(
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
}
