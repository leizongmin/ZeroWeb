//! Compositor 进程扩展类型（RFC 4.3-S2 / 4.4）。

use serde::{Deserialize, Serialize};

/// GPU shared image 描述符（4.3-S2+：mailbox 经 shm；S4 头含 sync_token fence）。
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
    /// 是否经 mmap 零拷贝路径发布（`ZW_COMPOSITOR_GPU_ZERO_COPY=1`）。
    #[serde(default)]
    pub zero_copy: bool,
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
