//! 发布构建中的空 worker 接口。
//!
//! `TabManager` 的进程内 worker 分支只保留给单元测试；产品构建始终由
//! `ProcessTabBackend` 创建 `zero-renderer`。此接口让共享 tab 管理代码不携带
//! WebView、脚本运行时或其依赖进入 `zero-browser`。

use zero_browser_shell::TabId;
use zero_engine::{MediaType, PrefersColorSchemeValue};
use zero_protocol::message::ImeEventParams;

pub enum TabWorkerCommand {
    SetJavascriptEnabled(bool),
    SetColorScheme(PrefersColorSchemeValue),
    SetMediaType(MediaType),
    Resize {
        width: u32,
        height: u32,
    },
    Navigate(String),
    NavigateRequest {
        url: String,
        method: String,
        body: Option<String>,
    },
    LoadHtml {
        html: String,
        css: Option<String>,
        url: Option<String>,
    },
    DispatchDomEvent {
        dispatch_id: u64,
        selector: String,
        event_type: String,
        key: Option<String>,
        code: Option<String>,
        shift: bool,
        selection: Option<(u32, u32)>,
    },
    ImeEvent {
        selector: Option<String>,
        params: ImeEventParams,
    },
    UserScroll {
        delta_x: f32,
        delta_y: f32,
    },
}

pub struct TabWorkerMessage;

pub struct TabWorkerHandle;

impl TabWorkerHandle {
    pub fn spawn(_tab_id: TabId, _viewport: (u32, u32), _color_scheme: PrefersColorSchemeValue) -> Self {
        unreachable!("production tabs must use zero-renderer")
    }

    pub fn send(&self, _command: TabWorkerCommand) {}

    pub fn try_recv(&self) -> Option<TabWorkerMessage> {
        None
    }

    pub fn shutdown(&mut self) {}
}
