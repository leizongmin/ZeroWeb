//! Compositor 帧像素 POSIX 共享内存传输（RFC 4.3 切片 S1）。
//!
//! Linux 上经 `/dev/shm/zeroweb-cmp-*` 传递 front 缓冲，IPC 消息只带元数据，
//! 避免 PipeTransport bincode 内联巨大 `rgba` Vec。`ZW_COMPOSITOR_SHM=1` 启用；
//! 非 Linux 或未设置时由调用方回退内联 `rgba`。

use crate::ProtocolError;

#[cfg(target_os = "linux")]
const SHM_PREFIX: &str = "zeroweb-cmp-";

/// 是否启用 compositor POSIX shm 帧传输（Linux + `ZW_COMPOSITOR_SHM=1`）。
pub fn compositor_shm_enabled() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::env::var("ZW_COMPOSITOR_SHM").is_ok_and(|v| v == "1")
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = std::env::var("ZW_COMPOSITOR_SHM");
        false
    }
}

/// 是否启用 compositor 侧 scroll 烘焙（RFC 4.2-S2；`ZW_COMPOSITOR_SCROLL_TRANSFORM=1`）。
pub fn compositor_scroll_transform_enabled() -> bool {
    std::env::var("ZW_COMPOSITOR_SCROLL_TRANSFORM").is_ok_and(|v| v == "1")
}

/// 期望 RGBA 字节数；`width`/`height` 为 0 时允许 0。
pub fn expected_rgba_len(width: u32, height: u32) -> usize {
    (width as usize).saturating_mul(height as usize).saturating_mul(4)
}

/// compositor 侧：写入像素到 shm，返回不含前缀的 buffer 名。
pub fn publish_compositor_frame(surface_id: u64, frame_id: u64, pixels: &[u8]) -> Result<String, ProtocolError> {
    #[cfg(target_os = "linux")]
    {
        use std::time::{SystemTime, UNIX_EPOCH};

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let name = format!("{surface_id}-{frame_id}-{nonce}");
        let path = shm_path(&name);
        std::fs::write(&path, pixels).map_err(|e| ProtocolError::Channel(format!("compositor shm 写入失败: {e}")))?;
        Ok(name)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (surface_id, frame_id, pixels);
        Err(ProtocolError::Channel("compositor shm 仅 Linux 可用".into()))
    }
}

/// Browser 侧：从 shm 读取并删除文件。
pub fn consume_compositor_frame(name: &str, expected_len: usize) -> Result<Vec<u8>, ProtocolError> {
    #[cfg(target_os = "linux")]
    {
        let path = shm_path(name);
        let data = std::fs::read(&path).map_err(|e| ProtocolError::Channel(format!("compositor shm 读取失败: {e}")))?;
        let _ = std::fs::remove_file(&path);
        if data.len() != expected_len {
            return Err(ProtocolError::Channel(format!(
                "compositor shm 大小不匹配: 期望 {expected_len}, 实际 {}",
                data.len()
            )));
        }
        Ok(data)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (name, expected_len);
        Err(ProtocolError::Channel("compositor shm 仅 Linux 可用".into()))
    }
}

/// 从内联 `rgba` 或 shm 名解析完整像素。
pub fn resolve_compositor_frame_rgba(
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    shm_name: Option<String>,
) -> Result<Vec<u8>, ProtocolError> {
    let expected = expected_rgba_len(width, height);
    match shm_name {
        Some(name) if !name.is_empty() => consume_compositor_frame(&name, expected),
        Some(_) => Err(ProtocolError::Channel("compositor shm 名为空".into())),
        None => {
            if width == 0 && height == 0 && rgba.is_empty() {
                return Ok(rgba);
            }
            if rgba.len() != expected {
                return Err(ProtocolError::Channel(format!(
                    "compositor 帧像素大小不匹配: 期望 {expected}, 实际 {}",
                    rgba.len()
                )));
            }
            Ok(rgba)
        }
    }
}

#[cfg(target_os = "linux")]
fn shm_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("/dev/shm/{SHM_PREFIX}{name}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_rgba_len_matches_dimensions() {
        assert_eq!(expected_rgba_len(2, 2), 16);
        assert_eq!(expected_rgba_len(0, 0), 0);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn publish_and_consume_round_trip() {
        let pixels = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        let name = publish_compositor_frame(9, 1, &pixels).expect("publish");
        let read = consume_compositor_frame(&name, pixels.len()).expect("consume");
        assert_eq!(read, pixels);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn resolve_prefers_shm_over_inline_rgba() {
        let pixels = vec![255u8, 0, 0, 255];
        let name = publish_compositor_frame(1, 2, &pixels).expect("publish");
        let resolved = resolve_compositor_frame_rgba(1, 1, vec![0; 4], Some(name)).expect("resolve");
        assert_eq!(resolved, pixels);
    }
}
