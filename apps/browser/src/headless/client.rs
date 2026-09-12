//! 无头协议客户端测试基础设施（Phase 4）— 请求/响应/事件/截图/DOM 快照解析辅助。

// ── 协议客户端（Phase 4 自动化测试基础设施）──

/// 无头浏览器协议客户端，用于自动化测试。
///
/// 通过 WebSocket 连接到 HeadlessServer，发送命令并接收响应和事件。
#[cfg(test)]
pub struct HeadlessClient;

#[cfg(test)]
impl HeadlessClient {
    /// 解析服务端 JSON 响应，提取 result 字段。
    ///
    /// 如果响应包含 error，返回错误描述。
    pub fn parse_response(raw: &str) -> Result<serde_json::Value, String> {
        let v: serde_json::Value = serde_json::from_str(raw).map_err(|e| format!("JSON parse: {e}"))?;
        if let Some(error) = v.get("error") {
            let code = error.get("code").and_then(|c| c.as_i64()).unwrap_or(-1);
            let message = error.get("message").and_then(|m| m.as_str()).unwrap_or("unknown");
            return Err(format!("Error {code}: {message}"));
        }
        Ok(v.get("result").cloned().unwrap_or(serde_json::json!({})))
    }

    /// 构建协议请求 JSON 字符串。
    pub fn build_request(id: u64, method: &str, params: serde_json::Value) -> String {
        serde_json::json!({
            "id": id,
            "method": method,
            "params": params,
        })
        .to_string()
    }

    /// 解析事件通知 JSON，返回 (method, params) 对。
    pub fn parse_event(raw: &str) -> Result<(String, serde_json::Value), String> {
        let v: serde_json::Value = serde_json::from_str(raw).map_err(|e| format!("JSON parse: {e}"))?;
        let method = v.get("method").and_then(|m| m.as_str()).unwrap_or("").to_string();
        let params = v.get("params").cloned().unwrap_or(serde_json::json!({}));
        Ok((method, params))
    }

    /// 解析截图响应，返回 (width, height, pixel_count)。
    pub fn parse_screenshot(result: &serde_json::Value) -> Result<(u32, u32, usize), String> {
        let data = result.get("data").ok_or("Missing data field")?;
        let width = data.get("width").and_then(|v| v.as_u64()).ok_or("Missing width")? as u32;
        let height = data.get("height").and_then(|v| v.as_u64()).ok_or("Missing height")? as u32;
        let pixel_count = data
            .get("pixelCount")
            .and_then(|v| v.as_u64())
            .ok_or("Missing pixelCount")? as usize;
        Ok((width, height, pixel_count))
    }

    /// 解析 DOM 快照响应，返回各图元计数。
    pub fn parse_dom_snapshot(result: &serde_json::Value) -> DomSnapshotStats {
        let rp = result.get("renderPrimitives");
        DomSnapshotStats {
            fills: rp.and_then(|r| r.get("fills")).and_then(|v| v.as_u64()).unwrap_or(0) as usize,
            glyphs: rp.and_then(|r| r.get("glyphs")).and_then(|v| v.as_u64()).unwrap_or(0) as usize,
            gradients: rp
                .and_then(|r| r.get("gradients"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize,
            shadows: rp.and_then(|r| r.get("shadows")).and_then(|v| v.as_u64()).unwrap_or(0) as usize,
            images: rp.and_then(|r| r.get("images")).and_then(|v| v.as_u64()).unwrap_or(0) as usize,
        }
    }
}

/// DOM 快照统计信息。
#[cfg(test)]
#[derive(Debug, PartialEq)]
pub struct DomSnapshotStats {
    pub fills: usize,
    pub glyphs: usize,
    pub gradients: usize,
    pub shadows: usize,
    pub images: usize,
}

#[cfg(test)]
impl DomSnapshotStats {
    /// 总图元数。
    pub fn total(&self) -> usize {
        self.fills + self.glyphs + self.gradients + self.shadows + self.images
    }
}
