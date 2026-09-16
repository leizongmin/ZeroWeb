//! CDP Target 域 — autoAttach/生命周期/会话附接。

use serde_json::Value;
use zero_browser_shell::TabId;

use crate::headless::HeadlessServer;

use crate::headless::protocol::{ProtocolError, ServerEvent};
use crate::headless::session::HeadlessSession;

impl HeadlessServer {
    /// Target.setAutoAttach — flatten 模式：对现存每个 page target 分配 sessionId、
    /// 登记注册表并逐个发 `Target.attachedToTarget`；此后新建 target（createTarget）
    /// 自动附接。Playwright 连接的第一条 Target 命令。
    pub(super) fn cmd_target_set_auto_attach(
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
    pub(super) fn cmd_target_create_target(
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
    pub(super) fn cmd_target_close_target(
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
    /// 事件盖发起会话的 sessionId（命令发起的应答送达发起方——flat 路由按 sessionId
    /// 投递；closeTarget 广播路径保持无盖章，见 cmd_target_close_target）。
    pub(super) fn cmd_target_detach_from_target(
        &self,
        params: &Value,
        cdp_session: Option<&str>,
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
                session_id: cdp_session.map(str::to_string),
            });
        }
        Ok(serde_json::json!({}))
    }

    /// Target.attachToBrowserTarget — PW 附加 CDP 会话入口（`browser.newBrowserCDPSession`
    /// / `context.newCDPSession(page)` 均经此建会话）。flat 模型：分配 sessionId 并登记
    /// 到活跃 target（attached_sessions 注册表——未登记 sessionId 会被 `-32001` 拒绝），
    /// 返回 `{sessionId}`；Chromium 形状一致。
    pub(super) fn cmd_target_attach_to_browser_target(
        &self,
        session: &mut HeadlessSession,
    ) -> Result<Value, ProtocolError> {
        let sid = self.next_cdp_session();
        let target_id = session
            .shell
            .active_tab_id()
            .map(|id| format!("zeroweb-tab-{}", id.0))
            .unwrap_or_default();
        self.attach_session(&sid, &target_id);
        Ok(serde_json::json!({ "sessionId": sid }))
    }

    /// Target.attachToTarget — 附接会话到指定 target，返回 `{sessionId}`（无事件——
    /// PW createSession 按响应配对）。target 缺参 `-32602`、未知 `-32000`。
    pub(super) fn cmd_target_attach_to_target(
        &self,
        session: &mut HeadlessSession,
        params: Value,
    ) -> Result<Value, ProtocolError> {
        let target_id = params
            .get("targetId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'targetId' parameter".into(),
            })?;
        let tab_id = Self::parse_target_id(target_id).ok_or_else(|| ProtocolError {
            code: -32000,
            message: format!("Unknown targetId '{target_id}'"),
        })?;
        if session.shell.tab(tab_id).is_none() {
            return Err(ProtocolError {
                code: -32000,
                message: format!("Unknown targetId '{target_id}'"),
            });
        }
        let sid = self.next_cdp_session();
        self.attach_session(&sid, target_id);
        Ok(serde_json::json!({ "sessionId": sid }))
    }

    /// Target.getTargetInfo — 无 targetId = 浏览器级 target；带 targetId = 查标签页。
    pub(super) fn cmd_target_get_target_info(
        &self,
        session: &mut HeadlessSession,
        params: Value,
    ) -> Result<Value, ProtocolError> {
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
        // 无 targetId：浏览器级连接（Playwright connectOverCDP 握手查询）→ browser
        // 占位；page-direct 连接（DevTools frontend per-page ws，devtools goal M1-S1）→
        // 活跃页 targetInfo——frontend 按它分类连接形态，误报 browser 会装载
        // ScreencastView（浏览器调试 UI）并因缺 screencast 域崩溃、面板全部停摆。
        if self.page_direct_connection() {
            if let Some(tab) = session.shell.tabs().next() {
                return Ok(serde_json::json!({ "targetInfo": Self::target_info_json(tab) }));
            }
        }
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
    pub(super) fn cmd_target_get_targets(&self, session: &mut HeadlessSession) -> Result<Value, ProtocolError> {
        let infos: Vec<Value> = session.shell.tabs().map(Self::target_info_json).collect();
        Ok(serde_json::json!({ "targetInfos": infos }))
    }

    /// 解析 targetId（`zeroweb-tab-<n>`，与 HTTP 发现枚举一致）→ TabId。
    pub(super) fn parse_target_id(target_id: &str) -> Option<TabId> {
        target_id
            .strip_prefix("zeroweb-tab-")
            .and_then(|n| n.parse::<u64>().ok())
            .map(TabId)
    }

    /// TabInfo 的 CDP targetInfo 形状（Target 域与 attachedToTarget 事件共用）。
    pub(super) fn target_info_json(tab: &zero_browser_shell::Tab) -> Value {
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
}
