//! CDP DOM 域 — 句柄桥几何/描述/解析面。

use serde_json::Value;
use zero_protocol::message::AutomationOperation;
use zero_protocol::message::AutomationResult;
use zero_protocol::message::AutomationValue;

use crate::headless::HeadlessServer;

use super::remote_object::{automation_value_to_remote_object_value, object_id_string, parse_object_id};
use crate::headless::protocol::ProtocolError;
use crate::headless::session::HeadlessSession;

impl HeadlessServer {
    /// DOM.scrollIntoViewIfNeeded — 元素滚动入视口（shim scrollIntoView 面；
    /// 无布局对象 → PW 可识别的 notvisible 语义错误）。
    pub(super) fn cmd_dom_scroll_into_view_if_needed(
        &self,
        session: &mut HeadlessSession,
        params: Value,
    ) -> Result<Value, ProtocolError> {
        let result = self.dom_element_probe(
            session,
            &params,
            "(e) => { if (!e || typeof e.getBoundingClientRect !== 'function') return 'no-layout';\
             try { if (typeof e.scrollIntoViewIfNeeded === 'function') e.scrollIntoViewIfNeeded();\
             else if (typeof e.scrollIntoView === 'function') e.scrollIntoView(); } catch (_e) {} return true; }",
        )?;
        if result == AutomationValue::String("no-layout".into()) {
            return Err(ProtocolError {
                code: -32000,
                message: "Node does not have a layout object".into(),
            });
        }
        Ok(serde_json::json!({}))
    }

    /// DOM.getContentQuads — 元素内容四边形（视口坐标 flat 8 数；PW 据此算点击点）。
    pub(super) fn cmd_dom_get_content_quads(
        &self,
        session: &mut HeadlessSession,
        params: Value,
    ) -> Result<Value, ProtocolError> {
        let result = self.dom_element_probe(
            session,
            &params,
            "(e) => { if (!e || typeof e.getBoundingClientRect !== 'function') return null;\
             var r = e.getBoundingClientRect();\
             if (r.width === 0 && r.height === 0) return null;\
             return [r.x, r.y, r.x + r.width, r.y, r.x + r.width, r.y + r.height, r.x, r.y + r.height]; }",
        )?;
        let quad = automation_value_to_remote_object_value(&result);
        if !quad.is_array() {
            return Err(ProtocolError {
                code: -32000,
                message: "Node does not have a layout object".into(),
            });
        }
        Ok(serde_json::json!({ "quads": [quad] }))
    }

    /// DOM.getBoxModel — 盒模型（content/padding/border/margin 四 quad + 尺寸；headless
    /// 单一面板：全部同 content quad）。
    pub(super) fn cmd_dom_get_box_model(
        &self,
        session: &mut HeadlessSession,
        params: Value,
    ) -> Result<Value, ProtocolError> {
        let result = self.dom_element_probe(
            session,
            &params,
            "(e) => { if (!e || typeof e.getBoundingClientRect !== 'function') return null;\
             var r = e.getBoundingClientRect();\
             if (r.width === 0 && r.height === 0) return null;\
             return [r.x, r.y, r.width, r.height]; }",
        )?;
        let AutomationValue::Array(rect) = &result else {
            return Err(ProtocolError {
                code: -32000,
                message: "Node does not have a layout object".into(),
            });
        };
        let nums = |i: usize| {
            rect.get(i).and_then(|v| match v {
                AutomationValue::Number(n) => Some(*n),
                _ => None,
            })
        };
        let (x, y, w, h) = match (nums(0), nums(1), nums(2), nums(3)) {
            (Some(x), Some(y), Some(w), Some(h)) => (x, y, w, h),
            _ => {
                return Err(ProtocolError {
                    code: -32000,
                    message: "Node does not have a layout object".into(),
                });
            }
        };
        let quad = [x, y, x + w, y, x + w, y + h, x, y + h];
        let model = serde_json::json!({
            "content": quad, "padding": quad, "border": quad, "margin": quad,
            "width": w, "height": h,
        });
        Ok(serde_json::json!({ "model": model }))
    }

    /// DOM.describeNode — 节点描述 + `backendNodeId`（= 句柄 id；PW adopt 流程以它
    /// 经 `DOM.resolveNode` 把 utility world 句柄跨上下文重析为 objectId）。
    pub(super) fn cmd_dom_describe_node(
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
        let result = self.dom_element_probe(
            session,
            &params,
            "(e) => ({ nodeName: e.nodeName || '', tagName: e.tagName || '',\
             nodeType: e.nodeType || 1, childElementCount: e.childElementCount || 0,\
             attributes: e.attributes ? Array.from(e.attributes).map(function (a) { return [a.name, a.value]; }) : [] })",
        )?;
        let mut node = automation_value_to_remote_object_value(&result);
        if let Some(map) = node.as_object_mut() {
            map.insert("backendNodeId".into(), serde_json::json!(handle));
            map.insert("nodeId".into(), serde_json::json!(handle));
        }
        Ok(serde_json::json!({ "node": node }))
    }

    /// DOM.resolveNode — backendNodeId → objectId（NodeId↔句柄桥的另一半）。
    /// backendNodeId 即原句柄 id：对注册表条目**重新保留**为新句柄返回（原句柄可能已
    /// 被 `Runtime.releaseObject` 释放——PW adopt 后 dispose 原句柄、只保留析出的新句柄；
    /// subtype:"node" 使 PW 侧生成 ElementHandle 而非 JSHandle）。
    pub(super) fn cmd_dom_resolve_node(
        &self,
        session: &mut HeadlessSession,
        params: Value,
    ) -> Result<Value, ProtocolError> {
        let backend_node_id = params
            .get("backendNodeId")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'backendNodeId' parameter".into(),
            })?;
        let result = session.automation_request(AutomationOperation::CallFunctionOnHandle {
            handle: backend_node_id,
            function_declaration: "(e) => e".into(),
            arguments: vec![AutomationValue::Handle(zero_protocol::message::AutomationHandleRef {
                id: backend_node_id,
                node: true,
            })],
            return_by_value: false,
            await_promise: false,
            group: None,
        });
        match result {
            Ok(AutomationResult::Value(AutomationValue::Handle(new_handle))) => Ok(serde_json::json!({
                "object": {
                    "type": "object",
                    "subtype": "node",
                    "objectId": object_id_string(new_handle.id),
                    "description": "",
                },
            })),
            Ok(_) => Err(ProtocolError {
                code: -32000,
                message: "Node is detached from document".into(),
            }),
            Err(e) => Err(ProtocolError {
                code: -32000,
                message: format!("Node is detached from document: {e}"),
            }),
        }
    }

    /// DOM.getDocument — 全树序列化（DevTools frontend Elements 面板唯一数据源，
    /// devtools goal M1-S3a；cdp-protocol 矩阵面的 Playwright 流不用此方法）。
    ///
    /// 实现：renderer 页面上下文 JS 探测（EvaluateRetaining，return_by_value）把 shim
    /// DOM 拍平为紧凑 JSON，headless 侧转换为 CDP Node 形状并顺序分配 `nodeId`。
    /// nodeId/backendNodeId 为本次调用内的独立空间（与 objectId 句柄注册表无关联——
    /// frontend 以 nodeId 引用节点时暂无 DOM.getNode 面，S3b 随样式域补）。
    ///
    /// https://chromedevtools.github.io/devtools-protocol/tot/DOM/#method-getDocument
    pub(super) fn cmd_dom_get_document(
        &self,
        session: &mut HeadlessSession,
        params: &Value,
    ) -> Result<Value, ProtocolError> {
        let depth = params.get("depth").and_then(|v| v.as_i64()).unwrap_or(-1);
        let script = format!(
            "(function() {{\n function ser(n, d) {{\n  var t = n.nodeType;\n  var o = {{t: t, n: n.nodeName || ''}};\n  if (t === 3 || t === 8) o.v = n.nodeValue || '';\n  if (t === 1) {{\n   var a = [];\n   try {{ var at = n.attributes; for (var i = 0; i < at.length; i++) {{ a.push(at[i].name, at[i].value); }} }} catch (e) {{}}\n   o.a = a;\n  }}\n  var c = [];\n  var kids = n.childNodes || [];\n  for (var j = 0; j < kids.length; j++) {{\n   var k = kids[j];\n   var kt = k.nodeType;\n   if (kt !== 1 && kt !== 3 && kt !== 8 && kt !== 10) continue;\n   if ({depth} >= 0 && d >= {depth}) {{ c.push({{t: kt, n: k.nodeName || ''}}); continue; }}\n   c.push(ser(k, d + 1));\n  }}\n  o.c = c;\n  o.cc = c.length;\n  if (t === 9) o.u = n.documentURI || '';\n  return o;\n }}\n return JSON.stringify(ser(document, 0));\n}})()"
        );
        let json = match session.automation_request(AutomationOperation::EvaluateRetaining {
            script,
            group: None,
            return_by_value: true,
        }) {
            Ok(AutomationResult::Value(AutomationValue::String(s))) => s,
            Ok(other) => {
                return Err(ProtocolError {
                    code: -32000,
                    message: format!("DOM serialization returned non-string: {other:?}"),
                });
            }
            Err(e) => {
                return Err(ProtocolError {
                    code: -32000,
                    message: format!("DOM serialization failed: {e}"),
                });
            }
        };
        let raw: Value = serde_json::from_str(&json).map_err(|e| ProtocolError {
            code: -32000,
            message: format!("DOM probe JSON invalid: {e}"),
        })?;
        let mut next_id = 1u64;
        let root = convert_cdp_node(&raw, &mut next_id);
        Ok(serde_json::json!({ "root": root }))
    }

    /// DOM 域 objectId 面（M4+）：经 objectId 桥对保留元素求值——geometry/身份探测
    /// 复用既有 CallFunctionOnHandle 原语，页面侧 rect 来自 shim `getBoundingClientRect`
    ///（RectBridge 真实布局矩形）。
    ///
    /// https://chromedevtools.github.io/devtools-protocol/tot/DOM/
    pub(super) fn dom_element_probe(
        &self,
        session: &mut HeadlessSession,
        params: &Value,
        probe: &str,
    ) -> Result<AutomationValue, ProtocolError> {
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
        match session.automation_request(AutomationOperation::CallFunctionOnHandle {
            handle,
            function_declaration: probe.to_string(),
            arguments: vec![AutomationValue::Handle(zero_protocol::message::AutomationHandleRef {
                id: handle,
                node: true,
            })],
            return_by_value: true,
            await_promise: false,
            group: None,
        }) {
            Ok(AutomationResult::Value(value)) => Ok(value),
            Ok(_) => Ok(AutomationValue::Null),
            // 句柄失效 = 文档已换代/节点已脱离（PW 识别该文案 → error:notconnected）。
            Err(e) => Err(ProtocolError {
                code: -32000,
                message: format!("Node is detached from document: {e}"),
            }),
        }
    }
}

/// 探测 JSON → CDP Node 形状递归转换（`nodeId` 顺序分配）。
///
/// 探测节点字段：`t`=nodeType、`n`=nodeName、`v`=nodeValue、`a`=attributes flat 数组、
/// `c`=children、`cc`=childNodeCount、`u`=documentURL。`c` 缺省（depth 截断的桩节点
/// 在探测侧即无 `c`/`cc`）→ 不输出 `children`，仅 `childNodeCount: 0` 之外的场景由
/// 探测侧 `cc` 兜底。
pub(crate) fn convert_cdp_node(raw: &Value, next_id: &mut u64) -> Value {
    let id = *next_id;
    *next_id += 1;
    let node_type = raw.get("t").and_then(|v| v.as_i64()).unwrap_or(1);
    let node_name = raw.get("n").and_then(|v| v.as_str()).unwrap_or("");
    let mut node = serde_json::Map::new();
    node.insert("nodeId".into(), serde_json::json!(id));
    node.insert("backendNodeId".into(), serde_json::json!(id));
    node.insert("nodeType".into(), serde_json::json!(node_type));
    match node_type {
        // 文档节点
        9 => {
            node.insert("nodeName".into(), serde_json::json!("#document"));
            node.insert("nodeValue".into(), serde_json::json!(""));
            let url = raw.get("u").and_then(|v| v.as_str()).unwrap_or("");
            node.insert("documentURL".into(), serde_json::json!(url));
            node.insert("baseURL".into(), serde_json::json!(url));
            node.insert("xmlVersion".into(), serde_json::json!(""));
        }
        // 文档类型节点（nodeName = doctype 名）
        10 => {
            node.insert("nodeName".into(), serde_json::json!(node_name));
            node.insert("publicId".into(), serde_json::json!(""));
            node.insert("systemId".into(), serde_json::json!(""));
        }
        // 文本 / 注释
        3 | 8 => {
            node.insert("nodeName".into(), serde_json::json!(node_name));
            node.insert(
                "nodeValue".into(),
                serde_json::json!(raw.get("v").cloned().unwrap_or_default()),
            );
        }
        // 元素（含 depth 截断桩：无 `a`/`c`）
        _ => {
            node.insert("nodeName".into(), serde_json::json!(node_name));
            node.insert("nodeValue".into(), serde_json::json!(""));
            if let Some(attrs) = raw.get("a") {
                node.insert("attributes".into(), attrs.clone());
            }
        }
    }
    let children: Vec<Value> = raw
        .get("c")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().map(|child| convert_cdp_node(child, next_id)).collect())
        .unwrap_or_default();
    let child_count = raw.get("cc").and_then(|v| v.as_i64()).unwrap_or(children.len() as i64);
    node.insert("childNodeCount".into(), serde_json::json!(child_count));
    if !children.is_empty() {
        node.insert("children".into(), serde_json::json!(children));
    }
    Value::Object(node)
}
