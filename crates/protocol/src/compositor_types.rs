//! Compositor 进程扩展类型（RFC 4.3-S2 / 4.4）。

use serde::{Deserialize, Serialize};

/// GPU shared image 描述符（4.3-S2 协议预留；mailbox/fence 接线为后续切片）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GpuSharedImageDescriptor {
    /// Mailbox 或等价跨进程资源名。
    pub mailbox_name: String,
    /// 宽度（像素）。
    pub width: u32,
    /// 高度（像素）。
    pub height: u32,
}

/// UI 层 surface 注册元数据（4.4 Viz 切片；最终 present 仍为后续）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositorUiSurfaceInfo {
    /// UI surface 标识（与页面 surface_id 命名空间独立）。
    pub surface_id: u64,
    /// 逻辑宽度。
    pub width: u32,
    /// 逻辑高度。
    pub height: u32,
}
