//! CDP CSS 域 — DevTools frontend 样式侧栏数据面（devtools goal M1-S3b）。
//!
//! 消费侧实现（零 engine/style-system 改动）：按 `DOM.getDocument` 注册的
//! nodeId → shim `__zwSelector` 映射解析节点，经 renderer 页面上下文 JS 探测调用
//! 既有 host 回调 `__zw_get_computed_style(sel, prop)`（style-system 计算值桥）。
//! 规则级内省（matched rules）暂无 IPC 消费面——`matchedCSSRules` 返空数组记账，
//! Styles 侧栏现阶段呈现 inline style，Computed 侧栏呈现计算样式表。

use serde_json::Value;
use zero_protocol::message::AutomationOperation;
use zero_protocol::message::AutomationResult;
use zero_protocol::message::AutomationValue;

use crate::headless::HeadlessServer;

use crate::headless::protocol::ProtocolError;
use crate::headless::session::HeadlessSession;

/// 计算样式查询属性清单（Computed 侧栏）。host 侧 `serialize_computed_property`
/// 现覆盖 display/position/visibility/opacity + 颜色族——未覆盖属性回 ''，出参滤除，
/// 清单可随 host 覆盖面扩属无损。
const COMPUTED_PROBE_PROPERTIES: &[&str] = &[
    "display",
    "position",
    "visibility",
    "opacity",
    "color",
    "background-color",
    "border-top-color",
    "border-right-color",
    "border-bottom-color",
    "border-left-color",
    "outline-color",
    "z-index",
    "font-size",
    "font-family",
    "margin-top",
];

impl HeadlessServer {
    /// CSS.getComputedStyle — 计算样式名值对（Computed 侧栏数据源）。
    ///
    /// https://chromedevtools.github.io/devtools-protocol/tot/CSS/#method-getComputedStyle
    pub(super) fn cmd_css_get_computed_style(
        &self,
        session: &mut HeadlessSession,
        params: &Value,
    ) -> Result<Value, ProtocolError> {
        let node_id = params
            .get("nodeId")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'nodeId' parameter".into(),
            })?;
        let selector = self.selector_for_devtools_node(node_id)?;
        let script = format!(
            "(function() {{\n if (typeof __zw_get_computed_style !== 'function') return '[]';\n var props = {props};\n var sel = {sel};\n var out = [];\n for (var i = 0; i < props.length; i++) {{\n  var v = '';\n  try {{ v = __zw_get_computed_style(sel, props[i]) || ''; }} catch (e) {{}}\n  if (v !== '') out.push({{name: props[i], value: v}});\n }}\n return JSON.stringify(out);\n}})()",
            props = serde_json::json!(COMPUTED_PROBE_PROPERTIES),
            sel = serde_json::json!(selector),
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
                    message: format!("Computed style probe returned non-string: {other:?}"),
                });
            }
            Err(e) => {
                return Err(ProtocolError {
                    code: -32000,
                    message: format!("Computed style probe failed: {e}"),
                });
            }
        };
        let computed: Value = serde_json::from_str(&json).map_err(|e| ProtocolError {
            code: -32000,
            message: format!("Computed style probe JSON invalid: {e}"),
        })?;
        Ok(serde_json::json!({ "computedStyle": computed }))
    }

    /// CSS.getMatchedStylesForNode — Styles 侧栏。规则级内省无 IPC 消费面（记账）：
    /// `matchedCSSRules` 返空数组 + `inlineStyle`（元素 `style` 属性解析）。
    ///
    /// https://chromedevtools.github.io/devtools-protocol/tot/CSS/#method-getMatchedStylesForNode
    pub(super) fn cmd_css_get_matched_styles(
        &self,
        session: &mut HeadlessSession,
        params: &Value,
    ) -> Result<Value, ProtocolError> {
        let node_id = params
            .get("nodeId")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'nodeId' parameter".into(),
            })?;
        let selector = self.selector_for_devtools_node(node_id)?;
        let script = format!(
            "(function() {{\n try {{\n  var el = document.querySelector({sel});\n  if (!el || !el.style) return '';\n  return el.style.cssText || '';\n }} catch (e) {{ return ''; }}\n}})()",
            sel = serde_json::json!(selector),
        );
        let css_text = match session.automation_request(AutomationOperation::EvaluateRetaining {
            script,
            group: None,
            return_by_value: true,
        }) {
            Ok(AutomationResult::Value(AutomationValue::String(s))) => s,
            Ok(_) => String::new(),
            Err(_) => String::new(),
        };
        Ok(serde_json::json!({
            "inlineStyle": { "cssProperties": parse_inline_style_properties(&css_text), "shorthandEntries": [] },
            "matchedCSSRules": [],
            "pseudoElements": [],
            "inherited": [],
        }))
    }
}

/// inline `style` 文本 → CDP `cssProperties`（`name:value` 分号切分；注入防御：
/// 值按原文保留，不重组转义）。
fn parse_inline_style_properties(css_text: &str) -> Vec<Value> {
    css_text
        .split(';')
        .filter_map(|decl| {
            let decl = decl.trim();
            if decl.is_empty() {
                return None;
            }
            let (name, value) = decl.split_once(':')?;
            let name = name.trim();
            if name.is_empty() {
                return None;
            }
            Some(serde_json::json!({ "name": name, "value": value.trim() }))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_inline_style_declarations() {
        let props = parse_inline_style_properties("color: red; display : none ; ; opacity: 0.5");
        assert_eq!(props.len(), 3);
        assert_eq!(props[0]["name"], "color");
        assert_eq!(props[0]["value"], "red");
        assert_eq!(props[1]["name"], "display");
        assert_eq!(props[1]["value"], "none");
        assert_eq!(props[2]["value"], "0.5");
        assert!(parse_inline_style_properties("").is_empty());
        assert!(parse_inline_style_properties("no-colon-here").is_empty());
    }
}
