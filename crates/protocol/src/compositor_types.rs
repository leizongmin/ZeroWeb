//! Compositor 进程扩展类型（RFC 4.3-S2 / 4.4）。

use serde::{Deserialize, Serialize};

/// GPU 图像跨进程传输方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GpuImageTransport {
    /// RGBA 经 POSIX shm + 可选 mailbox 头（默认）。
    #[default]
    ShmRgba,
    /// Linux dma-buf fd 经 Unix socket SCM_RIGHTS（`ZW_COMPOSITOR_GPU_TEXTURE_EXPORT=1`）。
    DmaBuf,
}

/// GPU shared image 描述符（4.3-S2+：mailbox 经 shm；S4 fence；S5 dma-buf fd）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GpuSharedImageDescriptor {
    /// Mailbox 或等价跨进程资源名。
    pub mailbox_name: String,
    /// 宽度（像素）。
    pub width: u32,
    /// 高度（像素）。
    pub height: u32,
    /// 同步代际（fence；单调递增，Browser 须 ≥ 期望 frame_id）。
    #[serde(default)]
    pub sync_token: u64,
    /// 是否经 mmap 零拷贝路径发布（Linux 默认开）。
    #[serde(default)]
    pub zero_copy: bool,
    /// 传输方式（默认 shm RGBA）。
    #[serde(default)]
    pub transport: GpuImageTransport,
    /// DRM fourcc（DmaBuf；如 ABGR8888 = 0x34324241）。
    #[serde(default)]
    pub drm_fourcc: u32,
    /// 行 stride（字节；DmaBuf）。
    #[serde(default)]
    pub stride: u32,
    /// DRM modifier（DmaBuf；线性为 0）。
    #[serde(default)]
    pub drm_modifier: u64,
    /// fd 辅助 socket 名（DmaBuf；空则使用 `mailbox_name`）。
    #[serde(default)]
    pub fd_socket_name: String,
}

/// UI 层 surface 注册元数据（4.4 Viz 切片）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositorUiSurfaceInfo {
    /// UI surface 标识（与页面 surface_id 命名空间独立）。
    pub surface_id: u64,
    /// 逻辑宽度。
    pub width: u32,
    /// 逻辑高度。
    pub height: u32,
}

/// 最终窗口 surface 登记（RFC 4.4-S4；compositor 拥有 present 输出）。
pub type CompositorWindowSurfaceInfo = CompositorUiSurfaceInfo;
