//! CDP Input 域 — 鼠标/键盘/文本注入。

use serde_json::Value;
use zero_protocol::message::{KeyboardEventType, MouseEventType};

use crate::headless::HeadlessServer;

use crate::headless::protocol::ProtocolError;
use crate::headless::session::HeadlessSession;

impl HeadlessServer {
    /// Input.dispatchMouseEvent — mousePressed/Released/Moved/Wheel →
    /// renderer MouseEvent/ScrollEvent。click 语义：released 时按 clickCount 合成
    /// Click/DblClick（renderer 侧 `Click` 类型即完整 click DOM 事件合成）。
    pub(super) fn cmd_input_dispatch_mouse_event(
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
    pub(super) fn cmd_input_dispatch_key_event(
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
    pub(super) fn cmd_input_insert_text(
        &self,
        session: &mut HeadlessSession,
        params: Value,
    ) -> Result<Value, ProtocolError> {
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

    /// CDP modifiers 位掩码 → KeyboardEventParams 布尔组（Alt=1 Ctrl=2 Meta=4 Shift=8）。
    pub(super) fn decode_modifiers(mask: i64) -> (bool, bool, bool, bool) {
        (mask & 2 != 0, mask & 8 != 0, mask & 1 != 0, mask & 4 != 0)
    }
}
