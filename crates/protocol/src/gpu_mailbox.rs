//! Compositor GPU mailbox 二进制头（RFC 4.3-S4）。
//!
//! `gpu_image` 路径在 shm 前缀写入 magic/sync_token，Browser 侧做 fence 校验；
//! `ZW_COMPOSITOR_GPU_ZERO_COPY=1` 时用 mmap 读取 payload。

/// Mailbox magic `ZWCM`.
pub const GPU_MAILBOX_MAGIC: u32 = 0x5A57_434D;

/// 头部长度：magic + width + height + sync_token + payload_len + reserved（28 字节）。
pub const GPU_MAILBOX_HEADER_LEN: usize = 28;

/// 解析后的 mailbox 视图。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuMailboxHeader {
    /// 宽度（像素）。
    pub width: u32,
    /// 高度（像素）。
    pub height: u32,
    /// Fence/sync 序号（单调；须 ≥ 期望 frame_id）。
    pub sync_token: u64,
    /// RGBA payload 字节数。
    pub payload_len: u32,
}

/// 将 RGBA payload 与 fence 头写入连续字节。
pub fn encode_gpu_mailbox(pixels: &[u8], width: u32, height: u32, sync_token: u64) -> Vec<u8> {
    let mut buf = Vec::with_capacity(GPU_MAILBOX_HEADER_LEN + pixels.len());
    buf.extend_from_slice(&GPU_MAILBOX_MAGIC.to_le_bytes());
    buf.extend_from_slice(&width.to_le_bytes());
    buf.extend_from_slice(&height.to_le_bytes());
    buf.extend_from_slice(&sync_token.to_le_bytes());
    buf.extend_from_slice(&(pixels.len() as u32).to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(pixels);
    buf
}

/// 解析 mailbox 头；不拷贝 payload。
pub fn decode_gpu_mailbox(data: &[u8]) -> Result<(GpuMailboxHeader, usize), crate::ProtocolError> {
    if data.len() < GPU_MAILBOX_HEADER_LEN {
        return Err(crate::ProtocolError::Channel(format!(
            "gpu mailbox 过短: {} < {GPU_MAILBOX_HEADER_LEN}",
            data.len()
        )));
    }
    let magic = u32::from_le_bytes(data[0..4].try_into().expect("magic"));
    if magic != GPU_MAILBOX_MAGIC {
        return Err(crate::ProtocolError::Channel(format!(
            "gpu mailbox magic 不匹配: {magic:#010x}"
        )));
    }
    let width = u32::from_le_bytes(data[4..8].try_into().expect("width"));
    let height = u32::from_le_bytes(data[8..12].try_into().expect("height"));
    let sync_token = u64::from_le_bytes(data[12..20].try_into().expect("sync_token"));
    let payload_len = u32::from_le_bytes(data[20..24].try_into().expect("payload_len"));
    let total = GPU_MAILBOX_HEADER_LEN.saturating_add(payload_len as usize);
    if data.len() < total {
        return Err(crate::ProtocolError::Channel(format!(
            "gpu mailbox payload 不完整: 文件 {} 期望 {total}",
            data.len()
        )));
    }
    Ok((
        GpuMailboxHeader {
            width,
            height,
            sync_token,
            payload_len,
        },
        GPU_MAILBOX_HEADER_LEN,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_round_trip() {
        let pixels = vec![1u8, 2, 3, 4];
        let blob = encode_gpu_mailbox(&pixels, 1, 1, 42);
        let (header, off) = decode_gpu_mailbox(&blob).expect("decode");
        assert_eq!(header.sync_token, 42);
        assert_eq!(&blob[off..], pixels.as_slice());
    }
}
