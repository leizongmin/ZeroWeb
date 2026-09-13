//! CDP remoteObject / objectId 线格式与 AutomationValue 转换助手 + 截图 PNG 编解码。

use serde_json::Value;
use zero_protocol::message::AutomationValue;
use zero_render_foundation::surface::FrameBuffer;

/// R1601：把 RGBA8 FrameBuffer 编码为 base64 PNG 字符串，供 `captureScreenshot`
/// 协议响应携带像素数据（headless 截图用于像素对比，DC-13 line 315）。
pub(super) fn framebuffer_to_png_base64(fb: &FrameBuffer) -> String {
    use base64::Engine;
    use png::{BitDepth, ColorType, Encoder};
    let mut png_buf: Vec<u8> = Vec::new();
    {
        let mut encoder = Encoder::new(&mut png_buf, fb.width, fb.height);
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);
        let mut writer = encoder.write_header().expect("PNG header encode");
        writer.write_image_data(&fb.data).expect("PNG image data encode");
    }
    base64::engine::general_purpose::STANDARD.encode(&png_buf)
}

/// 解码 PNG 字节为 (width, height, RGBA8 像素)（clip 裁剪前置步骤；Rgb 补 alpha）。
pub(super) fn decode_png_to_rgba(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), String> {
    use png::ColorType;
    let decoder = png::Decoder::new(bytes);
    let mut reader = decoder.read_info().map_err(|e| format!("png read_info: {e}"))?;
    let info = reader.info().clone();
    let mut buf = vec![0u8; reader.output_buffer_size()];
    reader.next_frame(&mut buf).map_err(|e| format!("png decode: {e}"))?;
    let rgba = if info.color_type == ColorType::Rgb {
        buf.chunks(3).flat_map(|px| [px[0], px[1], px[2], 255u8]).collect()
    } else {
        buf
    };
    Ok((info.width, info.height, rgba))
}

/// CDP remoteObject `objectId` 的 ZeroWeb 线格式：`zw:<handle>`（对客户端不透明；
/// Playwright 仅按不透明串回传）。注册表本体在 renderer 侧，headless 无状态映射。
pub(in crate::headless) fn object_id_string(handle: u64) -> String {
    format!("zw:{handle}")
}

/// 解析本服务发出的 objectId 线格式；非本服务格式 → None（调用方 -32602）。
pub(in crate::headless) fn parse_object_id(object_id: &str) -> Option<u64> {
    object_id.strip_prefix("zw:")?.parse().ok()
}

/// remoteObject 形状（注册表句柄 → `{type:"object", objectId}`；DOM 节点加
/// `subtype:"node"`——PW 据此生成 ElementHandle，locator/adopt 管线分叉点）。
pub(super) fn handle_to_remote_object(handle: zero_protocol::message::AutomationHandleRef) -> Value {
    let mut obj = serde_json::json!({
        "type": "object",
        "objectId": object_id_string(handle.id),
    });
    if handle.node {
        obj["subtype"] = serde_json::json!("node");
    }
    obj
}

/// AutomationValue → CDP remoteObject 形状；句柄变体产出 `objectId`（不落 value）。
pub(super) fn automation_value_to_remote_object(value: &AutomationValue) -> Value {
    match value {
        AutomationValue::Handle(handle) => handle_to_remote_object(*handle),
        AutomationValue::Null => serde_json::json!({ "type": "object", "subtype": "null", "value": null }),
        AutomationValue::Bool(v) => serde_json::json!({ "type": "boolean", "value": v }),
        AutomationValue::Number(v) => serde_json::json!({ "type": "number", "value": v }),
        AutomationValue::String(v) => serde_json::json!({ "type": "string", "value": v }),
        AutomationValue::Array(items) => serde_json::json!({
            "type": "object",
            "subtype": "array",
            "value": items.iter().map(automation_value_to_remote_object_value).collect::<Vec<_>>(),
        }),
        AutomationValue::Object(entries) => serde_json::json!({
            "type": "object",
            "value": entries
                .iter()
                .map(|(k, v)| (k.clone(), automation_value_to_remote_object_value(v)))
                .collect::<serde_json::Map<String, Value>>(),
        }),
    }
}

/// 嵌套值走纯 JSON（remoteObject 嵌套层不重复包 type/value 外壳，Chromium returnByValue 同；
/// PW 的 value 线格式依赖嵌套数组/对象原样保真——包装致 `{o:[...]}` 变非迭代对象）。
pub(super) fn automation_value_to_remote_object_value(value: &AutomationValue) -> Value {
    match value {
        AutomationValue::Null => Value::Null,
        AutomationValue::Bool(v) => serde_json::json!(v),
        AutomationValue::Number(v) => serde_json::json!(v),
        AutomationValue::String(v) => serde_json::json!(v),
        AutomationValue::Array(items) => Value::Array(
            items
                .iter()
                .map(automation_value_to_remote_object_value)
                .collect::<Vec<_>>(),
        ),
        AutomationValue::Object(entries) => Value::Object(
            entries
                .iter()
                .map(|(k, v)| (k.clone(), automation_value_to_remote_object_value(v)))
                .collect::<serde_json::Map<String, Value>>(),
        ),
        // 句柄仅在 remoteObject 顶层有 `objectId` 形态；嵌套句柄无纯 JSON 形态，防御性降级。
        AutomationValue::Handle(_) => Value::Null,
    }
}

/// CDP `arguments[]` 单个实参 → AutomationValue：`{value}` 走纯 JSON，
/// `{objectId}` 还原为本服务签发的句柄引用（其他键忽略，Chromium 同宽容语义）。
pub(super) fn cdp_call_argument_to_automation_value(argument: &Value) -> AutomationValue {
    if let Some(object_id) = argument.get("objectId").and_then(|v| v.as_str()) {
        return match parse_object_id(object_id) {
            // 实参回传的句柄 node 性不可知（不影响还原——页面侧只用 id）。
            Some(id) => AutomationValue::Handle(zero_protocol::message::AutomationHandleRef { id, node: false }),
            None => AutomationValue::Null,
        };
    }
    match argument.get("value") {
        Some(value) => json_to_automation_value(value),
        None => AutomationValue::Null,
    }
}

/// 纯 JSON → AutomationValue（CDP `{value}` 实参的嵌套形态）。
pub(super) fn json_to_automation_value(value: &Value) -> AutomationValue {
    match value {
        Value::Null => AutomationValue::Null,
        Value::Bool(v) => AutomationValue::Bool(*v),
        Value::Number(v) => AutomationValue::Number(v.as_f64().unwrap_or_default()),
        Value::String(v) => AutomationValue::String(v.clone()),
        Value::Array(items) => AutomationValue::Array(items.iter().map(json_to_automation_value).collect()),
        Value::Object(entries) => AutomationValue::Object(
            entries
                .iter()
                .map(|(k, v)| (k.clone(), json_to_automation_value(v)))
                .collect(),
        ),
    }
}

/// 脚本异常的 CDP 形状：`{result:{type:"undefined"}, exceptionDetails:{text}}`。
pub(super) fn exception_details_response(message: String) -> Value {
    serde_json::json!({
        "result": { "type": "undefined" },
        "exceptionDetails": {
            "text": message,
        },
    })
}
