//! WebDriver session actor backed by a live `zero-renderer` child.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use zero_net::{HttpClient, HttpMethod, HttpRequest};
use zero_protocol::ProtocolError;
use zero_protocol::message::{
    AutomationElementRef, AutomationError, AutomationErrorCode, AutomationKey, AutomationLocatorStrategy,
    AutomationOperation, AutomationRequest, AutomationResult, AutomationValue, FetchParams, FramePublishMode,
    IpcMessage, IpcMessageKind, SetViewportParams,
};
use zero_protocol::process::RendererHandle;

const NAVIGATION_TIMEOUT: Duration = Duration::from_secs(15);
const AUTOMATION_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_ELEMENT_REFERENCES: usize = 4096;

pub struct Driver {
    sessions: HashMap<String, Session>,
    next_session_id: u64,
    renderer_bin: PathBuf,
}

struct Session {
    renderer: RendererHandle,
    http: HttpClient,
    title: String,
    navigation_epoch: u64,
    next_request_id: u64,
    next_element_id: u64,
    elements: HashMap<String, AutomationElementRef>,
    reverse_elements: HashMap<AutomationElementRef, String>,
}

#[derive(Debug)]
pub struct DriverError {
    pub code: &'static str,
    pub message: String,
}

impl DriverError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl Driver {
    pub fn new() -> Result<Self, DriverError> {
        let renderer_bin = resolve_renderer_binary()
            .ok_or_else(|| DriverError::new("session not created", "zero-renderer binary not found"))?;
        Ok(Self {
            sessions: HashMap::new(),
            next_session_id: 1,
            renderer_bin,
        })
    }

    pub fn create_session(&mut self) -> Result<String, DriverError> {
        let id = format!("{:016x}", self.next_session_id);
        self.next_session_id += 1;
        let mut renderer = RendererHandle::spawn(self.renderer_bin.to_string_lossy().as_ref())
            .map_err(|error| DriverError::new("session not created", error.to_string()))?;
        renderer
            .send(IpcMessage {
                id: 0,
                kind: IpcMessageKind::SetFramePublishMode(FramePublishMode::Legacy),
            })
            .map_err(protocol_error)?;
        renderer
            .send(IpcMessage {
                id: 0,
                kind: IpcMessageKind::SetViewport(SetViewportParams {
                    width: 800,
                    height: 600,
                    device_scale_factor: 1.0,
                }),
            })
            .map_err(protocol_error)?;
        self.sessions.insert(
            id.clone(),
            Session {
                renderer,
                http: HttpClient::new(),
                title: String::new(),
                navigation_epoch: 0,
                next_request_id: 1,
                next_element_id: 1,
                elements: HashMap::new(),
                reverse_elements: HashMap::new(),
            },
        );
        Ok(id)
    }

    pub fn delete_session(&mut self, id: &str) -> bool {
        self.sessions.remove(id).is_some()
    }

    pub fn navigate(&mut self, id: &str, url: &str) -> Result<(), DriverError> {
        self.session_mut(id)?.navigate(url)
    }

    pub fn title(&mut self, id: &str) -> Result<String, DriverError> {
        let result = self.session_mut(id)?.request(AutomationOperation::ExecuteScript {
            script: "return document.title;".into(),
            arguments: Vec::new(),
        })?;
        match result {
            AutomationResult::Value(AutomationValue::String(title)) => Ok(title),
            _ => Ok(String::new()),
        }
    }

    pub fn find_element(&mut self, id: &str, selector: String) -> Result<String, DriverError> {
        let session = self.session_mut(id)?;
        let result = session.request(AutomationOperation::FindElement {
            using: AutomationLocatorStrategy::CssSelector,
            value: selector,
        })?;
        let AutomationResult::Element(Some(element)) = result else {
            return Err(DriverError::new("no such element", "element not found"));
        };
        session.register_element(element)
    }

    pub fn click_element(&mut self, id: &str, opaque_id: &str) -> Result<(), DriverError> {
        let session = self.session_mut(id)?;
        let element = session.element(opaque_id)?;
        session
            .request(AutomationOperation::ElementClick { element })
            .map(|_| ())
    }

    pub fn send_keys(&mut self, id: &str, opaque_id: &str, keys: Vec<AutomationKey>) -> Result<(), DriverError> {
        let session = self.session_mut(id)?;
        let element = session.element(opaque_id)?;
        session
            .request(AutomationOperation::SendKeys { element, keys })
            .map(|_| ())
    }

    pub fn active_element(&mut self, id: &str) -> Result<Option<String>, DriverError> {
        let session = self.session_mut(id)?;
        let result = session.request(AutomationOperation::GetActiveElement)?;
        let AutomationResult::Element(element) = result else {
            return Err(DriverError::new("unknown error", "invalid active element response"));
        };
        element.map(|element| session.register_element(element)).transpose()
    }

    pub fn execute_script(
        &mut self,
        id: &str,
        script: String,
        arguments: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value, DriverError> {
        let arguments = arguments.into_iter().map(automation_value_from_json).collect();
        let result = self
            .session_mut(id)?
            .request(AutomationOperation::ExecuteScript { script, arguments })?;
        let AutomationResult::Value(value) = result else {
            return Ok(serde_json::Value::Null);
        };
        Ok(automation_value_to_json(value))
    }

    fn session_mut(&mut self, id: &str) -> Result<&mut Session, DriverError> {
        self.sessions
            .get_mut(id)
            .ok_or_else(|| DriverError::new("no such session", "session not found"))
    }
}

impl Session {
    fn navigate(&mut self, url: &str) -> Result<(), DriverError> {
        self.navigation_epoch = self.navigation_epoch.wrapping_add(1).max(1);
        self.title.clear();
        self.renderer
            .navigate(url, None, self.navigation_epoch)
            .map_err(protocol_error)?;
        let deadline = Instant::now() + NAVIGATION_TIMEOUT;
        loop {
            if Instant::now() >= deadline {
                return Err(DriverError::new("timeout", "navigation timed out"));
            }
            match self.renderer.try_recv().map_err(protocol_error)? {
                Some(message) => {
                    match handle_renderer_message(&self.http, &mut self.title, &mut self.renderer, message)? {
                        RendererEvent::LoadComplete => return Ok(()),
                        RendererEvent::LoadFailed(message) => {
                            return Err(DriverError::new("unknown error", message));
                        }
                        RendererEvent::Other => {}
                    }
                }
                None => {
                    if !self.renderer.is_alive() {
                        return Err(DriverError::new("unknown error", "renderer exited during navigation"));
                    }
                    std::thread::sleep(Duration::from_millis(2));
                }
            }
        }
    }

    fn request(&mut self, operation: AutomationOperation) -> Result<AutomationResult, DriverError> {
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.wrapping_add(1).max(1);
        let http = &self.http;
        let title = &mut self.title;
        let response = self
            .renderer
            .request_automation(
                request_id,
                AutomationRequest { operation },
                AUTOMATION_TIMEOUT,
                |renderer, message| {
                    handle_renderer_message(http, title, renderer, message)
                        .map(|_| ())
                        .map_err(|error| ProtocolError::Process(error.message))
                },
            )
            .map_err(protocol_error)?;
        response.result.map_err(automation_error)
    }

    fn register_element(&mut self, element: AutomationElementRef) -> Result<String, DriverError> {
        if let Some(existing) = self.reverse_elements.get(&element) {
            return Ok(existing.clone());
        }
        if self.elements.len() >= MAX_ELEMENT_REFERENCES {
            return Err(DriverError::new("unknown error", "element reference limit reached"));
        }
        let id = format!("e{:016x}", self.next_element_id);
        self.next_element_id += 1;
        self.elements.insert(id.clone(), element);
        self.reverse_elements.insert(element, id.clone());
        Ok(id)
    }

    fn element(&self, opaque_id: &str) -> Result<AutomationElementRef, DriverError> {
        self.elements
            .get(opaque_id)
            .copied()
            .ok_or_else(|| DriverError::new("no such element", "unknown element reference"))
    }
}

enum RendererEvent {
    LoadComplete,
    LoadFailed(String),
    Other,
}

fn handle_renderer_message(
    http: &HttpClient,
    title: &mut String,
    renderer: &mut RendererHandle,
    message: IpcMessage,
) -> Result<RendererEvent, DriverError> {
    match message.kind {
        IpcMessageKind::FetchRequest(params) => {
            proxy_fetch(http, renderer, params)?;
            Ok(RendererEvent::Other)
        }
        IpcMessageKind::TitleChanged(value) => {
            *title = value;
            Ok(RendererEvent::Other)
        }
        IpcMessageKind::LoadComplete => Ok(RendererEvent::LoadComplete),
        IpcMessageKind::LoadFailed(message) => Ok(RendererEvent::LoadFailed(message)),
        IpcMessageKind::CrashNotification(message) => Ok(RendererEvent::LoadFailed(message)),
        _ => Ok(RendererEvent::Other),
    }
}

fn proxy_fetch(http: &HttpClient, renderer: &mut RendererHandle, params: FetchParams) -> Result<(), DriverError> {
    let method = match params.method.to_ascii_uppercase().as_str() {
        "POST" => HttpMethod::Post,
        "PUT" => HttpMethod::Put,
        "DELETE" => HttpMethod::Delete,
        "PATCH" => HttpMethod::Patch,
        "HEAD" => HttpMethod::Head,
        "OPTIONS" => HttpMethod::Options,
        _ => HttpMethod::Get,
    };
    let response = http.send(HttpRequest {
        method,
        url: params.url,
        headers: params.headers,
        body: params.body,
    });
    match response {
        Ok(response) => renderer
            .send_fetch_response(params.request_id, response.status_code, response.headers, response.body)
            .map_err(protocol_error),
        Err(error) => renderer
            .send_fetch_response(params.request_id, 0, Vec::new(), error.to_string().into_bytes())
            .map_err(protocol_error),
    }
}

fn automation_error(error: AutomationError) -> DriverError {
    let code = match error.code {
        AutomationErrorCode::NoSuchElement => "no such element",
        AutomationErrorCode::StaleElementReference => "stale element reference",
        AutomationErrorCode::InvalidArgument => "invalid argument",
        AutomationErrorCode::UnsupportedOperation => "unsupported operation",
        AutomationErrorCode::JavascriptError => "javascript error",
        AutomationErrorCode::Timeout => "timeout",
        AutomationErrorCode::Internal => "unknown error",
    };
    DriverError::new(code, error.message)
}

fn protocol_error(error: ProtocolError) -> DriverError {
    let code = if error.to_string().contains("timeout") {
        "timeout"
    } else {
        "unknown error"
    };
    DriverError::new(code, error.to_string())
}

pub fn parse_webdriver_keys(text: &str) -> Vec<AutomationKey> {
    let mut result = Vec::new();
    let mut buffer = String::new();
    let mut shift = false;
    let flush = |result: &mut Vec<AutomationKey>, buffer: &mut String| {
        if !buffer.is_empty() {
            result.push(AutomationKey::Text(std::mem::take(buffer)));
        }
    };
    for character in text.chars() {
        match character {
            '\u{E000}' => {
                flush(&mut result, &mut buffer);
                shift = false;
            }
            '\u{E003}' => {
                flush(&mut result, &mut buffer);
                result.push(AutomationKey::Backspace);
            }
            '\u{E004}' => {
                flush(&mut result, &mut buffer);
                result.push(if shift {
                    AutomationKey::ShiftTab
                } else {
                    AutomationKey::Tab
                });
            }
            '\u{E006}' | '\u{E007}' => {
                flush(&mut result, &mut buffer);
                result.push(AutomationKey::Enter);
            }
            '\u{E008}' => {
                flush(&mut result, &mut buffer);
                shift = true;
            }
            _ => buffer.push(character),
        }
    }
    flush(&mut result, &mut buffer);
    result
}

fn automation_value_from_json(value: serde_json::Value) -> AutomationValue {
    match value {
        serde_json::Value::Null => AutomationValue::Null,
        serde_json::Value::Bool(value) => AutomationValue::Bool(value),
        serde_json::Value::Number(value) => AutomationValue::Number(value.as_f64().unwrap_or_default()),
        serde_json::Value::String(value) => AutomationValue::String(value),
        serde_json::Value::Array(values) => {
            AutomationValue::Array(values.into_iter().map(automation_value_from_json).collect())
        }
        serde_json::Value::Object(entries) => AutomationValue::Object(
            entries
                .into_iter()
                .map(|(key, value)| (key, automation_value_from_json(value)))
                .collect(),
        ),
    }
}

fn automation_value_to_json(value: AutomationValue) -> serde_json::Value {
    match value {
        AutomationValue::Null => serde_json::Value::Null,
        AutomationValue::Bool(value) => serde_json::Value::Bool(value),
        AutomationValue::Number(value) => serde_json::Number::from_f64(value)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        AutomationValue::String(value) => serde_json::Value::String(value),
        AutomationValue::Array(values) => {
            serde_json::Value::Array(values.into_iter().map(automation_value_to_json).collect())
        }
        AutomationValue::Object(entries) => serde_json::Value::Object(
            entries
                .into_iter()
                .map(|(key, value)| (key, automation_value_to_json(value)))
                .collect(),
        ),
    }
}

fn renderer_binary_filename() -> &'static str {
    if cfg!(windows) {
        "zero-renderer.exe"
    } else {
        "zero-renderer"
    }
}

fn resolve_renderer_binary() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("ZERO_RENDERER_PATH") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }
    if let Some(path) = std::env::var_os("CARGO_BIN_EXE_zero-renderer") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }
    if let Ok(executable) = std::env::current_exe()
        && let Some(parent) = executable.parent()
    {
        for directory in [Some(parent), parent.parent()].into_iter().flatten() {
            let candidate = directory.join(renderer_binary_filename());
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
        .map(|directory| directory.join(renderer_binary_filename()))
        .find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webdriver_special_keys_are_typed() {
        assert_eq!(
            parse_webdriver_keys("A\u{E003}\u{E008}\u{E004}\u{E000}B"),
            vec![
                AutomationKey::Text("A".into()),
                AutomationKey::Backspace,
                AutomationKey::ShiftTab,
                AutomationKey::Text("B".into()),
            ]
        );
    }
}
