//! Compositor 帧像素 POSIX 共享内存传输（RFC 4.3 切片 S1）。
//!
//! Linux 上经 `/dev/shm/zeroweb-cmp-*` 传递 front 缓冲，IPC 消息只带元数据，
//! 避免 PipeTransport bincode 内联巨大 `rgba` Vec。Linux 默认启用
//! （`ZW_COMPOSITOR_SHM=0` 禁用）；非 Linux 时由调用方回退内联 `rgba`。

use crate::ProtocolError;
use crate::gpu_mailbox::{GpuMailboxHeader, decode_gpu_mailbox, encode_gpu_mailbox};

#[cfg(target_os = "linux")]
const SHM_PREFIX: &str = "zeroweb-cmp-";

/// Linux 环境变量：未设置时默认开，`0`/`false` 禁用。
fn env_linux_default_on(name: &str) -> bool {
    #[cfg(target_os = "linux")]
    {
        zero_runtime_config::enabled_by_default(name)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = name;
        false
    }
}

/// 是否启用 compositor headless GPU 光栅（Linux 默认开；`ZW_COMPOSITOR_GPU=0` 禁用）。
pub fn compositor_gpu_enabled() -> bool {
    env_linux_default_on("ZW_COMPOSITOR_GPU")
}

/// 是否启用 compositor POSIX shm 帧传输（Linux 默认开；`ZW_COMPOSITOR_SHM=0` 禁用）。
pub fn compositor_shm_enabled() -> bool {
    env_linux_default_on("ZW_COMPOSITOR_SHM")
}

/// 是否启用 compositor 侧 scroll 视口重绘（默认开，`0` 禁用）。
///
/// 合成器保留最近的绘制快照，并在滚动时按当前视口重光栅化；这避免了只平移
/// 首屏 front buffer 时，滚入首屏外内容变为空白。依赖 ASYNC_SCROLL 同时开启
/// （Browser 仅在异步滚动开启时经 `CompositorSetScroll` 推送滚动值）。
pub fn compositor_scroll_transform_enabled() -> bool {
    zero_runtime_config::enabled_by_default("ZW_COMPOSITOR_SCROLL_TRANSFORM")
}

/// 是否启用 GPU shared image 元数据通道（RFC 4.3-S2；Linux 默认开；`ZW_COMPOSITOR_GPU_IMAGE=0` 禁用）。
///
/// mailbox 当前复用 POSIX shm 后端；S4 头含 sync_token fence；真 GPU 纹理为后续。
pub fn compositor_gpu_image_enabled() -> bool {
    env_linux_default_on("ZW_COMPOSITOR_GPU_IMAGE")
}

/// 是否启用 gpu_image mmap 零拷贝 consume（Linux 默认开；`ZW_COMPOSITOR_GPU_ZERO_COPY=0` 禁用）。
pub fn compositor_gpu_zero_copy_enabled() -> bool {
    env_linux_default_on("ZW_COMPOSITOR_GPU_ZERO_COPY")
}

/// 是否启用 GPU 纹理 dma-buf fd 导出（Linux 默认开；`ZW_COMPOSITOR_GPU_TEXTURE_EXPORT=0` 禁用）。
pub fn compositor_gpu_texture_export_enabled() -> bool {
    env_linux_default_on("ZW_COMPOSITOR_GPU_TEXTURE_EXPORT")
}

/// 是否启用 Browser GPU dma-buf 导入（Linux 默认开；`ZW_BROWSER_GPU_DMABUF_IMPORT=0` 禁用）。
pub fn browser_gpu_dmabuf_import_enabled() -> bool {
    env_linux_default_on("ZW_BROWSER_GPU_DMABUF_IMPORT")
}

/// 是否启用 compositor 拥有最终窗口 present（RFC 4.4-S4；默认开，`0` 禁用）。
pub fn compositor_owned_present_enabled() -> bool {
    zero_runtime_config::enabled_by_default("ZW_COMPOSITOR_OWNED_PRESENT")
}

/// 是否启用 compositor Viz present（page+UI 合成；默认开，`0` 禁用）。
pub fn compositor_present_enabled() -> bool {
    zero_runtime_config::enabled_by_default("ZW_COMPOSITOR_PRESENT")
}

/// 期望 RGBA 字节数；`width`/`height` 为 0 时允许 0。
pub fn expected_rgba_len(width: u32, height: u32) -> usize {
    (width as usize).saturating_mul(height as usize).saturating_mul(4)
}

/// compositor 侧：写入像素到 shm，返回不含前缀的 buffer 名。
pub fn publish_compositor_frame(surface_id: u64, frame_id: u64, pixels: &[u8]) -> Result<String, ProtocolError> {
    publish_compositor_blob(surface_id, frame_id, pixels)
}

/// compositor 侧：写入 gpu mailbox（头 + RGBA）。
pub fn publish_compositor_gpu_mailbox(
    surface_id: u64,
    frame_id: u64,
    pixels: &[u8],
    width: u32,
    height: u32,
    sync_token: u64,
) -> Result<String, ProtocolError> {
    let blob = encode_gpu_mailbox(pixels, width, height, sync_token);
    publish_compositor_blob(surface_id, frame_id, &blob)
}

fn publish_compositor_blob(surface_id: u64, frame_id: u64, blob: &[u8]) -> Result<String, ProtocolError> {
    #[cfg(target_os = "linux")]
    {
        use std::time::{SystemTime, UNIX_EPOCH};

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let name = format!("{surface_id}-{frame_id}-{nonce}");
        let path = shm_path(&name);
        std::fs::write(&path, blob).map_err(|e| ProtocolError::Channel(format!("compositor shm 写入失败: {e}")))?;
        Ok(name)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (surface_id, frame_id, blob);
        Err(ProtocolError::Channel("compositor shm 仅 Linux 可用".into()))
    }
}

/// Browser 侧：从 shm 读取并删除文件（常规 read 路径）。
pub fn consume_compositor_frame(name: &str, expected_len: usize) -> Result<Vec<u8>, ProtocolError> {
    let data = read_compositor_blob(name)?;
    if data.len() != expected_len {
        return Err(ProtocolError::Channel(format!(
            "compositor shm 大小不匹配: 期望 {expected_len}, 实际 {}",
            data.len()
        )));
    }
    Ok(data)
}

fn read_compositor_blob(name: &str) -> Result<Vec<u8>, ProtocolError> {
    #[cfg(target_os = "linux")]
    {
        let path = shm_path(name);
        let data = std::fs::read(&path).map_err(|e| ProtocolError::Channel(format!("compositor shm 读取失败: {e}")))?;
        let _ = std::fs::remove_file(&path);
        Ok(data)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = name;
        Err(ProtocolError::Channel("compositor shm 仅 Linux 可用".into()))
    }
}

#[cfg(target_os = "linux")]
fn consume_compositor_blob_mmap(name: &str) -> Result<Vec<u8>, ProtocolError> {
    use std::fs::File;
    use std::os::unix::io::AsRawFd;

    let path = shm_path(name);
    let file = File::open(&path).map_err(|e| ProtocolError::Channel(format!("compositor shm 打开失败: {e}")))?;
    let len = file
        .metadata()
        .map_err(|e| ProtocolError::Channel(format!("compositor shm stat 失败: {e}")))?
        .len() as usize;
    if len == 0 {
        let _ = std::fs::remove_file(&path);
        return Err(ProtocolError::Channel("compositor shm 为空".into()));
    }
    // SAFETY: MAP_PRIVATE 只读映射；随后 munmap。
    let ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            len,
            libc::PROT_READ,
            libc::MAP_PRIVATE,
            file.as_raw_fd(),
            0,
        )
    };
    if ptr == libc::MAP_FAILED {
        let _ = std::fs::remove_file(&path);
        return Err(ProtocolError::Channel(format!(
            "compositor shm mmap 失败: {}",
            std::io::Error::last_os_error()
        )));
    }
    let data = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) }.to_vec();
    unsafe {
        libc::munmap(ptr, len);
    }
    let _ = std::fs::remove_file(&path);
    Ok(data)
}

fn consume_gpu_mailbox_blob(
    name: &str,
    desc: &crate::GpuSharedImageDescriptor,
    min_sync_token: Option<u64>,
) -> Result<Vec<u8>, ProtocolError> {
    #[cfg(target_os = "linux")]
    let blob = if compositor_gpu_zero_copy_enabled() || desc.zero_copy {
        consume_compositor_blob_mmap(name)?
    } else {
        read_compositor_blob(name)?
    };
    #[cfg(not(target_os = "linux"))]
    let blob = read_compositor_blob(name)?;

    let (header, payload_off) = decode_gpu_mailbox(&blob)?;
    validate_gpu_mailbox_header(&header, desc, min_sync_token)?;
    let end = payload_off.saturating_add(header.payload_len as usize);
    Ok(blob[payload_off..end].to_vec())
}

fn validate_gpu_mailbox_header(
    header: &GpuMailboxHeader,
    desc: &crate::GpuSharedImageDescriptor,
    min_sync_token: Option<u64>,
) -> Result<(), ProtocolError> {
    if header.width != desc.width || header.height != desc.height {
        return Err(ProtocolError::Channel(format!(
            "gpu mailbox 尺寸不匹配: 头 {}x{} 描述符 {}x{}",
            header.width, header.height, desc.width, desc.height
        )));
    }
    if header.sync_token != desc.sync_token {
        return Err(ProtocolError::Channel(format!(
            "gpu mailbox sync_token 不匹配: 头 {} 描述符 {}",
            header.sync_token, desc.sync_token
        )));
    }
    if let Some(min) = min_sync_token
        && header.sync_token < min
    {
        return Err(ProtocolError::Channel(format!(
            "gpu fence stale: sync_token {} < 期望 {min}",
            header.sync_token
        )));
    }
    let expected_pixels = expected_rgba_len(desc.width, desc.height);
    if header.payload_len as usize != expected_pixels {
        return Err(ProtocolError::Channel(format!(
            "gpu mailbox payload 大小不匹配: 期望 {expected_pixels}, 头 {}",
            header.payload_len
        )));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn consume_dma_buf_fd(desc: &crate::GpuSharedImageDescriptor) -> Result<std::os::fd::OwnedFd, ProtocolError> {
    use std::time::Duration;

    if desc.drm_modifier != 0 {
        return Err(ProtocolError::Channel(format!(
            "dma-buf modifier {} 尚未支持",
            desc.drm_modifier
        )));
    }
    let name = if desc.fd_socket_name.is_empty() {
        desc.mailbox_name.as_str()
    } else {
        desc.fd_socket_name.as_str()
    };
    if name.is_empty() {
        return Err(ProtocolError::Channel("dma-buf fd socket 名为空".into()));
    }
    crate::fd_socket_linux::consume_fd(name, Duration::from_secs(2))
}

/// Browser 解析 compositor 帧：RGBA 或 dma-buf fd（P0 GPU 导入）。
#[derive(Debug)]
pub enum CompositorResolvedFrame {
    /// CPU RGBA 像素。
    Rgba(Vec<u8>),
    /// Linux dma-buf / memfd fd（`ZW_BROWSER_GPU_DMABUF_IMPORT=1`）。
    #[cfg(target_os = "linux")]
    Dmabuf {
        /// 导入用 fd（SCM_RIGHTS 接收）。
        fd: std::os::fd::OwnedFd,
        /// 宽度（像素）。
        width: u32,
        /// 高度（像素）。
        height: u32,
        /// 行 stride（字节）。
        stride: u32,
        /// DRM fourcc。
        drm_fourcc: u32,
        /// DRM modifier（线性为 0）。
        drm_modifier: u64,
    },
}

/// 带 fence 校验的帧解析；dma-buf 导入时不拷贝 RGBA。
pub fn resolve_compositor_frame_delivery_fenced(
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    shm_name: Option<String>,
    gpu_image: Option<crate::GpuSharedImageDescriptor>,
    min_sync_token: Option<u64>,
) -> Result<CompositorResolvedFrame, ProtocolError> {
    let expected = expected_rgba_len(width, height);
    if let Some(desc) = gpu_image {
        if desc.width != width || desc.height != height {
            return Err(ProtocolError::Channel(format!(
                "gpu_image 尺寸不匹配: 期望 {width}x{height}, 描述符 {}x{}",
                desc.width, desc.height
            )));
        }
        if let Some(min) = min_sync_token
            && desc.sync_token < min
        {
            return Err(ProtocolError::Channel(format!(
                "gpu fence stale: sync_token {} < 期望 {min}",
                desc.sync_token
            )));
        }
        if desc.transport == crate::GpuImageTransport::DmaBuf && browser_gpu_dmabuf_import_enabled() {
            #[cfg(target_os = "linux")]
            {
                let fd = consume_dma_buf_fd(&desc)?;
                return Ok(CompositorResolvedFrame::Dmabuf {
                    fd,
                    width: desc.width,
                    height: desc.height,
                    stride: desc.stride,
                    drm_fourcc: desc.drm_fourcc,
                    drm_modifier: desc.drm_modifier,
                });
            }
            #[cfg(not(target_os = "linux"))]
            {
                let _ = desc;
                return Err(ProtocolError::Channel("dma-buf 导入仅 Linux 可用".into()));
            }
        }
        if desc.transport == crate::GpuImageTransport::DmaBuf {
            return consume_dma_buf_rgba(&desc).map(CompositorResolvedFrame::Rgba);
        }
        if desc.mailbox_name.is_empty() {
            return Err(ProtocolError::Channel("gpu_image mailbox 名为空".into()));
        }
        return consume_gpu_mailbox_blob(&desc.mailbox_name, &desc, min_sync_token).map(CompositorResolvedFrame::Rgba);
    }
    let _ = min_sync_token;
    match shm_name {
        Some(name) if !name.is_empty() => consume_compositor_frame(&name, expected).map(CompositorResolvedFrame::Rgba),
        Some(_) => Err(ProtocolError::Channel("compositor shm 名为空".into())),
        None => {
            if width == 0 && height == 0 && rgba.is_empty() {
                return Ok(CompositorResolvedFrame::Rgba(rgba));
            }
            if rgba.len() != expected {
                return Err(ProtocolError::Channel(format!(
                    "compositor 帧像素大小不匹配: 期望 {expected}, 实际 {}",
                    rgba.len()
                )));
            }
            Ok(CompositorResolvedFrame::Rgba(rgba))
        }
    }
}

#[cfg(target_os = "linux")]
fn consume_dma_buf_rgba(desc: &crate::GpuSharedImageDescriptor) -> Result<Vec<u8>, ProtocolError> {
    use std::os::fd::AsRawFd;
    use std::time::Duration;

    if desc.drm_modifier != 0 {
        return Err(ProtocolError::Channel(format!(
            "dma-buf modifier {} 尚未支持",
            desc.drm_modifier
        )));
    }
    let name = if desc.fd_socket_name.is_empty() {
        desc.mailbox_name.as_str()
    } else {
        desc.fd_socket_name.as_str()
    };
    if name.is_empty() {
        return Err(ProtocolError::Channel("dma-buf fd socket 名为空".into()));
    }
    let fd = crate::fd_socket_linux::consume_fd(name, Duration::from_secs(2))?;
    let expected = (desc.stride as usize).saturating_mul(desc.height as usize);
    let ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            expected,
            libc::PROT_READ,
            libc::MAP_SHARED,
            fd.as_raw_fd(),
            0,
        )
    };
    if ptr == libc::MAP_FAILED {
        return Err(ProtocolError::Channel(format!(
            "dma-buf mmap 失败: {}",
            std::io::Error::last_os_error()
        )));
    }
    let rgba = unsafe { std::slice::from_raw_parts(ptr as *const u8, expected) }.to_vec();
    unsafe {
        libc::munmap(ptr, expected);
    }
    Ok(rgba)
}

#[cfg(not(target_os = "linux"))]
fn consume_dma_buf_rgba(_desc: &crate::GpuSharedImageDescriptor) -> Result<Vec<u8>, ProtocolError> {
    Err(ProtocolError::Channel("dma-buf 仅 Linux 可用".into()))
}

/// compositor → Browser 帧像素交付方式（内联 / shm / gpu_image mailbox / dma-buf fd）。
#[derive(Debug, Clone, PartialEq)]
pub struct CompositorFrameDelivery {
    /// 内联 RGBA（非空时优先于 shm/gpu_image）。
    pub rgba: Vec<u8>,
    /// POSIX shm 名（不含前缀）。
    pub shm_name: Option<String>,
    /// GPU shared image 描述符（mailbox 或 dma-buf fd）。
    pub gpu_image: Option<crate::GpuSharedImageDescriptor>,
    /// dma-buf 导出待发布 fd（仅 compositor 内部；IPC 发出后须调用 [`publish_compositor_fd`]）。
    #[cfg(target_os = "linux")]
    pub pending_fd: Option<std::os::fd::RawFd>,
}

/// compositor 侧：构建 dma-buf fd 交付描述符（不含 shm 文件写入）。
#[cfg(target_os = "linux")]
#[allow(clippy::too_many_arguments)]
pub fn build_compositor_dma_buf_delivery(
    surface_id: u64,
    frame_id: u64,
    width: u32,
    height: u32,
    stride: u32,
    drm_fourcc: u32,
    drm_modifier: u64,
    sync_token: u64,
    fd: std::os::fd::RawFd,
) -> CompositorFrameDelivery {
    let fd_name = crate::fd_socket_linux::fd_socket_name(surface_id, frame_id);
    CompositorFrameDelivery {
        rgba: Vec::new(),
        shm_name: None,
        gpu_image: Some(crate::GpuSharedImageDescriptor {
            mailbox_name: fd_name.clone(),
            width,
            height,
            sync_token,
            zero_copy: true,
            transport: crate::GpuImageTransport::DmaBuf,
            drm_fourcc,
            stride,
            drm_modifier,
            fd_socket_name: fd_name,
        }),
        pending_fd: Some(fd),
    }
}

/// compositor 侧：经 Unix socket SCM_RIGHTS 发送 pending fd。
///
/// **所有权（R3340）**：`pending_fd` 转移给 [`crate::fd_socket_linux::publish_fd`]，
/// 后者在成功路径（SCM_RIGHTS 后关闭发送方副本）与所有错误路径上都关闭 fd。
/// 故 `publish_fd` 返回后本函数**不再** close（否则 double-close）。仅在「缺
/// `gpu_image` 描述符」分支（尚未调用 `publish_fd`）自行 close。
#[cfg(target_os = "linux")]
pub fn publish_compositor_fd(delivery: &mut CompositorFrameDelivery) -> Result<(), ProtocolError> {
    use std::time::Duration;

    let Some(fd) = delivery.pending_fd.take() else {
        return Ok(());
    };
    let Some(desc) = delivery.gpu_image.as_ref() else {
        unsafe {
            libc::close(fd);
        }
        return Err(ProtocolError::Channel("dma-buf 交付缺少 gpu_image 描述符".into()));
    };
    let name = if desc.fd_socket_name.is_empty() {
        desc.mailbox_name.as_str()
    } else {
        desc.fd_socket_name.as_str()
    };
    // fd 所有权转入 publish_fd（成功与错误路径均关闭），此处不再 close。
    crate::fd_socket_linux::publish_fd(name, fd, Duration::from_secs(2))?;
    Ok(())
}

/// 按 env 选择交付方式写入像素。
pub fn deliver_compositor_frame_pixels(
    surface_id: u64,
    frame_id: u64,
    pixels: &[u8],
    width: u32,
    height: u32,
) -> Result<CompositorFrameDelivery, ProtocolError> {
    if compositor_gpu_image_enabled() {
        let zero_copy = compositor_gpu_zero_copy_enabled();
        let mailbox_name = publish_compositor_gpu_mailbox(surface_id, frame_id, pixels, width, height, frame_id)?;
        return Ok(CompositorFrameDelivery {
            rgba: Vec::new(),
            shm_name: None,
            gpu_image: Some(crate::GpuSharedImageDescriptor {
                mailbox_name,
                width,
                height,
                sync_token: frame_id,
                zero_copy,
                transport: crate::GpuImageTransport::ShmRgba,
                drm_fourcc: 0,
                stride: 0,
                drm_modifier: 0,
                fd_socket_name: String::new(),
            }),
            #[cfg(target_os = "linux")]
            pending_fd: None,
        });
    }
    if compositor_shm_enabled() {
        let name = publish_compositor_frame(surface_id, frame_id, pixels)?;
        return Ok(CompositorFrameDelivery {
            rgba: Vec::new(),
            shm_name: Some(name),
            gpu_image: None,
            #[cfg(target_os = "linux")]
            pending_fd: None,
        });
    }
    Ok(CompositorFrameDelivery {
        rgba: pixels.to_vec(),
        shm_name: None,
        gpu_image: None,
        #[cfg(target_os = "linux")]
        pending_fd: None,
    })
}

/// 从内联 `rgba`、shm 或 gpu_image mailbox 解析完整像素。
pub fn resolve_compositor_frame_rgba(
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    shm_name: Option<String>,
    gpu_image: Option<crate::GpuSharedImageDescriptor>,
) -> Result<Vec<u8>, ProtocolError> {
    resolve_compositor_frame_rgba_fenced(width, height, rgba, shm_name, gpu_image, None)
}

/// 带 sync_token fence 校验的像素解析（`min_sync_token` 通常为期望 `frame_id`）。
pub fn resolve_compositor_frame_rgba_fenced(
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    shm_name: Option<String>,
    gpu_image: Option<crate::GpuSharedImageDescriptor>,
    min_sync_token: Option<u64>,
) -> Result<Vec<u8>, ProtocolError> {
    match resolve_compositor_frame_delivery_fenced(width, height, rgba, shm_name, gpu_image, min_sync_token)? {
        CompositorResolvedFrame::Rgba(bytes) => Ok(bytes),
        #[cfg(target_os = "linux")]
        CompositorResolvedFrame::Dmabuf { .. } => Err(ProtocolError::Channel(
            "dma-buf 导入路径：请使用 resolve_compositor_frame_delivery_fenced".into(),
        )),
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
        let resolved = resolve_compositor_frame_rgba(1, 1, vec![0; 4], Some(name), None).expect("resolve");
        assert_eq!(resolved, pixels);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn resolve_prefers_gpu_image_mailbox() {
        let pixels = vec![10u8, 20, 30, 40];
        let name = publish_compositor_gpu_mailbox(2, 3, &pixels, 1, 1, 3).expect("publish");
        let desc = crate::GpuSharedImageDescriptor {
            mailbox_name: name,
            width: 1,
            height: 1,
            sync_token: 3,
            zero_copy: false,
            transport: crate::GpuImageTransport::ShmRgba,
            drm_fourcc: 0,
            stride: 0,
            drm_modifier: 0,
            fd_socket_name: String::new(),
        };
        let resolved =
            resolve_compositor_frame_rgba_fenced(1, 1, vec![0; 4], None, Some(desc), Some(3)).expect("resolve");
        assert_eq!(resolved, pixels);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn gpu_fence_rejects_stale_sync_token() {
        let pixels = vec![10u8, 20, 30, 40];
        let name = publish_compositor_gpu_mailbox(2, 3, &pixels, 1, 1, 3).expect("publish");
        let desc = crate::GpuSharedImageDescriptor {
            mailbox_name: name,
            width: 1,
            height: 1,
            sync_token: 3,
            zero_copy: false,
            transport: crate::GpuImageTransport::ShmRgba,
            drm_fourcc: 0,
            stride: 0,
            drm_modifier: 0,
            fd_socket_name: String::new(),
        };
        let err =
            resolve_compositor_frame_rgba_fenced(1, 1, vec![0; 4], None, Some(desc), Some(4)).expect_err("stale fence");
        assert!(err.to_string().contains("stale"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[serial_test::serial]
    fn compositor_gpu_flags_default_on_linux() {
        // SAFETY: 测试专用 env 写入。
        unsafe {
            std::env::remove_var("ZW_COMPOSITOR_GPU");
            std::env::remove_var("ZW_COMPOSITOR_GPU_IMAGE");
            std::env::remove_var("ZW_COMPOSITOR_GPU_TEXTURE_EXPORT");
            std::env::remove_var("ZW_BROWSER_GPU_DMABUF_IMPORT");
        }
        assert!(compositor_gpu_enabled());
        assert!(compositor_gpu_image_enabled());
        assert!(compositor_gpu_texture_export_enabled());
        assert!(browser_gpu_dmabuf_import_enabled());

        // SAFETY: 测试专用 env 写入。
        unsafe {
            std::env::set_var("ZW_COMPOSITOR_GPU", "0");
            std::env::set_var("ZW_BROWSER_GPU_DMABUF_IMPORT", "0");
        }
        assert!(!compositor_gpu_enabled());
        assert!(!browser_gpu_dmabuf_import_enabled());
        // SAFETY: 清理测试 env。
        unsafe {
            std::env::remove_var("ZW_COMPOSITOR_GPU");
            std::env::remove_var("ZW_BROWSER_GPU_DMABUF_IMPORT");
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[serial_test::serial]
    fn resolve_delivery_fenced_returns_dmabuf_when_import_enabled() {
        use std::os::fd::IntoRawFd;
        use std::thread;

        // SAFETY: 测试专用 env 写入。
        unsafe {
            std::env::set_var("ZW_BROWSER_GPU_DMABUF_IMPORT", "1");
        }

        let memfd = unsafe { libc::memfd_create(c"zeroweb-test-dmabuf".as_ptr(), libc::MFD_CLOEXEC) };
        assert!(memfd >= 0);
        let width = 2u32;
        let height = 2u32;
        let stride = width * 4;
        let expected = (stride as usize) * (height as usize);
        assert_eq!(unsafe { libc::ftruncate(memfd, expected as libc::off_t) }, 0);
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                expected,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                memfd,
                0,
            )
        };
        assert_ne!(ptr, libc::MAP_FAILED);
        let pixels = [255u8, 128, 64, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255];
        unsafe {
            std::ptr::copy_nonoverlapping(pixels.as_ptr(), ptr as *mut u8, expected);
            libc::munmap(ptr, expected);
        }

        let mut delivery = build_compositor_dma_buf_delivery(99, 1, width, height, stride, 0x3432_4241, 0, 1, memfd);
        let desc = delivery.gpu_image.clone().expect("gpu_image");
        let handle = thread::spawn(move || publish_compositor_fd(&mut delivery).expect("publish fd"));

        let resolved = resolve_compositor_frame_delivery_fenced(width, height, Vec::new(), None, Some(desc), Some(1))
            .expect("resolve dmabuf");
        handle.join().expect("publish join");

        match resolved {
            CompositorResolvedFrame::Dmabuf {
                width: w,
                height: h,
                stride: s,
                ..
            } => {
                assert_eq!((w, h, s), (width, height, stride));
            }
            CompositorResolvedFrame::Rgba(_) => panic!("expected Dmabuf variant"),
        }

        // SAFETY: 测试 env 清理。
        unsafe {
            std::env::remove_var("ZW_BROWSER_GPU_DMABUF_IMPORT");
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[serial_test::serial]
    fn resolve_delivery_fenced_returns_rgba_when_import_disabled() {
        use std::os::fd::IntoRawFd;
        use std::thread;

        // SAFETY: 测试专用 env 写入。
        unsafe {
            std::env::set_var("ZW_BROWSER_GPU_DMABUF_IMPORT", "0");
        }

        let memfd = unsafe { libc::memfd_create(c"zeroweb-test-dmabuf-cpu".as_ptr(), libc::MFD_CLOEXEC) };
        assert!(memfd >= 0);
        let width = 1u32;
        let height = 1u32;
        let stride = 4u32;
        let expected = 4usize;
        assert_eq!(unsafe { libc::ftruncate(memfd, expected as libc::off_t) }, 0);
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                expected,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                memfd,
                0,
            )
        };
        assert_ne!(ptr, libc::MAP_FAILED);
        unsafe {
            std::ptr::copy_nonoverlapping([10u8, 20, 30, 40].as_ptr(), ptr as *mut u8, expected);
            libc::munmap(ptr, expected);
        }

        let mut delivery = build_compositor_dma_buf_delivery(100, 2, width, height, stride, 0x3432_4241, 0, 2, memfd);
        let desc = delivery.gpu_image.clone().expect("gpu_image");
        let handle = thread::spawn(move || publish_compositor_fd(&mut delivery).expect("publish fd"));

        let resolved = resolve_compositor_frame_delivery_fenced(width, height, Vec::new(), None, Some(desc), Some(2))
            .expect("resolve rgba fallback");
        handle.join().expect("publish join");

        match resolved {
            CompositorResolvedFrame::Rgba(bytes) => assert_eq!(bytes, vec![10, 20, 30, 40]),
            #[cfg(target_os = "linux")]
            CompositorResolvedFrame::Dmabuf { .. } => panic!("expected Rgba fallback"),
        }

        // SAFETY: 测试 env 清理。
        unsafe {
            std::env::remove_var("ZW_BROWSER_GPU_DMABUF_IMPORT");
        }
    }
}
