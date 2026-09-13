//! CDP Runtime 域 — evaluate / callFunctionOn / releaseObject(Group)。

use serde_json::Value;
use zero_protocol::message::AutomationOperation;
use zero_protocol::message::AutomationResult;

use crate::headless::HeadlessServer;

use super::remote_object::{
    automation_value_to_remote_object, cdp_call_argument_to_automation_value, exception_details_response,
    parse_object_id,
};
use crate::headless::protocol::ProtocolError;
use crate::headless::session::HeadlessSession;

impl HeadlessServer {
    /// Runtime.evaluate — remoteObject 形状返回（BiDi `script.evaluate` 的扁平字符串
    /// 语义不受影响，走独立实现）。`returnByValue:false`（Playwright 句柄获取路径）
    /// 走 objectId 桥：对象结果保留在 renderer 注册表，返回 `objectId` 引用。
    pub(super) fn cmd_runtime_evaluate(
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
        let return_by_value = params.get("returnByValue").and_then(|v| v.as_bool()).unwrap_or(false);
        let group = params.get("objectGroup").and_then(|v| v.as_str()).map(str::to_string);
        // 双分支统一走 objectId 桥：表达式语义（W3C ExecuteScript 是函数体语义，
        // 不满足 CDP evaluate 的表达式形态——裸表达式读数恒 undefined，实测）。
        // contextId 忽略：单引擎共享一个页面脚本上下文（无 world 隔离，矩阵已记账）。
        match session.automation_request(AutomationOperation::EvaluateRetaining {
            script: expression.to_string(),
            group,
            return_by_value,
        }) {
            Ok(AutomationResult::Value(value)) => Ok(serde_json::json!({
                "result": automation_value_to_remote_object(&value),
            })),
            Ok(_) => Ok(serde_json::json!({ "result": { "type": "undefined" } })),
            Err(e) => Ok(exception_details_response(e)),
        }
    }

    /// Runtime.callFunctionOn — 双路径：
    /// - `objectId`（Playwright utilityScript 主路径）：以保留句柄对象为 `this` 调用
    ///   函数，`arguments[].objectId` 还原为保留对象，`awaitPromise` 落定后返回；
    /// - 无 objectId（既有 value 路径）：函数体包装为表达式按值执行。
    pub(super) fn cmd_runtime_call_function_on(
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
        let return_by_value = params.get("returnByValue").and_then(|v| v.as_bool()).unwrap_or(false);
        let await_promise = params.get("awaitPromise").and_then(|v| v.as_bool()).unwrap_or(false);
        let group = params.get("objectGroup").and_then(|v| v.as_str()).map(str::to_string);
        if let Some(object_id) = params.get("objectId").and_then(|v| v.as_str()) {
            let handle = parse_object_id(object_id).ok_or_else(|| ProtocolError {
                code: -32602,
                message: format!("Invalid objectId '{object_id}'"),
            })?;
            let arguments = params
                .get("arguments")
                .and_then(|v| v.as_array())
                .map(|args| args.iter().map(cdp_call_argument_to_automation_value).collect())
                .unwrap_or_default();
            return match session.automation_request(AutomationOperation::CallFunctionOnHandle {
                handle,
                function_declaration: function_declaration.to_string(),
                arguments,
                return_by_value,
                await_promise,
                group,
            }) {
                Ok(AutomationResult::Value(value)) => Ok(serde_json::json!({
                    "result": automation_value_to_remote_object(&value),
                })),
                Ok(_) => Ok(serde_json::json!({ "result": { "type": "undefined" } })),
                Err(e) => Ok(exception_details_response(e)),
            };
        }
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
            Err(e) => Ok(exception_details_response(e)),
        }
    }

    /// Runtime.releaseObject — 释放 renderer 保留句柄（objectId 桥配对命令）。
    pub(super) fn cmd_runtime_release_object(
        &self,
        session: &mut HeadlessSession,
        params: Value,
    ) -> Result<Value, ProtocolError> {
        let object_id = params
            .get("objectId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'objectId' parameter".into(),
            })?;
        let handle = parse_object_id(object_id).ok_or_else(|| ProtocolError {
            code: -32602,
            message: format!("Invalid objectId '{object_id}'"),
        })?;
        session
            .automation_request(AutomationOperation::ReleaseHandle { handle })
            .map_err(|e| ProtocolError {
                code: -32000,
                message: e,
            })?;
        Ok(serde_json::json!({}))
    }

    /// Runtime.releaseObjectGroup — 整组释放 renderer 保留句柄。
    pub(super) fn cmd_runtime_release_object_group(
        &self,
        session: &mut HeadlessSession,
        params: Value,
    ) -> Result<Value, ProtocolError> {
        let group = params
            .get("objectGroup")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'objectGroup' parameter".into(),
            })?;
        session
            .automation_request(AutomationOperation::ReleaseObjectGroup {
                group: group.to_string(),
            })
            .map_err(|e| ProtocolError {
                code: -32000,
                message: e,
            })?;
        Ok(serde_json::json!({}))
    }
}
